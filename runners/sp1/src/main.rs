use anyhow::Result;
use clap::Parser;
use rust_eq_oracle::{RunResult, Status};
use sp1_sdk::{ProverClient, SP1Stdin};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

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
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Read the ELF file
    let elf_bytes = fs::read(&args.elf)?;

    // Read the input JSON
    let input_bytes = fs::read(&args.input)?;

    // Create SP1 stdin and write the input
    let mut stdin = SP1Stdin::new();
    stdin.write(&input_bytes);

    // Create the prover client
    let client = ProverClient::from_env();

    // Execute (not prove) the program and measure time
    let start = Instant::now();
    let (mut public_values, report) = client
        .execute(&elf_bytes, &stdin)
        .run()
        .map_err(|e| anyhow::anyhow!("SP1 execution failed: {}", e))?;
    let elapsed = start.elapsed();

    // Extract commits from public values
    // The guest commits in order: n, a, b
    let n: u32 = public_values.read();
    let a: u32 = public_values.read();
    let b: u32 = public_values.read();

    let commits = vec![
        serde_json::to_value(&n)?,
        serde_json::to_value(&a)?,
        serde_json::to_value(&b)?,
    ];

    // Create RunResult
    let result = RunResult {
        status: Status::Ok,
        elapsed_ms: elapsed.as_millis(),
        commits,
        meta: serde_json::json!({
            "runner": "sp1",
            "mode": "execute",
            "cycles": report.total_instruction_count(),
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

