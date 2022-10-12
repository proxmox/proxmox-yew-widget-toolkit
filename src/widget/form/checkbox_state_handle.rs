use yew::prelude::*;
use yew::html::Scope;

use crate::widget::form::{FieldOptions, FormContext};

/// Checkbox state handling (used for Checkbox and MenuCheckbox)
///
/// Handles FormContext interaction.
pub(crate) struct CheckboxStateHandle {
    form_ctx: Option<FormContext>,
    _form_ctx_handle: Option<ContextHandle<FormContext>>,
    name: Option<AttrValue>,
    group: Option<AttrValue>,

    // local state, usage depends whether we have a form_ctx
    // None => store checked state locally
    // Some => use it to track/detect changes
    checked: bool,
}

impl CheckboxStateHandle {

    pub fn new<COMP: Component>(
        scope: &Scope<COMP>,
        on_form_ctx_change: impl Into<Callback<FormContext>>,
        name: Option<AttrValue>,
        group: Option<AttrValue>,
        checked: bool,
        options: FieldOptions,
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
            checked,
            name,
            group,
        }
    }

    pub fn update(&mut self, form_ctx: FormContext) -> bool {
        self.form_ctx = Some(form_ctx);
        let value = self.get_value();
        // we use self.checked to track changes
        let changed = self.checked != value;
        self.checked = value;
        changed
    }

    pub fn get_value(&self) -> bool {
        if self.name.is_some() && self.form_ctx.is_some() {
            let name = self.name.as_ref().unwrap();
            let form_ctx = self.form_ctx.as_ref().unwrap();
            if let Some(group) = &self.group {
                form_ctx.get_field_value(group).as_str() == Some(name)
            } else {
                form_ctx.get_field_value(name).as_bool().unwrap_or(false)
            }
        } else {
            self.checked
        }
    }

    pub fn set_value(&mut self, checked: bool) {

        if self.name.is_some() && self.form_ctx.is_some() {
            let name = self.name.as_ref().unwrap();
            let form_ctx = self.form_ctx.as_ref().unwrap();
            if let Some(group) = &self.group {
                form_ctx.set_value(group, name.as_str().into());
            } else {
                form_ctx.set_value(name, checked.into());
            }
        } else {
            self.checked = checked;
        }
    }
}
