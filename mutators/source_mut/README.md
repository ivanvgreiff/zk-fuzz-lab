# Source Mutators (A1/A2)

**Phase 5**: Input mutation engine for systematic exploration of input space.

## Purpose

Generate systematic input variations for existing cores to explore boundary conditions, capacity limits, and edge cases. This provides a bridge to Phase 6 (RustSmith) by building the input generation infrastructure.

## Philosophy

**Deterministic and systematic**:
- Generate predictable mutations (same base → same mutations)
- Cover important boundaries and edge cases
- Prepare infrastructure for random generation later

**Note**: Randomness can be added in future phases by populating the `rng_seed` column.

## Phase 5 Implementation: Input Mutation Engine

**Current Focus**: Input mutations only (not source code mutations)

**Rationale**:
- Simpler to implement than AST-level mutations
- Directly addresses input space exploration
- Reusable for Phase 6 (RustSmith programs need inputs too!)
- Source-level mutations deferred to Phase 5.1 or later

## Mutation Strategies (Per Core)

### `io_echo` - Length Biasing (32 mutations)
**Strategy**: Hybrid approach
- **Powers of 2**: {0, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1KB, 2KB, 4KB, 8KB, 16KB, 32KB, 64KB, 128KB, 256KB, 512KB, 1MB}
- **Boundaries**: {127, 255, 1023, 4095, 65535}
- **Edge cases**: {3, 7, 15, 31, 63}
- **Max size**: 1MB (practical limit for testing)
- **Purpose**: Explore allocator capacity limits

### `arithmetic` - Boundary Values (~24 mutations)
**Strategy**: Boundary value testing
- **Values**: {0, 1, 2, u32::MAX/2, u32::MAX-1, u32::MAX}
- **Operations**: add, sub, mul, div
- **Purpose**: Test overflow/underflow handling

### `simple_struct` - String Variations (10 mutations)
**Strategy**: String edge cases
- **Lengths**: {0, 1, "hello", 100 chars, 1000 chars, 10000 chars}
- **Special**: Unicode (🦀), newlines, tabs
- **Purpose**: Test string encoding and serialization

### `fib` - Fibonacci Values (11 mutations)
**Strategy**: Representative n values
- **Values**: {0, 1, 2, 5, 10, 20, 30, 40, 50, 100, 1000}
- **Purpose**: Edge cases and various computation depths

### `panic_test` - Boolean Variations (4 mutations)
**Strategy**: Boolean combinations
- **Values**: {true, false} with various messages
- **Purpose**: Exercise panic paths

### `timeout_test` - Iteration Variations (9 mutations)
**Strategy**: Exponential iteration counts
- **Values**: {0, 1, 10, 100, 1K, 10K, 100K, 1M, 10M}
- **Purpose**: Timeout boundary exploration

**Total**: ~90 mutations across all 6 cores

## Usage (Phase 5)

### Fuzz a Single Core
```bash
make fuzz CORE=io_echo
```

**Output**: Tests 32 input variations, logs results to CSV

### Fuzz Multiple Cores
```bash
make fuzz CORE=io_echo,arithmetic
```

### Fuzz All Cores
```bash
make fuzz CORE=all
```

**Output**: Tests ~90 mutations across all 6 cores (~45 minutes)

### Direct Harness Usage
```bash
cargo run --bin harness -- fuzz --cores io_echo
cargo run --bin harness -- fuzz --cores "io_echo,arithmetic"
cargo run --bin harness -- fuzz --cores all
```

## Output Format

```
🔄 Starting input mutation fuzzing...
   Cores: io_echo

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📦 Core: io_echo
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

   Base input: inputs/io_echo_1kb.json
   Generating mutations...
   ✅ Generated 32 mutations

   📊 Size Distribution:
      Min: 0 bytes
      Max: 1048576 bytes (1.00 MB)
      Total sizes: 32

   🧪 Testing mutations...

   ✅ Mutation 1/32: length_bias:0b | Native: Ok (0ms) | SP1: Ok (36ms) | Equal: true
   ...
   ✅ Mutation 32/32: length_bias:1mb | Native: Ok (21ms) | SP1: Ok (17975ms) | Equal: true

   📊 Timing Statistics:
      Native: avg 1.2ms, max 21ms
      SP1: avg 1150.5ms, max 17975ms

   ✅ Core 'io_echo' fuzzing complete!
      Total: 32
      Passed: 32 (100.0%)
      Divergences: 0

🎯 Fuzzing Complete!
   Total mutations: 32
   Passed: 32 (100.0%)
   Total time: 959.5s

💾 All results logged to artifacts/summary.csv
```

## Artifacts Generated

### Mutation Plan
**Location**: `artifacts/mutations/<timestamp>_fuzz_<core>/plan.json`

```json
[
  {"mutation_op": "length_bias:0b", "base": "inputs/io_echo_1kb.json"},
  {"mutation_op": "length_bias:1b", "base": "inputs/io_echo_1kb.json"},
  ...
]
```

### Mutated Inputs
**Location**: `artifacts/mutations/<timestamp>_fuzz_<core>/input_N.json`

Saved for all mutations during fuzzing (enables resume capability in future).

### CSV Summary
Every mutation logged to `artifacts/summary.csv` with:
- `generator=mutated`
- `base_seed=inputs/<base>.json`
- `mutation_ops=<strategy>:<details>`
- `rng_seed=` (empty for deterministic)

## Phase Schedule

- **Phase 0-4**: Not implemented (stub directory)
- **Phase 5**: ✅ Input mutation engine (deterministic, systematic)
- **Phase 5.1**: (Future) Source-level AST mutations
- **Phase 6**: RustSmith integration (uses this input generator)
- **Phase 13**: Refine based on productive operators

## Future Enhancements

### Randomness (Phase 5.1+)
- Add RNG-based mutation selection
- Populate `rng_seed` column for reproducibility
- Enable `--seed` flag for deterministic random runs

### Source Mutations (Phase 5.1+)
- AST-level mutations (constants, booleans, branches)
- Requires `syn` crate for parsing
- Deferred until RustSmith integration (Phase 6) informs need

### Resume Capability (Phase 5.1+)
- Resume from saved mutation plan
- Skip already-tested mutations
- Useful for long fuzzing campaigns

