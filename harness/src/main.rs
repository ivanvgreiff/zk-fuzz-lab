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
    let elf_path = guest_path
        .join("target/elf-compilation/riscv32im-succinct-zkvm-elf/release")
        .join(format!("{}-guest", core_name));

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
    let native_result = run_native_runner(input_path)?;
    println!("   ✅ Native completed in {}ms\n", native_result.elapsed_ms);

    // Step 3: Run SP1 runner
    println!("🏃 Running SP1...");
    let sp1_result = run_sp1_runner(&elf_path, input_path)?;
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

fn run_native_runner(input_path: &PathBuf) -> Result<RunResult> {
    let output = Command::new("cargo")
        .args(["run", "--release", "--bin", "native-runner", "--"])
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

fn run_sp1_runner(elf_path: &PathBuf, input_path: &PathBuf) -> Result<RunResult> {
    let output = Command::new("cargo")
        .args(["run", "--release", "--bin", "sp1-runner", "--"])
        .args(["--elf", elf_path.to_str().unwrap()])
        .args(["--input", input_path.to_str().unwrap()])
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
        native_result,
        sp1_result,
        diff,
    };

    // Write detailed JSON log
    let log_path = PathBuf::from("artifacts").join(format!("{}.json", run_id));
    let log_json = serde_json::to_string_pretty(&log)?;
    fs::write(&log_path, log_json)?;

    println!("   📄 Detailed log: {}", log_path.display());

    Ok(())
}

