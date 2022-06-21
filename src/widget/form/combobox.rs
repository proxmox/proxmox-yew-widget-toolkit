use std::rc::Rc;

use anyhow::Error;

use yew::prelude::*;
use yew::virtual_dom::Key;
use yew::html::IntoEventCallback;

use proxmox_schema::Schema;

use crate::prelude::*;
use crate::props::{FieldStdProps, RenderFn};
use crate::widget::{DataTableColumn, Dropdown, GridPicker};
use crate::widget::form::ValidateFn;

use pwt_macros::widget;

/// Combobox widget
///
/// Allows to select text options.

#[widget(PwtCombobox, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Combobox {
    pub default: Option<String>,

    #[prop_or_default]
    pub editable: bool,

    #[prop_or_default]
    pub items: Rc<Vec<String>>,

    pub on_change: Option<Callback<String>>,

    pub validate: Option<ValidateFn<String>>,
}

impl Combobox {

    pub fn new() -> Self {
        yew::props!(Self { input_props: FieldStdProps::new() })
    }

    pub fn default(mut self, default: impl Into<String>) -> Self {
        self.set_default(default);
        self
    }

    pub fn set_default(&mut self, default: impl Into<String>) {
        self.default = Some(default.into());
    }

    /// Builder style method to set the editable flag
    pub fn editable(mut self, editable: bool) -> Self {
        self.set_editable(editable);
        self
    }

    /// Method to set the editable flag
    pub fn set_editable(&mut self, editable: bool) {
        self.editable = editable;
    }

    pub fn with_item(mut self, item: impl Into<String>) -> Self {
        self.add_item(item);
        self
    }

    pub fn add_item(&mut self, item: impl Into<String>) {
        Rc::make_mut(&mut self.items).push(item.into());
    }

    /// Builder style method to set items
    pub fn items(mut self, items: Rc<Vec<String>>) -> Self {
        self.set_items(items);
        self
    }

    /// Method to set items
    pub fn set_items(&mut self, items: Rc<Vec<String>>) {
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
    pub fn schema(mut self, schema: &'static Schema, optional: bool) -> Self {
        self.set_schema(schema, optional);
        self
    }

    /// Method to set the validation schema
    pub fn set_schema(&mut self, schema: &'static Schema, optional: bool) {
        self.validate = Some(ValidateFn::new(move |value: &String| {
            if optional && value.is_empty() {
                Ok(())
            } else {
                schema.parse_simple_value(value)?;
                Ok(())
            }
        }));
    }
}

pub enum Msg {
    Select(String),
}

pub struct PwtCombobox {
    value: String,
    valid: Result<(), String>,
}

impl PwtCombobox {

    fn create_picker(&self, ctx: &Context<Self>, selected: &str) -> RenderFn<Callback<String>> {
        let props = ctx.props();

        RenderFn::new({
            let items = Rc::clone(&props.items);
            let selected = selected.to_owned();

            move |onselect: &Callback<String>| {
                let columns = vec![
                    DataTableColumn::new("Value")
                        .render(|value: &String| html!{value.clone()}),
                ];
                let picker = GridPicker::new(columns)
                    .show_header(false)
                    .onselect(onselect)
                    .extract_key(|value: &String| Key::from(value.clone()))
                    .selection(items.iter().enumerate().find_map(|(n, value)| (value == &selected).then(|| n)))
                    .data(Rc::clone(&items));

                picker.into()
            }
        })
    }

    fn validate(&self, ctx: &Context<Self>, key: &String) -> Result<(), String> {
        match &ctx.props().validate {
            Some(cb) => {
                cb.validate(key).map_err(|e| e.to_string())
            }
            None => Ok(()),
        }
    }

    fn get_valid(&self, ctx: &Context<Self>) -> Result<(), String> {
        match &ctx.props().input_props.form_ref {
            Some(form_ref) => form_ref.get_valid(),
            None => self.valid.clone()
        }
    }

    fn get_value(&self, ctx: &Context<Self>) -> String {
        match  &ctx.props().input_props.form_ref {
            Some(form_ref) => form_ref.get_text(),
            None => self.value.clone(),
        }
    }

    fn set_value(&mut self, ctx: &Context<Self>, value: String) {
        let props = ctx.props();

        self.value = value;
        self.valid = self.validate(ctx, &self.value);

        if let Some(form_ref) = &props.input_props.form_ref {
            form_ref.form.with_field_state_mut(&form_ref.field_name, |field| {
                field.value = self.value.clone().into();
                field.valid = self.valid.clone();
                if let Some(default) = &props.default {
                    field.initial_value = default.as_str().into();
                }
            });
        }

        if let Some(on_change) = &props.on_change {
            on_change.emit(self.value.clone());
        }
    }
}

impl Component for PwtCombobox {
    type Message = Msg;
    type Properties = Combobox;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let value = props.default.as_ref().map(|s| s.as_str()).unwrap_or("").to_string();

        let mut me = Self { value, valid: Ok(()) };

        me.valid = me.validate(ctx, &me.value);

        props.input_props.register_form_field(me.value.clone().into(), me.valid.clone());

        me
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Select(key) => {
                self.set_value(ctx, key);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let valid = self.get_valid(ctx);
        let value = self.get_value(ctx);

        Dropdown::new(self.create_picker(ctx, &value))
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .editable(props.editable)
            .class(if valid.is_ok() { "is-valid" } else { "is-invalid" })
            .on_change(ctx.link().callback(|key: String| Msg::Select(key)))
            .value(value)
            .into()
    }
}
