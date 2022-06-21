use std::rc::Rc;

use anyhow::Error;
use serde_json::Value;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoEventCallback;

use crate::widget::prelude::*;
use crate::props::{
    LoadCallback, IntoLoadCallback, SubmitCallback, IntoSubmitCallback,
    RenderFn,
};
use crate::state::FormState;
use crate::widget::{Dialog, Mask, Row, Toolbar};
use crate::widget::form::{Checkbox, Submit, Reset};
use crate::component::AlertDialog;

#[derive(Clone, PartialEq, Properties)]
pub struct EditWindow {
    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,

    pub title: AttrValue,

    /// Show advanced checkbox
    #[prop_or_default]
    pub advanced_checkbox: bool,


    pub renderer: Option<RenderFn<FormState>>,
    pub loader: Option<LoadCallback<Value>>,
    pub ondone: Option<Callback<()>>,
    pub onsubmit: Option<SubmitCallback>,
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

    pub fn renderer(mut self, renderer: impl 'static + Fn(&FormState) -> Html) -> Self {
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

    pub fn is_edit(&self) -> bool {
        self.loader.is_some()
    }
}

pub enum Msg {
    FormChange,
    Submit,
    SubmitResult(Result<Value,Error>),
    Load,
    LoadResult(Result<Value, Error>),
    ClearError,
}

pub struct PbsEditWindow {
    loading: bool,
    form_state: FormState,
    submit_error: Option<String>,
}

impl Component for PbsEditWindow {
    type Message = Msg;
    type Properties = EditWindow;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::Load);
        let form_change = ctx.link().callback(|_| Msg::FormChange);
        Self {
            form_state: FormState::new(form_change),
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
                        self.form_state.load_form(value);
                    }
                }
                true
            }
            Msg::FormChange => {
                self.submit_error = None;

                let password = self.form_state.get_field_value("password");
                let confirm = self.form_state.get_field_value("confirm-password");

                let valid = if password == confirm {
                    Ok(())
                } else {
                    Err("Password does not match!".into())
                };

                self.form_state.with_field_state_mut("confirm-password", move |state| {
                    state.valid = valid.clone();
                });

                true
            }
            Msg::Submit => {
                if let Some(onsubmit) = props.onsubmit.clone() {
                    let link = ctx.link().clone();
                    let state = self.form_state.clone();
                    self.loading = true;
                    wasm_bindgen_futures::spawn_local(async move {
                        let result = onsubmit.apply(state).await;
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
        let form = match &props.renderer {
            Some(renderer) => renderer.apply(&self.form_state),
            None => html!{},
        };

        let mut toolbar = Toolbar::new()
            .class("pwt-border-top emphased")
            .with_flex_spacer();

        if props.advanced_checkbox {
            let advanced_label_id = crate::widget::get_unique_element_id();
            let advanced_field = Checkbox::new()
                .class("pwt-ms-1")
                .label_id(advanced_label_id.clone())
                .on_change({
                    let form_state = self.form_state.clone();
                    move |show| {
                        log::info!("ADV {}", show);
                        form_state.set_show_advanced(show);
                    }
                });

            let advanced = Row::new()
                .class("pwt-align-items-center")
                .with_child(html!{<label id={advanced_label_id}>{"Advanced"}</label>})
                .with_child(advanced_field);

            toolbar.add_child(advanced);
        }

        toolbar.add_child(Reset::new().form(&self.form_state));
        toolbar.add_child(
            Submit::new()
                .text(if edit_mode { "Update" } else { "Add" })
                .form(&self.form_state)
                .disabled(!self.form_state.dirty())
                .onsubmit(submit)
        );

        let alert = match self.submit_error.as_ref() {
            None => None,
            Some(msg) => Some(
                AlertDialog::new(msg)
                    .onclose(ctx.link().callback(|_| Msg::ClearError))
            ),
        };

        let panel = Mask::new()
            .form_wrapper(true)
            .visible(self.loading)
            .with_child(form)
            .with_optional_child(alert)
            .with_child(toolbar);

        Dialog::new(props.title.clone())
            .node_ref(props.node_ref.clone())
            .onclose(props.ondone.clone())
            .with_child(panel)
            .into()
    }
}

impl Into<VNode> for EditWindow {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PbsEditWindow>(Rc::new(self), NodeRef::default(), key);
        VNode::from(comp)
    }
}
