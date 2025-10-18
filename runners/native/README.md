# Native Runner

Builds and executes guest cores as **native Rust binaries**.

## Purpose

Run the plain Rust core logic from `guest/cores/` directly, without any ZKVM:
1. Read input from JSON file
2. Deserialize into `Input` type
3. Call `core::run(input)`
4. Serialize output into commit-stream format
5. Capture status (OK | PANIC | TIMEOUT) and timing

## Output Format

The native runner produces a `RunResult` JSON:

```json
{
  "status": "OK",
  "elapsed_ms": 2,
  "commits": [24, 46368, 75025],
  "meta": {}
}
```

### Status Values
- `OK`: Completed successfully
- `PANIC`: Panicked with error message in `meta.panic_msg`
- `TIMEOUT`: Exceeded time limit

### Commits Array
Must match exactly what the SP1 guest commits, in the same order.

## Phase Schedule

- **Phase 1**: Basic runner that executes cores and captures output
- **Phase 2**: Add panic message capture and timeout handling
- **Phase 4**: Integrate with structured logging to `artifacts/`

