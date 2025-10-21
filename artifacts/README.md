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
  summary.csv                 # High-level overview of all runs (Phase 2)
  20251021_040225_fib.json    # Run log (all runs)
  20251021_041600_timeout_test.json
  20251021_041600_timeout_test/  # Divergence subdirectory (Phase 2)
    input.json                    # Copy of input that triggered divergence
    run_log.json                  # Detailed run log
    repro.sh                      # Executable reproduction script
  reports/                    # Future: validation reports (Phase 7+)
    a1_pilot.md
    a1_round_1.md
```

### Phase 2 Implementation
- **Summary CSV**: Appended after every run for bulk analysis
- **Run Logs**: One JSON file per run at root (`<run_id>.json`)
- **Divergence Subdirectories**: Created only when `diff.equal == false`
- **Repro Scripts**: Shell scripts in divergence subdirectories, made executable on Unix

## Summary CSV Schema (Phase 2)

```csv
run_id,core,input,native_status,sp1_status,equal,reason,elapsed_native_ms,elapsed_sp1_ms,timing_delta_ms
20251021_040225_fib,fib,inputs/fib_24.json,Ok,Ok,true,,0,31,31
20251021_041600_timeout_test,timeout_test,inputs/timeout_infinite.json,Timeout,Timeout,true,,30000,30000,0
```

### Columns (Phase 2)
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
- **Phase 4**: Enhanced structured logging (additional fields/formats)
- **Phase 7**: Validation reports
- **Phase 13**: Iteration reports

