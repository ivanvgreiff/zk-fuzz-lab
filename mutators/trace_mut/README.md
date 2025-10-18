# Trace Mutators (A3)

**Phase 11+**: ZKVM-specific mutations for soundness testing.

## Purpose

Mutate ZKVM-specific artifacts to surface **soundness issues**:
- Witness tampering
- Public value modifications
- Serialization/deserialization corruption
- Commitment structure manipulation

## Why A3 is Different

A1 and A2 are **ZKVM-agnostic** (compare behaviors).

A3 is **ZKVM-specific** (mutate proof artifacts that should cause verification failure).

## Research Items (Phase 11)

Per meeting context, investigate:
1. **`is_complete` lifecycle**: Witness → Public Value
   - Where does it transition?
   - When can we mutate the public value struct?
2. **Capacity control**: How guest data size drives capacity
   - Create targeted inputs for capacity attacks
3. **Chip ordering / VK root**: Where does chip ordering "live"?
   - What commitments/VK root does it depend on?

## Mutation Strategies

### Public Value Tampering
```rust
// After witness → public value transition
public_values.is_complete = !public_values.is_complete;
```

### Serialization Corruption
```rust
// Corrupt proof bytes before verification
proof_bytes[rand_offset] ^= 0xFF;
```

### Commitment Manipulation
```rust
// Modify intermediate commitments
commitments[chip_id] = random_field_element();
```

## Expected Outcomes

A3 mutations should **fail verification** if the ZKVM is sound.

If verification **passes** after mutation → potential soundness bug.

## Phase Schedule

- **Phase 0-10**: Not implemented (stub directory)
- **Phase 11**: Research + initial A3 prep
- **Phase 12+**: Implementation (SP1 first)

## Relation to Other Approaches

- **A1 finds**: Divergences in Rust behavior
- **A2 finds**: Divergences in RISC-V state
- **A3 finds**: Soundness holes (proofs that should fail verification)

Sometimes only A3 catches certain bugs (e.g., under-constrained registers).

