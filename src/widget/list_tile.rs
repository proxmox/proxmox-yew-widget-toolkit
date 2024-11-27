use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::VTag;

use crate::props::{ListenersWrapper, WidgetStdProps};

use pwt_macros::{builder, widget};

/// List tile. A container with flex row layout.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::ListTile;
/// # fn create_list_tile() -> ListTile {
///   ListTile::new()
///       .with_child(html!{<span>{"A simple list tile"}</span>})
///       .with_child(html!{<span>{"second column"}</span>})
///       .interactive(true)
///       .disabled(false)
///       .style("justify-content", "space-between")
///       .class(pwt::css::ColorScheme::Primary)
/// # }
/// ```
#[widget(pwt=crate, @element, @container)]
#[derive(Default, Debug, Clone, PartialEq, Properties)]
#[builder]
pub struct ListTile {
    #[prop_or_default]
    #[builder]
    interactive: bool,

    #[prop_or_default]
    #[builder]
    disabled: bool,
}

impl ListTile {
    /// Creates new ListTile instance
    pub fn new() -> Self {
        yew::props! { Self {} }
    }

    /// Creates a new instance from existing properties
    pub fn from_widget_props(
        std_props: WidgetStdProps,
        listeners: Option<ListenersWrapper>,
    ) -> Self {
        yew::props! { Self { std_props, listeners: listeners.unwrap_or_default() } }
    }
}

impl Into<VTag> for ListTile {
    fn into(self) -> VTag {
        let classes = classes!(
            "pwt-list-tile",
            self.interactive.then(|| "pwt-interactive"),
            self.disabled.then(|| "disabled")
        );

        self.std_props.into_vtag(
            Cow::Borrowed("div"),
            Some(classes),
            Some(self.listeners),
            Some(self.children),
        )
    }
}
