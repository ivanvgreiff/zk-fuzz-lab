use anyhow::Result;
use clap::Parser;
use fib_core::{FibInput, run};
use rust_eq_oracle::{RunResult, Status};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(name = "native-runner")]
#[command(about = "Runs plain Rust cores natively and outputs RunResult JSON")]
struct Args {
    /// Path to the input JSON file
    #[arg(short, long)]
    input: PathBuf,

    /// Path to write the RunResult JSON (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read and parse input
    let input_bytes = fs::read(&args.input)?;
    let input: FibInput = serde_json::from_slice(&input_bytes)?;

    // Run the core and measure time
    let start = Instant::now();
    let output = run(input);
    let elapsed = start.elapsed();

    // Create commit stream (must match SP1 guest adapter order)
    let commits = vec![
        serde_json::to_value(&output.n)?,
        serde_json::to_value(&output.a)?,
        serde_json::to_value(&output.b)?,
    ];

    // Create RunResult
    let result = RunResult {
        status: Status::Ok,
        elapsed_ms: elapsed.as_millis(),
        commits,
        meta: serde_json::json!({
            "runner": "native"
        }),
    };

    // Serialize and output
    let result_json = serde_json::to_string_pretty(&result)?;
    
    if let Some(output_path) = args.output {
        fs::write(output_path, result_json)?;
    } else {
        println!("{}", result_json);
    }

    Ok(())
}

