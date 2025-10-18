# Source Mutators (A1/A2)

**Phase 5+**: Natural, conservative source-level mutations.

## Purpose

Apply "natural" transformations to Rust source code or RISC-V assembly to generate program variants for differential testing.

## Philosophy

**Natural-first** per meeting guidance:
- Prioritize mutations that could plausibly occur in real code
- Avoid overly synthetic transformations initially
- Instruction reordering is **not** a first-choice operator

## Mutation Operators

### Constants (Phase 5)
- Boundary values: `x → 0`, `x → max(T)`
- Small deltas: `x → x ± 1`, `x → x ± Δ` around boundaries
- Sign flips: `+x → -x`

### Booleans (Phase 5)
- Flip: `true ↔ false`
- Negate conditions: `x < y → x >= y`

### Branches (Phase 5)
- Swap arms: `if c { A } else { B } → if c { B } else { A }`
- Condition swaps: `if a && b → if a || b`

### Input Biasing (Phase 5)
- Vary `Vec<u8>` lengths to stress **capacity**:
  - `{0, 1, 2, 1024, 1MB, MAX_SIZE}`
- Per meeting: capacity scales with **guest-controlled data size**

## Configuration

```toml
[source_mut]
enable_constant_mutations = true
enable_boolean_mutations = true
enable_branch_mutations = true
enable_input_biasing = true

constant_boundary_prob = 0.3
constant_delta_max = 16
```

## Usage (Phase 5+)

```bash
# Mutate a single core
mutate --input guest/cores/fib.rs --output guest/cores/fib_mut1.rs --ops "const_boundary,bool_flip"

# Log mutations applied
mutate --log artifacts/mutations.json
```

## Phase Schedule

- **Phase 0-4**: Not implemented (stub directory)
- **Phase 5**: Natural operators (constants, booleans, branches, input biasing)
- **Phase 13**: Refine based on which operators are productive

## Output

Each mutation logs:
- Original code location
- Mutation operator applied
- Resulting code
- Seed for reproduction

