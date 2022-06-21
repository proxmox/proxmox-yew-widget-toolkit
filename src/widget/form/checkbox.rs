use std::rc::Rc;
use std::borrow::Cow;

use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VTag};
use yew::html::IntoEventCallback;

use crate::props::FieldStdProps;

use pwt_macros::widget;

#[widget(PwtCheckbox, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Checkbox {
    pub default: Option<bool>,
    pub on_change: Option<Callback<bool>>,
}

impl Checkbox {

    pub fn new() -> Self {
        yew::props!(Checkbox { input_props: FieldStdProps::new() })
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
}

pub struct PwtCheckbox {
    value: bool,
}

impl PwtCheckbox {

    fn get_value(&self, ctx: &Context<Self>) -> bool {
        match  &ctx.props().input_props.form_ref {
            Some(form_ref) => form_ref.get_value().as_bool().unwrap_or(false),
            None => self.value,
        }
    }

    fn set_value(&mut self, ctx: &Context<Self>, value: bool, default: Option<bool>) {
        let props = ctx.props();

        self.value = value;

        if let Some(form_ref) = &props.input_props.form_ref {
            form_ref.form.with_field_state_mut(&form_ref.field_name, |field| {
                field.value = self.value.into();
                field.valid = Ok(());
                if let Some(default) = default {
                    field.initial_value = default.into();
                }
            });
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

        props.input_props.register_form_field(value.into(), Ok(()));

        Self { value }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
             Msg::Toggle => {
                 if props.input_props.disabled { return false; }
                 self.set_value(ctx, !self.get_value(ctx), None);
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

        attr_map.insert(AttrValue::Static("role"), AttrValue::Static("checkbox"));

        if props.input_props.tabindex.is_none() {
            attr_map.insert(AttrValue::Static("tabindex"), AttrValue::Static("0"));
        }

        if checked {
            attr_map.insert(AttrValue::Static("aria-checked"), AttrValue::Static("true"));
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
