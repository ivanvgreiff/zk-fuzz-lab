use serde::{Deserialize, Serialize};

/// Input for simple struct core
/// Tests struct serialization, string handling, and ABI compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleStructInput {
    /// Numeric field
    pub field1: u32,
    /// String field (tests UTF-8 handling and allocation)
    pub field2: String,
    /// Boolean field
    pub field3: bool,
}

/// Output for simple struct core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleStructOutput {
    /// Echo of field1
    pub field1_echo: u32,
    /// Length of field2 in bytes
    pub field2_len: u32,
    /// Number of chars in field2 (may differ from bytes for unicode)
    pub field2_chars: u32,
    /// Echo of field3
    pub field3_echo: bool,
}

/// Run the simple struct core
/// 
/// Tests:
/// - Struct layout and padding
/// - String encoding (UTF-8 validation)
/// - Byte vs char length (important for unicode)
/// - Serialization consistency between native and RISC-V
/// 
/// Target vulnerabilities:
/// - Struct alignment differences
/// - String encoding mismatches
/// - Serialization format incompatibilities
pub fn run(input: SimpleStructInput) -> SimpleStructOutput {
    let field2_len = input.field2.len() as u32;        // Byte length
    let field2_chars = input.field2.chars().count() as u32; // Character count

    SimpleStructOutput {
        field1_echo: input.field1,
        field2_len,
        field2_chars,
        field3_echo: input.field3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_struct() {
        let input = SimpleStructInput {
            field1: 42,
            field2: "hello".to_string(),
            field3: true,
        };
        let output = run(input);
        assert_eq!(output.field1_echo, 42);
        assert_eq!(output.field2_len, 5);
        assert_eq!(output.field2_chars, 5);
        assert_eq!(output.field3_echo, true);
    }

    #[test]
    fn test_empty_string() {
        let input = SimpleStructInput {
            field1: 0,
            field2: "".to_string(),
            field3: false,
        };
        let output = run(input);
        assert_eq!(output.field1_echo, 0);
        assert_eq!(output.field2_len, 0);
        assert_eq!(output.field2_chars, 0);
        assert_eq!(output.field3_echo, false);
    }

    #[test]
    fn test_unicode_string() {
        let input = SimpleStructInput {
            field1: 1,
            field2: "🦀 Rust".to_string(), // Emoji is 4 bytes, but 1 char
            field3: true,
        };
        let output = run(input);
        assert_eq!(output.field1_echo, 1);
        assert_eq!(output.field2_len, 9); // 4 (emoji) + 1 (space) + 4 ("Rust")
        assert_eq!(output.field2_chars, 6); // 1 (emoji) + 1 (space) + 4 ("Rust")
        assert_eq!(output.field3_echo, true);
    }

    #[test]
    fn test_long_string() {
        let long_string = "a".repeat(1000);
        let input = SimpleStructInput {
            field1: 99,
            field2: long_string,
            field3: false,
        };
        let output = run(input);
        assert_eq!(output.field1_echo, 99);
        assert_eq!(output.field2_len, 1000);
        assert_eq!(output.field2_chars, 1000);
        assert_eq!(output.field3_echo, false);
    }

    #[test]
    fn test_max_values() {
        let input = SimpleStructInput {
            field1: u32::MAX,
            field2: "max".to_string(),
            field3: true,
        };
        let output = run(input);
        assert_eq!(output.field1_echo, u32::MAX);
        assert_eq!(output.field2_len, 3);
        assert_eq!(output.field2_chars, 3);
        assert_eq!(output.field3_echo, true);
    }
}

