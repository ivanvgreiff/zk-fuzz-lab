use serde::{Deserialize, Serialize};

/// Input for panic test core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicInput {
    /// If true, the program will panic
    pub should_panic: bool,
    /// Message to panic with
    pub panic_msg: Option<String>,
}

/// Output for panic test core (only returned if no panic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicOutput {
    /// Echo of should_panic as u32 (0 = false, 1 = true)
    pub should_panic_u32: u32,
    /// Status code (0 = success)
    pub status_code: u32,
}

/// Run the panic test core
/// 
/// Panics if input.should_panic is true, otherwise returns success.
pub fn run(input: PanicInput) -> PanicOutput {
    if input.should_panic {
        let msg = input.panic_msg.unwrap_or_else(|| "Intentional panic for testing".to_string());
        panic!("{}", msg);
    }

    PanicOutput {
        should_panic_u32: if input.should_panic { 1 } else { 0 },
        status_code: 0, // 0 = success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_panic() {
        let input = PanicInput {
            should_panic: false,
            panic_msg: None,
        };
        let output = run(input);
        assert_eq!(output.should_panic_u32, 0);
        assert_eq!(output.status_code, 0);
    }

    #[test]
    #[should_panic(expected = "Intentional panic")]
    fn test_panic_default_msg() {
        let input = PanicInput {
            should_panic: true,
            panic_msg: None,
        };
        run(input);
    }

    #[test]
    #[should_panic(expected = "Custom error message")]
    fn test_panic_custom_msg() {
        let input = PanicInput {
            should_panic: true,
            panic_msg: Some("Custom error message".to_string()),
        };
        run(input);
    }
}

