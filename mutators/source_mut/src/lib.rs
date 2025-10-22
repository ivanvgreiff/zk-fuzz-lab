use anyhow::Result;
use serde_json::Value;

/// Represents a generated input mutation
#[derive(Debug, Clone)]
pub struct MutatedInput {
    /// The mutated input as JSON
    pub input_json: Value,
    /// Description of the mutation applied (e.g., "length_bias:1024->2048")
    pub mutation_op: String,
    /// The base input that was mutated
    pub base_input_path: String,
}

/// Input mutation strategies
#[derive(Debug, Clone)]
pub enum MutationStrategy {
    /// Length biasing for Vec<u8> inputs (io_echo)
    LengthBias,
    /// Boundary value testing for arithmetic inputs
    BoundaryValues,
    /// String variations for struct inputs
    StringVariations,
    /// Fibonacci number variations
    FibonacciValues,
    /// Boolean variations
    BooleanVariations,
    /// Iteration count variations
    IterationVariations,
}

/// Generate mutations for a given core
pub fn generate_mutations(
    core_name: &str,
    base_input_json: &Value,
    base_input_path: &str,
) -> Result<Vec<MutatedInput>> {
    match core_name {
        "io_echo" => generate_io_echo_mutations(base_input_json, base_input_path),
        "arithmetic" => generate_arithmetic_mutations(base_input_json, base_input_path),
        "simple_struct" => generate_simple_struct_mutations(base_input_json, base_input_path),
        "fib" => generate_fib_mutations(base_input_json, base_input_path),
        "panic_test" => generate_panic_test_mutations(base_input_json, base_input_path),
        "timeout_test" => generate_timeout_test_mutations(base_input_json, base_input_path),
        _ => anyhow::bail!("Unknown core: {}", core_name),
    }
}

/// Generate io_echo mutations with length biasing (hybrid strategy)
fn generate_io_echo_mutations(
    _base_input: &Value,
    base_input_path: &str,
) -> Result<Vec<MutatedInput>> {
    let mut mutations = Vec::new();

    // Powers of 2: {0, 1, 2, 4, 8, ..., 1MB}
    let powers_of_2 = vec![
        0, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512,
        1024,           // 1KB
        2048,           // 2KB
        4096,           // 4KB
        8192,           // 8KB
        16384,          // 16KB
        32768,          // 32KB
        65536,          // 64KB
        131072,         // 128KB
        262144,         // 256KB
        524288,         // 512KB
        1048576,        // 1MB
    ];

    // Boundaries: {127, 255, 1023, 4095, 65535}
    let boundaries = vec![127, 255, 1023, 4095, 65535];

    // Edge cases: {3, 7, 15, 31, 63}
    let edge_cases = vec![3, 7, 15, 31, 63];

    // Combine all sizes (deduplicate)
    let mut all_sizes = powers_of_2;
    all_sizes.extend(boundaries);
    all_sizes.extend(edge_cases);
    all_sizes.sort_unstable();
    all_sizes.dedup();

    // Generate input for each size
    for size in all_sizes {
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let input_json = serde_json::json!({
            "data": data
        });

        let size_desc = if size < 1024 {
            format!("{}b", size)
        } else if size < 1048576 {
            format!("{}kb", size / 1024)
        } else {
            format!("{}mb", size / 1048576)
        };

        mutations.push(MutatedInput {
            input_json,
            mutation_op: format!("length_bias:{}", size_desc),
            base_input_path: base_input_path.to_string(),
        });
    }

    Ok(mutations)
}

/// Generate arithmetic mutations with boundary values
fn generate_arithmetic_mutations(
    _base_input: &Value,
    base_input_path: &str,
) -> Result<Vec<MutatedInput>> {
    let mut mutations = Vec::new();

    let operations = vec!["add", "sub", "mul", "div"];
    let boundary_values = vec![
        0,
        1,
        2,
        u32::MAX / 2,
        u32::MAX - 1,
        u32::MAX,
    ];

    // Generate combinations for each operation
    for op in &operations {
        for &a in &boundary_values {
            for &b in &boundary_values {
                // Skip some redundant combinations to keep count manageable
                if a == 0 && b == 0 {
                    continue; // Keep one zero case
                }
                
                let input_json = serde_json::json!({
                    "a": a,
                    "b": b,
                    "operation": op
                });

                mutations.push(MutatedInput {
                    input_json,
                    mutation_op: format!("boundary_values:{}_{}_op_{}", a, b, op),
                    base_input_path: base_input_path.to_string(),
                });

                // Limit to ~6 per operation to keep total manageable
                if mutations.len() >= operations.len() * 6 {
                    break;
                }
            }
            if mutations.len() >= operations.len() * 6 {
                break;
            }
        }
    }

    Ok(mutations)
}

/// Generate simple_struct mutations with string variations
fn generate_simple_struct_mutations(
    _base_input: &Value,
    base_input_path: &str,
) -> Result<Vec<MutatedInput>> {
    let mut mutations = Vec::new();

    // Pre-generate repeated strings to avoid lifetime issues
    let string_100 = "a".repeat(100);
    let string_1000 = "a".repeat(1000);
    let string_10000 = "a".repeat(10000);

    let string_cases: Vec<(String, &str)> = vec![
        (String::new(), "empty"),
        ("a".to_string(), "single"),
        ("hello".to_string(), "short"),
        (string_100, "100chars"),
        (string_1000, "1000chars"),
        (string_10000, "10kchars"),
        ("🦀".to_string(), "emoji"),
        ("🦀 Rust zkVM".to_string(), "unicode_mixed"),
        ("Hello\nWorld".to_string(), "newline"),
        ("Tab\tSeparated".to_string(), "tab"),
    ];

    let field1_values = vec![0, 1, 42, u32::MAX];
    let field3_values = vec![true, false];

    // Generate combinations
    for (idx, (string, string_desc)) in string_cases.iter().enumerate() {
        let field1 = field1_values[idx % field1_values.len()];
        let field3 = field3_values[idx % field3_values.len()];

        let input_json = serde_json::json!({
            "field1": field1,
            "field2": string,
            "field3": field3
        });

        mutations.push(MutatedInput {
            input_json,
            mutation_op: format!("string_variation:{}", string_desc),
            base_input_path: base_input_path.to_string(),
        });
    }

    Ok(mutations)
}

/// Generate fib mutations with various n values
fn generate_fib_mutations(
    _base_input: &Value,
    base_input_path: &str,
) -> Result<Vec<MutatedInput>> {
    let mut mutations = Vec::new();

    let n_values = vec![0, 1, 2, 5, 10, 20, 30, 40, 50, 100, 1000];

    for n in n_values {
        let input_json = serde_json::json!({
            "n": n
        });

        mutations.push(MutatedInput {
            input_json,
            mutation_op: format!("fib_value:n={}", n),
            base_input_path: base_input_path.to_string(),
        });
    }

    Ok(mutations)
}

/// Generate panic_test mutations
fn generate_panic_test_mutations(
    _base_input: &Value,
    base_input_path: &str,
) -> Result<Vec<MutatedInput>> {
    let mut mutations = Vec::new();

    let cases = vec![
        (false, "no_panic"),
        (true, "panic_simple"),
        (true, "panic_with_long_message"),
        (false, "no_panic_alternate"),
    ];

    for (should_panic, desc) in cases {
        let message = if should_panic {
            format!("Test panic: {}", desc)
        } else {
            String::new()
        };

        let input_json = serde_json::json!({
            "should_panic": should_panic,
            "panic_message": message
        });

        mutations.push(MutatedInput {
            input_json,
            mutation_op: format!("bool_variation:{}", desc),
            base_input_path: base_input_path.to_string(),
        });
    }

    Ok(mutations)
}

/// Generate timeout_test mutations with iteration variations
fn generate_timeout_test_mutations(
    _base_input: &Value,
    base_input_path: &str,
) -> Result<Vec<MutatedInput>> {
    let mut mutations = Vec::new();

    let iteration_counts = vec![
        0,
        1,
        10,
        100,
        1_000,
        10_000,
        100_000,
        1_000_000,
        10_000_000,
    ];

    for iterations in iteration_counts {
        let input_json = serde_json::json!({
            "iterations": iterations
        });

        mutations.push(MutatedInput {
            input_json,
            mutation_op: format!("iteration_variation:{}", iterations),
            base_input_path: base_input_path.to_string(),
        });
    }

    Ok(mutations)
}

/// Statistics about generated mutations
#[derive(Debug, Clone)]
pub struct MutationStats {
    pub total_count: usize,
    pub min_size: Option<usize>,
    pub max_size: Option<usize>,
    pub strategy: String,
}

/// Calculate statistics for io_echo mutations (size distribution)
pub fn calculate_size_stats(mutations: &[MutatedInput]) -> MutationStats {
    let mut min_size = None;
    let mut max_size = None;

    for mutation in mutations {
        if let Some(data) = mutation.input_json.get("data").and_then(|d| d.as_array()) {
            let size = data.len();
            min_size = Some(min_size.map_or(size, |m: usize| m.min(size)));
            max_size = Some(max_size.map_or(size, |m: usize| m.max(size)));
        }
    }

    MutationStats {
        total_count: mutations.len(),
        min_size,
        max_size,
        strategy: "length_bias".to_string(),
    }
}

