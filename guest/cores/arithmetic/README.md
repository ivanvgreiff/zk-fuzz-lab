# Arithmetic Core

**Purpose**: Test integer arithmetic boundary cases and overflow/underflow handling.

## Design

This core performs basic arithmetic operations (add, sub, mul, div) on u32 values and reports both the result and whether overflow occurred. This tests whether native Rust and SP1 RISC-V have matching integer semantics.

## Input Format

```json
{
  "a": 4294967295,
  "b": 1,
  "operation": "add"
}
```

### Fields
- `a` (u32): First operand
- `b` (u32): Second operand
- `operation` (String): One of "add", "sub", "mul", or "div"

## Output Format

The core commits two values:

```rust
pub struct ArithmeticOutput {
    pub result: u32,      // Result of the operation (wrapping)
    pub overflowed: bool, // Whether overflow occurred
}
```

### Commit Order (SP1)
1. `result` (u32)
2. `overflowed` (bool as u32: 0 for false, 1 for true)

## Usage

### Test Case 1: Normal Addition
```bash
make run CORE=guest/cores/arithmetic INPUT=inputs/arithmetic_add_normal.json
```

**Expected Output**: Both runners succeed, result=30, overflowed=false

### Test Case 2: Overflow Addition (u32::MAX + 1)
```bash
make run CORE=guest/cores/arithmetic INPUT=inputs/arithmetic_add_overflow.json
```

**Expected Output**: Both runners succeed, result=0 (wrapping), overflowed=true

### Test Case 3: Underflow Subtraction (0 - 1)
```bash
make run CORE=guest/cores/arithmetic INPUT=inputs/arithmetic_sub_underflow.json
```

**Expected Output**: Both runners succeed, result=4294967295 (wrapping), overflowed=true

### Test Case 4: Division by Zero
```bash
make run CORE=guest/cores/arithmetic INPUT=inputs/arithmetic_div_by_zero.json
```

**Expected Output**: Both runners panic with "Division by zero"

## Target Vulnerabilities

### Integer Overflow/Underflow Semantics
Rust uses wrapping arithmetic by default for release builds. SP1's RISC-V implementation must match this behavior.

**Attack Vectors**:
- `u32::MAX + 1` → should wrap to 0
- `0 - 1` → should wrap to u32::MAX
- `65536 * 65536` → should wrap to 0

### Division Behavior
Division by zero should panic consistently across both platforms.

### Potential Divergences
- **Overflow traps**: RISC-V might trap on overflow where Rust wraps
- **Sign extension**: Treating u32 as i32 internally
- **Panic handling**: Different error messages or exit codes

## Implementation Notes

### Why overflowing_* Methods?
We use `overflowing_add`, `overflowing_sub`, `overflowing_mul` to:
1. Get the wrapped result
2. Know if overflow occurred
3. Test both values are consistent

### Boundary Values Tested
- `0` (minimum)
- `1` (minimal non-zero)
- `u32::MAX` (4,294,967,295)
- `65536` (2^16, for mul overflow)

## Phase Context

**Phase 3**: This core is part of the seed corpus that exercises arithmetic edge cases before mutation in Phase 5.

