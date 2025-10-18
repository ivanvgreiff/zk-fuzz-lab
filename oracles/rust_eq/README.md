# Rust Equality Oracle (A1)

Compares **native** vs **ZKVM** execution results for Rust-level differential fuzzing.

## Purpose

Given two `RunResult` structs (one from native runner, one from SP1/ZKVM runner), determine if they are equivalent or diverged.

## Comparison Logic

```rust
pub struct RunResult {
    pub status: Status,          // OK | PANIC | TIMEOUT
    pub elapsed_ms: u128,
    pub commits: Vec<serde_json::Value>,
    pub meta: serde_json::Value,
}

pub struct Diff {
    pub equal: bool,
    pub reason: Option<String>,
}
```

### Comparison Steps

1. **Status check**: `native.status == sp1.status`
   - If mismatch → DIVERGED
   - Reason: "status mismatch: native=OK, sp1=PANIC"

2. **Commit stream check** (if both OK): `native.commits == sp1.commits`
   - If mismatch → DIVERGED
   - Reason: "commit stream mismatch: native=[24,46368] vs sp1=[24,46369]"

3. **Timing delta** (optional): `|native.elapsed_ms - sp1.elapsed_ms|`
   - Recorded for analysis
   - Not a failure condition (ZKVMs are slower)
   - Large deltas or TIMEOUT on one side are strong signals

## Output

```json
{
  "equal": false,
  "reason": "commit stream mismatch: native=[24,46368,75025] vs sp1=[24,46368,75026]",
  "timing_delta_ms": 140
}
```

## Phase Schedule

- **Phase 1**: Basic status + commit comparison
- **Phase 2**: Add panic message comparison, timeout handling
- **Phase 3**: Add tertiary signals (vec lengths, pointer addresses)
- **Phase 4**: Integrate with structured artifact logging

## Relation to Other Oracles

- **rust_eq** (A1): Compares Rust-level behavior (this oracle)
- **riscv_eq** (A2): Compares RISC-V state (emulator vs ZKVM)
- **spec_violations** (A3): Checks ZKVM-specific invariants

