//! SP1 guest adapter for io_echo core
//! 
//! This adapter wraps the plain Rust io_echo core with SP1's I/O layer.
//! It reads input from SP1's stdin, runs the core, and commits outputs.

#![no_main]
sp1_zkvm::entrypoint!(main);

use io_echo_core::{IoEchoInput, run};

pub fn main() {
    // 1. Read JSON input from SP1 I/O
    let input_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    let input: IoEchoInput = serde_json::from_slice(&input_bytes)
        .expect("Failed to deserialize IoEchoInput");

    // 2. Run the plain Rust core (zkVM-agnostic business logic)
    let output = run(input);

    // 3. Commit outputs in order (matching native runner)
    sp1_zkvm::io::commit(&output.length);
    
    // Commit Option<u8> as u32: 0 for None, 1+value for Some(value)
    match output.first_byte {
        None => sp1_zkvm::io::commit(&0u32),
        Some(byte) => sp1_zkvm::io::commit(&(1u32 + byte as u32)),
    }
    
    match output.last_byte {
        None => sp1_zkvm::io::commit(&0u32),
        Some(byte) => sp1_zkvm::io::commit(&(1u32 + byte as u32)),
    }
}

