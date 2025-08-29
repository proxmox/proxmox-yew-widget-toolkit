use std::borrow::Cow;

use wasm_bindgen::JsCast;
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::VTag;

use crate::props::{EventSubscriber, ListenersWrapper, WidgetBuilder, WidgetStdProps};

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

impl From<ListTile> for VTag {
    fn from(mut val: ListTile) -> Self {
        let classes = classes!(
            "pwt-list-tile",
            val.interactive.then_some("pwt-interactive"),
            val.disabled.then_some("disabled")
        );

        if !val.disabled {
            if let Some(on_activate) = val.on_activate.clone() {
                val.set_tabindex(0);
                val.add_onclick({
                    let on_activate = on_activate.clone();
                    move |event: MouseEvent| {
                        event.stop_propagation();
                        on_activate.emit(event.unchecked_into());
                    }
                });
                val.add_onkeydown({
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

        val.std_props.into_vtag(
            Cow::Borrowed("div"),
            Some(classes),
            Some(val.listeners),
            Some(val.children),
        )
    }
}
