use serde::{Deserialize, Serialize};

/// Input for I/O echo core
/// Tests allocator behavior and capacity handling with varying data sizes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoEchoInput {
    /// Arbitrary binary data - can be empty, small, or very large
    pub data: Vec<u8>,
}

/// Output for I/O echo core
/// Returns information about the input data without raw pointer addresses
/// (Pointer addresses differ between native and SP1 address spaces)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoEchoOutput {
    /// Length of the input data
    pub length: u32,
    /// First byte of data if present
    pub first_byte: Option<u8>,
    /// Last byte of data if present
    pub last_byte: Option<u8>,
}

/// Run the I/O echo core
/// 
/// This core exercises:
/// - Memory allocation (Vec with varying sizes)
/// - Capacity calculations (internal allocator behavior)
/// - Slice indexing (bounds checking)
/// 
/// Target vulnerability: Allocator capacity overflow (ptr + capacity > MAX_MEMORY)
/// where capacity scales with guest-controlled data size.
pub fn run(input: IoEchoInput) -> IoEchoOutput {
    let length = input.data.len() as u32;
    let first_byte = input.data.first().copied();
    let last_byte = input.data.last().copied();

    IoEchoOutput {
        length,
        first_byte,
        last_byte,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_data() {
        let input = IoEchoInput { data: vec![] };
        let output = run(input);
        assert_eq!(output.length, 0);
        assert_eq!(output.first_byte, None);
        assert_eq!(output.last_byte, None);
    }

    #[test]
    fn test_single_byte() {
        let input = IoEchoInput { data: vec![42] };
        let output = run(input);
        assert_eq!(output.length, 1);
        assert_eq!(output.first_byte, Some(42));
        assert_eq!(output.last_byte, Some(42));
    }

    #[test]
    fn test_multiple_bytes() {
        let input = IoEchoInput { data: vec![1, 2, 3, 4, 5] };
        let output = run(input);
        assert_eq!(output.length, 5);
        assert_eq!(output.first_byte, Some(1));
        assert_eq!(output.last_byte, Some(5));
    }

    #[test]
    fn test_large_data() {
        let data = vec![0u8; 10000];
        let input = IoEchoInput { data };
        let output = run(input);
        assert_eq!(output.length, 10000);
        assert_eq!(output.first_byte, Some(0));
        assert_eq!(output.last_byte, Some(0));
    }
}

