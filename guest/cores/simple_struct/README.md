# Simple Struct Core

**Purpose**: Test struct serialization, string handling, and ABI compatibility.

## Design

This core processes a struct with mixed field types (u32, String, bool) and returns information about it. This tests whether native Rust and SP1 handle struct layout, string encoding, and serialization consistently.

## Input Format

```json
{
  "field1": 42,
  "field2": "hello",
  "field3": true
}
```

### Fields
- `field1` (u32): Numeric field
- `field2` (String): String field (tests UTF-8 handling)
- `field3` (bool): Boolean field

## Output Format

The core commits four values:

```rust
pub struct SimpleStructOutput {
    pub field1_echo: u32,    // Echo of input field1
    pub field2_len: u32,     // Byte length of field2
    pub field2_chars: u32,   // Character count of field2 (may differ for unicode)
    pub field3_echo: bool,   // Echo of input field3
}
```

### Commit Order (SP1)
1. `field1_echo` (u32)
2. `field2_len` (u32)
3. `field2_chars` (u32)
4. `field3_echo` (bool as u32: 0 for false, 1 for true)

## Usage

### Test Case 1: Normal Struct
```bash
make run CORE=guest/cores/simple_struct INPUT=inputs/simple_struct_normal.json
```

**Expected Output**: Both runners succeed, all fields match

### Test Case 2: Empty String
```bash
make run CORE=guest/cores/simple_struct INPUT=inputs/simple_struct_empty.json
```

**Expected Output**: Both runners succeed, field2_len=0, field2_chars=0

### Test Case 3: Unicode String
```bash
make run CORE=guest/cores/simple_struct INPUT=inputs/simple_struct_unicode.json
```

**Expected Output**: Both runners succeed, byte length differs from char count

### Test Case 4: Long String
```bash
make run CORE=guest/cores/simple_struct INPUT=inputs/simple_struct_long.json
```

**Expected Output**: Both runners succeed, field2_len=1000

## Target Vulnerabilities

### Struct Layout Differences
- Padding between fields
- Alignment requirements
- Field ordering

### String Encoding
- UTF-8 validation differences
- Byte vs character length calculation
- Unicode handling (multi-byte characters)

### Serialization
- JSON serialization format consistency
- serde behavior differences
- Memory layout in RISC-V vs x86-64

## Implementation Notes

### Why Track Both Bytes and Chars?
For ASCII strings, byte length equals char count. But for unicode:
- `"🦀"` = 4 bytes, 1 char
- `"🦀 Rust"` = 9 bytes, 6 chars

This tests that both platforms calculate string metrics identically.

### Potential Divergences
- **String allocation**: Different heap layouts
- **UTF-8 validation**: Stricter or looser checks
- **Struct padding**: Different alignment on RISC-V

## Phase Context

**Phase 3**: This core is part of the seed corpus that exercises serialization and ABI compatibility before mutation in later phases.

