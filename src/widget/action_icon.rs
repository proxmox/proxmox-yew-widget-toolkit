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
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    pub on_activate: Option<Callback<()>>,
}

impl ActionIcon {
    /// Create a new instance.
    pub fn new(icon_class: impl Into<Classes>) -> Self {
        yew::props!(Self {}).class(icon_class.into())
    }
}

impl From<ActionIcon> for VTag {
    fn from(mut props: ActionIcon) -> Self {
        let disabled = props.disabled;

        let tabindex = match props.tabindex {
            Some(tabindex) => format!("{tabindex}"),
            None => String::from("-1"),
        };

        props.set_attribute("role", "button");
        props.set_attribute("tabindex", (!props.disabled).then_some(tabindex));
        props.set_attribute("aria-label", props.aria_label.clone());

        props.add_class("pwt-action-icon");
        props.add_class(disabled.then_some("disabled"));

        props.set_onclick({
            let on_activate = props.on_activate.clone();
            move |event: MouseEvent| {
                event.stop_propagation();
                if disabled {
                    return;
                }
                if let Some(on_activate) = &on_activate {
                    on_activate.emit(());
                }
            }
        });

        props.set_onkeydown({
            let on_activate = props.on_activate.clone();
            move |event: KeyboardEvent| match event.key().as_ref() {
                "Enter" | " " => {
                    event.stop_propagation();
                    if disabled {
                        return;
                    }
                    if let Some(on_activate) = &on_activate {
                        on_activate.emit(());
                    }
                }
                _ => {}
            }
        });

        // suppress double click to avoid confusion when used inside tables/trees
        props.set_ondblclick(move |event: MouseEvent| {
            event.stop_propagation();
        });

        props
            .std_props
            .into_vtag("i".into(), None::<&str>, Some(props.listeners), None)
    }
}
