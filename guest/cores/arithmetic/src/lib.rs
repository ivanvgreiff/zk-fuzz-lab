use serde::{Deserialize, Serialize};

/// Input for arithmetic core
/// Tests integer arithmetic boundary cases and overflow/underflow handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArithmeticInput {
    /// First operand
    pub a: u32,
    /// Second operand
    pub b: u32,
    /// Operation: "add", "sub", "mul", or "div"
    pub operation: String,
}

/// Output for arithmetic core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArithmeticOutput {
    /// Result of the operation (wrapping for overflow)
    pub result: u32,
    /// Whether overflow occurred (true for overflow)
    pub overflowed: bool,
}

/// Run the arithmetic core
/// 
/// Performs basic arithmetic operations with overflow detection.
/// Tests boundary cases where native and RISC-V semantics might differ.
/// 
/// Target vulnerabilities:
/// - Integer overflow/underflow handling differences
/// - Division by zero behavior
/// - Sign extension mismatches
pub fn run(input: ArithmeticInput) -> ArithmeticOutput {
    match input.operation.as_str() {
        "add" => {
            let (result, overflowed) = input.a.overflowing_add(input.b);
            ArithmeticOutput { result, overflowed }
        }
        "sub" => {
            let (result, overflowed) = input.a.overflowing_sub(input.b);
            ArithmeticOutput { result, overflowed }
        }
        "mul" => {
            let (result, overflowed) = input.a.overflowing_mul(input.b);
            ArithmeticOutput { result, overflowed }
        }
        "div" => {
            if input.b == 0 {
                panic!("Division by zero");
            }
            // Division can't overflow for unsigned integers
            ArithmeticOutput {
                result: input.a / input.b,
                overflowed: false,
            }
        }
        _ => panic!("Unknown operation: {}", input.operation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_normal() {
        let input = ArithmeticInput {
            a: 10,
            b: 20,
            operation: "add".to_string(),
        };
        let output = run(input);
        assert_eq!(output.result, 30);
        assert_eq!(output.overflowed, false);
    }

    #[test]
    fn test_add_overflow() {
        let input = ArithmeticInput {
            a: u32::MAX,
            b: 1,
            operation: "add".to_string(),
        };
        let output = run(input);
        assert_eq!(output.result, 0); // Wrapping
        assert_eq!(output.overflowed, true);
    }

    #[test]
    fn test_sub_normal() {
        let input = ArithmeticInput {
            a: 20,
            b: 10,
            operation: "sub".to_string(),
        };
        let output = run(input);
        assert_eq!(output.result, 10);
        assert_eq!(output.overflowed, false);
    }

    #[test]
    fn test_sub_underflow() {
        let input = ArithmeticInput {
            a: 0,
            b: 1,
            operation: "sub".to_string(),
        };
        let output = run(input);
        assert_eq!(output.result, u32::MAX); // Wrapping
        assert_eq!(output.overflowed, true);
    }

    #[test]
    fn test_mul_normal() {
        let input = ArithmeticInput {
            a: 10,
            b: 20,
            operation: "mul".to_string(),
        };
        let output = run(input);
        assert_eq!(output.result, 200);
        assert_eq!(output.overflowed, false);
    }

    #[test]
    fn test_mul_overflow() {
        let input = ArithmeticInput {
            a: 65536,
            b: 65536,
            operation: "mul".to_string(),
        };
        let output = run(input);
        assert_eq!(output.result, 0); // Wraps to 2^32 % 2^32 = 0
        assert_eq!(output.overflowed, true);
    }

    #[test]
    fn test_div_normal() {
        let input = ArithmeticInput {
            a: 20,
            b: 10,
            operation: "div".to_string(),
        };
        let output = run(input);
        assert_eq!(output.result, 2);
        assert_eq!(output.overflowed, false);
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_div_by_zero() {
        let input = ArithmeticInput {
            a: 1,
            b: 0,
            operation: "div".to_string(),
        };
        run(input);
    }

    #[test]
    #[should_panic(expected = "Unknown operation")]
    fn test_unknown_operation() {
        let input = ArithmeticInput {
            a: 1,
            b: 1,
            operation: "modulo".to_string(),
        };
        run(input);
    }
}

