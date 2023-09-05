use serde_json::Value;

use yew::html::Scope;
use yew::prelude::*;

use super::{FieldHandle, FieldOptions, FormContext, FormContextObserver, ValidateFn};
use crate::props::FieldBuilder;

pub struct ManagedFieldState {
    // local state, usage depends whether we have a name/form_ctx
    // None => store checked state locally
    // Some => use it to track/detect changes
    pub value: Value,
    pub valid: Result<(), String>,

    pub validate: ValidateFn<Value>,

    pub default: Value,
    pub radio_group: bool,
    pub unique: bool,
}

pub struct ManagedFieldContext<'a, MF: ManagedField + Sized + 'static> {
    ctx: &'a Context<ManagedFieldMaster<MF>>,
    comp_state: &'a ManagedFieldState,
}

impl<'a, MF: ManagedField + Sized> ManagedFieldContext<'a, MF> {
    /// The component’s props.
    pub fn props(&self) -> &MF::Properties {
        self.ctx.props()
    }

    /// The component scope (wrapped).
    pub fn link(&self) -> ManagedFieldLink<MF> {
        ManagedFieldLink {
            link: self.ctx.link().clone(),
        }
    }

    /// Current field state.
    pub fn state(&self) -> &ManagedFieldState {
        &self.comp_state
    }
}

pub struct ManagedFieldLink<MF: ManagedField + Sized + 'static> {
    link: Scope<ManagedFieldMaster<MF>>,
}

impl<MF: ManagedField + Sized> ManagedFieldLink<MF> {
    pub fn send_message(&self, msg: impl Into<MF::Message>) {
        let msg = msg.into();
        self.link.send_message(Msg::ChildMessage(msg));
    }

    pub fn callback<F, IN, M>(&self, function: F) -> Callback<IN>
    where
        M: Into<MF::Message>,
        F: Fn(IN) -> M + 'static,
    {
        self.link.callback(move |p: IN| {
            let msg: MF::Message = function(p).into();
            Msg::ChildMessage(msg)
        })
    }

    // Set value for managed fields
    pub fn update_value(&self, value: impl Into<Value>) {
        let msg = Msg::UpdateValue(value.into());
        self.link.send_message(msg);
    }

    /// Set valus/valid for unmanaged fields
    pub fn force_value(&self, value: impl Into<Value>, valid: Result<(), String>) {
        let msg = Msg::ForceValue(value.into(), valid);
        self.link.send_message(msg);
    }
}

impl<MF: ManagedField + Sized> Clone for ManagedFieldLink<MF> {
    fn clone(&self) -> Self {
        Self {
            link: self.link.clone(),
        }
    }
}

pub trait ManagedField: Sized {
    type Properties: Properties + FieldBuilder;
    type Message: 'static;

    fn setup(props: &Self::Properties) -> ManagedFieldState;

    fn create(ctx: &ManagedFieldContext<Self>) -> Self;

    fn update(&mut self, _ctx: &ManagedFieldContext<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn value_changed(&mut self, _ctx: &ManagedFieldContext<Self>) {}

    fn changed(&mut self, _ctx: &ManagedFieldContext<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    fn view(&self, _ctx: &ManagedFieldContext<Self>) -> Html;
}

pub enum Msg<M> {
    UpdateValue(Value),
    ForceValue(Value, Result<(), String>),
    ChildMessage(M),
    FormCtxUpdate(FormContext), // FormContext object changed
    FormCtxDataChange,          // Data inside FormContext changed
}

pub struct ManagedFieldMaster<MF: ManagedField> {
    state: MF,
    comp_state: ManagedFieldState,
    form_ctx: Option<FormContext>,
    field_handle: Option<FieldHandle>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
    _form_ctx_observer: Option<FormContextObserver>,
}

impl<MF: ManagedField + 'static> ManagedFieldMaster<MF> {
    // Get current field value and validation result.
    fn get_field_data(&self) -> (Value, Result<(), String>) {
        if let Some(field_handle) = &self.field_handle {
            field_handle.get_data()
        } else {
            (self.comp_state.value.clone(), self.comp_state.valid.clone())
        }
    }

    // Register the field in the FormContext
    fn register_field(&mut self, ctx: &Context<Self>) {
        let props = ctx.props();
        let input_props = props.as_input_props();

        let name = match &input_props.name {
            Some(name) => name,
            None => {
                self.field_handle = None;
                return;
            }
        };

        let form_ctx = match &self.form_ctx {
            Some(form_ctx) => form_ctx.clone(),
            None => {
                self.field_handle = None;
                return;
            }
        };

        let options = FieldOptions {
            submit: input_props.submit,
            submit_empty: input_props.submit_empty,
            disabled: input_props.disabled,
            required: input_props.required,
        };

        let field_handle = form_ctx.register_field(
            name,
            self.comp_state.value.clone(),
            self.comp_state.default.clone(),
            self.comp_state.radio_group,
            Some(self.comp_state.validate.clone()),
            options,
            self.comp_state.unique,
        );
        self.field_handle = Some(field_handle);
    }
}

impl<MF: ManagedField + 'static> Component for ManagedFieldMaster<MF> {
    type Message = Msg<MF::Message>;
    type Properties = MF::Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let comp_state = MF::setup(props);

        let sub_context = ManagedFieldContext {
            ctx,
            comp_state: &comp_state,
        };
        let state = MF::create(&sub_context);

        let input_props = props.as_input_props();

        let on_form_ctx_change = ctx.link().callback(Msg::FormCtxUpdate);

        let on_form_data_change = ctx.link().callback(|_| Msg::FormCtxDataChange);

        let mut _form_ctx_handle = None;
        let mut _form_ctx_observer = None;
        let mut form_ctx = None;

        if input_props.name.is_some() {
            if let Some((form, handle)) = ctx.link().context::<FormContext>(on_form_ctx_change) {
                _form_ctx_handle = Some(handle);
                _form_ctx_observer = Some(form.add_listener(on_form_data_change));
                form_ctx = Some(form);
            }
        }
        let mut me = Self {
            state,
            comp_state,
            _form_ctx_handle,
            _form_ctx_observer,
            field_handle: None,
            form_ctx,
        };
        me.register_field(ctx);
        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ForceValue(value, valid) => {
                if let Some(name) = &props.as_input_props().name {
                    log::error!("Field '{name}' is managed - unable to force value.");
                    return false;
                }
                let value_changed = value != self.comp_state.value;
                let valid_changed = valid != self.comp_state.valid;

                self.comp_state.value = value;
                self.comp_state.valid = valid;

                if value_changed || valid_changed {
                    let sub_context = ManagedFieldContext {
                        ctx,
                        comp_state: &self.comp_state,
                    };
                    self.state.value_changed(&sub_context);
                }
                true
            }
            Msg::UpdateValue(value) => {
                let valid = self
                    .comp_state
                    .validate
                    .validate(&self.comp_state.value)
                    .map_err(|e| e.to_string());

                let value_changed = value != self.comp_state.value;
                let valid_changed = valid != self.comp_state.valid;

                self.comp_state.value = value;
                self.comp_state.valid = valid;

                if let Some(field_handle) = &mut self.field_handle {
                    field_handle.set_value(self.comp_state.value.clone());
                }
                if value_changed || valid_changed {
                    let sub_context = ManagedFieldContext {
                        ctx,
                        comp_state: &self.comp_state,
                    };
                    self.state.value_changed(&sub_context);
                }
                true
            }
            Msg::ChildMessage(child_msg) => {
                let sub_context = ManagedFieldContext {
                    ctx,
                    comp_state: &self.comp_state,
                };
                self.state.update(&sub_context, child_msg);
                true
            }
            Msg::FormCtxUpdate(form_ctx) => {
                let on_form_data_change = ctx.link().callback(|_| Msg::FormCtxDataChange);
                self._form_ctx_observer = Some(form_ctx.add_listener(on_form_data_change));
                self.form_ctx = Some(form_ctx);
                self.register_field(ctx);
                true
            }
            Msg::FormCtxDataChange => {
                if self.field_handle.is_some() {
                    let (value, valid) = self.get_field_data();
                    let value_changed = value != self.comp_state.value;
                    let valid_changed = valid != self.comp_state.valid;

                    if value_changed || valid_changed {
                        self.comp_state.value = value;
                        self.comp_state.valid = valid;

                        let sub_context = ManagedFieldContext {
                            ctx,
                            comp_state: &self.comp_state,
                        };
                        self.state.value_changed(&sub_context);
                    }

                    return value_changed || valid_changed;
                }
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        if let Some(field_handle) = &mut self.field_handle {
            let props = props.as_input_props();
            let old_props = old_props.as_input_props();

            if props.submit != old_props.submit
                || props.submit_empty != old_props.submit_empty
                || props.disabled != old_props.disabled
                || props.required != old_props.required
            {
                let options = FieldOptions {
                    submit: props.submit,
                    submit_empty: props.submit_empty,
                    disabled: props.disabled,
                    required: props.required,
                };
                field_handle.update_field_options(options);
            }
        }
        let sub_context = ManagedFieldContext {
            ctx,
            comp_state: &self.comp_state,
        };
        self.state.changed(&sub_context, old_props)
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let sub_context = ManagedFieldContext {
            ctx,
            comp_state: &self.comp_state,
        };
        self.state.view(&sub_context)
    }
}
