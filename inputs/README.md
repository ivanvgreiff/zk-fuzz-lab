# Inputs

Deterministic input corpora for differential testing.

## Purpose

Store JSON input files that are fed **identically** to both native and zkVM runners.

## Format

Each input file is JSON that can be deserialized into the core's `Input` type.

### Example: `fib_24.json`
```json
{
  "n": 24
}
```

### Example: `io_echo_large.json`
```json
{
  "data": [0, 1, 2, 3, ..., 255]
}
```

## Naming Convention

```
<core_name>_<variant>.json

Examples:
- fib_0.json          (edge case: n=0)
- fib_1.json          (minimal)
- fib_24.json         (small)
- fib_100.json        (medium)
- io_echo_empty.json  (empty input)
- io_echo_1kb.json    (1KB of data)
- io_echo_1mb.json    (1MB of data, for testing the `ptr + capacity > MAX_MEMORY` bug)
```

## Determinism

- **Phase 1-3**: All inputs are **fixed** and **versioned** (stored in this directory)
- **Phase 5**: Input mutations are **generated on-the-fly** (deterministic, not stored in git)
- **Phase 6+**: Randomized inputs will have **RNG seed** logged for reproducibility
- This ensures every run is **reproducible**

## Input Mutations (Phase 5)

**Generated vs Stored**: Phase 5 introduces `make fuzz` which generates ~93 input mutations on-the-fly:
- **Base inputs** (this directory): Used as starting point for mutations
- **Generated mutations** (`artifacts/mutations/`): Created during fuzzing, not in git
- **Mutation strategies**: Length biasing, boundary values, string variations, etc.

Example: `io_echo` generates 32 sizes (0b → 1MB) from base `io_echo_1kb.json`

**Why not store mutations?**: Would bloat git with ~93 JSON files. Instead, we generate deterministically from base inputs.

## Available Inputs

### Phase 1
- `fib_24.json` - Fibonacci with n=24

### Phase 2
- `panic_no.json` - Panic test configured to succeed
- `panic_yes.json` - Panic test configured to panic with custom message
- `timeout_finite.json` - Timeout test with 1M iterations (completes quickly)
- `timeout_infinite.json` - Timeout test with infinite loop (triggers timeout)

### Phase 3
**I/O Echo (3 inputs)**
- `io_echo_empty.json` - Empty data (0 bytes)
- `io_echo_small.json` - Small data sample (10 bytes, 0-9)
- `io_echo_1kb.json` - 1KB of data (1024 bytes, 0-255 pattern repeated 4x)

**Arithmetic (8 inputs)**
- `arithmetic_add_normal.json` - Normal addition (10 + 20)
- `arithmetic_add_overflow.json` - Overflow addition (u32::MAX + 1)
- `arithmetic_sub_normal.json` - Normal subtraction (20 - 10)
- `arithmetic_sub_underflow.json` - Underflow subtraction (0 - 1)
- `arithmetic_mul_normal.json` - Normal multiplication (10 * 20)
- `arithmetic_mul_overflow.json` - Overflow multiplication (65536 * 65536)
- `arithmetic_div_normal.json` - Normal division (20 / 10)
- `arithmetic_div_by_zero.json` - Division by zero (triggers panic)

**Simple Struct (4 inputs)**
- `simple_struct_normal.json` - Normal struct (42, "hello", true)
- `simple_struct_empty.json` - Empty string field (0, "", false)
- `simple_struct_unicode.json` - Unicode string (1, "🦀 Rust", true)
- `simple_struct_long.json` - Long string (99, "a"*1000, false)

## Phase Schedule

- **Phase 1**: Manual inputs for fibonacci ✅
- **Phase 2**: Inputs for panic and timeout testing ✅
- **Phase 3**: Inputs for I/O echo, arithmetic, struct seeds ✅
- **Phase 5**: On-the-fly input generation (mutations from base inputs) ✅
- **Phase 6**: Auto-generated inputs from RustSmith programs

