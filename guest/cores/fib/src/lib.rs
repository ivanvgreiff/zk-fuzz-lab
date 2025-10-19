use serde::{Deserialize, Serialize};

/// Input for the fibonacci computation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FibInput {
    pub n: u32,
}

/// Output of the fibonacci computation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FibOutput {
    /// The input value (echoed back)
    pub n: u32,
    /// The (n-1)th fibonacci number
    pub a: u32,
    /// The nth fibonacci number
    pub b: u32,
}

/// Pure Rust implementation of fibonacci computation
/// This is ZKVM-agnostic business logic
pub fn run(input: FibInput) -> FibOutput {
    let n = input.n;
    
    // Compute the n'th fibonacci number using normal Rust code
    let mut a = 0u32;
    let mut b = 1u32;
    
    for _ in 0..n {
        let mut c = a + b;
        c %= 7919; // Modulus to prevent overflow (same as SP1 example)
        a = b;
        b = c;
    }
    
    FibOutput { n, a, b }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fib_0() {
        let result = run(FibInput { n: 0 });
        assert_eq!(result.n, 0);
        assert_eq!(result.a, 0);
        assert_eq!(result.b, 1);
    }

    #[test]
    fn test_fib_1() {
        let result = run(FibInput { n: 1 });
        assert_eq!(result.n, 1);
        assert_eq!(result.a, 1);
        assert_eq!(result.b, 1);
    }

    #[test]
    fn test_fib_10() {
        let result = run(FibInput { n: 10 });
        assert_eq!(result.n, 10);
        assert_eq!(result.a, 55);
        assert_eq!(result.b, 89);
    }
}

