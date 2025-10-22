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
    
    /// Run input mutation fuzzing on one or more cores
    Fuzz {
        /// Core name to fuzz (e.g., "io_echo") or comma-separated list (e.g., "io_echo,arithmetic") or "all"
        #[arg(short, long)]
        cores: String,

        /// Skip building the SP1 guests (use existing ELFs)
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
        Commands::Fuzz {
            cores,
            skip_build,
        } => run_fuzzing(&cores, skip_build),
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

/// Get SP1 version string
fn get_sp1_version() -> String {
    Command::new("cargo")
        .args(["prove", "--version"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get rustc version string
fn get_rustc_version() -> String {
    Command::new("rustc")
        .args(["--version"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
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
            // Phase 4: Future-proofing columns
            "repro_path",
            "generator",
            "base_seed",
            "mutation_ops",
            "rng_seed",
            "zkvm_target",
            "sp1_version",
            "rustc_version",
        ])?;
    }

    // Determine repro_path (artifacts/<run_id>/ if divergence, empty otherwise)
    let repro_path = if !diff.equal {
        format!("artifacts/{}/", run_id)
    } else {
        String::new()
    };

    // Get version strings (cached for performance in future batch runs)
    let sp1_version = get_sp1_version();
    let rustc_version = get_rustc_version();

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
        // Phase 4: Future-proofing columns
        &repro_path,
        "hand_written",  // generator (Phase 5 will populate with "mutated", Phase 6 with "rustsmith")
        "",              // base_seed (empty for now, Phase 5 will populate)
        "",              // mutation_ops (empty for now, Phase 5 will populate)
        "",              // rng_seed (empty for now, Phase 6 will populate)
        "sp1",           // zkvm_target (Phase 8 will add risc0, openvm)
        &sp1_version,
        &rustc_version,
    ])?;

    writer.flush()?;

    Ok(())
}

/// Run input mutation fuzzing on specified cores
fn run_fuzzing(cores_arg: &str, skip_build: bool) -> Result<()> {
    // Parse cores argument
    let available_cores = vec!["fib", "panic_test", "timeout_test", "io_echo", "arithmetic", "simple_struct"];
    
    let cores_to_fuzz: Vec<&str> = if cores_arg == "all" {
        available_cores.clone()
    } else {
        cores_arg.split(',').map(|s| s.trim()).collect()
    };

    // Validate cores
    for core in &cores_to_fuzz {
        if !available_cores.contains(core) {
            anyhow::bail!(
                "Unknown core: '{}'\n\nAvailable cores: {}",
                core,
                available_cores.join(", ")
            );
        }
    }

    println!("🔄 Starting input mutation fuzzing...");
    println!("   Cores: {}", cores_to_fuzz.join(", "));
    println!();

    let mut total_mutations = 0;
    let mut total_passed = 0;
    let mut total_divergences = 0;
    let overall_start = std::time::Instant::now();

    // Fuzz each core
    for core_name in cores_to_fuzz {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📦 Core: {}", core_name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();

        let result = fuzz_single_core(core_name, skip_build)?;
        
        total_mutations += result.total;
        total_passed += result.passed;
        total_divergences += result.divergences;

        println!();
    }

    let overall_elapsed = overall_start.elapsed();

    // Overall summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎯 Fuzzing Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("📊 Overall Summary:");
    println!("   Total mutations: {}", total_mutations);
    println!("   Passed: {} ({:.1}%)", total_passed, (total_passed as f64 / total_mutations as f64) * 100.0);
    println!("   Divergences: {} ({:.1}%)", total_divergences, (total_divergences as f64 / total_mutations as f64) * 100.0);
    println!("   Total time: {:.1}s", overall_elapsed.as_secs_f64());
    println!();
    println!("💾 All results logged to artifacts/summary.csv");
    
    if total_divergences > 0 {
        println!("   🔧 Divergence artifacts in artifacts/");
    }

    Ok(())
}

#[derive(Debug)]
struct FuzzResult {
    total: usize,
    passed: usize,
    divergences: usize,
}

/// Fuzz a single core with input mutations
fn fuzz_single_core(core_name: &str, skip_build: bool) -> Result<FuzzResult> {
    // Determine base input path for this core
    let base_input_path = get_base_input_for_core(core_name)?;
    
    println!("   Base input: {}", base_input_path.display());

    // Load base input
    let base_input_json: serde_json::Value = serde_json::from_slice(&fs::read(&base_input_path)?)?;

    // Generate mutations
    println!("   Generating mutations...");
    let mutations = source_mutator::generate_mutations(
        core_name,
        &base_input_json,
        base_input_path.to_str().unwrap(),
    )?;

    println!("   ✅ Generated {} mutations", mutations.len());

    // Calculate and display statistics
    if core_name == "io_echo" {
        let stats = source_mutator::calculate_size_stats(&mutations);
        println!();
        println!("   📊 Size Distribution:");
        println!("      Min: {} bytes", stats.min_size.unwrap_or(0));
        println!("      Max: {} bytes", stats.max_size.unwrap_or(0));
        if let Some(max) = stats.max_size {
            if max >= 1024 {
                println!("           ({:.2} KB)", max as f64 / 1024.0);
            }
            if max >= 1048576 {
                println!("           ({:.2} MB)", max as f64 / 1048576.0);
            }
        }
        println!("      Total sizes: {}", stats.total_count);
    }

    println!();
    println!("   🧪 Testing mutations...");
    println!();

    // Create artifacts directory for this fuzzing run
    let timestamp = Utc::now();
    let fuzz_run_id = format!("{}_fuzz_{}", timestamp.format("%Y%m%d_%H%M%S"), core_name);
    let fuzz_artifacts_dir = PathBuf::from("artifacts/mutations").join(&fuzz_run_id);
    fs::create_dir_all(&fuzz_artifacts_dir)?;

    // Save mutation plan
    let plan_path = fuzz_artifacts_dir.join("plan.json");
    let plan_json = serde_json::to_string_pretty(&mutations.iter().map(|m| {
        serde_json::json!({
            "mutation_op": &m.mutation_op,
            "base": &m.base_input_path,
        })
    }).collect::<Vec<_>>())?;
    fs::write(&plan_path, plan_json)?;

    let mut passed = 0;
    let mut divergences = 0;
    let mut native_times = Vec::new();
    let mut sp1_times = Vec::new();

    let core_path = PathBuf::from(format!("guest/cores/{}", core_name));

    // Build SP1 guest once (unless skip_build)
    if !skip_build {
        let guest_path = PathBuf::from(format!("adapters/sp1_guest/{}_guest", core_name));
        println!("   📦 Building SP1 guest for {}...", core_name);
        build_sp1_guest(&guest_path)?;
        println!("   ✅ SP1 guest built");
        println!();
    }

    // Test each mutation
    for (idx, mutation) in mutations.iter().enumerate() {
        let mutation_num = idx + 1;
        let total = mutations.len();

        // Save mutated input temporarily
        let temp_input_path = fuzz_artifacts_dir.join(format!("input_{}.json", mutation_num));
        fs::write(&temp_input_path, serde_json::to_string_pretty(&mutation.input_json)?)?;

        // Run differential test
        let native_result = run_native_runner(core_name, &temp_input_path)?;
        let elf_name = core_name.replace("_", "-");
        let elf_path = PathBuf::from(format!("adapters/sp1_guest/{}_guest", core_name))
            .join("target/elf-compilation/riscv32im-succinct-zkvm-elf/release")
            .join(format!("{}-guest", elf_name));
        let sp1_result = run_sp1_runner(&elf_path, &temp_input_path, core_name)?;

        // Compare
        let diff = compare(&native_result, &sp1_result);

        // Track stats
        native_times.push(native_result.elapsed_ms);
        sp1_times.push(sp1_result.elapsed_ms);

        if diff.equal {
            passed += 1;
        } else {
            divergences += 1;
        }

        // Display progress
        let status_icon = if diff.equal { "✅" } else { "❌" };
        println!(
            "   {} Mutation {}/{}: {} | Native: {:?} ({}ms) | SP1: {:?} ({}ms) | Equal: {}",
            status_icon,
            mutation_num,
            total,
            mutation.mutation_op,
            native_result.status,
            native_result.elapsed_ms,
            sp1_result.status,
            sp1_result.elapsed_ms,
            diff.equal,
        );

        if !diff.equal {
            if let Some(reason) = &diff.reason {
                println!("      Reason: {}", reason);
            }
        }

        // Log to CSV with mutation metadata
        log_mutation_result(
            &core_path,
            &temp_input_path,
            native_result,
            sp1_result,
            diff,
            &mutation.mutation_op,
            &mutation.base_input_path,
        )?;
    }

    // Calculate timing stats
    let native_avg = native_times.iter().sum::<u128>() as f64 / native_times.len() as f64;
    let sp1_avg = sp1_times.iter().sum::<u128>() as f64 / sp1_times.len() as f64;
    let native_max = native_times.iter().max().unwrap_or(&0);
    let sp1_max = sp1_times.iter().max().unwrap_or(&0);

    println!();
    println!("   📊 Timing Statistics:");
    println!("      Native: avg {:.1}ms, max {}ms", native_avg, native_max);
    println!("      SP1: avg {:.1}ms, max {}ms", sp1_avg, sp1_max);
    println!();
    println!("   ✅ Core '{}' fuzzing complete!", core_name);
    println!("      Total: {}", mutations.len());
    println!("      Passed: {} ({:.1}%)", passed, (passed as f64 / mutations.len() as f64) * 100.0);
    println!("      Divergences: {}", divergences);

    Ok(FuzzResult {
        total: mutations.len(),
        passed,
        divergences,
    })
}

/// Get the base input path for a given core
fn get_base_input_for_core(core_name: &str) -> Result<PathBuf> {
    let base_input = match core_name {
        "fib" => "inputs/fib_24.json",
        "panic_test" => "inputs/panic_no.json",
        "timeout_test" => "inputs/timeout_finite.json",
        "io_echo" => "inputs/io_echo_1kb.json",
        "arithmetic" => "inputs/arithmetic_add_normal.json",
        "simple_struct" => "inputs/simple_struct_normal.json",
        _ => anyhow::bail!("Unknown core: {}", core_name),
    };
    Ok(PathBuf::from(base_input))
}

/// Log mutation result to CSV with mutation metadata
fn log_mutation_result(
    core_path: &PathBuf,
    input_path: &PathBuf,
    native_result: RunResult,
    sp1_result: RunResult,
    diff: rust_eq_oracle::Diff,
    mutation_op: &str,
    base_input_path: &str,
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
            // Phase 4: Future-proofing columns
            "repro_path",
            "generator",
            "base_seed",
            "mutation_ops",
            "rng_seed",
            "zkvm_target",
            "sp1_version",
            "rustc_version",
        ])?;
    }

    // Generate run ID
    let timestamp = Utc::now();
    let run_id = format!(
        "{}_{}",
        timestamp.format("%Y%m%d_%H%M%S"),
        core_path.file_name().unwrap().to_str().unwrap()
    );

    // Determine repro_path (artifacts/<run_id>/ if divergence, empty otherwise)
    let repro_path = if !diff.equal {
        format!("artifacts/{}/", run_id)
    } else {
        String::new()
    };

    // Get version strings
    let sp1_version = get_sp1_version();
    let rustc_version = get_rustc_version();

    // Convert core name to String to avoid &&str issue
    let core_name_str = core_path.file_name().unwrap().to_str().unwrap().to_string();
    
    // Write data row with mutation metadata
    writer.write_record(&[
        &run_id,
        &core_name_str,
        &input_path.display().to_string(),
        &format!("{:?}", native_result.status),
        &format!("{:?}", sp1_result.status),
        &diff.equal.to_string(),
        &diff.reason.clone().unwrap_or_else(|| "".to_string()),
        &native_result.elapsed_ms.to_string(),
        &sp1_result.elapsed_ms.to_string(),
        &diff.timing_delta_ms.map(|d| d.to_string()).unwrap_or_else(|| "".to_string()),
        // Phase 5: Mutation metadata
        &repro_path,
        "mutated",          // generator
        base_input_path,    // base_seed
        mutation_op,        // mutation_ops
        "",                 // rng_seed (empty for deterministic)
        "sp1",              // zkvm_target
        &sp1_version,
        &rustc_version,
    ])?;

    writer.flush()?;

    // If divergence, create repro folder (same as run_differential_test)
    if !diff.equal {
        let repro_dir = PathBuf::from("artifacts").join(&run_id);
        fs::create_dir_all(&repro_dir)?;

        // Copy input
        fs::copy(input_path, repro_dir.join("input.json"))?;

        // Generate repro script
        let repro_script = generate_repro_script(core_path, input_path);
        let repro_path = repro_dir.join("repro.sh");
        fs::write(&repro_path, repro_script)?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&repro_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&repro_path, perms)?;
        }

        // Write detailed log
        let log = RunLog {
            run_id: run_id.clone(),
            timestamp: timestamp.to_rfc3339(),
            core_path: core_path.display().to_string(),
            input_path: input_path.display().to_string(),
            native_result,
            sp1_result,
            diff,
        };
        fs::write(repro_dir.join("run_log.json"), serde_json::to_string_pretty(&log)?)?;
    }

    Ok(())
}
