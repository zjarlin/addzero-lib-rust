//! 基于 [`TypeId`] 的线程本地类型存储工具。
//!
//! [`ThreadLocalUtil`] 允许在当前线程内按类型存取任意 `'static` 值，内部使用
//! `thread_local!` + [`HashMap<TypeId, Box<dyn Any>>`] 实现，每个线程拥有独立的存储槽。
//!
//! ## 核心操作
//!
//! - [`ThreadLocalUtil::set`]：将值写入当前线程的存储，同类型旧值会被替换。
//! - [`ThreadLocalUtil::get`]：克隆读取（要求 `T: Clone`）。
//! - [`ThreadLocalUtil::with`]：通过闭包以共享引用访问存储中的值。
//! - [`ThreadLocalUtil::remove`] / [`take`]：取出并移除值（获取所有权）。
//! - [`ThreadLocalUtil::contains`]：检查当前线程是否存有某类型的值。
//! - [`ThreadLocalUtil::clear`]：清空当前线程的所有存储。
//!
//! ## 适用场景
//!
//! 在请求处理链路中传递隐式上下文（如请求 ID、用户信息），避免在每个函数签名中显式传参。
//! 注意：值不会跨线程传递，新线程需重新设置。

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
