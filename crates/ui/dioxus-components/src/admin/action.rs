use std::rc::Rc;

use crate::Tone;

#[derive(Clone)]
pub struct AdminCommand<R> {
    pub to: Option<R>,
    pub on_run: Option<Rc<dyn Fn()>>,
}

impl<R: PartialEq> PartialEq for AdminCommand<R> {
    fn eq(&self, other: &Self) -> bool {
        self.to == other.to
    }
}

impl<R> Default for AdminCommand<R> {
    fn default() -> Self {
        Self {
            to: None,
            on_run: None,
        }
    }
}

impl<R> AdminCommand<R> {
    pub fn run(on_run: impl Fn() + 'static) -> Self {
        Self {
            to: None,
            on_run: Some(Rc::new(on_run)),
        }
    }

    pub fn run_to(to: R, on_run: impl Fn() + 'static) -> Self {
        Self {
            to: Some(to),
            on_run: Some(Rc::new(on_run)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AdminActionIcon {
    Bell,
    LogOut,
    Moon,
    Search,
    Sun,
}

#[derive(Clone)]
pub struct AdminAction<R> {
    pub class: String,
    pub title: String,
    pub tone: Option<Tone>,
    pub icon: AdminActionIcon,
    pub cmd: AdminCommand<R>,
}

impl<R: PartialEq> PartialEq for AdminAction<R> {
    fn eq(&self, other: &Self) -> bool {
        self.class == other.class
            && self.title == other.title
            && self.tone == other.tone
            && self.icon == other.icon
            && self.cmd == other.cmd
    }
}
