use std::rc::Rc;
use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{ApplyAttributeAs, Listeners, VList, VTag};
use yew::html::{Scope, IntoEventCallback, IntoPropValue};

use pwt_macros::widget;

use super::{FormContext, FieldOptions};

#[widget(PwtCheckbox, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Checkbox {
    /// Name of the form field (or radio-group value).
    ///
    /// The field register itself with this `name` in the FormContext
    /// (if any).
    pub name: Option<AttrValue>,
    /// Radio group name.
    ///
    /// The field is part of this radio-group.
    ///
    /// The field register itself as `group` in the FormContext, and
    /// use `name` as group value.
    pub group: Option<AttrValue>,
    /// Force value.
    pub checked: Option<bool>,
    /// Default value.
    pub default: Option<bool>,
    /// Change callback
    pub on_change: Option<Callback<bool>>,
}

impl Checkbox {

    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
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
    pub fn on_change(mut self, cb: impl IntoEventCallback<bool>) -> Self {
        self.on_change = cb.into_event_callback();
        self
    }
}

pub enum Msg {
    Toggle,
    FormCtxUpdate(FormContext),
}

/// Checkbox state handling (used for Checkbox and MenuCheckbox)
///
/// Handles FormContext interaction. 
pub(crate) struct CheckBoxStateHandle {
    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
    name: Option<AttrValue>,
    group: Option<AttrValue>,
    checked: bool,
    on_change: Option<Callback<bool>>,
}

impl CheckBoxStateHandle {

    pub fn new<COMP: Component>(
        scope: &Scope<COMP>,
        on_form_ctx_change: impl Into<Callback<FormContext>>,
        name: Option<AttrValue>,
        group: Option<AttrValue>,
        checked: bool,
        options: FieldOptions,
        on_change: impl IntoEventCallback<bool>,
    ) -> Self {
        let mut _form_ctx_handle = None;
        let mut form_ctx = None;

        if let Some(name) = &name {

            let on_form_ctx_change = on_form_ctx_change.into();

            if let Some((form, handle)) = scope.context::<FormContext>(on_form_ctx_change) {
                if let Some(group) = &group {
                    form.register_radio_group_option(
                        group,
                        name,
                        checked,
                        options,
                    );
                } else {
                    form.register_field(
                        name,
                        checked.into(),
                        None,
                        options,
                    );
                }
                form_ctx = Some(form);
                _form_ctx_handle = Some(handle);
            }
        }

        Self {
            _form_ctx_handle,
            form_ctx,
            on_change: on_change.into_event_callback(),
            checked,
            name,
            group,
        }
    }

    pub fn update(&mut self, form_ctx: FormContext) -> bool {
        self.form_ctx = Some(form_ctx);
        let value = self.get_value();
        let changed = self.checked != value;
        self.checked = value;
        changed
    }

    pub fn get_value(&self) -> bool {
        if let Some(name) = &self.name {
            if let Some(form_ctx) = &self.form_ctx {
                if let Some(group) = &self.group {
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

    pub fn set_value(&mut self, checked: bool) {
        self.checked = checked;

        if let Some(name) = &self.name {
            if let Some(form_ctx) = &self.form_ctx {
                if let Some(group) = &self.group {
                    form_ctx.set_value(group, name.as_str().into());
                } else {
                    form_ctx.set_value(name, self.checked.into());
                }
            }
        }

        if let Some(on_change) = &self.on_change {
            on_change.emit(self.checked);
        }
    }
}

#[doc(hidden)]
pub struct PwtCheckbox {
    state: CheckBoxStateHandle,
}

impl Component for PwtCheckbox {
    type Message = Msg;
    type Properties = Checkbox;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let checked = props.checked.or(props.default).unwrap_or(false);

        let on_form_ctx_change = Callback::from({
            let link = ctx.link().clone();
            move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
        });

        let state = CheckBoxStateHandle::new(
            ctx.link(),
            on_form_ctx_change,
            props.name.clone(),
            props.group.clone(),
            checked,
            FieldOptions::from_field_props(&props.input_props),
            props.on_change.clone(),
        );

        Self { state }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => self.state.update(form_ctx),
            Msg::Toggle => {
                if props.input_props.disabled { return false; }
                let value = props.checked.unwrap_or_else(|| self.state.get_value());
                self.state.set_value(!value);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let disabled = props.input_props.disabled;
        let checked = props.checked.unwrap_or_else(|| self.state.get_value());

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
