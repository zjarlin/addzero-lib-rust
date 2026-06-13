//! 存储层围绕 Git 原语建立的类型安全包装。

use az_derive_aliases::{
    apply, impl_default, plain_copy_eq, plain_copy_eq_hash_ord_display, plain_eq,
    plain_eq_display, plain_string_value_object, serde_string_value_object,
};
use git2::Oid;
use std::path::PathBuf;
use anyhow::bail;

/// Git 提交 ID。
///
/// 包装 `git2::Oid` 是为了避免把 blob/tree/commit ID 误传到错误位置；
/// 原始 OID 只在存储模块内部暴露。
#[apply(plain_copy_eq_hash_ord_display)]
#[display("{_0}")]
pub struct CommitId(pub(crate) Oid);

impl CommitId {
    pub(crate) fn new(oid: Oid) -> Self {
        Self(oid)
    }

    /// 返回原始 OID，仅供存储模块内部使用。
    pub(crate) fn raw(&self) -> Oid {
        self.0
    }

    /// 从十六进制字符串解析提交 ID。
    pub fn from_hex(hex: &str) -> Result<Self, git2::Error> {
        Oid::from_str(hex).map(CommitId)
    }
    /// 返回提交 ID 的短格式。
    pub fn short(&self) -> String {
        self.0.to_string()[..7].to_string()
    }
}

/// Git blob 标识符。
#[apply(plain_copy_eq_hash_ord_display)]
#[display("{_0}")]
pub struct BlobId(pub(crate) Oid);

impl BlobId {
    pub(crate) fn new(oid: Oid) -> Self {
        Self(oid)
    }
    pub(crate) fn raw(&self) -> Oid {
        self.0
    }
}

/// Git tree 标识符。
#[apply(plain_copy_eq_hash_ord_display)]
#[display("{_0}")]
pub struct TreeId(pub(crate) Oid);

impl TreeId {
    pub(crate) fn new(oid: Oid) -> Self {
        Self(oid)
    }

    pub(crate) fn raw(&self) -> Oid {
        self.0
    }
}

/// 已验证的表名。
///
/// 表名会进入 Git 路径，因此这里限制字符集，避免路径穿越，并保持和
/// 文件系统/Git 引用约束兼容。合法表名长度为 1 到 64，只允许 ASCII
/// 字母、数字、下划线、连字符，且必须以字母或下划线开头。
#[apply(serde_string_value_object)]
#[display("{_0}")]
pub struct TableName(String);

impl TableName {
    /// 不能作为用户表使用的保留名称。
    const RESERVED: &'static [&'static str] = &["_schema", "_meta", "_system", "_git"];

    /// 创建并校验表名。
    pub fn new(name: impl Into<String>) -> anyhow::Result<Self> {
        let name = name.into();
        Self::validate(&name)?;
        Ok(Self(name))
    }

    /// 校验表名。
    fn validate(name: &str) -> anyhow::Result<()> {
        if name.is_empty() {
            bail!("name cannot be empty");
        }

        if name.len() > 64 {
            bail!("name too long: {} characters", name.len());
        }

        let Some(first_char) = name.chars().next() else {
            bail!("name cannot be empty");
        };

        // Must start with a letter or underscore (not a digit)
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            bail!("name cannot start with '{first_char}'");
        }

        for (i, c) in name.chars().enumerate() {
            if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
                bail!("invalid character '{c}' at position {i}");
            }
        }

        if Self::RESERVED.contains(&name.to_lowercase().as_str()) {
            bail!("'{name}' is a reserved name");
        }

        Ok(())
    }
}

/// 已验证的行键，也就是主键。
///
/// 行键会作为 JSON 文件名使用，因此需要和表名类似的路径安全约束。
/// 实际调用中通常由 ULID/UUID 生成，而不是人工输入。
#[apply(serde_string_value_object)]
#[display("{_0}")]
pub struct RowKey(String);

impl RowKey {
    pub fn new(key: impl Into<String>) -> anyhow::Result<Self> {
        let key = key.into();
        Self::validate(&key)?;
        Ok(Self(key))
    }

    /// 校验行键。
    fn validate(key: &str) -> anyhow::Result<()> {
        if key.is_empty() {
            bail!("name cannot be empty");
        }

        if key.len() > 128 {
            bail!("name too long: {} characters", key.len());
        }

        for (i, c) in key.chars().enumerate() {
            // alphanumeric, underscore, hyphen allowed
            if !c.is_ascii_alphanumeric() && c != '_' && c != '-' {
                bail!("invalid character '{c}' at position {i}");
            }
        }

        Ok(())
    }

    /// 生成基于 ULID 的新行键。
    pub fn generate() -> Self {
        Self(ulid::Ulid::new().to_string().to_lowercase())
    }
}

/// 仓库中一行数据的完整路径。
///
/// 路径格式固定为 `{table}/{row_key}.json`。
#[apply(plain_eq_display)]
#[display("{table}/{key}.json")]
pub struct RowPath {
    pub table: TableName,
    pub key: RowKey,
}

impl RowPath {
    /// 创建行路径。
    pub fn new(table: TableName, key: RowKey) -> Self {
        Self { table, key }
    }

    /// 转换为文件系统路径。
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(format!("{}/{}.json", self.table, self.key))
    }
}

/// Git 分支名，对事务分支有固定前缀约定。
#[apply(plain_string_value_object)]
#[display("{_0}")]
pub struct BranchName(String);

impl BranchName {
    /// 主分支名。
    pub const MAIN: &'static str = "main";

    /// 事务分支前缀。
    pub const TX_PREFIX: &'static str = "tx/";

    /// 创建并校验分支名。
    pub fn new(name: impl Into<String>) -> anyhow::Result<Self> {
        let name = name.into();
        // basic validation , git is more permissive but we gon be restrictive
        if name.is_empty() {
            bail!("name cannot be empty");
        }
        if name.contains("..") || name.ends_with('/') || name.starts_with('/') {
            bail!("invalid path: '{name}'");
        }
        Ok(Self(name))
    }

    /// 创建主分支名。
    pub fn main() -> Self {
        Self(Self::MAIN.to_string())
    }

    /// 根据事务 ID 创建事务分支名。
    pub fn for_transaction(tx_id: &str) -> Self {
        Self(format!("{}{}", Self::TX_PREFIX, tx_id))
    }

    /// 判断是否为事务分支。
    pub fn is_transaction_branch(&self) -> bool {
        self.0.starts_with(Self::TX_PREFIX)
    }

    /// 如果是事务分支，提取事务 ID。
    pub fn transaction_id(&self) -> Option<&str> {
        if self.is_transaction_branch() {
            Some(&self.0[Self::TX_PREFIX.len()..])
        } else {
            None
        }
    }

    /// 返回完整 Git ref 路径，例如 `refs/heads/main`。
    pub fn as_ref_path(&self) -> String {
        format!("refs/heads/{}", self.0)
    }
}

/// Git 提交签名，包含作者/提交者信息。
#[apply(plain_eq)]
pub struct GitSignature {
    pub name: String,
    pub email: String,
}

impl GitSignature {
    /// 创建提交签名。
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: email.into(),
        }
    }

    /// GitDB 内部操作使用的默认提交签名。
    pub fn gitdb() -> Self {
        Self::new("GitDB", "gitdb@localhost")
    }

    /// 转换为 `git2::Signature`。
    pub(crate) fn to_git2_signature(&self) -> Result<git2::Signature<'static>, git2::Error> {
        git2::Signature::now(&self.name, &self.email)
    }
}

impl_default!(GitSignature => GitSignature::gitdb());

/// 两个提交之间的一条路径变更。
#[apply(plain_eq)]
pub struct Change {
    pub path: PathBuf,
    pub status: ChangeStatus,
}

/// diff 中的路径变更类型。
#[apply(plain_copy_eq)]
pub enum ChangeStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_name_valid() {
        assert!(TableName::new("users").is_ok());
        assert!(TableName::new("user_accounts").is_ok());
        assert!(TableName::new("User123").is_ok());
        assert!(TableName::new("_private").is_ok());
        assert!(TableName::new("my-table").is_ok());
    }

    #[test]
    fn test_table_name_invalid() {
        assert!(TableName::new("").is_err());
        assert!(TableName::new("123users").is_err()); // starts with number
        assert!(TableName::new("users/admin").is_err()); // contains slash
        assert!(TableName::new("_schema").is_err()); // reserved
        assert!(TableName::new("a".repeat(65)).is_err()); // too long
    }

    #[test]
    fn test_row_key_valid() {
        assert!(RowKey::new("abc123").is_ok());
        assert!(RowKey::new("01ARZ3NDEKTSV4RRFFQ69G5FAV").is_ok()); // ULID
        assert!(RowKey::new("550e8400-e29b-41d4-a716-446655440000").is_ok()); // UUID with hyphens is valid
        assert!(RowKey::new("simple_key").is_ok());
    }

    #[test]
    fn test_row_key_generate() {
        let key1 = RowKey::generate();
        let key2 = RowKey::generate();
        assert_ne!(key1, key2);
        assert_eq!(key1.as_str().len(), 26); // ULID length
    }

    #[test]
    fn test_row_path_display() {
        let path = RowPath::new(
            TableName::new("users").unwrap(),
            RowKey::new("abc123").unwrap(),
        );
        assert_eq!(path.to_string(), "users/abc123.json");
    }

    #[test]
    fn test_branch_name_transaction() {
        let branch = BranchName::for_transaction("abc123");
        assert!(branch.is_transaction_branch());
        assert_eq!(branch.transaction_id(), Some("abc123"));
        assert_eq!(branch.as_ref_path(), "refs/heads/tx/abc123");
    }

    #[test]
    fn test_branch_name_main() {
        let branch = BranchName::main();
        assert!(!branch.is_transaction_branch());
        assert_eq!(branch.transaction_id(), None);
        assert_eq!(branch.as_ref_path(), "refs/heads/main");
    }
}
