# Fibonacci Core

Plain Rust implementation of the Fibonacci sequence calculation (modulo 7919).

## Purpose

This is the first **walking skeleton** core for Phase 1, demonstrating the zkVM-agnostic design:
- No zkVM-specific APIs
- Simple function signature: `fn run(input: FibInput) -> FibOutput`
- Serializable input/output types

## Implementation

```rust
pub struct FibInput {
    pub n: u32,
}

pub struct FibOutput {
    pub n: u32,    // Echo the input
    pub a: u32,    // (n-1)th fibonacci number
    pub b: u32,    // nth fibonacci number
}

pub fn run(input: FibInput) -> FibOutput
```

The implementation:
1. Computes the nth Fibonacci number using iterative approach
2. Applies modulo 7919 to prevent overflow (same as SP1's example)
3. Returns both `fib(n-1)` and `fib(n)` as `a` and `b`

## Testing

### Unit Tests
```bash
cd guest/cores/fib
cargo test
```

### Differential Test
```bash
# From repo root
make run CORE=guest/cores/fib INPUT=inputs/fib_24.json
```

Expected output:
- Native: ~0ms, commits: `[24, 6773, 3754]`
- SP1: ~20-30ms, commits: `[24, 6773, 3754]`
- Oracle: ✅ PASS

## Commit Stream

The core commits three values in order:
1. `n` (input, echoed back)
2. `a` (fib(n-1) mod 7919)
3. `b` (fib(n) mod 7919)

Both native and SP1 runners must produce identical commit streams.

## Phase

- **Phase 1**: Initial implementation ✅
- **Phase 2+**: Used as baseline for mutation testing

