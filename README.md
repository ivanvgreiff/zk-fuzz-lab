# zk-fuzz-lab

A zkVM-agnostic differential fuzzing framework for finding bugs in zero-knowledge virtual machines.

## Three-Pronged Approach

This framework implements three complementary fuzzing strategies:

### A1: Rust-Level Differential (Current Focus)
Build a harness that compiles **the same Rust guest program** to:
1. **Native** (standard Rust compilation)
2. **Target zkVM** (starting with SP1)

Then run both with **identical deterministic inputs** and **diff** the results to surface divergences.

**What We Compare:**
- **Primary**: Commit stream equality (sequence of values committed by the program)
- **Secondary**: Status (OK | PANIC | TIMEOUT) and execution timing
- **Tertiary** (later phases): Allocator/ABI hints (pointer addresses, vector lengths)

### A2: RISC-V Level Differential (Phase 10+)
Compare execution at the **RISC-V instruction level**:
- Generate/mutate RISC-V programs directly
- Run through emulator vs zkVM
- Compare final CPU state (registers, memory, PC)
- More degrees of freedom than Rust-level

### A3: zkVM-Specific Invariant Testing (Phase 11+)
Test zkVM-specific properties:
- Proof/witness mutations (malformed proofs should fail verification)
- Serialization/deserialization edge cases
- VK root consistency
- Public value tampering

### Design Principles

- **zkVM-agnostic**: Plain Rust core logic, thin adapters per zkVM
- **Deterministic**: Single JSON input source, logged RNG seeds
- **Observable**: Rich logging of outputs, panics, timeouts, and timing
- **Reproducible**: Every divergence saved with repro script

## Repository Structure

```
guest/
  cores/              # Plain Rust business logic (zkVM-agnostic)
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
  sp1/                # Builds and runs via SP1 zkVM
oracles/
  rust_eq/            # A1: Compares native vs zkVM outputs
  riscv_eq/           # A2: Compares emulator vs zkVM state
  spec_violations/    # A3: zkVM-specific invariant checks
harness/              # Orchestrates runs, diffing, and logging
inputs/               # Deterministic input corpora (JSON)
artifacts/            # Crashes, divergences, repros, logs
ci/                   # Smoke tests and nightly fuzzing runs
```

## Implementation Phases

### Completed
- **Phase 0** ✅ - Bootstrap & repo scaffold
- **Phase 1** ✅ - Walking skeleton (native+SP1 differential for fibonacci)

### A1: Rust-Level Differential (In Progress)
- **Phase 2** ⏳ - Observability (panic capture, timeout handling, repro scripts)
- **Phase 3** ⏳ - Seed programs (I/O echo, arithmetic, structs)
- **Phase 4** ⏳ - Logging schema (CSV exports, structured artifacts)
- **Phase 5** ⏳ - Mutators v0 (natural: constants, booleans, branches, input biasing)
- **Phase 6** ⏳ - RustSmith integration (randomized program generation)
- **Phase 7** ⏳ - A1 validation (attempt to rediscover 1 SP1 + 1 RISC Zero bug)

### Multi-ZKVM & A2/A3 Preparation
- **Phase 8** ⏳ - Portability hooks (Runner trait, support for RISC Zero/OpenVM)
- **Phase 9** ⏳ - Hygiene & comms (public repo redaction, documentation)
- **Phase 10** ⏳ - A2 scaffolding (RISC-V level differential)
- **Phase 11** ⏳ - A3 preparation (proof/witness mutation research)

### Infrastructure & Validation
- **Phase 12** ⏳ - CI/CD (PR smoke tests, nightly fuzzing runs)
- **Phase 13** ⏳ - Validation loop (track productive operators, tune based on findings)
- **Phase 14** ⏳ - Coordination (pairing sessions, reviews)

## Quick Start

```bash
# Run differential test
make run CORE=guest/cores/fib INPUT=inputs/fib_24.json

# Run smoke test (verify scaffold)
make smoke

# Build all packages
make build

# Run tests
make test
```

## Architecture Notes

See `context_from_beginning.md` for architectural decisions and justifications.

### Key Decisions

1. **Plain Rust cores** with thin zkVM adapters (not zkVM-first code)
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

