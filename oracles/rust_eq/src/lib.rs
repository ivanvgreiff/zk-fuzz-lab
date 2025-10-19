use serde::{Deserialize, Serialize};

/// Status of a program execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    /// Execution completed successfully
    #[serde(rename = "OK")]
    Ok,
    /// Execution panicked
    #[serde(rename = "PANIC")]
    Panic,
    /// Execution timed out
    #[serde(rename = "TIMEOUT")]
    Timeout,
}

/// Result of running a program (native or ZKVM)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    /// Execution status
    pub status: Status,
    /// Elapsed time in milliseconds
    pub elapsed_ms: u128,
    /// Sequence of committed values (in order)
    pub commits: Vec<serde_json::Value>,
    /// Optional metadata (panic message, etc.)
    #[serde(default)]
    pub meta: serde_json::Value,
}

/// Result of comparing two RunResults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diff {
    /// Whether the results are equal
    pub equal: bool,
    /// Reason for inequality (if any)
    pub reason: Option<String>,
    /// Timing delta in milliseconds (informational only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing_delta_ms: Option<u128>,
}

/// Compare two RunResults for equality
///
/// This is the core oracle logic for A1 differential testing.
/// It compares:
/// 1. Status (OK/PANIC/TIMEOUT)
/// 2. Commit streams (must be exactly equal if both OK)
/// 3. Timing (recorded but not used for equality)
pub fn compare(native: &RunResult, zkvm: &RunResult) -> Diff {
    // 1. Compare status first
    if native.status != zkvm.status {
        return Diff {
            equal: false,
            reason: Some(format!(
                "status mismatch: native={:?}, zkvm={:?}",
                native.status, zkvm.status
            )),
            timing_delta_ms: Some(native.elapsed_ms.abs_diff(zkvm.elapsed_ms)),
        };
    }

    // 2. If both OK, compare the commit streams exactly
    if native.status == Status::Ok && native.commits != zkvm.commits {
        return Diff {
            equal: false,
            reason: Some(format!(
                "commit stream mismatch: native={:?} vs zkvm={:?}",
                native.commits, zkvm.commits
            )),
            timing_delta_ms: Some(native.elapsed_ms.abs_diff(zkvm.elapsed_ms)),
        };
    }

    // 3. Results are equal
    Diff {
        equal: true,
        reason: None,
        timing_delta_ms: Some(native.elapsed_ms.abs_diff(zkvm.elapsed_ms)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_compare_equal() {
        let native = RunResult {
            status: Status::Ok,
            elapsed_ms: 10,
            commits: vec![json!(24), json!(46368), json!(75025)],
            meta: json!({}),
        };
        let zkvm = RunResult {
            status: Status::Ok,
            elapsed_ms: 150,
            commits: vec![json!(24), json!(46368), json!(75025)],
            meta: json!({}),
        };

        let diff = compare(&native, &zkvm);
        assert!(diff.equal);
        assert_eq!(diff.timing_delta_ms, Some(140));
    }

    #[test]
    fn test_compare_status_mismatch() {
        let native = RunResult {
            status: Status::Ok,
            elapsed_ms: 10,
            commits: vec![json!(24)],
            meta: json!({}),
        };
        let zkvm = RunResult {
            status: Status::Panic,
            elapsed_ms: 5,
            commits: vec![],
            meta: json!({"panic_msg": "overflow"}),
        };

        let diff = compare(&native, &zkvm);
        assert!(!diff.equal);
        assert!(diff.reason.unwrap().contains("status mismatch"));
    }

    #[test]
    fn test_compare_commit_mismatch() {
        let native = RunResult {
            status: Status::Ok,
            elapsed_ms: 10,
            commits: vec![json!(24), json!(46368), json!(75025)],
            meta: json!({}),
        };
        let zkvm = RunResult {
            status: Status::Ok,
            elapsed_ms: 150,
            commits: vec![json!(24), json!(46368), json!(75026)], // Off by one
            meta: json!({}),
        };

        let diff = compare(&native, &zkvm);
        assert!(!diff.equal);
        assert!(diff.reason.unwrap().contains("commit stream mismatch"));
    }
}

