//! 低代码文本校验内置。

use anyhow::{Result, bail};

/// 创建可组合的文本校验器。
pub fn text(value: &str) -> TextValidation<'_> {
    TextValidation { value }
}

/// 面向低代码规则的文本校验器。
///
/// 每个内置规则在失败时立即返回其配置的业务错误消息，成功时继续传递当前文本。
#[derive(Clone, Copy, Debug)]
pub struct TextValidation<'a> {
    value: &'a str,
}

impl<'a> TextValidation<'a> {
    /// 去除文本首尾空白，供后续规则和最终取值使用。
    pub fn trim(self) -> Self {
        Self {
            value: self.value.trim(),
        }
    }

    /// 要求文本不是空串或纯空白。
    pub fn not_blank(self, message: &str) -> Result<Self> {
        if self.value.trim().is_empty() {
            bail!("{message}");
        }
        Ok(self)
    }

    /// 要求文本以任一给定前缀开始。
    pub fn starts_with_any(self, prefixes: &[&str], message: &str) -> Result<Self> {
        if prefixes.iter().any(|prefix| self.value.starts_with(prefix)) {
            return Ok(self);
        }
        bail!("{message}");
    }

    /// 返回已通过所有规则的文本。
    pub fn value(self) -> &'a str {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_then_validates_built_in_rules() {
        let result = text("  postgresql://engine  ")
            .trim()
            .not_blank("连接串不能为空")
            .and_then(|value| {
                value.starts_with_any(&["postgres://", "postgresql://"], "只支持 PostgreSQL")
            })
            .map(TextValidation::value);
        let value = match result {
            Ok(value) => value,
            Err(error) => panic!("内置规则应校验成功: {error}"),
        };

        // 归一化后的文本应通过内置规则并保留可直接使用的值。
        assert_eq!(value, "postgresql://engine");
    }

    #[test]
    fn rejects_blank_and_unsupported_prefix() {
        // 空白值必须由 not_blank 内置阻断。
        assert!(text("  ").not_blank("连接串不能为空").is_err());
        // 非 PostgreSQL 协议必须由 starts_with_any 内置阻断。
        assert!(
            text("mysql://engine")
                .starts_with_any(&["postgres://", "postgresql://"], "只支持 PostgreSQL")
                .is_err()
        );
    }
}
