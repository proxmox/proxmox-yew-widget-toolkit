// Fontawesome icons

use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::props::{AsClassesMut, AsCssStylesMut, CssStyles, WidgetStyleBuilder};
use crate::touch::prelude::{CssMarginBuilder, CssPaddingBuilder};

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

    #[prop_or_default]
    #[doc(hidden)]
    pub style: CssStyles,
}

impl Fa {
    /// Create a new instrtance from the icon name.
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            class: classes!("fa", format!("fa-{}", name.as_ref())),
            style: Default::default(),
        }
    }

    /// Create a new instance using the passed CSS class name.
    pub fn from_class(class: impl Into<Classes>) -> Self {
        Self {
            class: class.into(),
            style: Default::default(),
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

impl AsClassesMut for Fa {
    fn as_classes_mut(&mut self) -> &mut yew::Classes {
        &mut self.class
    }
}

impl AsCssStylesMut for Fa {
    fn as_css_styles_mut(&mut self) -> &mut CssStyles {
        &mut self.style
    }
}

impl CssPaddingBuilder for Fa {}
impl CssMarginBuilder for Fa {}
impl WidgetStyleBuilder for Fa {}

#[function_component(PwtFa)]
#[doc(hidden)]
pub fn pwt_fa(props: &Fa) -> Html {
    let style = props.style.compile_style_attribute(None);
    html! {
        <i class={props.class.clone()} {style} role="none"/>
    }
}

impl From<Fa> for VNode {
    fn from(val: Fa) -> Self {
        let comp = VComp::new::<PwtFa>(Rc::new(val), None);
        VNode::from(comp)
    }
}
