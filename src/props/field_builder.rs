use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::VNode;

use super::FieldStdProps;

/// Defines common builder methods for form field.
pub trait FieldBuilder: Into<VNode> {
    /// Mutable access to the field [properties](FieldStdProps).
    fn as_input_props_mut(&mut self) -> &mut FieldStdProps;
    /// Access to the field [properties](FieldStdProps).
    fn as_input_props(&self) -> &FieldStdProps;

    /// Copy properties from another [FieldStdProps]
    ///
    /// This overwrites all previously set properties.
    fn with_input_props(mut self, props: &FieldStdProps) -> Self {
        *self.as_input_props_mut() = props.clone();
        self
    }

    /// Builder style method to set the field name.
    fn name(mut self, name: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_name(name);
        self
    }

    /// Method to set the field name.
    fn set_name(&mut self, name: impl IntoPropValue<Option<AttrValue>>) {
        self.as_input_props_mut().name = name.into_prop_value();
    }

    /// Builder style method to set the html aria-label attribute
    fn aria_label(mut self, label: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_aria_label(label);
        self
    }

    /// Method to set the html aria-label attribute
    fn set_aria_label(&mut self, label: impl IntoPropValue<Option<AttrValue>>) {
        self.as_input_props_mut().aria_label = label.into_prop_value();
    }

    /// Builder style method to set the html tabindex attribute
    fn tabindex(mut self, index: impl IntoPropValue<Option<i32>>) -> Self {
        self.set_tabindex(index);
        self
    }

    /// Method to set the html tabindex attribute
    fn set_tabindex(&mut self, index: impl IntoPropValue<Option<i32>>) {
        self.as_input_props_mut().tabindex = index.into_prop_value();
    }

    /// Builder style method to set the autofocus flag
    fn autofocus(mut self, autofocus: bool) -> Self {
        self.set_autofocus(autofocus);
        self
    }

    /// Method to set the autofocus flag
    fn set_autofocus(&mut self, autofocus: bool) {
        self.as_input_props_mut().autofocus = autofocus;
    }

    /// Builder style method to set the disabled flag
    fn disabled(mut self, disabled: bool) -> Self {
        self.set_disabled(disabled);
        self
    }

    /// Method to set the disabled flag
    fn set_disabled(&mut self, disabled: bool) {
        self.as_input_props_mut().disabled = disabled;
    }

    fn is_disabled(&self) -> bool {
        self.as_input_props().disabled
    }

    /// Builder style method to set the required flag
    fn required(mut self, required: bool) -> Self {
        self.set_required(required);
        self
    }

    /// Method to set the required flag
    fn set_required(&mut self, required: bool) {
        self.as_input_props_mut().required = required;
    }

    /// Builder style method to set the label_id
    fn label_id(mut self, id: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_label_id(id);
        self
    }

    /// Method to set the label_id
    fn set_label_id(&mut self, id: impl IntoPropValue<Option<AttrValue>>) {
        self.as_input_props_mut().label_id = id.into_prop_value();
    }

    /// Builder style method to set the submit flag
    fn submit(mut self, submit: bool) -> Self {
        self.set_submit(submit);
        self
    }

    /// Method to set the submit flag
    fn set_submit(&mut self, submit: bool) {
        self.as_input_props_mut().submit = submit;
    }

    /// Builder style method to set the submit_empty flag
    fn submit_empty(mut self, submit_empty: bool) -> Self {
        self.set_submit_empty(submit_empty);
        self
    }

    /// Method to set the submit_empty flag
    fn set_submit_empty(&mut self, submit_empty: bool) {
        self.as_input_props_mut().submit_empty = submit_empty;
    }

    /// Builder style method to set the placeholder text
    fn placeholder(mut self, placeholder: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_placeholder(placeholder);
        self
    }

    /// Method to set the placeholder text
    fn set_placeholder(&mut self, placeholder: impl IntoPropValue<Option<AttrValue>>) {
        self.as_input_props_mut().placeholder = placeholder.into_prop_value();
    }
}
