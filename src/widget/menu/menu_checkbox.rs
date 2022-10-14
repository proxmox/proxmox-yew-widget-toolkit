use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::widget::{Container, MenuEvent};
use crate::widget::form::{CheckboxStateHandle, FieldOptions, FormContext};

use super::MenuControllerMsg;

/// Checkbox/RadioGroup widget for [Menu](crate::widget::Menu)s.
#[derive(Clone, PartialEq, Properties)]
pub struct MenuCheckbox {
    /// Menu text (html inline text)
    pub text: Html,
    /// Name of the form field (or radio-group value).
    ///
    /// The field register itself with this `name` in the FormContext
    /// (if any).
    pub name: Option<AttrValue>,
    /// Radio group name.
    ///
    /// The field is part of this radio-group.
    ///
    /// The field register itself as `group` in the FormContext, and use
    /// `name` as group value.
    pub group: Option<AttrValue>,
    /// Disable field
    #[prop_or_default]
    pub disabled: bool,
    /// Force value.
    pub checked: Option<bool>,
    /// Default value.
    pub default: Option<bool>,
    /// Include value in [FormContext::get_submit_data].
    #[prop_or(true)]
    pub submit: bool,
    /// Change callback
    pub on_change: Option<Callback<MenuEvent>>,

    pub(crate) menu_controller: Option<Callback<MenuControllerMsg>>,
}

impl MenuCheckbox {

    /// Create a new menu button
    pub fn new(text: impl Into<Html>) -> Self {
        yew::props!(Self {
            text: text.into()
        })
    }

    pub fn checked(mut self, checked: impl IntoPropValue<Option<bool>>) -> Self {
        self.set_checked(checked);
        self
    }

    pub fn set_checked(&mut self, checked: impl IntoPropValue<Option<bool>>) {
        self.checked = checked.into_prop_value();
    }

    pub fn default(mut self, default: impl IntoPropValue<Option<bool>>) -> Self {
        self.set_default(default);
        self
    }

    pub fn set_default(&mut self, default: impl IntoPropValue<Option<bool>>) {
        self.default = default.into_prop_value();
    }

    /// Builder style method to set the field name.
    pub fn name(mut self, name: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_name(name);
        self
    }

    /// Method to set the field name.
    pub fn set_name(&mut self, name: impl IntoPropValue<Option<AttrValue>>) {
        self.name = name.into_prop_value();
    }

    /// Builder style method to set the radio group name.
    pub fn group(mut self, group: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_group(group);
        self
    }

    /// Method to set the radio group name.
    pub fn set_group(&mut self, group: impl IntoPropValue<Option<AttrValue>>) {
        self.group = group.into_prop_value();
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
    FormCtxUpdate(FormContext),
}

#[doc(hidden)]
pub struct PwtMenuCheckbox {
    state: CheckboxStateHandle,
}

impl Component for PwtMenuCheckbox {
    type Message = Msg;
    type Properties = MenuCheckbox;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let checked = props.checked.or(props.default).unwrap_or(false);

        let on_form_ctx_change = Callback::from({
            let link = ctx.link().clone();
            move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
        });

        let state = CheckboxStateHandle::new(
            ctx.link(),
            on_form_ctx_change,
            props.name.clone(),
            props.group.clone(),
            checked,
            FieldOptions { submit: props.submit, submit_empty: false },
        );

        Self { state }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => self.state.update(form_ctx),
            Msg::Toggle => {
                if props.disabled { return false; }
                let value = !props.checked.unwrap_or_else(|| self.state.get_value());
                self.state.set_value(value);

                if let Some(on_change) = &props.on_change {
                    let mut event = MenuEvent::new();
                    event.checked = value;
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

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let checked = props.checked.unwrap_or_else(|| self.state.get_value());

        let icon_class = classes!(
            "fa",
            "fa-fw",
            if props.group.is_some() {
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
            .attribute("tabindex", (!props.disabled).then(|| "-1"))
            .attribute("disabled", props.disabled.then(|| ""))
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
