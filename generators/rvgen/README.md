# RISC-V Generator (A2)

**Phase 10+**: RISC-V program generation for lower-level differential testing.

## Purpose

Generate RISC-V assembly programs to explore more degrees of freedom than Rust-level testing (A1).

## Approach

- Generate RV32I/RV32IM instruction sequences
- Build matching input data
- Compare emulator execution vs zkVM execution
- Use `oracles/riscv_eq` for state comparison

## Why A2 After A1?

- A1 gives fast feedback across multiple zkVMs
- A2 provides deeper control over instruction-level behavior
- Some bugs A1 finds, A2 might not (and vice versa)
- Or, all bugs A1 finds should be findable via A2
- The relationship is still being understood

## Phase Schedule

- **Phase 0-9**: Not implemented (stub directory)
- **Phase 10**: Initial RV32 instruction mixer + input builder
- **Phase 11+**: Refinement based on A1 learnings

## Relation to A1

- Reuse `mutators/source_mut/` where possible
- Keep "natural-first" philosophy
- Instruction reordering is a later-stage operator (not first)

