use std::{fmt, slice};

use anyhow::Error;

/// 收集一个阶段内允许继续执行的非致命诊断。
///
/// 诊断按发生顺序保存。无法继续或会使结果失真的错误仍应通过
/// `anyhow::Result` 和 `?` 立即返回，不应放入本收集器。
#[derive(Debug, Default)]
pub struct Diagnostics {
    errors: Vec<Error>,
}

impl Diagnostics {
    /// 当前是否没有收集到诊断。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// 返回已收集的诊断数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// 按发生顺序记录一个非致命错误。
    pub fn record<E>(&mut self, error: E)
    where
        E: Into<Error>,
    {
        self.errors.push(error.into());
    }

    /// 返回成功值；失败时记录错误并返回 `None`。
    ///
    /// 调用方应只在存在明确领域降级策略时处理返回的 `None`。
    #[must_use]
    pub fn capture<T, E>(&mut self, result: Result<T, E>) -> Option<T>
    where
        E: Into<Error>,
    {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.record(error);
                None
            }
        }
    }

    /// 返回成功值；失败时记录错误并执行显式降级闭包。
    ///
    /// 降级闭包仅在错误路径执行，且应返回领域上安全、可解释的值。
    pub fn recover<T, E, F>(&mut self, result: Result<T, E>, fallback: F) -> T
    where
        E: Into<Error>,
        F: FnOnce() -> T,
    {
        match result {
            Ok(value) => value,
            Err(error) => {
                self.record(error);
                fallback()
            }
        }
    }

    /// 按发生顺序迭代全部诊断。
    pub fn iter(&self) -> slice::Iter<'_, Error> {
        self.errors.iter()
    }

    /// 消耗收集器并返回全部原始诊断。
    #[must_use]
    pub fn into_errors(self) -> Vec<Error> {
        self.errors
    }

    /// 在阶段边界决定是否接受结果。
    ///
    /// 没有诊断时返回给定值；存在诊断时把完整收集器包装为一个
    /// `anyhow::Error`，调用方仍可向下转型为 [`Diagnostics`] 检查各项错误。
    pub fn finish<T>(self, value: T) -> anyhow::Result<T> {
        if self.is_empty() {
            return Ok(value);
        }

        Err(Error::new(self))
    }

    /// 在无返回值的阶段边界聚合全部诊断。
    pub fn into_result(self) -> anyhow::Result<()> {
        self.finish(())
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "收集到 {} 个可恢复诊断", self.len())?;

        for (index, error) in self.errors.iter().enumerate() {
            write!(formatter, "\n[{}] {error:#}", index + 1)?;
        }

        Ok(())
    }
}

impl std::error::Error for Diagnostics {}
