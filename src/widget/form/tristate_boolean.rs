use std::rc::Rc;

use anyhow::Error;
use serde_json::Value;

use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::Key;

use crate::prelude::*;
use crate::props::RenderFn;
use crate::state::{Selection, Store};
use crate::widget::data_table::{DataTable, DataTableColumn, DataTableHeader};
use crate::widget::{Dropdown, GridPicker};

use super::{ManagedField, ManagedFieldContext, ManagedFieldMaster, ManagedFieldState, ValidateFn};

use pwt_macros::{builder, widget};

#[derive(Copy, Clone, PartialEq)]
pub enum Tristate {
    Null,
    Yes,
    No,
}

/// Tristate Boolean widget (yes, no, null)
#[widget(pwt=crate, comp=ManagedFieldMaster<PwtTristateBoolean>, @input)]
#[derive(Clone, PartialEq, Properties)]
#[builder]
pub struct TristateBoolean {
    /// Default value.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub default: Option<Tristate>,

    /// Display text for [Tristate::Yes] (default is "Yes").
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub yes_text: Option<AttrValue>,

    /// Display text for [Tristate::No] (default is "No").
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub no_text: Option<AttrValue>,

    /// Display text for [Tristate::Null] (default is "Default").
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub null_text: Option<AttrValue>,

    /// Change callback
    #[builder_cb(IntoEventCallback, into_event_callback, Tristate)]
    #[prop_or_default]
    pub on_change: Option<Callback<Tristate>>,
}

impl TristateBoolean {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }
}

pub enum Msg {
    Select(String),
}

#[doc(hidden)]
pub struct PwtTristateBoolean {
    store: Store<AttrValue>,
    selection: Selection,
    columns: Rc<Vec<DataTableHeader<AttrValue>>>,
    render_value: RenderFn<AttrValue>,
}

fn tristate_to_value(tristate: Tristate) -> Value {
    match tristate {
        Tristate::Null => Value::Null,
        Tristate::Yes => Value::Bool(true),
        Tristate::No => Value::Bool(false),
    }
}

fn tristate_to_text(tristate: Tristate) -> String {
    match tristate {
        Tristate::Null => String::new(),
        Tristate::Yes => String::from("yes"),
        Tristate::No => String::from("no"),
    }
}

fn value_to_tristate(value: &Value) -> Option<Tristate> {
    match value {
        Value::Bool(true) => Some(Tristate::Yes),
        Value::Bool(false) => Some(Tristate::No),
        Value::Null => Some(Tristate::Null),
        _ => None,
    }
}

impl ManagedField for PwtTristateBoolean {
    type Message = Msg;
    type Properties = TristateBoolean;

    fn create_validation_fn(_props: &Self::Properties) -> ValidateFn<Value> {
        ValidateFn::new(move |value: &Value| match value {
            Value::Null | Value::Bool(_) => Ok(()),
            _ => Err(Error::msg(tr!("Got wrong data type!"))),
        })
    }

    fn setup(props: &Self::Properties) -> ManagedFieldState {
        let mut value = Value::Null;

        if let Some(default) = props.default {
            value = tristate_to_value(default);
        }

        // if let Some(force_value) = &props.value {
        // fixme: value = force_value.to_string();
        //}

        let value: Value = value.clone().into();

        let default = match props.default {
            None => Value::Null,
            Some(tristate) => tristate_to_value(tristate),
        };

        ManagedFieldState {
            value: value,
            valid: Ok(()), // fixme
            default,
            radio_group: false,
            unique: false,
            submit_converter: None,
        }
    }

    fn create(ctx: &ManagedFieldContext<Self>) -> Self {
        let props = ctx.props();

        let items = vec![
            AttrValue::Static(""),
            AttrValue::Static("yes"),
            AttrValue::Static("no"),
        ];

        let store = Store::with_extract_key(|item: &AttrValue| Key::from(item.as_str()));
        store.set_data(items);

        let yes_text = props
            .yes_text
            .clone()
            .map(|s| s.to_string())
            .unwrap_or(tr!("Yes"));
        let no_text = props
            .no_text
            .clone()
            .map(|s| s.to_string())
            .unwrap_or(tr!("No"));
        let null_text = props
            .null_text
            .clone()
            .map(|s| s.to_string())
            .unwrap_or(tr!("Default"));

        let render_value = RenderFn::new(move |value: &AttrValue| {
            let text = match value.as_str() {
                "" => null_text.clone(),
                "yes" => yes_text.clone(),
                "no" => no_text.clone(),
                unknown => unknown.to_string(), // should never happen
            };
            html! {text}
        });

        let columns = Rc::new(vec![DataTableColumn::new("Value")
            .show_menu(false)
            .render(render_value.clone())
            .into()]);

        Self {
            store,
            selection: Selection::new(),
            columns,
            render_value,
        }
    }

    fn value_changed(&mut self, ctx: &super::ManagedFieldContext<Self>) {
        let props = ctx.props();
        let state = ctx.state();

        if let Some(tristate) = value_to_tristate(&state.value) {
            if let Some(on_change) = &props.on_change {
                on_change.emit(tristate);
            }

            let key = Key::from(tristate_to_text(tristate));
            self.selection.select(key);
        } else {
            self.selection.clear();
        }

        //self.selection.select(key.clone());
    }

    fn update(&mut self, ctx: &ManagedFieldContext<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::Select(value) => {
                if props.input_props.disabled {
                    return false;
                }

                let value = match value.as_str() {
                    "yes" => Value::Bool(true),
                    "no" => Value::Bool(false),
                    "" => Value::Null,
                    _ => Value::Null, // should never happen
                };
                ctx.link().update_value(value);
                false
            }
        }
    }
    fn view(&self, ctx: &ManagedFieldContext<Self>) -> Html {
        let props = ctx.props();
        let state = ctx.state();

        let (value, valid) = (&state.value, &state.valid);
        let value_text = match value_to_tristate(value) {
            Some(tristate) => tristate_to_text(tristate),
            None => String::new(),
        };

        let columns = Rc::clone(&self.columns);

        let picker = {
            let store = self.store.clone();
            let selection = self.selection.clone();

            move |on_select: &Callback<Key>| {
                // TODO use a simpler list widget without virtual scroll support?
                let table = DataTable::new(columns.clone(), store.clone())
                    //.class("pwt-fit")
                    .borderless(true)
                    .striped(false)
                    .show_header(false);

                GridPicker::new(table)
                    .selection(selection.clone())
                    .show_filter(false)
                    .on_select(on_select.clone())
                    .into()
            }
        };

        let tip = match &valid {
            Err(msg) => Some(msg.to_string()),
            Ok(_) => None,
        };

        Dropdown::new(picker)
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .class(if valid.is_ok() {
                "is-valid"
            } else {
                "is-invalid"
            })
            .on_change(ctx.link().callback(|key: String| Msg::Select(key)))
            .value(value_text)
            .render_value(self.render_value.clone())
            .tip(tip)
            .into()
    }
}
