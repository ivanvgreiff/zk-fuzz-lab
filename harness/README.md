# Harness

Orchestrates differential testing runs: inputs → runners → oracles → artifacts.

## Purpose

The harness is the main entry point that:
1. Loads input from `inputs/` (JSON)
2. Invokes both **native** and **zkVM** runners
3. Calls the appropriate oracle (e.g., `rust_eq`) to diff results
4. Logs outcomes to `artifacts/`
5. Generates repro scripts for divergences

## Architecture

```
┌─────────┐
│ Harness │
└────┬────┘
     │
     ├──→ runners/native/ ──→ RunResult₁
     │
     ├──→ runners/sp1/    ──→ RunResult₂
     │
     └──→ oracles/rust_eq/
           compare(RunResult₁, RunResult₂) → Diff
           ├─→ artifacts/run_<id>.json
           └─→ artifacts/repro_<id>.sh (if diverged)
```

## CLI Interface (Phase 1+)

```bash
# Run single seed
harness run --core guest/cores/fib.rs --input inputs/fib_24.json

# Run batch of seeds
harness batch --seeds guest/cores/*.rs

# Fuzzing mode (Phase 5+)
harness fuzz --generator rustsmith --mutator source_mut --duration 1h
```

## Output Artifacts

### Run Log (`artifacts/run_<timestamp>.json`)
```json
{
  "run_id": "20241018_143022_fib",
  "seed_path": "guest/cores/fib.rs",
  "input_path": "inputs/fib_24.json",
  "native_result": { "status": "OK", "elapsed_ms": 2, "commits": [...] },
  "sp1_result": { "status": "OK", "elapsed_ms": 142, "commits": [...] },
  "diff": { "equal": true, "reason": null }
}
```

### Repro Script (`artifacts/repro_<run_id>.sh`)
Generated only on divergence:
```bash
#!/bin/bash
# Repro for divergence found at 2024-10-18 14:30:22
harness run --core guest/cores/fib.rs --input inputs/fib_24.json
```

## CSV Summary (`artifacts/summary.csv`) - Phase 4

Every run appends a row with 18 columns:

**Core Columns**:
- `run_id`, `core`, `input`
- `native_status`, `sp1_status`, `equal`, `reason`
- `elapsed_native_ms`, `elapsed_sp1_ms`, `timing_delta_ms`

**Future-Proofing Columns** (Phase 4):
- `repro_path` - Direct link to divergence folder
- `generator` - "hand_written", "mutated" (P5), "rustsmith" (P6)
- `base_seed` - Original seed for mutations (P5)
- `mutation_ops` - Comma-separated operators (P5)
- `rng_seed` - For reproducibility (P6)
- `zkvm_target` - "sp1", "risc0", "openvm" (P8)
- `sp1_version` - zkVM version tracking
- `rustc_version` - Compiler version tracking

See `artifacts/README.md` for full schema documentation.

## Phase Schedule

- **Phase 1**: Basic CLI that runs native+sp1+diff ✅
- **Phase 2**: Add repro script generation + CSV logging ✅
- **Phase 4**: Enhanced CSV schema (18 columns with future-proofing) ✅
- **Phase 5**: Integration with mutators
- **Phase 6**: Integration with generators

