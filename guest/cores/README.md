# Guest Cores

This directory contains **plain Rust** implementations of guest programs.

## Design Principle

Guest cores are **ZKVM-agnostic** business logic:
- No `#![no_main]`
- No ZKVM-specific APIs (no `sp1_zkvm::`, `risc0_zkvm::`, etc.)
- Simple function signature: `fn run(input: Input) -> Output`

## Example Structure

```rust
// fib.rs
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct FibInput {
    pub n: u32,
}

#[derive(Serialize)]
pub struct FibOutput {
    pub n: u32,
    pub a: u32,
    pub b: u32,
}

pub fn run(input: FibInput) -> FibOutput {
    let mut a = 0u32;
    let mut b = 1u32;
    
    for _ in 0..input.n {
        let mut c = a + b;
        c %= 7919; // Modulus to prevent overflow
        a = b;
        b = c;
    }
    
    FibOutput {
        n: input.n,
        a,
        b,
    }
}
```

## How This Runs

1. **Native runner** calls `run()` directly with deserialized input
2. **ZKVM adapters** wrap this logic:
   - Read input via `zkvm::io::read()`
   - Call `run(input)`
   - Commit output fields via `zkvm::io::commit()`

This keeps the business logic portable across all ZKVMs.

## Phase Schedule

- **Phase 1**: Add fibonacci core
- **Phase 3**: Add I/O echo, arithmetic boundary, simple struct seeds
- **Phase 6+**: Auto-generate cores via RustSmith

