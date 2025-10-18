# zk-fuzz-lab

A ZKVM-agnostic differential fuzzing framework for finding bugs in zero-knowledge virtual machines.

## Current Goal: A1 (Rust-level Differential Fuzzing)

Build a harness that compiles **the same Rust guest program** to:
1. **Native** (standard Rust compilation)
2. **Target ZKVM** (starting with SP1)

Then run both with **identical deterministic inputs** and **diff** the results to surface divergences.

### What We Compare

- **Primary**: Commit stream equality (sequence of values committed by the program)
- **Secondary**: Status (OK | PANIC | TIMEOUT) and execution timing
- **Tertiary** (later phases): Allocator/ABI hints (pointer addresses, vector lengths)

### Design Principles

- **ZKVM-agnostic**: Plain Rust core logic, thin adapters per ZKVM
- **Deterministic**: Single JSON input source, logged RNG seeds
- **Observable**: Rich logging of outputs, panics, timeouts, and timing
- **Reproducible**: Every divergence saved with repro script

## Repository Structure

```
guest/
  cores/              # Plain Rust business logic (ZKVM-agnostic)
generators/
  rustgen/            # A1: RustSmith + templates (Phase 6)
  rvgen/              # A2: RISC-V program generator (Phase 10)
mutators/
  source_mut/         # A1/A2: Source-level mutations (Phase 5)
  trace_mut/          # A3: Witness/proof mutations (Phase 11)
adapters/
  sp1_guest/          # Wraps plain Rust cores into SP1 guest shape
runners/
  native/             # Builds and runs cores natively
  sp1/                # Builds and runs via SP1 ZKVM
oracles/
  rust_eq/            # A1: Compares native vs ZKVM outputs
  riscv_eq/           # A2: Compares emulator vs ZKVM state
  spec_violations/    # A3: ZKVM-specific invariant checks
harness/              # Orchestrates runs, diffing, and logging
inputs/               # Deterministic input corpora (JSON)
artifacts/            # Crashes, divergences, repros, logs
ci/                   # Smoke tests and nightly fuzzing runs
```

## Phase Status

- **Phase 0** ✅ - Repository scaffold created
- **Phase 1** 🔄 - Walking skeleton (next)
- **Phase 2+** ⏳ - Pending

## Quick Start (After Phase 1)

```bash
# Run a single seed program through both runners
make run A=guest/cores/fib.rs

# Run smoke test
make smoke
```

## Architecture Notes

See `context_from_beginning.md` for architectural decisions and justifications.

### Key Decisions

1. **Plain Rust cores** with thin ZKVM adapters (not ZKVM-first code)
2. **JSON inputs** consumed identically by both runners
3. **Commit-stream normalization** for output comparison
4. **Execute-only first**, prove+verify later (Phase 6+)
5. **Natural mutations first** (constants, branches), not instruction reordering

## Validation Strategy

Once A1 is stable (Phase 7), we will attempt to re-detect:
- 1 known SP1 bug
- 1 known RISC Zero bug

If they appear **immediately**, we may have bias. If they appear eventually or not at all, we document why and adjust.

## References

- **Implementation Plan**: `implementation_plan.md`
- **Meeting Context**: `implementation_plan_context.md`
- **Architecture Decisions**: `context_from_beginning.md`

