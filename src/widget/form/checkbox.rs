use std::rc::Rc;
use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{ApplyAttributeAs, Listeners, VList, VTag};
use yew::html::{IntoEventCallback, IntoPropValue};

use pwt_macros::widget;

use super::{FormContext, FieldOptions};

#[widget(PwtCheckbox, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Checkbox {
    pub name: AttrValue,
    pub default: Option<bool>,
    pub on_change: Option<Callback<bool>>,
}

impl Checkbox {

    pub fn new(name: impl IntoPropValue<AttrValue>) -> Self {
        yew::props!(Self {
            name: name.into_prop_value(),
        })
    }

    pub fn default(mut self, default: bool) -> Self {
        self.set_default(default);
        self
    }

    pub fn set_default(&mut self, default: bool) {
        self.default = Some(default);
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

#[doc(hidden)]
pub struct PwtCheckbox {
    value: bool,
    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
}

impl PwtCheckbox {

    fn get_value(&self, ctx: &Context<Self>) -> bool {
        let props = ctx.props();
        match &self.form_ctx {
            Some(form_ctx) => form_ctx.get_field_value(&props.name).as_bool().unwrap_or(false),
            None => self.value,
        }
    }

    fn set_value(&mut self, ctx: &Context<Self>, value: bool) {
        //log::info!("CHECKBOX SET FALUE0 {} {}", self.value, value);
        if self.value == value { return; }

        let props = ctx.props();

        self.value = value;

        if let Some(form_ctx) = &self.form_ctx {
            //log::info!("CHECKBOX SET FALUE {:?}", value);
            form_ctx.set_value(&props.name, value.into());
        } else {
            log::info!("MISING FORM CTX");
        }

        if let Some(on_change) = &props.on_change {
            on_change.emit(self.value.clone());
        }
    }
}

impl Component for PwtCheckbox {
    type Message = Msg;
    type Properties = Checkbox;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let value = props.default.unwrap_or(false);

        let on_form_ctx_change = Callback::from({
            let link = ctx.link().clone();
            move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
        });

        let mut _form_ctx_handle = None;
        let mut form_ctx = None;

        if let Some((form, handle)) = ctx.link().context::<FormContext>(on_form_ctx_change) {
            form.register_field(
                &props.name,
                value.into(),
                None,
                FieldOptions::from_field_props(&props.input_props),
            );

            form_ctx = Some(form);
            _form_ctx_handle = Some(handle);
        }

        Self {
            _form_ctx_handle,
            form_ctx,
            value,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => {
                self.form_ctx = Some(form_ctx);
                let value = self.get_value(ctx);
                let changed = self.value != value;
                self.value = value;
                changed                
            }
            Msg::Toggle => {
                if props.input_props.disabled { return false; }
                self.set_value(ctx, !self.get_value(ctx));
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let disabled = props.input_props.disabled;
        let checked = self.get_value(ctx);

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
