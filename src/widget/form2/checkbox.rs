use std::rc::Rc;
use std::borrow::Cow;

use serde_json::Value;

use yew::prelude::*;
use yew::virtual_dom::{ApplyAttributeAs, Listeners, VList, VTag};
use yew::html::{IntoEventCallback, IntoPropValue};

use pwt_macros::widget;

use crate::widget::form::ValidateFn;
use super::{FieldState, FieldStateMsg};

/// Checkbox input element.
#[widget(pwt=crate, comp=PwtCheckbox, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Checkbox {
    /// Checkbox value (default is "on").
    pub value: Option<AttrValue>,
    /// Force value.
    pub checked: Option<bool>,
    /// Default value.
    pub default: Option<bool>,
    /// Change callback
    pub on_change: Option<Callback<String>>,
    //fixme: on_input()
}

impl Checkbox {

    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the value.
    pub fn value(mut self, value: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_value(value);
        self
    }

    /// Method to set the value.
    pub fn set_value(&mut self, value: impl IntoPropValue<Option<AttrValue>>) {
        self.value = value.into_prop_value();
    }

    /// Builder style method to set the checked value.
    pub fn checked(mut self, checked: impl IntoPropValue<Option<bool>>) -> Self {
        self.set_checked(checked);
        self
    }

    /// Method to set the checked value.
    pub fn set_checked(&mut self, checked: impl IntoPropValue<Option<bool>>) {
        self.checked = checked.into_prop_value();
    }

    /// Builder style method to set the field default value.
    pub fn default(mut self, default: impl IntoPropValue<Option<bool>>) -> Self {
        self.set_default(default);
        self
    }

    /// Method to set the field default value.
    pub fn set_default(&mut self, default: impl IntoPropValue<Option<bool>>) {
        self.default = default.into_prop_value();
    }

    /// Builder style method to set the on_change callback
    pub fn on_change(mut self, cb: impl IntoEventCallback<String>) -> Self {
        self.on_change = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Toggle,
    StateUpdate(FieldStateMsg),
}

fn create_checkbox_validation_cb() -> ValidateFn<Value> {
    ValidateFn::new(move |_value: &Value| {
        Ok(())
    })
}

#[doc(hidden)]
pub struct PwtCheckbox {
    state: FieldState,

}

impl Component for PwtCheckbox {
    type Message = Msg;
    type Properties = Checkbox;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let real_validate = create_checkbox_validation_cb();

        let on_change = match &props.on_change {
            Some(on_change) => Some(Callback::from({
                let on_change = on_change.clone();
                move |value: Value| {
                    on_change.emit(value.as_str().unwrap_or("").to_string());
                }
            })),
            None => None,
        };

        let state = FieldState::create(
            ctx,
            &props.input_props,
            ctx.link().callback(Msg::StateUpdate),
            on_change,
            real_validate.clone(),
        );

        let mut me = Self { state };

        let on_value = props.value.as_deref().unwrap_or("on").to_string();

        let default = match props.default {
            Some(true) => on_value.clone(),
            _ => String::new(),
        };

        if let Some(name) = &props.input_props.name {
            me.state.register_field(&props.input_props, default.clone(), default, false);
            if props.checked.is_some() {
                log::error!("Checkbox '{name}' is named - unable to force checked.");
            }
         } else {
            let value = match props.checked {
                Some(true) => on_value.clone(),
                _ => String::new(),
            };
            me.state.force_value(value, None);
        }

        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::StateUpdate(state_msg) => {
                let on_value = props.value.as_deref().unwrap_or("on").to_string();
                let default = match props.default {
                    Some(true) => on_value.clone(),
                    _ => String::new(),
                };
                self.state.update_hook(&props.input_props, state_msg, default, false)
            }
            Msg::Toggle => {
                if props.input_props.disabled { return true; }
                let on_value = props.value.as_deref().unwrap_or("on").to_string();
                let (value, _) = self.state.get_field_data();
                let new_value = if value == on_value {
                    String::new()
                } else {
                    on_value
                };

                self.state.set_value(new_value);
                //fixme
                //if let Some(on_input) = &props.on_input {
                //  on_input.emit(value);
                //}
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if let Some(name) = &props.input_props.name {
            if props.checked.is_some() {
                log::error!("Checkbox '{name}' is named - unable to force checked.");
            }
        } else {
            if props.checked != old_props.checked {
                let on_value = props.value.as_deref().unwrap_or("on").to_string();
                let value = match props.checked {
                    Some(true) => on_value.clone(),
                    _ => String::new(),
                };
                self.state.force_value(value.to_string(), None);
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let disabled = props.input_props.disabled;

        let on_value = props.value.as_deref().unwrap_or("on").to_string();
        let (value, _) = self.state.get_field_data();
        let checked = value == on_value;

        let class = classes!(
            "pwt-checkbox",
            "fa",
            "fa-check",
            checked.then(|| "checked"),
            disabled.then(|| "disabled"),
        );

        let mut attributes = props.std_props.cumulate_attributes(Some(class));
        let attr_map = attributes.get_mut_index_map();
        props.input_props.cumulate_attributes(attr_map);

        attr_map.insert(AttrValue::Static("role"), (AttrValue::Static("checkbox"), ApplyAttributeAs::Attribute));

        if props.input_props.tabindex.is_none() {
            attr_map.insert(AttrValue::Static("tabindex"), (AttrValue::Static("0"), ApplyAttributeAs::Attribute));
        }

        if checked {
            attr_map.insert(AttrValue::Static("aria-checked"), (AttrValue::Static("true"), ApplyAttributeAs::Attribute));
        }

        let onclick = ctx.link().callback(|_| Msg::Toggle);
        let onkeyup = Callback::from({
            let link = ctx.link().clone();
            move |event: KeyboardEvent| {
                if event.key_code() == 32 {
                    link.send_message(Msg::Toggle);
                }
            }
        });

        let mut listeners =  props.listeners.listeners.clone();
        listeners.push(Some(Rc::new(::yew::html::onkeyup::Wrapper::new(onkeyup))));
        listeners.push(Some(Rc::new(::yew::html::onclick::Wrapper::new(onclick))));

        let listeners = Listeners::Pending(listeners.into_boxed_slice());

        let input: Html = VTag::__new_other(
            Cow::Borrowed("div"),
            props.std_props.node_ref.clone(),
            None,
            attributes,
            listeners,
            VList::new(),
        ).into();

        html!{
            // Wrap inside div for fixed size
            <div>{input}</div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            if props.input_props.autofocus {
                if let Some(el) = props.std_props.node_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
        }
    }
}
