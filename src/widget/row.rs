use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};

use pwt_macros::widget;

use crate::css::Display;
use crate::prelude::*;
use crate::props::{AsClassesMut, AsCssStylesMut, PwtSpace};

/// Horizontal container with flex layout.
///
/// Creates a container with horizontal flexbox layout.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::{Row, Button};
/// # use pwt::css::AlignItems;
/// Row::new()
///   .padding(2)
///   .gap(2)
///   .class(AlignItems::Center)
///   .with_child(Button::new("Button1"))
///   .with_child(Button::new("Button2"))
///   .with_flex_spacer() // white space between left and right side
///   .with_child(Button::new("Button3"))
/// # ;
/// ```
#[widget(pwt=crate, @element, @container)]
#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Row {}

impl Default for Row {
    fn default() -> Self {
        Self::new()
    }
}

impl Row {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {}).class(Display::Flex)
    }

    /// Builder style method to add a CSS class to set gap between children.
    ///
    /// The default CSS template defines utility classes for gaps (`pwt-gap-{gap}`).
    pub fn gap(mut self, gap: impl Into<PwtSpace>) -> Self {
        self.add_gap(gap);
        self
    }

    /// Method to add a CSS class to set gap between children.
    pub fn add_gap(&mut self, gap: impl Into<PwtSpace>) {
        match gap.into() {
            PwtSpace::None => {}
            PwtSpace::Pwt(factor) if factor <= 4 => {
                self.as_classes_mut().push(format!("pwt-gap-{}", factor));
            }
            space => self.as_css_styles_mut().set_style("gap", space.to_string()),
        }
    }

    /// Builder style method to add a flexible spacer.
    ///
    /// A flexible spacer is a empty child with CSS `flex: 1 1 auto;`.
    pub fn with_flex_spacer(mut self) -> Self {
        self.add_flex_spacer();
        self
    }

    /// Method to add a flexible spacer.
    pub fn add_flex_spacer(&mut self) {
        self.add_child(html! {<div class="pwt-flex-fill"/>});
    }
}

impl From<Row> for VTag {
    fn from(val: Row) -> Self {
        let attributes = val.std_props.cumulate_attributes(None::<&str>);

        let listeners = Listeners::Pending(val.listeners.listeners.into_boxed_slice());

        let children = VList::with_children(val.children, None);

        VTag::__new_other(
            Cow::Borrowed("div"),
            val.std_props.node_ref,
            val.std_props.key,
            attributes,
            listeners,
            children.into(),
        )
    }
}
