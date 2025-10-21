# Panic Test Core

**Purpose**: Validate panic capture in both native and SP1 runners.

## Design

This core can be configured to either:
1. **Panic** with a custom message
2. **Succeed** and return normally

## Input Format

```json
{
  "should_panic": false,
  "panic_msg": null
}
```

### Fields
- `should_panic` (bool): Whether to trigger a panic
- `panic_msg` (Option<String>): Custom panic message (if `should_panic` is true)

## Output Format

If the core does **not** panic, it commits two `u32` values:

```rust
pub struct PanicOutput {
    pub should_panic_u32: u32,  // 0 for false, 1 for true
    pub status_code: u32,       // 0 for success
}
```

### Commit Order (SP1)
1. `should_panic_u32` (0 or 1)
2. `status_code` (always 0 if no panic)

## Usage

### Test Case 1: No Panic (Expected: Pass)
```bash
make run CORE=guest/cores/panic_test INPUT=inputs/panic_no.json
```

**Expected Output**: Both runners succeed, commits match.

### Test Case 2: Panic (Expected: Pass)
```bash
make run CORE=guest/cores/panic_test INPUT=inputs/panic_yes.json
```

**Expected Output**: Both runners panic with captured messages, status matches.

## Implementation Notes

### Native Runner
- Uses `std::panic::catch_unwind` to capture panics
- Extracts panic message using `extract_panic_message` helper
- Returns `Status::Panic` with `meta.panic_msg`

### SP1 Runner
- SP1 execution errors are caught and mapped to `Status::Panic`
- Error message is stored in `meta.panic_msg`
- Note: SP1's panic message may differ from native (e.g., "execution failed with exit code 1")

## Phase Context

**Phase 2**: This core is part of the observability enhancements to ensure both runners correctly capture and compare panic conditions.

