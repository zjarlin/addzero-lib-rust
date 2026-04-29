use dioxus::prelude::Element;

use crate::admin::AdminAction;

#[derive(Clone)]
pub struct AdminTopbar<R> {
    pub brand: Option<Element>,
    pub eyebrow: Option<String>,
    pub title: String,
    pub left: Vec<AdminAction<R>>,
    pub right: Vec<AdminAction<R>>,
}

impl<R: PartialEq> PartialEq for AdminTopbar<R> {
    fn eq(&self, other: &Self) -> bool {
        self.brand == other.brand
            && self.eyebrow == other.eyebrow
            && self.title == other.title
            && self.left == other.left
            && self.right == other.right
    }
}
