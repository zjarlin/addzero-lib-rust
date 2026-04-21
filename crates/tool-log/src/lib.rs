use std::any::{TypeId, type_name, type_name_of_val};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

fn logger_map() -> &'static RwLock<HashMap<TypeId, &'static str>> {
    static LOGGER_MAP: OnceLock<RwLock<HashMap<TypeId, &'static str>>> = OnceLock::new();
    LOGGER_MAP.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn logger_target<T>() -> &'static str
where
    T: 'static,
{
    let type_id = TypeId::of::<T>();
    if let Some(target) = logger_map()
        .read()
        .expect("logger cache should be readable")
        .get(&type_id)
        .copied()
    {
        return target;
    }

    let target = type_name::<T>();
    logger_map()
        .write()
        .expect("logger cache should be writable")
        .insert(type_id, target);
    target
}

pub fn value_logger_target<T>(value: &T) -> &'static str
where
    T: ?Sized,
{
    type_name_of_val(value)
}

#[macro_export]
macro_rules! trace_for {
    ($value:expr, $($arg:tt)+) => {
        ::log::trace!(target: $crate::value_logger_target(&$value), $($arg)+)
    };
}

#[macro_export]
macro_rules! debug_for {
    ($value:expr, $($arg:tt)+) => {
        ::log::debug!(target: $crate::value_logger_target(&$value), $($arg)+)
    };
}

#[macro_export]
macro_rules! info_for {
    ($value:expr, $($arg:tt)+) => {
        ::log::info!(target: $crate::value_logger_target(&$value), $($arg)+)
    };
}

#[macro_export]
macro_rules! warn_for {
    ($value:expr, $($arg:tt)+) => {
        ::log::warn!(target: $crate::value_logger_target(&$value), $($arg)+)
    };
}

#[macro_export]
macro_rules! error_for {
    ($value:expr, $($arg:tt)+) => {
        ::log::error!(target: $crate::value_logger_target(&$value), $($arg)+)
    };
}
