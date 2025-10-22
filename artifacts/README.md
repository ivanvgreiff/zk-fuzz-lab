# Artifacts

Storage for crashes, divergences, repros, and logs from fuzzing campaigns.

## Purpose

Persistent record of all fuzzing runs, with special emphasis on:
- **Divergences**: Native vs zkVM disagreements
- **Panics**: Crashes in either runner
- **Timeouts**: Hanging executions
- **Repros**: Scripts to reproduce any finding

## Directory Structure (Phase 2+)

```
artifacts/
  summary.csv                       # High-level overview of all runs (Phase 2)
  20251021_040225_fib.json          # Run log (single runs)
  20251021_041600_timeout_test/     # Divergence subdirectory (Phase 2)
    input.json                      # Copy of input that triggered divergence
    run_log.json                    # Detailed run log
    repro.sh                        # Executable reproduction script
  mutations/                        # Phase 5: Fuzzing runs
    20251022_020440_fuzz_io_echo/   # Mutation campaign subdirectory
      plan.json                     # List of all mutations generated
      input_1.json                  # Each generated input
      input_2.json
      ...
      input_32.json
  reports/                          # Future: validation reports (Phase 7+)
    a1_pilot.md
    a1_round_1.md
```

### Phase 2 Implementation
- **Summary CSV**: Appended after every run for bulk analysis
- **Run Logs**: One JSON file per run at root (`<run_id>.json`)
- **Divergence Subdirectories**: Created only when `diff.equal == false`
- **Repro Scripts**: Shell scripts in divergence subdirectories, made executable on Unix

### Phase 5 Addition
- **Mutation Subdirectories**: `mutations/<timestamp>_fuzz_<core>/`
  - `plan.json`: All mutations generated for this campaign
  - `input_N.json`: Each mutated input (saved for future resume capability)
- **CSV Integration**: All mutations logged to `summary.csv` with populated mutation columns

## Summary CSV Schema (Phase 4)

```csv
run_id,core,input,native_status,sp1_status,equal,reason,elapsed_native_ms,elapsed_sp1_ms,timing_delta_ms,repro_path,generator,base_seed,mutation_ops,rng_seed,zkvm_target,sp1_version,rustc_version
20251021_040225_fib,fib,inputs/fib_24.json,Ok,Ok,true,,0,31,31,,hand_written,,,,,sp1,cargo-prove-cli 3.3.0,rustc 1.82.0
20251021_041600_timeout_test,timeout_test,inputs/timeout_infinite.json,Timeout,Timeout,true,,30000,30000,0,,hand_written,,,,,sp1,cargo-prove-cli 3.3.0,rustc 1.82.0
20251021_041009_panic_test,panic_test,inputs/panic_no.json,Ok,Ok,false,"commit stream mismatch: ...",0,40,40,artifacts/20251021_041009_panic_test/,hand_written,,,,,sp1,cargo-prove-cli 3.3.0,rustc 1.82.0
```

### Columns (Phase 2 + Phase 4)

**Core Columns** (Phase 2):
- `run_id`: Unique identifier (timestamp + core name)
- `core`: Core name (e.g., "fib", "panic_test")
- `input`: Input file path
- `native_status`: Native runner status (Ok | Panic | Timeout)
- `sp1_status`: SP1 runner status (Ok | Panic | Timeout)
- `equal`: Whether results match (true | false)
- `reason`: Reason for mismatch (empty if equal=true)
- `elapsed_native_ms`: Native execution time in milliseconds
- `elapsed_sp1_ms`: SP1 execution time in milliseconds
- `timing_delta_ms`: Absolute timing difference

**Future-Proofing Columns** (Phase 4-5):
- `repro_path`: Path to divergence folder (empty if no divergence), e.g., "artifacts/20251021_041009_panic_test/"
- `generator`: Program source ("hand_written" for P1-3, "mutated" for P5, "rustsmith" for P6)
- `base_seed`: For mutations, the original input (e.g., "inputs/io_echo_1kb.json" in P5)
- `mutation_ops`: Mutation description (e.g., "length_bias:256kb" in P5)
- `rng_seed`: Random seed for reproducibility (empty for deterministic P5, populated in P6)
- `zkvm_target`: Target zkVM ("sp1" for P1-5, "risc0"/"openvm" in P8)
- `sp1_version`: SP1 toolchain version for reproducibility
- `rustc_version`: Rust compiler version for reproducibility

**Phase 5 Example Row**:
```csv
20251022_021805_io_echo,io_echo,artifacts/mutations/20251022_020440_fuzz_io_echo/input_30.json,Ok,Ok,true,,5,4415,4410,,mutated,inputs/io_echo_1kb.json,length_bias:256kb,,sp1,cargo-prove sp1 (bb91c6f),rustc 1.90.0
```

## Triage Workflow (Phase 2+)

1. Check `summary.csv` for divergences (`equal=false`)
2. Navigate to `<run_id>/` subdirectory
3. Review `run_log.json` for detailed comparison
4. Run `repro.sh` to reproduce locally
5. Minimize the input (manual or auto - Phase 5+)
6. Verify it's a real bug (not fuzzer false positive)

## Retention Policy

- Keep all divergences indefinitely
- Keep PASSes for 7 days (or until disk space needed)
- Archive reports permanently

## Phase Schedule

- **Phase 1**: Basic JSON logs per run ✅
- **Phase 2**: Repro scripts + CSV summary + divergence subdirectories ✅
- **Phase 4**: Enhanced CSV schema with future-proofing columns ✅
- **Phase 5**: Mutation campaign subdirectories + populated mutation columns ✅
- **Phase 7**: Validation reports
- **Phase 13**: Iteration reports

