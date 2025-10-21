#![no_main]
sp1_zkvm::entrypoint!(main);

use panic_test_core::{PanicInput, run};

pub fn main() {
    // Read input bytes from SP1 I/O
    let input_bytes = sp1_zkvm::io::read::<Vec<u8>>();
    
    // Deserialize input
    let input: PanicInput = serde_json::from_slice(&input_bytes)
        .expect("Failed to deserialize PanicInput");
    
    // Run the core (may panic)
    let output = run(input);
    
    // Commit outputs in order
    sp1_zkvm::io::commit(&output.should_panic_u32);
    sp1_zkvm::io::commit(&output.status_code);
}

