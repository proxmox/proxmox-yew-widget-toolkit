use std::borrow::Cow;

use web_sys::HtmlElement;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};
use yew::html::IntoPropValue;

use crate::widget::prelude::*;

use pwt_macros::widget;

#[widget(crate::widget::PwtButton, @input, @element)]
#[derive(Properties, PartialEq, Clone)]
pub struct Button {
    pub text: Option<AttrValue>,
    pub icon_class: Option<Classes>,
}

impl Button {

    pub fn new(text: impl IntoPropValue<Option<AttrValue>>) -> Self {
        yew::props!(Self { text: text.into_prop_value() })
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

pub struct PwtButton;

impl Component for PwtButton {
    type Message = ();
    type Properties = Button;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let props = ctx.props();
        if first_render && ctx.props().input_props.autofocus {
            if let Some(button) = props.std_props.node_ref.cast::<HtmlElement>() {
                let _ = button.focus();
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut attributes = props.std_props.cumulate_attributes(Some("pwt-button"));
        let attr_map = attributes.get_mut_index_map();

        props.input_props.cumulate_attributes(attr_map);

        let mut children = Vec::new();

        let has_text = props.text.as_ref().map(|t| !t.is_empty()).unwrap_or(false);

        if let Some(icon_class) = &props.icon_class {
            children.push(html!{
                <i class={classes!(
                    icon_class.clone(),
                    has_text.then(|| "pwt-me-2"),
                )} role="status" aria_hidden="true"></i>
            })
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
