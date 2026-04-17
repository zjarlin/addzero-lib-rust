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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct User {
        name: String,
        city: String,
        role: String,
    }

    fn users() -> Vec<User> {
        vec![
            User {
                name: "Alice".to_owned(),
                city: "Shanghai".to_owned(),
                role: "user".to_owned(),
            },
            User {
                name: "Bob".to_owned(),
                city: "Beijing".to_owned(),
                role: "admin".to_owned(),
            },
            User {
                name: "Alina".to_owned(),
                city: "Shenzhen".to_owned(),
                role: "user".to_owned(),
            },
        ]
    }

    #[test]
    fn eq_and_like_filters_are_chainable() {
        let result = lambdaquery(users())
            .eq(true, |user| user.city.as_str(), "Shanghai")
            .like(true, |user| user.name.as_str(), "ali")
            .list();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Alice");
    }

    #[test]
    fn in_filter_matches_any_candidate() {
        let result = lambdaquery(users())
            .r#in(true, |user| user.role.as_str(), ["admin", "owner"])
            .list();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Bob");
    }

    #[test]
    fn or_only_affects_next_condition_then_resets_to_and() {
        let result = lambdaquery(users())
            .eq(true, |user| user.city.as_str(), "Beijing")
            .or()
            .like(true, |user| user.name.as_str(), "ali")
            .eq(true, |user| user.role.as_str(), "user")
            .list();

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|user| user.role == "user"));
    }

    #[test]
    fn not_only_negates_next_condition() {
        let result = lambdaquery(users())
            .not()
            .eq(true, |user| user.city.as_str(), "Beijing")
            .list();

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|user| user.city != "Beijing"));
    }

    #[test]
    fn negate_flips_the_whole_accumulated_predicate() {
        let result = lambdaquery(users())
            .eq(true, |user| user.role.as_str(), "admin")
            .negate()
            .list();

        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|user| user.role != "admin"));
    }

    #[test]
    fn one_returns_first_matching_item() {
        let user = stream_query!(users())
            .like(true, |item| item.name.as_str(), "bob")
            .one();

        assert_eq!(user.map(|item| item.name), Some("Bob".to_owned()));
    }
}
