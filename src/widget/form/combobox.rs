use std::rc::Rc;

use anyhow::{bail, Error};
use serde_json::Value;

use yew::prelude::*;
use yew::virtual_dom::Key;
use yew::html::{IntoEventCallback, IntoPropValue};

use proxmox_schema::Schema;

use crate::prelude::*;
use crate::widget::{DataTableColumn, Dropdown, GridPicker};

use super::{FieldOptions, FormContext, TextFieldStateHandle, ValidateFn};

use pwt_macros::widget;

/// Combobox widget
///
/// Allows to select text options.

#[widget(PwtCombobox, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Combobox {
    /// Name of the form field.
    ///
    /// The field register itself with this `name` in the FormContext
    /// (if any).
    pub name: Option<AttrValue>,

    /// Default value.
    pub default: Option<AttrValue>,

    #[prop_or_default]
    pub editable: bool,

    #[prop_or_default]
    pub items: Rc<Vec<AttrValue>>,

    /// Change callback
    pub on_change: Option<Callback<String>>,

    /// Validation function.
    pub validate: Option<ValidateFn<String>>,
}

impl Combobox {

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

    /// Builder style method to set the default item.
    pub fn default(mut self, default: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_default(default);
        self
    }

    /// Method to set the default item.
    pub fn set_default(&mut self, default: impl IntoPropValue<Option<AttrValue>>) {
        self.default = default.into_prop_value();
    }

    /// Builder style method to set the editable flag.
    pub fn editable(mut self, editable: bool) -> Self {
        self.set_editable(editable);
        self
    }

    /// Method to set the editable flag.
    pub fn set_editable(&mut self, editable: bool) {
        self.editable = editable;
    }

    /// Builder style method to add an selectable item.
    pub fn with_item(mut self, item: impl IntoPropValue<AttrValue>) -> Self {
        self.add_item(item);
        self
    }

    /// Method to add an selectable item.
    pub fn add_item(&mut self, item: impl IntoPropValue<AttrValue>) {
        Rc::make_mut(&mut self.items).push(item.into_prop_value());
    }

    /// Builder style method to set items
    pub fn items(mut self, items: Rc<Vec<AttrValue>>) -> Self {
        self.set_items(items);
        self
    }

    /// Method to set items
    pub fn set_items(&mut self, items: Rc<Vec<AttrValue>>) {
        self.items = items;
    }

    /// Builder style method to set the on_change callback
    pub fn on_change(mut self, cb: impl IntoEventCallback<String>) -> Self {
        self.on_change = cb.into_event_callback();
        self
    }

    /// Builder style method to set the validate callback
    pub fn validate(
        mut self,
        validate: impl 'static + Fn(&String) -> Result<(), Error>,
    ) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(
        &mut self,
        validate: impl 'static + Fn(&String) -> Result<(), Error>,
    ) {
        self.validate = Some(ValidateFn::new(validate));
    }

    /// Builder style method to set the validation schema
    pub fn schema(mut self, schema: &'static Schema) -> Self {
        self.set_schema(schema);
        self
    }

    /// Method to set the validation schema
    pub fn set_schema(&mut self, schema: &'static Schema) {
        self.set_validate(move |value: &String| {
            schema.parse_simple_value(value)?;
            Ok(())
        });
    }
}

pub enum Msg {
    Select(String),
    FormCtxUpdate(FormContext),
    Reposition,
}

#[doc(hidden)]
pub struct PwtCombobox {
    state: TextFieldStateHandle,
}

fn create_combobox_validation_cb(props: Combobox) -> ValidateFn<Value> {
    ValidateFn::new(move |value: &Value| {
        let value = match value {
            Value::Null => String::new(),
            Value::String(v) => v.clone(),
            _ => { // should not happen
                log::error!("PwtField: got wrong data type in validate!");
                String::new()
            }
        };

        if value.is_empty() {
            if props.input_props.required {
                bail!("Field may not be empty.");
            } else {
                return Ok(());
            }
        }
        match &props.validate {
            Some(cb) => cb.validate(&value),
            None => Ok(()),
        }
    })
}

impl Component for PwtCombobox {
    type Message = Msg;
    type Properties = Combobox;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let value = props.default.as_ref().map(|s| s.as_str()).unwrap_or("").to_string();

        let on_form_ctx_change = Callback::from({
            let link = ctx.link().clone();
            move |form_ctx: FormContext| link.send_message(Msg::FormCtxUpdate(form_ctx))
        });


        let real_validate = create_combobox_validation_cb(props.clone());

        let state = TextFieldStateHandle::new(
            ctx.link(),
            on_form_ctx_change,
            props.name.clone(),
            value,
            Some(real_validate.clone()),
            FieldOptions::from_field_props(&props.input_props),
            props.on_change.clone(),
        );

        Self { state }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FormCtxUpdate(form_ctx) => self.state.update(form_ctx),
            Msg::Select(key) => {
                if props.input_props.disabled { return true; }
                self.state.set_value(key, false);
                true
            }
            Msg::Reposition => {
                true // just trigger a redraw
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let (value, valid) = self.state.get_field_data();

        let picker = {
            let items = Rc::clone(&props.items);
            let selected = value.clone();
            let link = ctx.link().clone();

            move |onselect: &Callback<Key>| {
                let columns = vec![
                    DataTableColumn::new("Value")
                        .render(|value: &AttrValue| html!{value}),
                ];
                GridPicker::new(columns)
                    .show_header(false)
                    .onselect(onselect)
                    .on_filter_change({
                        let link = link.clone();
                        move |()| link.send_message(Msg::Reposition)
                    })
                    .extract_key(|value: &AttrValue| Key::from(value.to_string()))
                    .selection(items.iter().enumerate().find_map(|(n, value)| (value == &selected).then(|| n)))
                    .data(Rc::clone(&items))
                    .into()
            }
        };

        Dropdown::new(picker)
            .popup_type("dialog")
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .editable(props.editable)
            .class(if valid.is_ok() { "is-valid" } else { "is-invalid" })
            .on_change(ctx.link().callback(|key: String| Msg::Select(key)))
            .value(value)
            .into()
    }
}
