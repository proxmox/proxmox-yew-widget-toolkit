use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::*;
use crate::widget::{Container};
use crate::widget::form::{FormContext, FieldOptions};


#[derive(Clone, PartialEq, Properties)]
pub struct MenuCheckbox {
    pub text: AttrValue,

    /// Name of the form field (or radio-group value).
    pub name: Option<AttrValue>,

    /// Radio group name.
    pub group: Option<AttrValue>,

    #[prop_or_default]
    pub disabled: bool,

    pub checked: Option<bool>,
    pub default: Option<bool>,

    pub on_change: Option<Callback<bool>>,
}

impl MenuCheckbox {

    /// Create a new menu button
    pub fn new(text: impl Into<AttrValue>) -> Self {
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
    pub fn on_change(mut self, cb: impl IntoEventCallback<bool>) -> Self {
        self.on_change = cb.into_event_callback();
        self
    }

}

pub enum Msg {
    Toggle,
    FormCtxUpdate(FormContext),
}

#[doc(hidden)]
pub struct PwtMenuCheckbox {
    checked: bool,
    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
}

impl PwtMenuCheckbox {

    fn get_value_from_state(&self, ctx: &Context<Self>) -> bool {
        let props = ctx.props();
        if let Some(name) = &props.name {
            if let Some(form_ctx) = &self.form_ctx {
                if let Some(group) = &props.group {
                    return form_ctx
                        .get_field_value(group)
                        .as_str() == Some(name);
                } else {
                    return form_ctx
                        .get_field_value(name)
                        .as_bool()
                        .unwrap_or(false);
                }
            }
        }
        self.checked
    }

    fn get_value(&self, ctx: &Context<Self>) -> bool {
        let props = ctx.props();

        if let Some(checked) = props.checked {
            return checked; // use forced value
        }

        self.get_value_from_state(ctx)
    }

    fn set_value(&mut self, ctx: &Context<Self>, checked: bool) {
        self.checked = checked;

        let props = ctx.props();

        if let Some(name) = &props.name {
            if let Some(form_ctx) = &self.form_ctx {
                if let Some(group) = &props.group {
                    form_ctx.set_value(group, name.as_str().into());
                } else {
                    form_ctx.set_value(name, self.checked.into());
                }
            }
        }

        if let Some(on_change) = &props.on_change {
            on_change.emit(self.checked);
        }
    }
}

impl Component for PwtMenuCheckbox {
    type Message = Msg;
    type Properties = MenuCheckbox;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let mut _form_ctx_handle = None;
        let mut form_ctx = None;

        if let Some(name) = &props.name {
            let value = props.checked.or(props.default).unwrap_or(false);

            let on_form_ctx_change = Callback::from({
                let link = ctx.link().clone();
                move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
            });


            if let Some((form, handle)) = ctx.link().context::<FormContext>(on_form_ctx_change) {
                if let Some(group) = &props.group {
                    form.register_radio_group_option(
                        group,
                        name,
                        value,
                        FieldOptions::new(),
                        // fixme: FieldOptions::from_field_props(&props.input_props),
                    );
                } else {
                    form.register_field(
                        name,
                        value.into(),
                        None,
                        FieldOptions::new(),
                        // fixme: FieldOptions::from_field_props(&props.input_props),
                    );
                }
                form_ctx = Some(form);
                _form_ctx_handle = Some(handle);
            }
        }

        Self {
            _form_ctx_handle,
            form_ctx,
            checked: false,
         }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => {
                self.form_ctx = Some(form_ctx);
                let value = self.get_value_from_state(ctx);
                let changed = self.checked != value;
                self.checked = value;
                changed
            }
            Msg::Toggle => {
                if props.disabled { return false; }
                self.set_value(ctx, !self.get_value(ctx));
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let checked = self.get_value(ctx);

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
            .with_child(html!{<i class="pwt-menu-item-indent">{&props.text}</i>})
            .into()
     }
}

impl Into<VNode> for MenuCheckbox {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtMenuCheckbox>(Rc::new(self), None);
        VNode::from(comp)
    }
}
