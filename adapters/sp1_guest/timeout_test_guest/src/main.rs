#![no_main]
sp1_zkvm::entrypoint!(main);

use timeout_test_core::{TimeoutInput, run};

pub fn main() {
    // Read input bytes from SP1 I/O
    let input_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    
    // Deserialize input
    let input: TimeoutInput = serde_json::from_slice(&input_bytes)
        .expect("Failed to deserialize TimeoutInput");
    
    // Run the core (may timeout if iterations == 0)
    let output = run(input);
    
    // Commit output
    sp1_zkvm::io::commit(&output.completed);
}

