use std::borrow::Cow;

use web_sys::HtmlElement;

use yew::prelude::*;
use yew::virtual_dom::{ApplyAttributeAs, Listeners, VList, VTag};
use yew::html::IntoPropValue;

use pwt_macros::widget;

#[widget(crate::widget::PwtButton, @element)]
#[derive(Properties, PartialEq, Clone)]
pub struct Button {
    pub text: Option<AttrValue>,
    pub icon_class: Option<Classes>,

    pub tabindex: Option<i32>,
    pub aria_label: Option<AttrValue>,
    pub placeholder: Option<AttrValue>,

    #[prop_or_default]
    pub autofocus: bool,

    #[prop_or_default]
    pub disabled: bool,

    #[prop_or_default]
    pub pressed: bool,
}

impl Button {
    /// Create a new button.
    pub fn new(text: impl IntoPropValue<Option<AttrValue>>) -> Self {
        yew::props!(Self {
            text: text.into_prop_value()
        })
    }

    /// Builder style method to set the html aria-label attribute
    pub fn aria_label(mut self, label: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_aria_label(label);
        self
    }

    /// Method to set the html aria-label attribute
    pub fn set_aria_label(&mut self, label: impl IntoPropValue<Option<AttrValue>>) {
        self.aria_label = label.into_prop_value();
    }

    /// Builder style method to set the html tabindex attribute
    pub fn tabindex(mut self, index: impl IntoPropValue<Option<i32>>) -> Self {
        self.set_tabindex(index);
        self
    }

    /// Method to set the html tabindex attribute
    pub fn set_tabindex(&mut self, index: impl IntoPropValue<Option<i32>>) {
        self.tabindex = index.into_prop_value();
    }

    /// Builder style method to set the autofocus flag
    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.set_autofocus(autofocus);
        self
    }

    /// Method to set the autofocus flag
    pub fn set_autofocus(&mut self, autofocus: bool) {
        self.autofocus = autofocus;
    }

    /// Builder style method to set the disabled flag
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.set_disabled(disabled);
        self
    }

    /// Method to set the disabled flag
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Builder style method to set the pressed flag
    pub fn pressed(mut self, pressed: bool) -> Self {
        self.set_pressed(pressed);
        self
    }

    /// Method to set the pressed flag
    pub fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    pub fn new_icon(icon_class: impl Into<Classes>) -> Self {
        yew::props!(Self {}).icon_class(icon_class)
    }

    /// Create a Refresh/Reload button
    pub fn refresh(loading: bool) -> Self {
        let icon_class = if loading {
            "fa fa-fw fa-spinner fa-pulse"
        } else {
            "fa fa-fw fa-refresh"
        };
        Self::new_icon(icon_class)
            .aria_label("Refresh")
            .disabled(loading)
    }

    pub fn icon_class(mut self, icon_class: impl Into<Classes>) -> Self {
        self.set_icon_class(icon_class);
        self
    }

    pub fn set_icon_class(&mut self, icon_class: impl Into<Classes>) {
        self.icon_class = Some(icon_class.into());
    }
}

#[doc(hidden)]
pub struct PwtButton;

impl Component for PwtButton {
    type Message = ();
    type Properties = Button;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let props = ctx.props();
        if first_render && ctx.props().autofocus {
            if let Some(button) = props.std_props.node_ref.cast::<HtmlElement>() {
                let _ = button.focus();
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut attributes = props.std_props.cumulate_attributes(Some(classes!(
            "pwt-button",
            if props.pressed { "pressed" } else { "" }
        )));
        let attr_map = attributes.get_mut_index_map();

        if props.disabled {
            attr_map.insert(AttrValue::Static("disabled"), (AttrValue::Static(""), ApplyAttributeAs::Attribute));
        }
        if props.autofocus {
            attr_map.insert(AttrValue::Static("autofocus"), (AttrValue::Static(""), ApplyAttributeAs::Attribute));
        }
        if let Some(ref aria_label) = props.aria_label {
            attr_map.insert(AttrValue::Static("aria-label"), (aria_label.clone(), ApplyAttributeAs::Attribute));
        }
        if let Some(ref tabindex) = props.tabindex {
            attr_map.insert(AttrValue::Static("tabindex"), (tabindex.to_string().into(), ApplyAttributeAs::Attribute));
        }

        let mut children = Vec::new();

        let has_text = props.text.as_ref().map(|t| !t.is_empty()).unwrap_or(false);

        if let Some(icon_class) = &props.icon_class {
            if !icon_class.is_empty() {
                children.push(html!{
                    <i role="none" aria-hidden="true" class={classes!(
                        icon_class.clone(),
                        has_text.then(|| "pwt-me-2"),
                    )}></i>
                })
            }
        }

        if let Some(text) = &props.text {
            children.push((&*text).into());
        }

        let listeners = Listeners::Pending(
            props.listeners.listeners.clone().into_boxed_slice()
        );

        VTag::__new_other(
            Cow::Borrowed("button"),
            props.std_props.node_ref.clone(),
            props.std_props.key.clone(),
            attributes,
            listeners,
            VList::with_children(children, None),
        ).into()
    }
}
