//! SP1 guest adapter for arithmetic core
//! 
//! This adapter wraps the plain Rust arithmetic core with SP1's I/O layer.
//! It reads input from SP1's stdin, runs the core, and commits outputs.

#![no_main]
sp1_zkvm::entrypoint!(main);

use arithmetic_core::{ArithmeticInput, run};

pub fn main() {
    // 1. Read JSON input from SP1 I/O
    let input_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    let input: ArithmeticInput = serde_json::from_slice(&input_bytes)
        .expect("Failed to deserialize ArithmeticInput");

    // 2. Run the plain Rust core (zkVM-agnostic business logic)
    let output = run(input);

    // 3. Commit outputs in order (matching native runner)
    sp1_zkvm::io::commit(&output.result);
    
    // Commit bool as u32: 0 for false, 1 for true
    let overflowed_u32 = if output.overflowed { 1u32 } else { 0u32 };
    sp1_zkvm::io::commit(&overflowed_u32);
}

