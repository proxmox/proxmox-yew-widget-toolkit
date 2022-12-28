use std::rc::Rc;

use anyhow::Error;
use serde_json::Value;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoEventCallback;

use crate::prelude::*;
use crate::props::{LoadCallback, IntoLoadCallback, RenderFn};
use crate::widget::{Column, Dialog, Mask, Row};
use crate::widget::form2::{
    Checkbox, Form, FormContext, SubmitButton, ResetButton,
    SubmitCallback, IntoSubmitCallback,
};
use crate::component::AlertDialog;

#[derive(Clone, PartialEq, Properties)]
pub struct EditWindow {
    /// Yew node ref
    #[prop_or_default]
    node_ref: NodeRef,
    /// Yew component key
    pub key: Option<Key>,

    /// Window title
    pub title: AttrValue,

    /// Show advanced checkbox
    #[prop_or_default]
    pub advanced_checkbox: bool,


    pub renderer: Option<RenderFn<FormContext>>,
    pub loader: Option<LoadCallback<Value>>,
    pub ondone: Option<Callback<()>>,
    pub onsubmit: Option<SubmitCallback>,
    pub on_change: Option<Callback<FormContext>>,
}

impl EditWindow {

    pub fn new(title: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            title: title.into(),
        })
    }

    pub fn advanced_checkbox(mut self, advanced_checkbox: bool) -> Self {
        self.set_advanced_checkbox(advanced_checkbox);
        self
    }

    pub fn set_advanced_checkbox(&mut self, advanced_checkbox: bool) {
        self.advanced_checkbox = advanced_checkbox;
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: ::yew::html::NodeRef) -> Self {
        self.node_ref = node_ref;
        self
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn renderer(mut self, renderer: impl 'static + Fn(&FormContext) -> Html) -> Self {
        self.renderer = Some(RenderFn::new(renderer));
        self
    }

    pub fn loader(mut self, callback: impl IntoLoadCallback<Value>) -> Self {
        self.loader = callback.into_load_callback();
        self
    }

    pub fn onsubmit(mut self, callback: impl IntoSubmitCallback) -> Self {
        self.onsubmit = callback.into_submit_callback();
        self
    }

    pub fn ondone(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.ondone = cb.into_event_callback();
        self
    }

    /// Builder style method to set the on_change callback.
    pub fn on_change(mut self, cb: impl IntoEventCallback<FormContext>) -> Self {
        self.on_change = cb.into_event_callback();
        self
    }

    pub fn is_edit(&self) -> bool {
        self.loader.is_some()
    }
}

pub enum Msg {
    FormDataChange,
    Submit,
    SubmitResult(Result<Value,Error>),
    Load,
    LoadResult(Result<Value, Error>),
    ClearError,
}

#[doc(hidden)]
pub struct PwtEditWindow {
    loading: bool,
    form_ctx: FormContext,
    submit_error: Option<String>,
}

impl Component for PwtEditWindow {
    type Message = Msg;
    type Properties = EditWindow;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);

        let form_ctx = FormContext::new()
            .on_change(ctx.link().callback(|_| Msg::FormDataChange));

        Self {
            form_ctx,
            loading: false,
            submit_error: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ClearError => {
                self.submit_error = None;
                true
            }
            Msg::Load => {
                if let Some(loader) = props.loader.clone() {
                    self.loading = true;
                    let link = ctx.link().clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let res = loader.apply().await;
                        link.send_message(Msg::LoadResult(res));
                    });
                }
                true
            }
            Msg::LoadResult(result) => {
                self.loading = false;
                match result {
                    Err(err) => log::error!("Load error: {}", err),
                    Ok(value) => {
                        self.form_ctx.load_form(value);
                    }
                }
                true
            }
            Msg::FormDataChange => {
                if self.submit_error != None {
                    self.submit_error = None;
                }
                if let Some(on_change) = &props.on_change {
                    on_change.emit(self.form_ctx.clone());
                }
                // Note: we redraw on any data change
                true
            }
            Msg::Submit => {
                if let Some(onsubmit) = props.onsubmit.clone() {
                    let link = ctx.link().clone();
                    let form_ctx = self.form_ctx.clone();
                    self.loading = true;
                    wasm_bindgen_futures::spawn_local(async move {
                        let result = onsubmit.apply(form_ctx).await;
                        link.send_message(Msg::SubmitResult(result));
                    });
                }
                true
            }
            Msg::SubmitResult(result) => {
                self.loading = false;
                 match result {
                    Ok(_) => {
                        self.submit_error = None;
                        if let Some(ondone) = &props.ondone {
                            ondone.emit(());
                        }
                    }
                    Err(err) => {
                        //log::info!("ERROR: {}", err);
                        self.submit_error = Some(err.to_string());
                    }
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let submit = ctx.link().callback(|_| Msg::Submit);

        let edit_mode = props.is_edit();

        let mut toolbar = Row::new()
            .padding(2)
            .gap(2)
            .class("pwt-bg-color-neutral-emphased")
            .with_flex_spacer();

        if props.advanced_checkbox {
            let advanced_label_id = crate::widget::get_unique_element_id();
            let advanced_field = Checkbox::new()
                .class("pwt-ms-1")
                .label_id(advanced_label_id.clone())
                .on_change({
                    let form_ctx = self.form_ctx.clone();
                    move |show| {
                        form_ctx.set_show_advanced(show == "on");
                    }
                });

            let advanced = Row::new()
                .class("pwt-align-items-center")
                .with_child(html!{<label id={advanced_label_id}>{"Advanced"}</label>})
                .with_child(advanced_field);

            toolbar.add_child(advanced);
        }

        toolbar.add_child(ResetButton::new());
        toolbar.add_child(
            SubmitButton::new()
                .text(if edit_mode { "Update" } else { "Add" })
                .on_submit(submit)
        );

        let renderer = props.renderer.clone();
        let loading = self.loading;

        let form = match &renderer {
            Some(renderer) => renderer.apply(&self.form_ctx),
            None => html!{},
        };

        let input_panel = Mask::new()
            .visible(loading)
            .with_child(
                Column::new()
                    .with_child(form)
                    .with_child(toolbar.clone())
            );

        let alert = match self.submit_error.as_ref() {
            None => None,
            Some(msg) => Some(
                AlertDialog::new(msg)
                    .on_close(ctx.link().callback(|_| Msg::ClearError))
            ),
        };

        Dialog::new(props.title.clone())
            .node_ref(props.node_ref.clone())
            .on_close(props.ondone.clone())
            .with_child(
                Form::new()
                    .form_context(self.form_ctx.clone())
                    .with_child(input_panel)
            )
            .with_optional_child(alert)
            .into()
    }
}

impl Into<VNode> for EditWindow {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtEditWindow>(Rc::new(self), key);
        VNode::from(comp)
    }
}
