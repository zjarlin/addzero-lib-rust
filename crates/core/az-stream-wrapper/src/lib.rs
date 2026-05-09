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
