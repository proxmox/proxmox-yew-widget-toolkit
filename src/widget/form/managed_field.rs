use std::ops::DerefMut;

use anyhow::Error;
use serde_json::Value;
use wasm_bindgen::{JsCast, closure::Closure};

use yew::html::Scope;
use yew::prelude::*;

use super::{FieldHandle, FieldOptions, FormContext, FormContextObserver, SubmitValidateFn};
use crate::props::FieldBuilder;

pub type ManagedFieldContext<MF> = Context<ManagedFieldMaster<MF>>;
pub type ManagedFieldLink<MF> = Scope<ManagedFieldMaster<MF>>;

/// Managed field state.
///
/// The initial state is create in [ManagedField::setup]. Primary use
/// of this struct is to access current field value, which is always
/// kept in sync with the form context (see [ManagedFieldContext::state])
pub struct ManagedFieldState {
    // local state, usage depends whether we have a name/form_ctx
    // None => store checked state locally
    // Some => use it to track/detect changes
    /// The field value (kept in sync with the form context)
    pub value: Value,

    /// Result of the last validation (updated on value changes)
    pub result: Result<Value, String>,

    /// Last valid value.
    pub last_valid: Option<Value>,

    /// Field default value
    pub default: Value,

    /// Radio group flag. Set when the field is part of a radio group.
    ///
    /// # Note
    ///
    /// When `radio_group` is true, multiple fields with the same name are allowed,
    /// each representing a different radio option. Only one can be selected at a time.
    pub radio_group: bool,

    /// Do not allow multiple fields with the same name.
    ///
    /// When `unique` is true, all fields with the same name share the same state.
    /// This is useful for fields that appear in multiple places but should be synchronized.
    ///
    /// # Note
    ///
    /// This is mutually exclusive with `radio_group`.
    pub unique: bool,

    form_ctx: Option<FormContext>,
    field_handle: Option<FieldHandle>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
    _form_ctx_observer: Option<FormContextObserver>,
    label_clicked_closure: Option<Closure<dyn Fn()>>,

    /// The validation function
    validate: SubmitValidateFn<Value>,
}

impl ManagedFieldState {
    pub fn new(value: Value, default: Value) -> Self {
        Self {
            unique: false,
            radio_group: false,
            last_valid: Some(default.clone()),
            result: Ok(default.clone()),
            default,
            value,

            form_ctx: None,
            field_handle: None,
            _form_ctx_handle: None,
            _form_ctx_observer: None,
            label_clicked_closure: None,
            validate: SubmitValidateFn::new(|v: &Value| Ok(v.clone())),
        }
    }
}

pub trait ManagedFieldScopeExt<M: ManagedField> {
    /// Set value for managed fields.
    ///
    /// Updates the value and re-validates it. If the field is connected to a [FormContext],
    /// the form's state is automatically synchronized.
    ///
    /// Use this for normal value updates from user interaction.
    fn update_value(&self, value: impl Into<Value>);

    /// Update default value.
    ///
    /// This updates the default value used for form resets.
    fn update_default(&self, default: impl Into<Value>);

    /// Set value/validity for unmanaged fields
    ///
    /// # Note
    ///
    /// This is ignored if the field is managed by a FormContext.
    fn force_value(
        &self,
        value: Option<impl Into<Value>>,
        validation_result: Option<Result<Value, String>>,
    );

    /// Trigger re-validation
    fn validate(&self);
}

impl<M: ManagedField> ManagedFieldScopeExt<M> for Scope<ManagedFieldMaster<M>> {
    fn update_value(&self, value: impl Into<Value>) {
        let msg = Msg::UpdateValue(value.into());
        self.send_message(msg);
    }
    fn update_default(&self, default: impl Into<Value>) {
        let msg = Msg::UpdateDefault(default.into());
        self.send_message(msg);
    }
    fn force_value(
        &self,
        value: Option<impl Into<Value>>,
        validation_result: Option<Result<Value, String>>,
    ) {
        let msg = Msg::ForceValue(value.map(|v| v.into()), validation_result);
        self.send_message(msg);
    }
    fn validate(&self) {
        self.send_message(Msg::Validate);
    }
}

/// Trait to simplify implementing managed fields.
///
/// This trait is used by the [ManagedFieldMaster] component, which simplifies
/// implementing managed fields by:
///
/// - automatically connect to the [FormContext] and register the field.
/// - observe and handle [FormContext] changes.
///
/// The trait is similar to the Yew [Component] trait, with some extra functions.
///
/// The [ManagedFieldContext] give you access to:
///
/// - component properties: `ctx.props()`
/// - managed state (field value, valid): `ctx.state()`
/// - component link (clonable) to send messages: `ctx.link()`
///
/// There are special link function [update](ManagedFieldLink::update_value) or
/// [force](ManagedFieldLink::force_value) field values:
pub trait ManagedField: Sized + DerefMut<Target = ManagedFieldState> + 'static {
    type Properties: Properties + FieldBuilder;
    type Message: 'static;
    type ValidateClosure: 'static + PartialEq;

    /// Extract arguments passed to the [`validator`](Self::validator)
    ///
    /// This is called when component properties changes. We rebuild the
    /// validation function if returend value changes.
    fn validation_args(props: &Self::Properties) -> Self::ValidateClosure;

    /// The validation function.
    ///
    /// Gets the result of [`validation_args`](Self::validation_args) and the value as parameter.
    /// If valid, it should return the value to be submitted.
    fn validator(_props: &Self::ValidateClosure, value: &Value) -> Result<Value, Error> {
        Ok(value.clone())
    }

    /// Create the component state.
    fn create(ctx: &ManagedFieldContext<Self>) -> Self;

    /// Process messages and update state.
    fn update(&mut self, _ctx: &ManagedFieldContext<Self>, _msg: Self::Message) -> bool {
        true
    }

    /// This is called whenever the managed value (or validity) changes.
    fn value_changed(&mut self, _ctx: &ManagedFieldContext<Self>) {}

    /// This is called when the associated label is clicked.
    fn label_clicked(&mut self, _ctx: &ManagedFieldContext<Self>) -> bool {
        false
    }

    /// Called on component property changes.
    fn changed(&mut self, _ctx: &ManagedFieldContext<Self>, _old_props: &Self::Properties) -> bool {
        true
    }

    /// Create the component view.
    fn view(&self, _ctx: &ManagedFieldContext<Self>) -> Html;

    /// The component rendered method.
    fn rendered(&mut self, _ctx: &ManagedFieldContext<Self>, _first_render: bool) {}
}

pub enum Msg<M> {
    UpdateValue(Value),
    UpdateDefault(Value),
    ForceValue(Option<Value>, Option<Result<Value, String>>),
    ChildMessage(M),
    Validate,
    LabelClicked,               // Associated label was clicked
    FormCtxUpdate(FormContext), // FormContext object changed
    FormCtxDataChange,          // Data inside FormContext changed
}

impl<CM> From<CM> for Msg<CM> {
    fn from(child_msg: CM) -> Self {
        Msg::ChildMessage(child_msg)
    }
}

/// Component implementation for [ManagedField]s.
pub struct ManagedFieldMaster<MF: ManagedField> {
    state: MF,
}

impl<MF: ManagedField + 'static> ManagedFieldMaster<MF> {
    /// Get current field value with the validation result and the last valid value.
    fn get_field_data(&self) -> (Value, Result<Value, String>, Option<Value>) {
        if let Some(field_handle) = &self.state.field_handle {
            return field_handle.get_data();
        }
        (
            self.state.value.clone(),
            self.state.result.clone(),
            self.state.last_valid.clone(),
        )
    }

    /// Update field state (value, validation result, and last valid value).
    ///
    /// This helper method consolidates the common pattern of updating all three
    /// state components and triggering the value_changed callback when needed.
    fn update_field_state(
        &mut self,
        ctx: &Context<Self>,
        new_value: Value,
        new_result: Result<Value, String>,
    ) -> bool {
        let value_changed = new_value != self.state.value;
        let valid_changed = new_result != self.state.result;

        self.state.value = new_value;
        self.state.result = new_result;
        if let Ok(submit_value) = &self.state.result {
            self.state.last_valid = Some(submit_value.clone());
        }

        if value_changed || valid_changed {
            self.state.value_changed(ctx);
        }

        value_changed || valid_changed
    }

    // Register the field in the FormContext
    fn register_field(&mut self, ctx: &Context<Self>) {
        let props = ctx.props();
        let input_props = props.as_input_props();

        let name = match &input_props.name {
            Some(name) => name,
            None => {
                self.state.field_handle = None;
                return;
            }
        };

        let form_ctx = match &self.state.form_ctx {
            Some(form_ctx) => form_ctx.clone(),
            None => {
                self.state.field_handle = None;
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
            self.state.value.clone(),
            self.state.default.clone(),
            self.state.radio_group,
            Some(self.state.validate.clone()),
            options,
            self.state.unique,
        );

        // FormContext may already have field data (i.e for unique fields), so sync back
        // data after field registration.
        let (value, result, last_valid) = field_handle.get_data();
        self.state.value = value;
        self.state.result = result;
        self.state.last_valid = last_valid;

        self.state.field_handle = Some(field_handle);
    }
}

impl<MF: ManagedField + 'static> Component for ManagedFieldMaster<MF> {
    type Message = Msg<MF::Message>;
    type Properties = MF::Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let mut state = MF::create(&ctx);

        let validation_args = MF::validation_args(props);
        state.validate = SubmitValidateFn::new(move |value| MF::validator(&validation_args, value));

        state.result = state
            .validate
            .apply(&state.value)
            .map_err(|err| err.to_string());
        if let Ok(submit_value) = &state.result {
            state.last_valid = Some(submit_value.clone())
        }

        let input_props = props.as_input_props();

        let on_form_ctx_change = ctx.link().callback(Msg::FormCtxUpdate);

        let on_form_data_change = ctx.link().callback(|_| Msg::FormCtxDataChange);

        if input_props.name.is_some() {
            if let Some((form, handle)) = ctx.link().context::<FormContext>(on_form_ctx_change) {
                state._form_ctx_handle = Some(handle);
                state._form_ctx_observer = Some(form.add_listener(on_form_data_change));
                state.form_ctx = Some(form);
            }
        }
        let mut me = Self { state };

        me.register_field(ctx);

        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ForceValue(value, result) => {
                if let Some(name) = &props.as_input_props().name {
                    if self.state.field_handle.is_some() {
                        log::error!("Field '{name}' is managed - unable to force value.");
                        return false;
                    }
                }

                let value = value.unwrap_or(self.state.value.clone());
                let result = result.unwrap_or_else(|| {
                    self.state.validate.apply(&value).map_err(|e| e.to_string())
                });

                self.update_field_state(ctx, value, result)
            }
            Msg::UpdateValue(value) => {
                let result = self.state.validate.apply(&value).map_err(|e| e.to_string());

                if let Some(field_handle) = &mut self.state.field_handle {
                    field_handle.set_value(value.clone());
                }

                self.update_field_state(ctx, value, result)
            }
            Msg::Validate => {
                if let Some(field_handle) = &mut self.state.field_handle {
                    field_handle.validate();
                    return false;
                }

                let result = self
                    .state
                    .validate
                    .apply(&self.state.value)
                    .map_err(|e| e.to_string());

                let valid_changed = result != self.state.result;
                self.state.result = result.clone();
                if let Ok(submit_value) = &result {
                    self.state.last_valid = Some(submit_value.clone());
                }

                if valid_changed {
                    self.state.value_changed(ctx);
                }
                valid_changed
            }
            Msg::UpdateDefault(default) => {
                self.state.default = default.clone();
                if let Some(field_handle) = &mut self.state.field_handle {
                    field_handle.set_default(default);
                }
                true
            }
            Msg::ChildMessage(child_msg) => self.state.update(ctx, child_msg),
            Msg::FormCtxUpdate(form_ctx) => {
                let on_form_data_change = ctx.link().callback(|_| Msg::FormCtxDataChange);
                self.state._form_ctx_observer = Some(form_ctx.add_listener(on_form_data_change));
                self.state.form_ctx = Some(form_ctx);
                self.register_field(ctx);
                true
            }
            Msg::FormCtxDataChange => {
                if self.state.field_handle.is_none() {
                    return false;
                }

                let (value, result, last_valid) = self.get_field_data();
                let value_changed = value != self.state.value;
                let valid_changed = result != self.state.result;

                if value_changed || valid_changed {
                    self.state.value = value;
                    self.state.result = result;
                    self.state.last_valid = last_valid;
                    self.state.value_changed(ctx);
                }

                value_changed || valid_changed
            }
            Msg::LabelClicked => self.state.label_clicked(ctx),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        let mut refresh1 = false;
        if let Some(field_handle) = &mut self.state.field_handle {
            let input_props = props.as_input_props();
            let old_input_props = old_props.as_input_props();

            if input_props.submit != old_input_props.submit
                || input_props.submit_empty != old_input_props.submit_empty
                || input_props.disabled != old_input_props.disabled
                || input_props.required != old_input_props.required
            {
                let options = FieldOptions {
                    submit: input_props.submit,
                    submit_empty: input_props.submit_empty,
                    disabled: input_props.disabled,
                    required: input_props.required,
                };
                field_handle.update_field_options(options);
                refresh1 = true;
            }
        }

        let old_validation_args = MF::validation_args(old_props);
        let validation_args = MF::validation_args(props);

        if validation_args != old_validation_args {
            let validate =
                SubmitValidateFn::new(move |value| MF::validator(&validation_args, value));
            refresh1 = true;
            self.state.validate = validate.clone();
            if let Some(field_handle) = &mut self.state.field_handle {
                field_handle.update_validate(Some(validate));
            } else {
                ctx.link().send_message(Msg::Validate); // re-evaluate
            }
        }

        let refresh2 = self.state.changed(ctx, old_props);

        refresh1 || refresh2
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.state.view(ctx)
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props().as_input_props();

            if let Some(label_id) = &props.label_id {
                let label_clicked_closure = Closure::wrap({
                    let link = ctx.link().clone();
                    Box::new(move || {
                        link.send_message(Msg::LabelClicked);
                    }) as Box<dyn Fn()>
                });

                if let Some(el) = gloo_utils::document().get_element_by_id(label_id) {
                    let _ = el.add_event_listener_with_callback(
                        "click",
                        label_clicked_closure.as_ref().unchecked_ref(),
                    );
                    self.state.label_clicked_closure = Some(label_clicked_closure);
                    // keep alive
                }
            }
        }

        self.state.rendered(ctx, first_render);
    }
}
