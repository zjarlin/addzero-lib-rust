//! Sandbox policy types shared by script and plugin runtimes.

use serde::{Deserialize, Serialize};

/// Permission policy for a sandboxed execution context.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SandboxPolicy {
    /// Allowed filesystem paths (empty = deny all).
    pub fs_allow: Vec<String>,
    /// Allowed network hosts/ports (empty = deny all).
    pub net_allow: Vec<String>,
    /// Allowed commands (empty = deny all).
    pub cmd_allow: Vec<String>,
    /// Max memory in MB (0 = unlimited).
    pub max_memory_mb: u64,
    /// Max execution time in seconds (0 = unlimited).
    pub max_time_secs: u64,
}

impl SandboxPolicy {
    /// A permissive policy for trusted scripts.
    pub fn permissive() -> Self {
        Self::default()
    }

    /// A restrictive policy that denies everything.
    pub fn deny_all() -> Self {
        Self {
            fs_allow: vec![],
            net_allow: vec![],
            cmd_allow: vec![],
            max_memory_mb: 0,
            max_time_secs: 0,
        }
    }
}
