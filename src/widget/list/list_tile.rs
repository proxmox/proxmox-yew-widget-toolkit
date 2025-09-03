use std::borrow::Cow;

use wasm_bindgen::JsCast;
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::VTag;

use crate::props::{EventSubscriber, IntoVTag, ListenersWrapper, WidgetBuilder, WidgetStdProps};

use pwt_macros::{builder, widget};

/// List tile. A container with grid/subgrid layout.
///
/// This is meant to be used inside [List](crate::widget::List).
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::{List, ListTile};
/// # fn create_list_tile() -> List {
///     List::new(100, |pos| {
///         ListTile::new()
///             .with_child(html!{<span>{format!("{pos}")}</span>})
///             .with_child(html!{<span>{"A simple list tile"}</span>})
///             .with_child(html!{<span>{"third column"}</span>})
///             .interactive(true)
///             .disabled(false)
///             .class(pwt::css::ColorScheme::Primary)
///     })
///     .grid_template_columns("auto 1fr auto")
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

    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    force_height: Option<u32>,

    /// Activate callback (click, enter, space)
    ///
    /// This also sets the tabindex property to "0" to enable keyboard focus.
    #[builder_cb(IntoEventCallback, into_event_callback, Event)]
    #[prop_or_default]
    on_activate: Option<Callback<Event>>,
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

impl IntoVTag for ListTile {
    fn into_vtag_with_ref(mut self, node_ref: NodeRef) -> VTag {
        let classes = classes!(
            "pwt-list-tile",
            self.interactive.then_some("pwt-interactive"),
            self.disabled.then_some("disabled")
        );

        if !self.disabled {
            if let Some(on_activate) = self.on_activate.clone() {
                self.set_tabindex(0);
                self.add_onclick({
                    let on_activate = on_activate.clone();
                    move |event: MouseEvent| {
                        event.stop_propagation();
                        on_activate.emit(event.unchecked_into());
                    }
                });
                self.add_onkeydown({
                    let on_activate = on_activate.clone();
                    move |event: KeyboardEvent| match event.key().as_str() {
                        "Enter" | " " => {
                            event.stop_propagation();
                            on_activate.emit(event.unchecked_into());
                        }
                        _ => {}
                    }
                });
            }
        }

        self.std_props.into_vtag(
            Cow::Borrowed("div"),
            node_ref,
            Some(classes),
            Some(self.listeners),
            Some(self.children),
        )
    }
}
