# RustSmith Generator (A1)

**Phase 6+**: Randomized Rust program generation using RustSmith.

## Purpose

Generate diverse, valid Rust programs for differential testing:
- Random but legal Rust syntax
- Configurable complexity (function count, loop bounds, etc.)
- Reproducible via RNG seed

## RustSmith Integration

[RustSmith](https://github.com/cbeuw/rustsmith) (Donaldson et al.) is a random Rust program generator similar to Csmith for C.

### Configuration Knobs
- Max function count
- Max loop depth
- Max expression nesting
- Type complexity
- Feature gates (e.g., disable unsafe, async)

### Output Format
Plain Rust source file that conforms to the `guest/cores/` interface.

## Usage (Phase 6+)

```bash
# Generate a single program
rustgen generate --seed 42 --output guest/cores/gen_42.rs

# Generate batch
rustgen batch --count 100 --output-dir guest/cores/generated/
```

## Phase Schedule

- **Phase 0-5**: Not implemented (stub directory)
- **Phase 6**: Initial RustSmith integration
- **Phase 7**: Use in validation (attempt to rediscover known bugs)
- **Phase 13**: Tune based on which operators are productive

## Notes

- If RustSmith has issues, contact **Ali** or **Shel** (mentioned in meeting)
- LLMs can help craft valid RustSmith configs

