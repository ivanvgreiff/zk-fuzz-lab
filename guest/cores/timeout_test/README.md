# Timeout Test Core

**Purpose**: Validate timeout handling in both native and SP1 runners.

## Design

This core can be configured to either:
1. **Run for a finite number of iterations** (completes quickly)
2. **Run indefinitely** (triggers timeout)

## Input Format

```json
{
  "iterations": 1000000
}
```

### Fields
- `iterations` (u64): Number of loop iterations
  - `> 0`: Finite loop (completes normally)
  - `0`: Infinite loop (triggers timeout)

## Output Format

If the core completes before timeout, it commits one boolean:

```rust
pub struct TimeoutOutput {
    pub completed: bool,  // Always true if no timeout
}
```

### Commit Order (SP1)
1. `completed` (boolean, always true)

## Usage

### Test Case 1: Finite Loop (Expected: Pass)
```bash
make run CORE=guest/cores/timeout_test INPUT=inputs/timeout_finite.json
```

**Expected Output**: Both runners complete in <1s, commits match.

### Test Case 2: Infinite Loop (Expected: Match on Timeout)
```bash
make run CORE=guest/cores/timeout_test INPUT=inputs/timeout_infinite.json
```

**Expected Output**: Both runners timeout after 30s, status matches (`TIMEOUT`).

## Timeout Configuration

Default timeout: **30 seconds** (configurable via CLI flag)

### Native Runner
```bash
native-runner --core timeout_test --input inputs/timeout_infinite.json --timeout 10
```

### SP1 Runner
```bash
sp1-runner --elf adapters/sp1_guest/timeout_test_guest/target/elf-compilation/riscv32im-succinct-zkvm-elf/release/timeout-test-guest --input inputs/timeout_infinite.json --timeout 10
```

## Implementation Notes

### Native Runner
- Spawns core execution in a separate thread
- Uses `mpsc::channel` with `recv_timeout` to enforce timeout
- Returns `Status::Timeout` with `meta.timeout_secs` on timeout

### SP1 Runner
- Spawns SP1 execution in a separate thread
- Uses `mpsc::channel` with `recv_timeout` to enforce timeout
- Returns `Status::Timeout` with `meta.timeout_secs` on timeout

### Determinism Note

Timeouts are inherently **non-deterministic** at the millisecond level due to:
- Thread scheduling
- System load
- OS jitter

The oracle compares **status** (OK vs TIMEOUT), not exact timing, so both runners timing out is considered a **match**.

## Phase Context

**Phase 2**: This core is part of the observability enhancements to ensure both runners correctly handle infinite loops and timeout conditions.

