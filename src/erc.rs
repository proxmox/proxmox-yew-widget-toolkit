use std::rc::Rc;
use std::fmt::{self, Display, Formatter};

use yew::html::ImplicitClone;

#[derive(Debug)]
pub struct Erc<T>(pub Rc<T>);

impl<T> Clone for Erc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Erc<T> {

    pub fn new(data: T) -> Self {
        Self(Rc::new(data))
    }

}

impl<T> PartialEq for Erc<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> std::ops::Deref for Erc<T> {
    type Target = Rc<T>;

    fn deref(&self) -> &Rc<T> {
        &self.0
    }
}

impl<T> ImplicitClone for Erc<T> {}

impl<T: Display> Display for Erc<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
