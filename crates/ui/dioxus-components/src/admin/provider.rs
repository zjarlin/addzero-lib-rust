use std::rc::Rc;

use crate::admin::{AdminSection, AdminTopbar};

pub type SharedAdminProvider<R> = Rc<dyn AdminProvider<R>>;

pub trait AdminProvider<R>: 'static {
    fn topbar(&self, current: &R) -> AdminTopbar<R>;

    fn menu(&self, current: &R) -> Vec<AdminSection<R>>;
}
