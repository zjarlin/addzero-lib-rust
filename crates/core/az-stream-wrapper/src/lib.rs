//! 对集合进行链式条件筛选的流式查询包装器。
//!
//! 提供类似 MyBatis-Plus LambdaQueryWrapper 的链式 API，通过 `eq`、`like`、`in`
//! 等字符串条件对 `Vec<T>` 进行过滤，并支持 `and`、`or`、`not` 等逻辑连接符组合谓词。
//!
//! # 核心类型
//!
//! - [`StreamWrapper<T>`] — 链式筛选的主入口，包装一个集合和可组合的谓词链。
//!
//! # 关键功能
//!
//! - **等值匹配**：[`StreamWrapper::eq`] 按精确字符串匹配过滤。
//! - **模糊匹配**：[`StreamWrapper::like`] 按大小写不敏感的子串匹配过滤。
//! - **集合匹配**：[`StreamWrapper::r#in`] 判断字段值是否在给定候选集合内。
//! - **逻辑组合**：[`StreamWrapper::or`]、[`StreamWrapper::not`]、[`StreamWrapper::negate`]
//!   控制后续条件与已有谓词的逻辑关系。
//! - **结果提取**：[`StreamWrapper::list`] 收集所有匹配项，[`StreamWrapper::one`] 返回第一个匹配项。
//!
//! # 快速开始
//!
//! ```rust
//! use az_stream_wrapper::lambdaquery;
//!
//! let items = vec!["apple", "banana", "avocado", "blueberry"];
//! let result = lambdaquery(&items)
//!     .like(true, |s: &&str| s, "av")
//!     .list();
//! assert_eq!(result, vec!["apple", "avocado"]);
//! ```

use std::sync::Arc;

type Predicate<T> = Arc<dyn Fn(&T) -> bool + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Junction {
    And,
    Or,
    Not,
}

pub struct StreamWrapper<T> {
    items: Vec<T>,
    predicate: Predicate<T>,
    next_junction: Junction,
}

impl<T: 'static> StreamWrapper<T> {
    pub fn lambdaquery<I>(items: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            items: items.into_iter().collect(),
            predicate: Arc::new(|_| true),
            next_junction: Junction::And,
        }
    }

    pub fn eq<F, S>(self, condition: bool, accessor: F, search: S) -> Self
    where
        F: for<'a> Fn(&'a T) -> &'a str + 'static,
        S: Into<String>,
    {
        let needle = search.into();
        self.with_filter(condition, move |item| accessor(item) == needle)
    }

    pub fn like<F, S>(self, condition: bool, accessor: F, search: S) -> Self
    where
        F: for<'a> Fn(&'a T) -> &'a str + 'static,
        S: Into<String>,
    {
        let needle = search.into().to_lowercase();
        self.with_filter(condition, move |item| {
            accessor(item).to_lowercase().contains(needle.as_str())
        })
    }

    pub fn r#in<F, I, S>(self, condition: bool, accessor: F, search_values: I) -> Self
    where
        F: for<'a> Fn(&'a T) -> &'a str + 'static,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let search_values = search_values
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();
        self.with_filter(condition, move |item| {
            let current = accessor(item);
            search_values.iter().any(|candidate| candidate == current)
        })
    }

    pub fn or(mut self) -> Self {
        self.next_junction = Junction::Or;
        self
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(mut self) -> Self {
        self.next_junction = Junction::Not;
        self
    }

    pub fn negate(mut self) -> Self {
        let predicate = Arc::clone(&self.predicate);
        self.predicate = Arc::new(move |item| !(predicate)(item));
        self
    }

    pub fn list(self) -> Vec<T> {
        let predicate = Arc::clone(&self.predicate);
        self.items
            .into_iter()
            .filter(move |item| (predicate)(item))
            .collect()
    }

    pub fn one(self) -> Option<T> {
        let predicate = Arc::clone(&self.predicate);
        self.items.into_iter().find(move |item| (predicate)(item))
    }

    fn with_filter<F>(mut self, condition: bool, filter: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        if !condition {
            return self;
        }

        let predicate = Arc::clone(&self.predicate);
        let next_predicate: Predicate<T> = match self.next_junction {
            Junction::And => Arc::new(move |item| predicate(item) && filter(item)),
            Junction::Or => Arc::new(move |item| predicate(item) || filter(item)),
            Junction::Not => Arc::new(move |item| predicate(item) && !filter(item)),
        };
        self.predicate = next_predicate;
        self.next_junction = Junction::And;
        self
    }
}

pub fn lambdaquery<T: 'static, I>(items: I) -> StreamWrapper<T>
where
    I: IntoIterator<Item = T>,
{
    StreamWrapper::lambdaquery(items)
}

#[macro_export]
macro_rules! stream_query {
    ($items:expr) => {
        $crate::lambdaquery($items)
    };
}
