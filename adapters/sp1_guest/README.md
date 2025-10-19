# SP1 Guest Adapters

This directory contains **one-way adapters** that wrap plain Rust cores into SP1 guest shape.

## Purpose

Transform zkVM-agnostic cores from `guest/cores/` into valid SP1 guest programs with:
- `#![no_main]`
- `sp1_zkVM::entrypoint!(main)`
- `sp1_zkVM::io::{read, commit}` for I/O

## Example Adapter

For a core `guest/cores/fib.rs` with signature:
```rust
pub fn run(input: FibInput) -> FibOutput
```

The adapter `adapters/sp1_guest/fib_guest.rs` would be:
```rust
#![no_main]
sp1_zkVM::entrypoint!(main);

use fib_core::{FibInput, run};

pub fn main() {
    // 1. Read input from SP1 I/O
    let input_bytes = sp1_zkVM::io::read::<Vec<u8>>();
    let input: FibInput = serde_json::from_slice(&input_bytes)
        .expect("Failed to parse input");
    
    // 2. Run the core logic
    let output = run(input);
    
    // 3. Commit outputs in order
    sp1_zkVM::io::commit(&output.n);
    sp1_zkVM::io::commit(&output.a);
    sp1_zkVM::io::commit(&output.b);
}
```

## Why One-Way?

We don't need a "reverse adapter" (SP1 → native) because:
- Core logic is already plain Rust
- Native runner calls the core directly
- Only SP1 needs the wrapper layer

## Phase Schedule

- **Phase 1**: Manual adapter for fibonacci
- **Phase 3+**: Consider templating/codegen for common patterns

