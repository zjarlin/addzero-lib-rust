use std::rc::Rc;

use dioxus::prelude::Element;

use crate::admin::{AdminSection, AdminTopbar};

pub type SharedAdminShellProvider<R> = Rc<dyn AdminShellProvider<R>>;

#[derive(Clone)]
pub struct AdminShellState<R> {
    pub topbar: AdminTopbar<R>,
    pub menu: Vec<AdminSection<R>>,
    pub right_panel: Option<Element>,
}

pub trait AdminShellProvider<R>: 'static {
    fn shell(&self, current: &R) -> AdminShellState<R>;
}
