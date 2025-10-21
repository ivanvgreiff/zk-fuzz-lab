use anyhow::{Context, Result};
use clap::Parser;
use rust_eq_oracle::{RunResult, Status};
use std::any::Any;
use std::fs;
use std::panic;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(name = "native-runner")]
#[command(about = "Runs plain Rust cores natively and outputs RunResult JSON")]
struct Args {
    /// Name of the core to run (e.g., "fib", "panic_test")
    #[arg(short, long)]
    core: String,

    /// Path to the input JSON file
    #[arg(short, long)]
    input: PathBuf,

    /// Path to write the RunResult JSON (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Timeout in seconds (0 = no timeout)
    #[arg(long, default_value = "30")]
    timeout: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read input bytes
    let input_bytes = fs::read(&args.input).context("Failed to read input file")?;

    // Run the core with panic capture and timeout
    let timeout_duration = if args.timeout > 0 {
        Some(Duration::from_secs(args.timeout))
    } else {
        None
    };

    let result = run_core_with_safeguards(&args.core, input_bytes, timeout_duration)?;

    // Serialize and output
    let result_json = serde_json::to_string_pretty(&result)?;
    
    if let Some(output_path) = args.output {
        fs::write(output_path, result_json)?;
    } else {
        println!("{}", result_json);
    }

    Ok(())
}

/// Run a core with panic capture and timeout handling
fn run_core_with_safeguards(
    core_name: &str,
    input_bytes: Vec<u8>,
    timeout: Option<Duration>,
) -> Result<RunResult> {
    let (tx, rx) = mpsc::channel();
    let core_name = core_name.to_string();

    // Spawn thread to run core
    let handle = thread::spawn(move || {
        // Capture panics
        let panic_result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let start = Instant::now();
            let commits = run_core_dispatch(&core_name, &input_bytes)?;
            let elapsed = start.elapsed();
            
            Ok::<_, anyhow::Error>(RunResult {
                status: Status::Ok,
                elapsed_ms: elapsed.as_millis(),
                commits,
                meta: serde_json::json!({"runner": "native"}),
            })
        }));

        match panic_result {
            Ok(Ok(result)) => tx.send(Ok(result)),
            Ok(Err(e)) => tx.send(Err(e)),
            Err(panic_err) => {
                let panic_msg = extract_panic_message(&panic_err);
                tx.send(Ok(RunResult {
                    status: Status::Panic,
                    elapsed_ms: 0,
                    commits: vec![],
                    meta: serde_json::json!({
                        "runner": "native",
                        "panic_msg": panic_msg,
                    }),
                }))
            }
        }
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
                        "runner": "native",
                        "timeout_secs": timeout_duration.as_secs(),
                    }),
                })
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                anyhow::bail!("Runner thread disconnected unexpectedly")
            }
        }
    } else {
        // No timeout
        rx.recv().context("Runner thread disconnected")?
    };

    // Clean up thread
    let _ = handle.join();

    result
}

/// Dispatch to the appropriate core based on name
fn run_core_dispatch(core_name: &str, input_bytes: &[u8]) -> Result<Vec<serde_json::Value>> {
    match core_name {
        "fib" => {
            let input: fib_core::FibInput = serde_json::from_slice(input_bytes)?;
            let output = fib_core::run(input);
            Ok(vec![
                serde_json::to_value(&output.n)?,
                serde_json::to_value(&output.a)?,
                serde_json::to_value(&output.b)?,
            ])
        }
        "panic_test" => {
            let input: panic_test_core::PanicInput = serde_json::from_slice(input_bytes)?;
            let output = panic_test_core::run(input);
            Ok(vec![
                serde_json::to_value(&output.should_panic_u32)?,
                serde_json::to_value(&output.status_code)?,
            ])
        }
        "timeout_test" => {
            let input: timeout_test_core::TimeoutInput = serde_json::from_slice(input_bytes)?;
            let output = timeout_test_core::run(input);
            Ok(vec![
                serde_json::to_value(&output.completed)?,
            ])
        }
        "io_echo" => {
            let input: io_echo_core::IoEchoInput = serde_json::from_slice(input_bytes)?;
            let output = io_echo_core::run(input);
            // Encode Option<u8> as u32: 0 for None, 1+value for Some
            let first_byte_u32 = match output.first_byte {
                None => 0u32,
                Some(byte) => 1u32 + byte as u32,
            };
            let last_byte_u32 = match output.last_byte {
                None => 0u32,
                Some(byte) => 1u32 + byte as u32,
            };
            Ok(vec![
                serde_json::to_value(&output.length)?,
                serde_json::to_value(&first_byte_u32)?,
                serde_json::to_value(&last_byte_u32)?,
            ])
        }
        "arithmetic" => {
            let input: arithmetic_core::ArithmeticInput = serde_json::from_slice(input_bytes)?;
            let output = arithmetic_core::run(input);
            // Encode bool as u32: 0 for false, 1 for true
            let overflowed_u32 = if output.overflowed { 1u32 } else { 0u32 };
            Ok(vec![
                serde_json::to_value(&output.result)?,
                serde_json::to_value(&overflowed_u32)?,
            ])
        }
        "simple_struct" => {
            let input: simple_struct_core::SimpleStructInput = serde_json::from_slice(input_bytes)?;
            let output = simple_struct_core::run(input);
            // Encode bool as u32: 0 for false, 1 for true
            let field3_u32 = if output.field3_echo { 1u32 } else { 0u32 };
            Ok(vec![
                serde_json::to_value(&output.field1_echo)?,
                serde_json::to_value(&output.field2_len)?,
                serde_json::to_value(&output.field2_chars)?,
                serde_json::to_value(&field3_u32)?,
            ])
        }
        _ => anyhow::bail!("Unknown core: {}", core_name),
    }
}

/// Extract panic message from panic payload
fn extract_panic_message(panic_err: &Box<dyn Any + Send>) -> String {
    if let Some(s) = panic_err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_err.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic".to_string()
    }
}

