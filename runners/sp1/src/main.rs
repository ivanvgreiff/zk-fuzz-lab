use anyhow::{Context, Result};
use clap::Parser;
use rust_eq_oracle::{RunResult, Status};
use sp1_sdk::{ProverClient, SP1Stdin};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "sp1-runner")]
#[command(about = "Runs SP1 guest programs and outputs RunResult JSON")]
struct Args {
    /// Path to the SP1 guest ELF file
    #[arg(short, long)]
    elf: PathBuf,

    /// Path to the input JSON file
    #[arg(short, long)]
    input: PathBuf,

    /// Path to write the RunResult JSON (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Timeout in seconds (0 = no timeout)
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Number of values to read from public_values (if not specified, read until exhausted)
    #[arg(long)]
    num_commits: Option<usize>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read the ELF file
    let elf_bytes = fs::read(&args.elf)?;

    // Read the input JSON
    let input_bytes = fs::read(&args.input)?;

    // Run with timeout and panic capture
    let timeout_duration = if args.timeout > 0 {
        Some(Duration::from_secs(args.timeout))
    } else {
        None
    };

    let result = run_sp1_with_safeguards(
        elf_bytes,
        input_bytes,
        timeout_duration,
        args.num_commits,
    )?;

    // Serialize and output
    let result_json = serde_json::to_string_pretty(&result)?;
    
    if let Some(output_path) = args.output {
        fs::write(output_path, result_json)?;
    } else {
        println!("{}", result_json);
    }

    Ok(())
}

/// Run SP1 guest with timeout and panic capture
fn run_sp1_with_safeguards(
    elf_bytes: Vec<u8>,
    input_bytes: Vec<u8>,
    timeout: Option<Duration>,
    num_commits: Option<usize>,
) -> Result<RunResult> {
    let (tx, rx) = mpsc::channel();

    // Spawn thread to run SP1
    let handle = thread::spawn(move || {
        let result = (|| -> Result<RunResult> {
            // Create SP1 stdin and write the input
            let mut stdin = SP1Stdin::new();
            stdin.write(&input_bytes);

            // Create the prover client
            let client = ProverClient::from_env();

            // Execute (not prove) the program and measure time
            let start = Instant::now();
            let execution_result = client.execute(&elf_bytes, &stdin).run();
            let elapsed = start.elapsed();

            match execution_result {
                Ok((mut public_values, report)) => {
                    // Extract commits from public values
                    let mut commits = Vec::new();

                    if let Some(n) = num_commits {
                        // Read exact number of commits
                        for _ in 0..n {
                            // Read as u32 (generic type - could be improved)
                            let value: u32 = public_values.read();
                            commits.push(serde_json::to_value(&value)?);
                        }
                    } else {
                        // Read until exhausted (for now, hardcode for common types)
                        // TODO: Make this more generic in future phases
                        while let Ok(value) = std::panic::catch_unwind(
                            std::panic::AssertUnwindSafe(|| public_values.read::<u32>())
                        ) {
                            commits.push(serde_json::to_value(&value)?);
                        }
                    }

                    Ok(RunResult {
                        status: Status::Ok,
                        elapsed_ms: elapsed.as_millis(),
                        commits,
                        meta: serde_json::json!({
                            "runner": "sp1",
                            "mode": "execute",
                            "cycles": report.total_instruction_count(),
                        }),
                    })
                }
                Err(e) => {
                    // SP1 execution failed (likely panic in guest)
                    let error_msg = format!("{}", e);
                    Ok(RunResult {
                        status: Status::Panic,
                        elapsed_ms: elapsed.as_millis(),
                        commits: vec![],
                        meta: serde_json::json!({
                            "runner": "sp1",
                            "mode": "execute",
                            "panic_msg": error_msg,
                        }),
                    })
                }
            }
        })();

        tx.send(result)
    });

    // Wait with timeout
    let result = if let Some(timeout_duration) = timeout {
        match rx.recv_timeout(timeout_duration) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Thread is still running, mark as timeout
                Ok(RunResult {
                    status: Status::Timeout,
                    elapsed_ms: timeout_duration.as_millis(),
                    commits: vec![],
                    meta: serde_json::json!({
                        "runner": "sp1",
                        "mode": "execute",
                        "timeout_secs": timeout_duration.as_secs(),
                    }),
                })
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                anyhow::bail!("SP1 runner thread disconnected unexpectedly")
            }
        }
    } else {
        // No timeout
        rx.recv().context("SP1 runner thread disconnected")?
    };

    // Clean up thread
    let _ = handle.join();

    result
}

