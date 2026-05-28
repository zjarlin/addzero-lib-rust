//! 脚本运行时和插件运行时共享的沙箱策略类型。

use az_derive_aliases::{apply, serde_eq_default};

/// 沙箱执行上下文的权限策略。
#[apply(serde_eq_default)]
pub struct SandboxPolicy {
    /// 允许访问的文件系统路径；为空表示全部拒绝。
    pub fs_allow: Vec<String>,
    /// 允许访问的网络主机或端口；为空表示全部拒绝。
    pub net_allow: Vec<String>,
    /// 允许执行的命令；为空表示全部拒绝。
    pub cmd_allow: Vec<String>,
    /// 最大内存，单位 MB；0 表示不限制。
    pub max_memory_mb: u64,
    /// 最大执行时间，单位秒；0 表示不限制。
    pub max_time_secs: u64,
}

impl SandboxPolicy {
    /// 面向可信脚本的宽松策略。
    pub fn permissive() -> Self {
        Self::default()
    }

    /// 拒绝所有外部能力的限制策略。
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
