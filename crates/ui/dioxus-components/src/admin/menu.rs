use std::rc::Rc;

#[derive(Clone)]
pub struct AdminMenu<R> {
    pub label: String,
    pub to: Option<R>,
    pub on_select: Option<Rc<dyn Fn()>>,
    pub children: Vec<AdminMenu<R>>,
    pub is_active: Rc<dyn Fn(&R) -> bool>,
}

impl<R: PartialEq> PartialEq for AdminMenu<R> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && self.to == other.to && self.children == other.children
    }
}

impl<R> AdminMenu<R> {
    pub fn leaf(label: impl Into<String>, to: R, is_active: impl Fn(&R) -> bool + 'static) -> Self {
        Self {
            label: label.into(),
            to: Some(to),
            on_select: None,
            children: Vec::new(),
            is_active: Rc::new(is_active),
        }
    }

    pub fn branch(
        label: impl Into<String>,
        to: Option<R>,
        children: Vec<AdminMenu<R>>,
        is_active: impl Fn(&R) -> bool + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            to,
            on_select: None,
            children,
            is_active: Rc::new(is_active),
        }
    }
}

#[derive(Clone)]
pub struct AdminSection<R> {
    pub label: String,
    pub menus: Vec<AdminMenu<R>>,
}

impl<R: PartialEq> PartialEq for AdminSection<R> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && self.menus == other.menus
    }
}
