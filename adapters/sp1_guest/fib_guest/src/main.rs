//! SP1 guest adapter for the fibonacci core
//!
//! This wraps the plain Rust fibonacci core with SP1's I/O interface.
//! The core business logic remains in `fib-core`, keeping it ZKVM-agnostic.

#![no_main]
sp1_zkvm::entrypoint!(main);

use fib_core::{FibInput, run};

pub fn main() {
    // 1. Read input from SP1 I/O (as JSON bytes)
    let input_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    
    // 2. Deserialize into FibInput
    let input: FibInput = serde_json::from_slice(&input_bytes)
        .expect("Failed to deserialize FibInput from JSON");
    
    // 3. Run the core business logic (ZKVM-agnostic)
    let output = run(input);
    
    // 4. Commit outputs in order (matching the commit stream format)
    //    This must match exactly what the native runner outputs
    sp1_zkvm::io::commit(&output.n);
    sp1_zkvm::io::commit(&output.a);
    sp1_zkvm::io::commit(&output.b);
}

