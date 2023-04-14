use serde_json::Value;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};

use pwt_macros::widget;

use crate::props::{WidgetBuilder, ContainerBuilder, EventSubscriber};
use crate::widget::Container;
use super::{FieldState, FieldStateMsg, ValidateFn};

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
    /// Radio group flag
    #[prop_or_default]
    pub radio_group: bool,
    /// Use switch style layout.
    #[prop_or_default]
    pub switch: bool,
    /// Change callback
    pub on_change: Option<Callback<String>>,
    //fixme: on_input()
}

impl Checkbox {

    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Creates a radio group member.
    pub fn radio() -> Self {
        yew::props!(Self { radio_group: true })
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

    /// Builder style method to set the checked flag.
    pub fn checked(mut self, checked: impl IntoPropValue<Option<bool>>) -> Self {
        self.set_checked(checked);
        self
    }

    /// Method to set the checked flag.
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

    /// Builder style method to set the switch flag
    pub fn switch(mut self, switch: bool) -> Self {
        self.set_switch(switch);
        self
    }

    /// Method to set the switch flag
    pub fn set_switch(&mut self, switch: bool) {
        self.switch = switch;
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

#[doc(hidden)]
pub struct PwtCheckbox {
    state: FieldState,
    label_clicked_closure: Option<Closure<dyn Fn()>>,
}

impl Component for PwtCheckbox {
    type Message = Msg;
    type Properties = Checkbox;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let real_validate = ValidateFn::new(move |_value: &Value| {
            Ok(())
        });

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

        let mut me = Self {
            state,
            label_clicked_closure: None,
        };

        let on_value = props.value.as_deref().unwrap_or("on").to_string();

        let default = match props.default {
            Some(true) => on_value.clone(),
            _ => String::new(),
        };

        if let Some(name) = &props.input_props.name {
            me.state.register_field(&props.input_props, default.clone(), default, props.radio_group, false);
            if props.checked.is_some() {
                log::error!("Checkbox '{name}' is named - unable to force checked.");
            }
         } else {
            let value = match props.checked {
                Some(true) => on_value.clone(),
                _ => default,
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
                self.state.update_hook(&props.input_props, state_msg, default, props.radio_group, false)
            }
            Msg::Toggle => {
                if props.input_props.disabled { return true; }
                let on_value = props.value.as_deref().unwrap_or("on").to_string();
                let (value, _) = self.state.get_field_data();
                let new_value = if value == on_value {
                    if props.radio_group {
                        // do not allow to deselect radio buttons (same behaviour as browser).
                        on_value
                    } else {
                        String::new()
                    }
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
            self.state.update_field_options(&props.input_props);
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

        let onclick = ctx.link().callback(|_| Msg::Toggle);
        let onkeyup = Callback::from({
            let link = ctx.link().clone();
            move |event: KeyboardEvent| {
                if event.key_code() == 32 {
                    link.send_message(Msg::Toggle);
                }
            }
        });

        let (layout_class, inner) = match (props.switch, props.radio_group) {
            (true, _) => {
                ("pwt-switch", html!{<span class="pwt-switch-slider"><i class="fa fa-check"/></span>})
            }
            (false, true) => {
                // Note: icon is invisible, but necessary for correct baseline alignment
                ("pwt-radio-button", html!{<span class="pwt-checkbox-icon"><i class="fa fa-check"/></span>})
            }
            (false, false) => {
                ("pwt-checkbox", html!{<span class="pwt-checkbox-icon"><i class="fa fa-check"/></span>})
            }
        };

        // TODO: add other props.input_props

        let checkbox = Container::new()
            .with_std_props(&props.std_props)
            .class(layout_class)
            .class(checked.then(|| "checked"))
            .class(disabled.then(|| "disabled"))
            .with_child(inner)
            .attribute("tabindex", props.input_props.tabindex.unwrap_or(0).to_string())
            .attribute("role", "checkbox")
            .attribute("aria-checked", checked.then(|| "true"))
            .onkeyup(onkeyup)
            .onclick(onclick);

        if props.switch {
            checkbox.into()
        } else {
            Container::new()
                .class("pwt-checkbox-state")
                .with_child(checkbox)
                .into()
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

            if let Some(label_id) = &props.input_props.label_id {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();

                let label_clicked_closure = Closure::wrap({
                    let link = ctx.link().clone();
                    Box::new(move || {
                        link.send_message(Msg::Toggle);
                    }) as Box<dyn Fn()>
                });

                if let Some(el) = document.get_element_by_id(&label_id) {
                    let _ = el.add_event_listener_with_callback(
                        "click",
                        label_clicked_closure.as_ref().unchecked_ref(),
                    );
                    self.label_clicked_closure = Some(label_clicked_closure); // keep alive
                }
            }

        }
    }
}
