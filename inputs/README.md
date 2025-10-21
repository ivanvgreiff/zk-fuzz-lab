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

- All inputs are **fixed** and **versioned**
- If randomization is used (Phase 5+), the **RNG seed** is stored in the input file or logged separately
- This ensures every run is **reproducible**

## Capacity Biasing (Phase 5+)

With respect to the SP1 allocator overflow bug, the `capacity` variable scales with **guest-controlled data size**. We bias input sizes to stress capacity:

- `io_echo_0.json`: 0 bytes
- `io_echo_1.json`: 1 byte
- `io_echo_1kb.json`: 1024 bytes
- `io_echo_1mb.json`: 1 MB
- `io_echo_max.json`: Maximum safe size

## Available Inputs

### Phase 1
- `fib_24.json` - Fibonacci with n=24

### Phase 2
- `panic_no.json` - Panic test configured to succeed
- `panic_yes.json` - Panic test configured to panic with custom message
- `timeout_finite.json` - Timeout test with 1M iterations (completes quickly)
- `timeout_infinite.json` - Timeout test with infinite loop (triggers timeout)

### Phase 3+ (Planned)
- I/O echo inputs (various sizes)
- Arithmetic boundary inputs
- Struct seed inputs

## Phase Schedule

- **Phase 1**: Manual inputs for fibonacci
- **Phase 2**: Inputs for panic and timeout testing
- **Phase 3**: Inputs for I/O echo, arithmetic, struct seeds
- **Phase 5**: Size-biased inputs for capacity exploration
- **Phase 6**: Auto-generated inputs from RustSmith

