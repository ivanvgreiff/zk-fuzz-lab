# SP1 Runner

Builds and executes guest programs via **SP1 zkVM**.

## Purpose

Compile adapters from `adapters/sp1_guest/` and run them through SP1:
1. Build the SP1 guest program (adapter + core)
2. Execute with input via `sp1_zkVM::io::read()`
3. Extract committed values from execution
4. Capture status (OK | PANIC | TIMEOUT) and timing

## Output Format

The SP1 runner produces a `RunResult` JSON identical to native runner:

```json
{
  "status": "OK",
  "elapsed_ms": 142,
  "commits": [24, 46368, 75025],
  "meta": {
    "mode": "execute"
  }
}
```

## Execution Modes

### Phase 1-2: Execute Only
```rust
let (output, report) = client.execute(elf, stdin).run()?;
```
- Fast feedback
- No proof generation
- Extracts public values (commits)

### Phase 6+: Prove + Verify
```rust
let (pk, vk) = client.setup(elf);
let proof = client.prove(&pk, stdin).run()?;
client.verify(&proof, &vk)?;
```
- Full proof generation
- Verification check
- Slower but catches soundness issues

## Phase Schedule

- **Phase 1**: Execute-only mode with commit extraction
- **Phase 2**: Add panic/timeout handling
- **Phase 6**: Add prove+verify mode behind flag

