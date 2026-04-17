use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static HOLDER: RefCell<HashMap<TypeId, Box<dyn Any>>> = RefCell::new(HashMap::new());
}

pub struct ThreadLocalUtil;

impl ThreadLocalUtil {
    pub fn set<T>(value: T)
    where
        T: 'static,
    {
        HOLDER.with(|holder| {
            holder
                .borrow_mut()
                .insert(TypeId::of::<T>(), Box::new(value) as Box<dyn Any>);
        });
    }

    pub fn get<T>() -> Option<T>
    where
        T: Clone + 'static,
    {
        Self::with::<T, _>(|value| value.cloned())
    }

    pub fn with<T, R>(f: impl FnOnce(Option<&T>) -> R) -> R
    where
        T: 'static,
    {
        HOLDER.with(|holder| {
            let holder = holder.borrow();
            let typed = holder
                .get(&TypeId::of::<T>())
                .and_then(|value| value.downcast_ref::<T>());
            f(typed)
        })
    }

    pub fn remove<T>() -> Option<T>
    where
        T: 'static,
    {
        HOLDER.with(|holder| {
            holder
                .borrow_mut()
                .remove(&TypeId::of::<T>())
                .and_then(|value| value.downcast::<T>().ok())
                .map(|value| *value)
        })
    }

    pub fn take<T>() -> Option<T>
    where
        T: 'static,
    {
        Self::remove::<T>()
    }

    pub fn contains<T>() -> bool
    where
        T: 'static,
    {
        HOLDER.with(|holder| holder.borrow().contains_key(&TypeId::of::<T>()))
    }

    pub fn clear() {
        HOLDER.with(|holder| holder.borrow_mut().clear());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn set_get_and_remove_work_for_single_type() {
        ThreadLocalUtil::clear();
        ThreadLocalUtil::set::<String>("addzero".to_owned());

        assert_eq!(ThreadLocalUtil::get::<String>(), Some("addzero".to_owned()));
        assert!(ThreadLocalUtil::contains::<String>());
        assert_eq!(
            ThreadLocalUtil::remove::<String>(),
            Some("addzero".to_owned())
        );
        assert_eq!(ThreadLocalUtil::get::<String>(), None);
    }

    #[test]
    fn values_are_separated_by_type() {
        ThreadLocalUtil::clear();
        ThreadLocalUtil::set::<String>("hello".to_owned());
        ThreadLocalUtil::set::<i32>(42);

        assert_eq!(ThreadLocalUtil::get::<String>(), Some("hello".to_owned()));
        assert_eq!(ThreadLocalUtil::get::<i32>(), Some(42));
    }

    #[test]
    fn with_borrows_without_clone_requirement() {
        ThreadLocalUtil::clear();
        ThreadLocalUtil::set::<Vec<i32>>(vec![1, 2, 3]);

        let sum = ThreadLocalUtil::with::<Vec<i32>, _>(|value| {
            value.map(|numbers| numbers.iter().sum::<i32>())
        });

        assert_eq!(sum, Some(6));
    }

    #[test]
    fn thread_local_values_are_isolated_per_thread() {
        ThreadLocalUtil::clear();
        ThreadLocalUtil::set::<String>("main".to_owned());

        let worker = thread::spawn(|| {
            assert_eq!(ThreadLocalUtil::get::<String>(), None);
            ThreadLocalUtil::set::<String>("worker".to_owned());
            ThreadLocalUtil::get::<String>()
        });

        assert_eq!(
            worker.join().expect("thread should join"),
            Some("worker".to_owned())
        );
        assert_eq!(ThreadLocalUtil::get::<String>(), Some("main".to_owned()));
    }
}
