use wasm_bindgen::JsCast;
use web_sys::Event;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::VTag;

use crate::prelude::*;

use pwt_macros::{builder, widget};

/// A clickable icon. Like [Button](super::Button) without any decoration (inline element).
///
/// This component is useful in data tables because it is visually lighter than a button.
#[widget(pwt=crate, @element)]
#[builder]
#[derive(Properties, PartialEq, Clone)]
pub struct ActionIcon {
    /// Html tabindex (defaults to -1)
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub tabindex: Option<i32>,

    /// Aria label
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub aria_label: Option<AttrValue>,

    /// Disable flag
    #[builder]
    #[prop_or_default]
    pub disabled: bool,

    /// Activate callback (click, enter, space)
    #[builder_cb(IntoEventCallback, into_event_callback, Event)]
    #[prop_or_default]
    pub on_activate: Option<Callback<Event>>,
}

impl ActionIcon {
    /// Create a new instance.
    pub fn new(icon_class: impl Into<Classes>) -> Self {
        yew::props!(Self {}).class(icon_class.into())
    }
}

impl IntoVTag for ActionIcon {
    fn into_vtag_with_ref(mut self, node_ref: NodeRef) -> VTag {
        let disabled = self.disabled;

        let tabindex = match self.tabindex {
            Some(tabindex) => format!("{tabindex}"),
            None => String::from("-1"),
        };

        self.set_attribute("role", "button");
        self.set_attribute("tabindex", (!self.disabled).then_some(tabindex));
        self.set_attribute("aria-label", self.aria_label.clone());

        self.add_class("pwt-action-icon");
        self.add_class(disabled.then_some("disabled"));

        self.add_onclick({
            let on_activate = self.on_activate.clone();
            move |event: MouseEvent| {
                event.stop_propagation();
                if disabled {
                    return;
                }
                if let Some(on_activate) = &on_activate {
                    on_activate.emit(event.unchecked_into());
                }
            }
        });

        self.add_onkeydown({
            let on_activate = self.on_activate.clone();
            move |event: KeyboardEvent| match event.key().as_ref() {
                "Enter" | " " => {
                    event.stop_propagation();
                    if disabled {
                        return;
                    }
                    if let Some(on_activate) = &on_activate {
                        on_activate.emit(event.unchecked_into());
                    }
                }
                _ => {}
            }
        });

        // suppress double click to avoid confusion when used inside tables/trees
        self.add_ondblclick(move |event: MouseEvent| {
            event.stop_propagation();
        });

        self.std_props.into_vtag(
            "i".into(),
            node_ref,
            None::<&str>,
            Some(self.listeners),
            None,
        )
    }
}
