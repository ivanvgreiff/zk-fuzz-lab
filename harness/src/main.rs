use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use rust_eq_oracle::{compare, RunResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "harness")]
#[command(about = "ZKVM differential fuzzing harness")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run differential test on a core program
    Run {
        /// Path to the core (e.g., guest/cores/fib)
        #[arg(short, long)]
        core: PathBuf,

        /// Path to input JSON file
        #[arg(short, long)]
        input: PathBuf,

        /// Skip building the SP1 guest (use existing ELF)
        #[arg(long)]
        skip_build: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct RunLog {
    run_id: String,
    timestamp: String,
    core_path: String,
    input_path: String,
    native_result: RunResult,
    sp1_result: RunResult,
    diff: rust_eq_oracle::Diff,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            core,
            input,
            skip_build,
        } => run_differential_test(&core, &input, skip_build),
    }
}

fn run_differential_test(core_path: &PathBuf, input_path: &PathBuf, skip_build: bool) -> Result<()> {
    println!("🚀 Starting differential test...");
    println!("   Core: {}", core_path.display());
    println!("   Input: {}", input_path.display());
    println!();

    // Determine guest path (assume convention: adapters/sp1_guest/{core_name}_guest)
    let core_name = core_path
        .file_name()
        .context("Invalid core path")?
        .to_str()
        .context("Non-UTF8 core name")?;
    
    let guest_path = PathBuf::from(format!("adapters/sp1_guest/{}_guest", core_name));
    // ELF filename uses hyphens instead of underscores
    let elf_name = core_name.replace("_", "-");
    let elf_path = guest_path
        .join("target/elf-compilation/riscv32im-succinct-zkvm-elf/release")
        .join(format!("{}-guest", elf_name));

    // Step 1: Build SP1 guest (unless skip_build is set)
    if !skip_build {
        println!("📦 Building SP1 guest...");
        build_sp1_guest(&guest_path)?;
        println!("   ✅ SP1 guest built\n");
    } else {
        println!("⏩ Skipping SP1 guest build\n");
    }

    // Step 2: Run native runner
    println!("🏃 Running native...");
    let native_result = run_native_runner(core_name, input_path)?;
    println!("   ✅ Native completed in {}ms\n", native_result.elapsed_ms);

    // Step 3: Run SP1 runner
    println!("🏃 Running SP1...");
    let sp1_result = run_sp1_runner(&elf_path, input_path, core_name)?;
    println!("   ✅ SP1 completed in {}ms\n", sp1_result.elapsed_ms);

    // Step 4: Compare results
    println!("🔍 Comparing results...");
    let diff = compare(&native_result, &sp1_result);

    if diff.equal {
        println!("   ✅ PASS - Results match!");
        if let Some(delta) = diff.timing_delta_ms {
            println!("   📊 Timing delta: {}ms", delta);
        }
    } else {
        println!("   ❌ FAIL - Results differ!");
        if let Some(reason) = &diff.reason {
            println!("   📋 Reason: {}", reason);
        }
    }
    println!();

    // Step 5: Log results
    println!("💾 Logging results...");
    log_results(core_path, input_path, native_result, sp1_result, diff)?;
    println!("   ✅ Results logged to artifacts/\n");

    Ok(())
}

fn build_sp1_guest(guest_path: &PathBuf) -> Result<()> {
    let status = Command::new("cargo")
        .args(["prove", "build"])
        .current_dir(guest_path)
        .status()
        .context("Failed to execute cargo prove build")?;

    if !status.success() {
        anyhow::bail!("cargo prove build failed");
    }

    Ok(())
}

fn run_native_runner(core_name: &str, input_path: &PathBuf) -> Result<RunResult> {
    let output = Command::new("cargo")
        .args(["run", "--release", "--bin", "native-runner", "--"])
        .args(["--core", core_name])
        .args(["--input", input_path.to_str().unwrap()])
        .output()
        .context("Failed to run native-runner")?;

    if !output.status.success() {
        anyhow::bail!(
            "native-runner failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let result: RunResult = serde_json::from_slice(&output.stdout)
        .context("Failed to parse native-runner output")?;

    Ok(result)
}

fn run_sp1_runner(elf_path: &PathBuf, input_path: &PathBuf, core_name: &str) -> Result<RunResult> {
    // Determine number of commits based on core
    let num_commits = match core_name {
        "fib" => 3,
        "panic_test" => 2,
        "timeout_test" => 1,
        "io_echo" => 3,          // length, first_byte, last_byte
        "arithmetic" => 2,       // result, overflowed
        "simple_struct" => 4,    // field1_echo, field2_len, field2_chars, field3_echo
        _ => {
            // For unknown cores, don't specify (will try to read until exhausted)
            0
        }
    };

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--release", "--bin", "sp1-runner", "--"])
        .args(["--elf", elf_path.to_str().unwrap()])
        .args(["--input", input_path.to_str().unwrap()]);

    // Add num-commits if known
    if num_commits > 0 {
        cmd.args(["--num-commits", &num_commits.to_string()]);
    }

    let output = cmd
        .output()
        .context("Failed to run sp1-runner")?;

    if !output.status.success() {
        anyhow::bail!(
            "sp1-runner failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let result: RunResult = serde_json::from_slice(&output.stdout)
        .context("Failed to parse sp1-runner output")?;

    Ok(result)
}

fn log_results(
    core_path: &PathBuf,
    input_path: &PathBuf,
    native_result: RunResult,
    sp1_result: RunResult,
    diff: rust_eq_oracle::Diff,
) -> Result<()> {
    // Create artifacts directory if it doesn't exist
    fs::create_dir_all("artifacts")?;

    // Generate run ID
    let timestamp = Utc::now();
    let run_id = format!(
        "{}_{}",
        timestamp.format("%Y%m%d_%H%M%S"),
        core_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    );

    // Create run log
    let log = RunLog {
        run_id: run_id.clone(),
        timestamp: timestamp.to_rfc3339(),
        core_path: core_path.display().to_string(),
        input_path: input_path.display().to_string(),
        native_result: native_result.clone(),
        sp1_result: sp1_result.clone(),
        diff: diff.clone(),
    };

    // Write detailed JSON log
    let log_path = PathBuf::from("artifacts").join(format!("{}.json", run_id));
    let log_json = serde_json::to_string_pretty(&log)?;
    fs::write(&log_path, &log_json)?;

    println!("   📄 Detailed log: {}", log_path.display());

    // If there's a divergence, create a repro folder
    if !diff.equal {
        let repro_dir = PathBuf::from("artifacts").join(&run_id);
        fs::create_dir_all(&repro_dir)?;

        // Generate repro script
        let repro_script = generate_repro_script(core_path, input_path);
        let repro_path = repro_dir.join("repro.sh");
        fs::write(&repro_path, repro_script)?;

        // Make script executable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&repro_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&repro_path, perms)?;
        }

        // Copy input file to repro folder
        let input_copy = repro_dir.join("input.json");
        fs::copy(input_path, &input_copy)?;

        // Write detailed log to repro folder as well
        let log_copy = repro_dir.join("run_log.json");
        fs::write(&log_copy, log_json)?;

        println!("   🔧 Repro folder: {}", repro_dir.display());
    }

    // Append to CSV summary
    append_to_csv_summary(&run_id, core_path, input_path, &native_result, &sp1_result, &diff)?;

    Ok(())
}

/// Generate a repro script for the given test case
fn generate_repro_script(core_path: &PathBuf, input_path: &PathBuf) -> String {
    format!(
        r#"#!/usr/bin/env bash
# Repro script generated by zk-fuzz-lab harness
# Run this script from the repository root

set -e

echo "🔁 Reproducing differential test..."
echo "   Core: {core}"
echo "   Input: {input}"
echo ""

# Run the differential test
make run CORE={core} INPUT={input}
"#,
        core = core_path.display(),
        input = input_path.display(),
    )
}

/// Append run results to CSV summary
fn append_to_csv_summary(
    run_id: &str,
    core_path: &PathBuf,
    input_path: &PathBuf,
    native_result: &RunResult,
    sp1_result: &RunResult,
    diff: &rust_eq_oracle::Diff,
) -> Result<()> {
    let csv_path = PathBuf::from("artifacts/summary.csv");
    
    // Check if file exists to determine if we need to write header
    let needs_header = !csv_path.exists();

    // Open file in append mode
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&csv_path)?;

    let mut writer = csv::Writer::from_writer(file);

    // Write header if this is a new file
    if needs_header {
        writer.write_record(&[
            "run_id",
            "core",
            "input",
            "native_status",
            "sp1_status",
            "equal",
            "reason",
            "elapsed_native_ms",
            "elapsed_sp1_ms",
            "timing_delta_ms",
        ])?;
    }

    // Write data row
    writer.write_record(&[
        run_id,
        &core_path.file_name().unwrap().to_str().unwrap(),
        &input_path.display().to_string(),
        &format!("{:?}", native_result.status),
        &format!("{:?}", sp1_result.status),
        &diff.equal.to_string(),
        &diff.reason.clone().unwrap_or_else(|| "".to_string()),
        &native_result.elapsed_ms.to_string(),
        &sp1_result.elapsed_ms.to_string(),
        &diff.timing_delta_ms.map(|d| d.to_string()).unwrap_or_else(|| "".to_string()),
    ])?;

    writer.flush()?;

    Ok(())
}

