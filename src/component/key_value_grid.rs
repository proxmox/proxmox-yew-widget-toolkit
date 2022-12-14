use std::rc::Rc;

use serde_json::Value;
use indexmap::IndexMap;
use derivative::Derivative;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode, Key};
use yew::html::IntoEventCallback;

use crate::props::{IntoEventCallbackMut, CallbackMut};
use crate::state::{Selection, Store};
use crate::widget::data_table2::{
    DataTable, DataTableColumn, DataTableHeader, DataTableMouseEvent,
    DataTableKeyboardEvent,
};

/// For use with KVGrid
#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct RenderKVGridRecordFn(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&str, &Value, &Value) -> Html>
);

impl RenderKVGridRecordFn {
    /// Creates a new [`RenderKVGridRecordFn`]
    pub fn new(renderer: impl 'static + Fn(&str, &Value, &Value) -> Html) -> Self {
        Self(Rc::new(renderer))
    }
}


#[derive(Clone, PartialEq)]
pub struct KVGridRow {
    pub name: String,
    pub header: String,
    pub required: bool,
    pub placeholder: Option<String>,
    pub renderer: Option<RenderKVGridRecordFn>,
}

impl KVGridRow {

    pub fn new(name: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            header: header.into(),
            required: false,
            placeholder: None,
            renderer: None,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.set_required(required);
        self
    }

    pub fn set_required(&mut self, required: bool) {
        self.required = required;
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.set_placeholder(placeholder);
        self
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        let placeholder = placeholder.into();
        if placeholder.is_empty() {
            self.placeholder = None;
        } else {
            self.placeholder = Some(placeholder.into());
        }
    }

    pub fn renderer(mut self, renderer: impl 'static + Fn(&str, &Value, &Value) -> Html) -> Self {
        self.set_renderer(renderer);
        self
    }

    pub fn set_renderer(&mut self, renderer: impl 'static + Fn(&str, &Value, &Value) -> Html) {
        self.renderer = Some(RenderKVGridRecordFn::new(renderer));
    }

}

#[derive(Properties, PartialEq, Clone)]
pub struct KVGrid {
    rows: Rc<Vec<KVGridRow>>,
    data: Rc<Value>,
    /// Select callback.
    pub on_select: Option<Callback<Option<Key>>>,
    /// Row click callback.
    pub on_row_click: Option<CallbackMut<DataTableMouseEvent>>,
    /// Row double click callback.
    pub on_row_dblclick: Option<CallbackMut<DataTableMouseEvent>>,
    /// Row keydown callback.
    pub on_row_keydown: Option<CallbackMut<DataTableKeyboardEvent>>,
}

impl Into<VNode> for KVGrid {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtKVGrid>(Rc::new(self), None);
        VNode::from(comp)
    }

}

impl KVGrid {

    pub fn new() -> Self {
        yew::props!(Self {
            rows: Rc::new(Vec::new()),
            data: Rc::new(Value::Null),
        })
    }

    pub fn data(mut self, data: Rc<Value>) -> Self {
        self.set_data(data);
        self
    }

    pub fn set_data(&mut self, data: Rc<Value>) {
        self.data = data;
    }

    pub fn on_select(mut self, cb: impl IntoEventCallback<Option<Key>>) -> Self {
        self.on_select = cb.into_event_callback();
        self
    }

    /// Builder style method to set the row click callback.
    pub fn on_row_click(mut self, cb: impl IntoEventCallbackMut<DataTableMouseEvent>) -> Self {
        self.on_row_click = cb.into_event_cb_mut();
        self
    }

    /// Builder style method to set the row double click callback.
    pub fn on_row_dblclick(mut self, cb: impl IntoEventCallbackMut<DataTableMouseEvent>) -> Self {
        self.on_row_dblclick = cb.into_event_cb_mut();
        self
    }

    /// Builder style method to set the row keydown callback.
    pub fn on_row_keydown(mut self, cb: impl IntoEventCallbackMut<DataTableKeyboardEvent>) -> Self {
        self.on_row_keydown = cb.into_event_cb_mut();
        self
    }

    pub fn with_row(mut self, row: KVGridRow) -> Self {
        self.add_row(row);
        self
    }

    pub fn add_row(&mut self, row: KVGridRow) {
        Rc::make_mut(&mut self.rows).push(row);
    }

    pub fn rows(mut self, rows: Rc<Vec<KVGridRow>>) -> Self {
        self.set_rows(rows);
        self
    }

    pub fn set_rows(&mut self, rows: Rc<Vec<KVGridRow>>) {
        self.rows = rows;
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.get_row(name)
            .map(|row| row.header.as_str())
    }

    pub fn get_row(&self, name: &str) -> Option<&KVGridRow> {
        // fixme: replace with somthing faster
        self.rows.iter().find(|row| row.name == name)
    }
}

struct Record {
    row: Rc<KVGridRow>,
    value: Value,
    store: Rc<Value>,
}

#[doc(hidden)]
pub struct PwtKVGrid {
    rows: Rc<IndexMap<String, Rc<KVGridRow>>>,
    store: Store<Record>,
    selection: Selection,
}

thread_local!{
    static COLUMNS: Rc<Vec<DataTableHeader<Record>>> = Rc::new(vec![
        DataTableColumn::new("Key")
            .show_menu(false)
            .render(|record: &Record| html!{record.row.header.clone()})
            .into(),
        DataTableColumn::new("Value")
            .width("100%")
            .show_menu(false)
            .render(|record: &Record|  {
                match &record.row.renderer {
                    Some(renderer) => (renderer.0)(&record.row.name, &record.value, &record.store),
                    None => render_value(&record.value),
                }
            })
            .into(),
    ]);
}

impl PwtKVGrid {

    fn data_update(&mut self, props: &KVGrid) {
        let mut visible_rows: Vec<Record> = Vec::new();

        for row in self.rows.values() {
            let name = row.name.as_str();
            let value = props.data.get(name);

            if value.is_some() || row.placeholder.is_some() || row.required {
                let value = match value {
                    None => {
                        if let Some(placeholder) = &row.placeholder {
                            placeholder.to_string().into()
                        } else {
                            Value::Null
                        }
                    }
                    Some(value) => value.clone(),
                };

                visible_rows.push(Record {
                    row: Rc::clone(row),
                    value,
                    store: Rc::clone(&props.data),
                });
            }
        }
        self.store.set_data(visible_rows);
    }
}

impl Component for PwtKVGrid {
    type Message = ();
    type Properties = KVGrid;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let rows: IndexMap<String, Rc<KVGridRow>> = props.rows
            .iter()
            .map(|row| (row.name.clone(), Rc::new(row.clone())))
            .collect();

        let store = Store::with_extract_key(|record: &Record| Key::from(record.row.name.as_str()));

        let selection = Selection::new()
            .on_select({
                let on_select = props.on_select.clone();
                move |selection: Selection| {
                    if let Some(on_select) = &on_select {
                        on_select.emit(selection.selected_key());
                    }
                }
            });

        let mut me = Self {
            rows: Rc::new(rows),
            store,
            selection,
        };
        me.data_update(props);
        me
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();

        if props.data != old_props.data || props.rows != old_props.rows {
            self.data_update(props);
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        DataTable::new(COLUMNS.with(Rc::clone), self.store.clone())
            .virtual_scroll(false)
            .show_header(false)
            .selection(self.selection.clone())
            .on_row_click(props.on_row_click.clone())
            .on_row_dblclick(props.on_row_dblclick.clone())
            .on_row_keydown(props.on_row_keydown.clone())
            .into()
    }
}


fn render_value(value: &Value) -> Html {
    match value {
        Value::Null => html!{ {"NULL"} },
        Value::Bool(v) => html!{ {v.to_string()} },
        Value::Number(v) =>  html!{ {v.to_string()} },
        Value::String(v) => html!{ {v} },
        v =>  html!{ {v.to_string()} },
    }
}
