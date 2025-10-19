# RISC-V Equality Oracle (A2)

**Phase 10+**: Compares RISC-V emulator vs zkVM execution state.

## Purpose

For A2 (RISC-V-level fuzzing), compare:
- Final register state
- Final memory state
- (Later) Intermediate execution traces

## Comparison

```rust
pub struct RiscVState {
    pub registers: [u32; 32],  // x0-x31
    pub pc: u32,
    pub memory: Vec<(u32, u8)>, // (addr, byte) pairs
}

pub fn compare(emulator: &RiscVState, zkVM: &RiscVState) -> Diff {
    // Compare registers
    // Compare PC
    // Compare memory (may need approximate comparison for allocator differences)
}
```

## Emulator Choice

Options:
- [riscv-emu](https://github.com/gamozolabs/riscv-emu)
- [riscv-rs](https://github.com/takahirox/riscv-rust)
- Custom lightweight RV32I interpreter

## Intermediate State (Future)

Later phases may compare intermediate states (cycle-by-cycle), not just final state.

## Phase Schedule

- **Phase 0-9**: Not implemented (stub directory)
- **Phase 10**: Basic final-state comparison
- **Later**: Intermediate state comparison

## Relation to rust_eq

- **rust_eq** (A1): High-level Rust behavior
- **riscv_eq** (A2): Low-level instruction execution
- Both are zkVM-agnostic (compare against reference implementation)

