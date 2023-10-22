use std::rc::Rc;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::props::FieldStdProps;
use crate::widget::form::{
    ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState,
};
use crate::widget::Container;

use super::{MenuControllerMsg, MenuEvent};

use pwt_macros::builder;

pub type PwtMenuCheckbox = ManagedFieldMaster<MenuCheckboxField>;

/// Checkbox/RadioGroup widget for [Menu](super::Menu)s.
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct MenuCheckbox {
    /// Menu text (html inline text)
    pub text: Html,

    /// Checkbox value (default is "on").
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub value: Option<AttrValue>,

    /// Radio group flag
    #[prop_or_default]
    #[builder]
    pub radio_group: bool,

    /// Force value.
    ///
    /// To implement controlled components (for use without a FormContext).
    /// This is ignored if the field has a name.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub checked: Option<bool>,

    /// Default value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<bool>,

    /// Standard input element properties
    #[prop_or_default]
    pub input_props: FieldStdProps,

    /// Click callback
    ///
    /// This callback is emited when the user clicks on the checkbox.
    #[builder_cb(IntoEventCallback, into_event_callback, MenuEvent)]
    #[prop_or_default]
    pub on_click: Option<Callback<MenuEvent>>,

    /// Change callback
    ///
    /// This callback is emited on any data change, i.e. if data
    /// inside the [FormContext](crate::widget::form::FormContext) changed.
    #[builder_cb(IntoEventCallback, into_event_callback, MenuEvent)]
    #[prop_or_default]
    pub on_change: Option<Callback<MenuEvent>>,

    #[prop_or_default]
    pub(crate) menu_controller: Option<Callback<MenuControllerMsg>>,
}

impl FieldBuilder for MenuCheckbox {
    fn as_input_props_mut(&mut self) -> &mut FieldStdProps {
        &mut self.input_props
    }
    fn as_input_props(&self) -> &FieldStdProps {
        &self.input_props
    }
}

impl MenuCheckbox {
    /// Create a new menu button
    pub fn new(text: impl Into<Html>) -> Self {
        yew::props!(Self { text: text.into() })
    }

    /// Create a new radio button
    pub fn radio(text: impl Into<Html>) -> Self {
        yew::props!(Self {
            text: text.into(),
            radio_group: true,
        })
    }

    pub(crate) fn menu_controller(mut self, cb: impl IntoEventCallback<MenuControllerMsg>) -> Self {
        self.menu_controller = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Toggle,
}

#[doc(hidden)]
pub struct MenuCheckboxField {}

impl ManagedField for MenuCheckboxField {
    type Message = Msg;
    type Properties = MenuCheckbox;
    type ValidateClosure = ();

    fn validation_args(_props: &Self::Properties) -> Self::ValidateClosure {
        ()
    }

    fn setup(props: &MenuCheckbox) -> ManagedFieldState {
        let on_value = props.value.as_deref().unwrap_or("on").to_string();

        let default = match props.default {
            Some(true) => on_value.clone(),
            _ => String::new(),
        };

        let value = match props.checked {
            Some(true) => on_value.clone(),
            Some(false) => String::new(),
            None => default.clone(),
        };

        ManagedFieldState {
            value: value.into(),
            valid: Ok(()),
            default: default.into(),
            radio_group: props.radio_group,
            unique: true,
        }
    }

    fn value_changed(&mut self, ctx: &ManagedFieldContext<Self>) {
        let props = ctx.props();
        let state = ctx.state();
        let value = state.value.as_str().unwrap_or("").to_string();
        let on_value = props.value.as_deref().unwrap_or("on").to_string();
        let checked = value == on_value;

        let mut event = MenuEvent::new();
        event.checked = checked;
        if let Some(on_change) = &props.on_change {
            on_change.emit(event);
        }
    }

    fn create(_ctx: &ManagedFieldContext<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        let state = ctx.state();
        match msg {
            Msg::Toggle => {
                if props.input_props.disabled {
                    return false;
                }

                let on_value = props.value.as_deref().unwrap_or("on").to_string();
                let value = state.value.clone();
                let checked = value == on_value;

                let new_value = if checked {
                    if props.radio_group {
                        // do not allow to deselect radio buttons (same behaviour as browser).
                        on_value
                    } else {
                        String::new()
                    }
                } else {
                    on_value
                };

                let mut event = MenuEvent::new();
                event.checked = checked;

                if let Some(on_click) = &props.on_click {
                    on_click.emit(event.clone());
                    if !event.get_keep_open() {
                        if let Some(menu_controller) = &props.menu_controller {
                            menu_controller.emit(MenuControllerMsg::Collapse);
                        }
                    }
                }
                ctx.link().update_value(new_value);

                true
            }
        }
    }

    fn changed(&mut self, ctx: &ManagedFieldContext<Self>, _old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if let Some(checked) = props.checked {
            let on_value = props.value.as_deref().unwrap_or("on").to_string();
            let value = if checked { on_value } else { String::new() };
            ctx.link().force_value(Some(value), None);
        }

        true
    }

    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let state = ctx.state();

        let disabled = props.input_props.disabled;

        let on_value = props.value.as_deref().unwrap_or("on").to_string();
        let value = state.value.clone();
        let checked = value == on_value;

        let icon_class = classes!(
            "fa",
            "fa-fw",
            if props.radio_group {
                if checked {
                    "fa-check-circle-o"
                } else {
                    "fa-circle-o"
                }
            } else {
                if checked {
                    "fa-check-square-o"
                } else {
                    "fa-square-o"
                }
            },
            "pwt-menu-item-icon",
        );
        let icon = html! {<i role="none" class={icon_class}/>};

        let onclick = ctx.link().callback(|_| Msg::Toggle);
        let onkeydown = Callback::from({
            let link = ctx.link().clone();
            move |event: KeyboardEvent| {
                if event.key() == " " {
                    link.send_message(Msg::Toggle);
                }
            }
        });

        Container::new()
            .class("pwt-menu-item")
            .attribute("tabindex", (!disabled).then(|| "-1"))
            .attribute("aria-disabled", disabled.then(|| "true"))
            .attribute(
                "role",
                if props.radio_group {
                    "menuitemradio"
                } else {
                    "menuitemcheckbox"
                },
            )
            .attribute("aria-checked", checked.then(|| "true"))
            .onclick(onclick)
            .onkeydown(onkeydown)
            .with_child(icon)
            .with_child(html! {<i class="pwt-menu-item-indent">{props.text.clone()}</i>})
            .into()
    }
}

impl Into<VNode> for MenuCheckbox {
    fn into(self) -> VNode {
        let comp = VComp::new::<ManagedFieldMaster<MenuCheckboxField>>(Rc::new(self), None);
        VNode::from(comp)
    }
}
