use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::Key;
use yew::html::{IntoEventCallback, IntoPropValue};

#[cfg(feature="proxmox-schema")]
use proxmox_schema::Schema;

use crate::prelude::*;
use crate::state::Store;
use crate::widget::GridPicker;
use crate::widget::data_table::{DataTable, DataTableColumn, DataTableHeader};

use super::{Selector, SelectorRenderArgs, IntoValidateFn, ValidateFn};

use pwt_macros::widget;

/// Combobox widget
///
/// Allows to select text options.
#[widget(pwt=crate, comp=PwtCombobox, @input, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct Combobox {
    /// Default value.
    pub default: Option<AttrValue>,

    /// Make the input editable.
    #[prop_or_default]
    pub editable: bool,

    /// Item list.
    #[prop_or_default]
    pub items: Rc<Vec<AttrValue>>,

    /// Change callback
    pub on_change: Option<Callback<String>>,

    /// Validation function.
    pub validate: Option<ValidateFn<(String, Store<AttrValue>)>>,

    /// Show filter
    ///
    /// Defaul behavior is to show the filter for pickers with more than 10 items.
    pub show_filter: Option<bool>,
}

impl Combobox {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
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
        validate: impl IntoValidateFn<(String, Store<AttrValue>)>,
    ) -> Self {
        self.set_validate(validate);
        self
    }

    /// Method to set the validate callback
    pub fn set_validate(
        &mut self,
        validate: impl IntoValidateFn<(String, Store<AttrValue>)>,
    ) {
        self.validate = validate.into_validate_fn();
    }

    /// Builder style method to set the validation schema
    #[cfg(feature="proxmox-schema")]
    pub fn schema(mut self, schema: &'static Schema) -> Self {
        self.set_schema(schema);
        self
    }

    /// Method to set the validation schema
    #[cfg(feature="proxmox-schema")]
    pub fn set_schema(&mut self, schema: &'static Schema) {
        self.set_validate(move |(value, _store): &(String, _)| {
            schema.parse_simple_value(value)?;
            Ok(())
        });
    }

    /// Builder style method to set the show_filter flag.
    pub fn show_filter(mut self, show_filter: impl IntoPropValue<Option<bool>>) -> Self {
        self.set_show_filter(show_filter);
        self
    }

    /// Method to set the show_filter flag.
    pub fn set_show_filter(&mut self, show_filter: impl IntoPropValue<Option<bool>>) {
        self.show_filter = show_filter.into_prop_value();
    }
}

pub enum Msg {
    Reposition,
}

#[doc(hidden)]
pub struct PwtCombobox {
    store: Store<AttrValue>,
}

thread_local!{
    static COLUMNS: Rc<Vec<DataTableHeader<AttrValue>>> = Rc::new(vec![
        DataTableColumn::new("Value")
            .show_menu(false)
            .render(|value: &AttrValue| html!{value})
            .into(),
    ]);
}

impl Component for PwtCombobox {
    type Message = Msg;
    type Properties = Combobox;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let store = Store::with_extract_key(|item: &AttrValue| Key::from(item.as_str()));
        if !props.items.is_empty() {
            store.set_data(props.items.as_ref().clone());
        }

        Self { store }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Reposition => true, // just trigger a redraw
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let link = ctx.link().clone();

        let show_filter = props.show_filter.unwrap_or_else(|| {
            if self.store.data_len() > 10 { true } else { false }
        });

        let picker = move |args: &SelectorRenderArgs<Store<AttrValue>>| {
            // TODO use a simpler list widget without virtual scroll support?
            let table = DataTable::new(COLUMNS.with(Rc::clone), args.store.clone())
                //.class("pwt-fit")
                .show_header(false);

            let mut picker = GridPicker::new(table)
                .selection(args.selection.clone())
                .on_select(args.on_select.clone());

            if show_filter {
                picker.set_on_filter_change({
                    let link = link.clone();
                    move |_| {
                        link.send_message(Msg::Reposition);
                    }
                });
            }

            picker.into()
        };

        Selector::new(self.store.clone(), picker)
            .with_std_props(&props.std_props)
            .with_input_props(&props.input_props)
            .editable(props.editable)
            .default(&props.default)
            .validate(props.validate.clone())
            .on_change({
                let on_change = props.on_change.clone();
                move |key: Key| {
                    if let Some(on_change) = &on_change {
                        on_change.emit(key.to_string());
                    }
                }
            })
            .into()
    }
}
