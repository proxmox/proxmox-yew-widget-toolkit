use std::rc::Rc;
use serde_json::Value;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::props::FieldStdProps;
use crate::widget::Container;
use crate::widget::form::{FieldState, FieldStateMsg, ValidateFn};

use super::{MenuControllerMsg, MenuEvent};

/// Checkbox/RadioGroup widget for [Menu](super::Menu)s.
#[derive(Clone, PartialEq, Properties)]
pub struct MenuCheckbox {
    /// Menu text (html inline text)
    pub text: Html,

    /// Checkbox value (default is "on").
    pub value: Option<AttrValue>,
    /// Radio group flag
    #[prop_or_default]
    pub radio_group: bool,

    /// Force value.
    pub checked: Option<bool>,
    /// Default value.
    pub default: Option<bool>,

    /// Standard input element properties
    #[prop_or_default]
    pub input_props: FieldStdProps,

    /// Change callback
    pub on_change: Option<Callback<MenuEvent>>,

    pub(crate) menu_controller: Option<Callback<MenuControllerMsg>>,
}

impl FieldBuilder for MenuCheckbox {
    fn as_input_props_mut(&mut self) -> &mut FieldStdProps  {
        &mut self.input_props
    }
    fn as_input_props(&self) -> &FieldStdProps {
        &self.input_props
    }
}

impl MenuCheckbox {

    /// Create a new menu button
    pub fn new(text: impl Into<Html>) -> Self {
        yew::props!(Self {
            text: text.into()
        })
    }

    /// Create a new radio button
    pub fn radio(text: impl Into<Html>) -> Self {
        yew::props!(Self {
            text: text.into(),
            radio_group: true,
        })
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

    /// Builder style method to set the on_change callback.
    pub fn on_change(mut self, cb: impl IntoEventCallback<MenuEvent>) -> Self {
        self.on_change = cb.into_event_callback();
        self
    }

    pub(crate) fn menu_controller(mut self, cb: impl IntoEventCallback<MenuControllerMsg>) -> Self {
        self.menu_controller = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Toggle,
    StateUpdate(FieldStateMsg),
}

#[doc(hidden)]
pub struct PwtMenuCheckbox {
    state: FieldState,
}

impl Component for PwtMenuCheckbox {
    type Message = Msg;
    type Properties = MenuCheckbox;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let real_validate = ValidateFn::new(move |_value: &Value| {
            Ok(())
        });

        let state = FieldState::create(
            ctx,
            &props.input_props,
            ctx.link().callback(Msg::StateUpdate),
            None, // fixme: on_change
            real_validate.clone(),
        );

        let mut me = Self { state };

        let on_value = props.value.as_deref().unwrap_or("on").to_string();

        let default = match props.default {
            Some(true) => on_value.clone(),
            _ => String::new(),
        };

        if let Some(name) = &props.input_props.name {
            me.state.register_field(&props.input_props, default.clone(), default, props.radio_group);
            if props.checked.is_some() {
                log::error!("MenuCheckbox '{name}' is named - unable to force checked.");
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
                self.state.update_hook(&props.input_props, state_msg, default, props.radio_group)
            }
            Msg::Toggle => {
                if props.input_props.disabled { return true; }
                let on_value = props.value.as_deref().unwrap_or("on").to_string();
                let (value, _) = self.state.get_field_data();
                let checked = value == on_value;
                let new_value = if checked {
                    String::new()
                } else {
                    on_value
                };

                self.state.set_value(new_value);

                if let Some(on_change) = &props.on_change {
                    let mut event = MenuEvent::new();
                    event.checked = checked;
                    on_change.emit(event.clone());
                    if !event.get_keep_open() {
                        if let Some(menu_controller) = &props.menu_controller {
                            menu_controller.emit(MenuControllerMsg::Collapse);
                        }
                    }
                }

                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if let Some(name) = &props.input_props.name {
            if props.checked.is_some() {
                log::error!("MenuCheckbox '{name}' is named - unable to force checked.");
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

        let icon_class = classes!(
            "fa",
            "fa-fw",
            if props.radio_group {
                if checked { "fa-check-circle-o" } else { "fa-circle-o" }
            } else {
                if checked { "fa-check-square-o" } else { "fa-square-o" }
            },
            "pwt-menu-item-icon",
        );
        let icon = html!{<i role="none" aria-hidden="true" class={icon_class}/>};

        let onclick = ctx.link().callback(|_| Msg::Toggle);
        let onkeydown = Callback::from({
            let link = ctx.link().clone();
            move |event: KeyboardEvent| {
                if event.key_code() == 32 {
                    link.send_message(Msg::Toggle);
                }
            }
        });

        Container::new()
            .class("pwt-menu-item")
            .attribute("tabindex", (!disabled).then(|| "-1"))
            .attribute("disabled", disabled.then(|| ""))
            .attribute("role", if props.radio_group { "menuitemradio" } else { "menuitemcheckbox" })
            .attribute("aria-checked", checked.then(|| "true"))
            .onclick(onclick)
            .onkeydown(onkeydown)
            .with_child(icon)
            .with_child(html!{<i class="pwt-menu-item-indent">{props.text.clone()}</i>})
            .into()
     }
}

impl Into<VNode> for MenuCheckbox {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtMenuCheckbox>(Rc::new(self), None);
        VNode::from(comp)
    }
}
