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

## Available Adapters

### Phase 1
- **fib_guest** - Wraps `fib-core` for SP1 execution

### Phase 2
- **panic_test_guest** - Wraps `panic-test-core` for SP1 execution
- **timeout_test_guest** - Wraps `timeout-test-core` for SP1 execution

### Phase 3+ (Planned)
- I/O echo guest
- Arithmetic boundary guest
- Consider templating/codegen for common patterns

## Build Process

Each adapter is a **standalone Cargo workspace** to allow independent RISC-V compilation:

```bash
cd adapters/sp1_guest/fib_guest
cargo prove build
```

This generates an ELF binary at:
```
target/elf-compilation/riscv32im-succinct-zkvm-elf/release/fib-guest
```

## Phase Schedule

- **Phase 1**: Manual adapter for fibonacci
- **Phase 2**: Manual adapters for panic and timeout testing
- **Phase 3+**: Consider templating/codegen for common patterns

