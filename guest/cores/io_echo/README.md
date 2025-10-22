# I/O Echo Core

**Purpose**: Test allocator behavior and capacity handling with varying data sizes.

## Design

This core reads arbitrary binary data (`Vec<u8>`) and returns information about it without comparing raw pointer addresses (which differ between native and SP1 address spaces).

## Input Format

```json
{
  "data": [0, 1, 2, 3, 4, 5]
}
```

### Fields
- `data` (Vec<u8>): Arbitrary binary data - can be empty, small, or very large

## Output Format

The core commits three values:

```rust
pub struct IoEchoOutput {
    pub length: u32,           // data.len()
    pub first_byte: Option<u8>, // data[0] if exists
    pub last_byte: Option<u8>,  // data[len-1] if exists
}
```

### Commit Order (SP1)
1. `length` (u32)
2. `first_byte` (Option<u8> as u32: 0 for None, 1+value for Some)
3. `last_byte` (Option<u8> as u32: 0 for None, 1+value for Some)

## Usage

### Test Case 1: Empty Data
```bash
make run CORE=guest/cores/io_echo INPUT=inputs/io_echo_empty.json
```

**Expected Output**: Both runners succeed, length=0, first_byte=None, last_byte=None

### Test Case 2: Small Data (1KB)
```bash
make run CORE=guest/cores/io_echo INPUT=inputs/io_echo_1kb.json
```

**Expected Output**: Both runners succeed, length=1024, commits match

### Test Case 3: Small Data (10 bytes)
```bash
make run CORE=guest/cores/io_echo INPUT=inputs/io_echo_small.json
```

**Expected Output**: Both runners succeed, length=10, commits match

## Target Vulnerabilities

### Allocator Capacity Overflow
The core exercises the allocator with guest-controlled data sizes. This targets bugs like:
- `ptr + capacity > MAX_MEMORY` wraparound
- Incorrect capacity calculations
- Integer overflow in allocation size

**Attack Vector**: Gradually increase `data` size:
- `[]` (0 bytes)
- `1KB` (1,024 bytes)
- `100KB` (102,400 bytes)
- Future: `1MB+` when testing extreme cases

## Implementation Notes

### Why Not Commit Pointer Addresses?
Native and SP1 use different address spaces:
- **Native**: Host x86-64 address space (e.g., `0x7ffe5c3b2000`)
- **SP1**: RISC-V VM address space (e.g., `0x00400000`)

Committing raw addresses would cause **false positive divergences**.

### What We DO Commit
- **Length**: Tests allocation succeeded
- **First/Last Bytes**: Tests slice indexing works
- **Allocation still happens**: SP1 still allocates the `Vec` internally, we just don't compare addresses

## Phase Context

**Phase 3**: This core is part of the seed corpus that exercises allocator code paths before mutation/generation in later phases.

