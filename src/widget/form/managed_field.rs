use anyhow::Error;
use serde_json::Value;
use wasm_bindgen::{closure::Closure, JsCast};

use yew::html::Scope;
use yew::prelude::*;

use super::{FieldHandle, FieldOptions, FormContext, FormContextObserver, ValidateFn};
use crate::props::FieldBuilder;

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
    pub valid: Result<(), String>,

    /// Field default value
    pub default: Value,

    /// Radio group flag. Set when the field is part of a radio group.
    pub radio_group: bool,

    /// Do not allow multiple fields with the same name.
    ///
    /// Instead, use the same state for all of those fields.
    pub unique: bool,

    /// Optional conversion called by [FormContext::get_submit_data]
    pub submit_converter: Option<Callback<Value, Option<Value>>>,
}

/// Managed field context.
///
/// This is a small wrapper around Yew [Context], and gives you access
/// to:
///
/// - the [ManagedFieldState] to get the field value and validity
/// - the [ManagedFieldLink] to send messges to the update function.
/// - the component properties.
pub struct ManagedFieldContext<'a, MF: ManagedField + Sized + 'static> {
    ctx: &'a Context<ManagedFieldMaster<MF>>,
    comp_state: &'a ManagedFieldState,
}

impl<'a, MF: ManagedField + Sized> ManagedFieldContext<'a, MF> {
    fn new(ctx: &'a Context<ManagedFieldMaster<MF>>, comp_state: &'a ManagedFieldState) -> Self {
        ManagedFieldContext { ctx, comp_state }
    }

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
    /// Send messages to the update function.
    pub fn send_message(&self, msg: impl Into<MF::Message>) {
        let msg = msg.into();
        self.link.send_message(Msg::ChildMessage(msg));
    }

    /// Create a callback which sends messages to the update function.
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

    /// Set value for managed fields.
    ///
    /// Updates the value and re-check validity. The connected [FormContext] is kept in sync.
    pub fn update_value(&self, value: impl Into<Value>) {
        let msg = Msg::UpdateValue(value.into());
        self.link.send_message(msg);
    }

    /// Update default value.
    pub fn update_default(&self, default: impl Into<Value>) {
        let msg = Msg::UpdateDefault(default.into());
        self.link.send_message(msg);
    }

    /// Trigger re-validation
    pub fn validate(&self) {
        self.link.send_message(Msg::Validate);
    }

    /// Set valus/valid for unmanaged fields
    ///
    /// # Note
    ///
    /// This is ignored if the field is managed by a FormContext.
    pub fn force_value(&self, value: Option<impl Into<Value>>, valid: Option<Result<(), String>>) {
        let msg = Msg::ForceValue(value.map(|v| v.into()), valid);
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
pub trait ManagedField: Sized {
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
    fn validator(_props: &Self::ValidateClosure, _value: &Value) -> Result<(), Error> {
        Ok(())
    }

    /// Returns the initial field setup.
    ///
    /// # Note
    ///
    /// The [ManagedFieldState::valid] property is ignored and
    /// immediately overwritten by a call to the validation function.
    fn setup(props: &Self::Properties) -> ManagedFieldState;

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
    ForceValue(Option<Value>, Option<Result<(), String>>),
    ChildMessage(M),
    Validate,
    LabelClicked,               // Associated label was clicked
    FormCtxUpdate(FormContext), // FormContext object changed
    FormCtxDataChange,          // Data inside FormContext changed
}

/// Component implementation for [ManagedField]s.
pub struct ManagedFieldMaster<MF: ManagedField> {
    slave: MF,
    comp_state: ManagedFieldState,
    form_ctx: Option<FormContext>,
    field_handle: Option<FieldHandle>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
    _form_ctx_observer: Option<FormContextObserver>,
    label_clicked_closure: Option<Closure<dyn Fn()>>,

    /// The validation function
    validate: ValidateFn<Value>,
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
            Some(self.validate.clone()),
            options,
            self.comp_state.unique,
            self.comp_state.submit_converter.clone(),
        );

        // FormContext may already have field data (i.e for unique fields), so sync back
        // data after field registration.
        let (value, valid) = field_handle.get_data();
        self.comp_state.value = value;
        self.comp_state.valid = valid;

        self.field_handle = Some(field_handle);
    }
}

impl<MF: ManagedField + 'static> Component for ManagedFieldMaster<MF> {
    type Message = Msg<MF::Message>;
    type Properties = MF::Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let validation_args = MF::validation_args(props);
        let validate = ValidateFn::new(move |value| MF::validator(&validation_args, value));

        let mut comp_state = MF::setup(props);
        comp_state.valid = validate
            .validate(&comp_state.value)
            .map_err(|err| err.to_string());

        let sub_context = ManagedFieldContext::new(ctx, &comp_state);
        let slave = MF::create(&sub_context);

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
            slave,
            comp_state,
            _form_ctx_handle,
            _form_ctx_observer,
            field_handle: None,
            form_ctx,
            label_clicked_closure: None,
            validate,
        };

        me.register_field(ctx);

        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::ForceValue(value, valid) => {
                if let Some(name) = &props.as_input_props().name {
                    if self.field_handle.is_some() {
                        log::error!("Field '{name}' is managed - unable to force value.");
                        return false;
                    }
                }

                let value = value.unwrap_or(self.comp_state.value.clone());
                let valid = valid
                    .unwrap_or_else(|| self.validate.validate(&value).map_err(|e| e.to_string()));

                let value_changed = value != self.comp_state.value;
                let valid_changed = valid != self.comp_state.valid;

                self.comp_state.value = value;
                self.comp_state.valid = valid;

                if value_changed || valid_changed {
                    let sub_context = ManagedFieldContext::new(ctx, &self.comp_state);
                    self.slave.value_changed(&sub_context);
                }
                true
            }
            Msg::UpdateValue(value) => {
                let valid = self.validate.validate(&value).map_err(|e| e.to_string());

                let value_changed = value != self.comp_state.value;
                let valid_changed = valid != self.comp_state.valid;

                self.comp_state.value = value;
                self.comp_state.valid = valid;

                if let Some(field_handle) = &mut self.field_handle {
                    field_handle.set_value(self.comp_state.value.clone());
                }
                if value_changed || valid_changed {
                    let sub_context = ManagedFieldContext::new(ctx, &self.comp_state);
                    self.slave.value_changed(&sub_context);
                }
                true
            }
            Msg::Validate => {
                if let Some(field_handle) = &mut self.field_handle {
                    field_handle.validate();
                    false
                } else {
                    let valid = self
                        .validate
                        .validate(&self.comp_state.value)
                        .map_err(|e| e.to_string());

                    let valid_changed = valid != self.comp_state.valid;
                    self.comp_state.valid = valid;
                    if valid_changed {
                        let sub_context = ManagedFieldContext::new(ctx, &self.comp_state);
                        self.slave.value_changed(&sub_context);
                    }
                    valid_changed
                }
            }
            Msg::UpdateDefault(default) => {
                self.comp_state.default = default.clone();
                if let Some(field_handle) = &mut self.field_handle {
                    field_handle.set_default(default);
                }
                true
            }
            Msg::ChildMessage(child_msg) => {
                let sub_context = ManagedFieldContext::new(ctx, &self.comp_state);
                self.slave.update(&sub_context, child_msg)
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

                        let sub_context = ManagedFieldContext::new(ctx, &self.comp_state);
                        self.slave.value_changed(&sub_context);
                    }

                    return value_changed || valid_changed;
                }
                false
            }
            Msg::LabelClicked => {
                let sub_context = ManagedFieldContext::new(ctx, &self.comp_state);
                self.slave.label_clicked(&sub_context)
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        let mut refresh1 = false;
        if let Some(field_handle) = &mut self.field_handle {
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
            log::info!("UPDATE VF {:?}", props.as_input_props().name);
            let validate = ValidateFn::new(move |value| MF::validator(&validation_args, value));
            refresh1 = true;
            self.validate = validate.clone();
            if let Some(field_handle) = &mut self.field_handle {
                field_handle.update_validate(Some(validate));
            }
        }

        let sub_context = ManagedFieldContext::new(ctx, &self.comp_state);
        let refresh2 = self.slave.changed(&sub_context, old_props);

        refresh1 || refresh2
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let sub_context = ManagedFieldContext::new(ctx, &self.comp_state);
        self.slave.view(&sub_context)
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props().as_input_props();
            /* fixme:
            if props.autofocus {
                if let Some(el) = props.std_props.node_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
            */

            if let Some(label_id) = &props.label_id {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();

                let label_clicked_closure = Closure::wrap({
                    let link = ctx.link().clone();
                    Box::new(move || {
                        link.send_message(Msg::LabelClicked);
                    }) as Box<dyn Fn()>
                });

                if let Some(el) = document.get_element_by_id(&label_id) {
                    let _ = el.add_event_listener_with_callback(
                        "click",
                        label_clicked_closure.as_ref().unchecked_ref(),
                    );
                    self.label_clicked_closure = Some(label_clicked_closure); // keep alive
                }
            }
        }

        let sub_context = ManagedFieldContext::new(ctx, &self.comp_state);
        self.slave.rendered(&sub_context, first_render);
    }
}
