use serde::{Deserialize, Serialize};

/// Input for timeout test core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutInput {
    /// Number of iterations (0 = infinite loop)
    pub iterations: u64,
}

/// Output for timeout test core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutOutput {
    /// Number of iterations completed
    pub completed: u64,
}

/// Run the timeout test core
/// 
/// If iterations == 0, runs an infinite loop (will timeout).
/// Otherwise, runs for the specified number of iterations.
pub fn run(input: TimeoutInput) -> TimeoutOutput {
    if input.iterations == 0 {
        // Infinite loop - will cause timeout
        loop {
            // Prevent optimization
            std::hint::black_box(1 + 1);
        }
    }

    // Finite loop - compute something to prevent optimization
    let mut sum = 0u64;
    for i in 0..input.iterations {
        sum = sum.wrapping_add(i);
        std::hint::black_box(sum);
    }

    TimeoutOutput {
        completed: input.iterations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finite_iterations() {
        let input = TimeoutInput { iterations: 100 };
        let output = run(input);
        assert_eq!(output.completed, 100);
    }

    #[test]
    fn test_large_iterations() {
        let input = TimeoutInput { iterations: 1_000_000 };
        let output = run(input);
        assert_eq!(output.completed, 1_000_000);
    }
}

