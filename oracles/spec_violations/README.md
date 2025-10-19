# Spec Violations Oracle (A3)

**Phase 11+**: zkVM-specific invariant checks for soundness testing.

## Purpose

Check **"should never pass"** invariants specific to each zkVM:
- Proof verification should fail after certain mutations
- Public values should satisfy constraints
- Commitments should be consistent with execution

## Example Checks

### Witness Tampering
```rust
// Mutate public value after witness → public value transition
public_values.is_complete = !public_values.is_complete;

// Verification MUST fail
assert!(verifier.verify(proof, public_values).is_err());
```

### Capacity Overflow
```rust
// Allocate beyond MAX_MEMORY
let ptr = alloc(size);
assert!(ptr + capacity <= MAX_MEMORY); // Should trigger error
```

### Under-Constrained Registers
```rust
// RISC Zero 3-register bug: when rs1 == rs2
// Verification should catch this, but doesn't (if bug present)
```

## zkVM-Specific

Each zkVM may have different invariants:
- **SP1**: Specific chip ordering, commitment structure
- **RISC Zero**: Receipt structure, journal constraints
- **OpenVM**: Circuit-specific checks

## Phase Schedule

- **Phase 0-10**: Not implemented (stub directory)
- **Phase 11**: Research + document invariants per zkVM
- **Phase 12+**: Implementation (SP1 first)

## Output

```rust
pub struct ViolationResult {
    pub violated: bool,
    pub invariant: String,      // Which invariant was checked
    pub expected: String,        // What should have happened
    pub actual: String,          // What actually happened
}
```

Example:
```json
{
  "violated": true,
  "invariant": "verification_must_fail_after_public_value_mutation",
  "expected": "VerificationError",
  "actual": "VerificationSuccess"
}
```

This indicates a **soundness bug**: a mutated proof verified when it shouldn't have.

