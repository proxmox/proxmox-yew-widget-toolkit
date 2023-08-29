// Fontawesome icons

use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

/// Font Awesome icons.
///
/// This is a helper to create Font Awesome icons from there name.
///
/// # Accessibility
///
/// This widget hides the icon from the accessibility tree
/// (`role="none"`).
///
/// If you are using the icon to convey meaning (rather than pure
/// decoration), the Font Awesome web site suggests to provide a text
/// alternative inside a `<span>` (or similar) element and include
/// appropriate CSS to visually hide that element while keeping it
/// accessible to assistive technologies.
#[derive(Properties, PartialEq, Clone)]
pub struct Fa {
    #[prop_or_default]
    #[doc(hidden)]
    pub class: Classes,
}

impl Fa {

    /// Create a new instrtance from the icon name.
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            class: classes!("fa", format!("fa-{}", name.as_ref())),
        }
    }

    /// Create a new instance using the passed CSS class name.
    pub fn from_class(class: impl Into<Classes>) -> Self {
        Self {
            class: class.into(),
        }
    }

    /// Builder style method to add a html class
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }

    /// Rotate icon with 8 steps.
    pub fn pulse(mut self) -> Self {
        self.add_class("fa-pulse");
        self
    }

    /// Rotatze icon. Works well with 'spinner', 'refresh' and 'cog'.
    pub fn spin(mut self) -> Self {
        self.add_class("fa-spin");
        self
    }

    /// Set icons at a fixed width.
    pub fn fixed_width(mut self) -> Self {
        self.add_class("fa-fw");
        self
    }

    /// Increase icon size by 33%.
    pub fn large(mut self) -> Self {
        self.add_class("fa-lg");
        self
    }

    /// Increase icon size 2 times.
    pub fn large_2x(mut self) -> Self {
        self.add_class("fa-2x");
        self
    }

    /// Increase icon size 3 times.
    pub fn large_3x(mut self) -> Self {
        self.add_class("fa-3x");
        self
    }

    /// Increase icon size 4 times.
    pub fn large_4x(mut self) -> Self {
        self.add_class("fa-4x");
        self
    }

    /// Increase icon size 5 times.
    pub fn large_5x(mut self) -> Self {
        self.add_class("fa-5x");
        self
    }
}

#[function_component(PwtFa)]
#[doc(hidden)]
pub fn pwt_fa(props: &Fa) -> Html {

    html!{
        <i class={props.class.clone()} role="none"/>
    }

}

impl Into<VNode> for Fa {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtFa>(Rc::new(self), None);
        VNode::from(comp)
    }
}
