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
