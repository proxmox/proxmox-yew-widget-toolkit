use std::rc::Rc;

use serde_json::Value;
use indexmap::IndexMap;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode, Key};
use yew::html::IntoEventCallback;

use crate::props::RenderRecordFn;
use crate::widget::focus::focus_next_tabable;

#[derive(Clone, PartialEq)]
pub struct KVGridRow {
    pub name: String,
    pub header: String,
    pub required: bool,
    pub placeholder: Option<String>,
    pub renderer: Option<RenderRecordFn>,
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
        self.renderer = Some(RenderRecordFn::new(renderer));
    }

}

pub enum Msg {
    SelectItem(Key),
}

#[derive(Properties, PartialEq, Clone)]
pub struct KVGrid {
    rows: IndexMap<String, KVGridRow>,
    data: Rc<Value>,
    onselect: Option<Callback<Option<Key>>>,
    // Mouse single click or Space key
    onrowclick: Option<Callback<Key>>,
    // Mouse double click or Return key
    onrowdblclick: Option<Callback<Key>>,
 }

impl Into<VNode> for KVGrid {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtKVGrid>(Rc::new(self), NodeRef::default(), None);
        VNode::from(comp)
    }

}

impl KVGrid {

    pub fn new() -> Self {
        Self {
            rows: IndexMap::new(),
            data: Rc::new(Value::Null),
            onselect: None,
            onrowclick: None,
            onrowdblclick: None,
       }
    }

    pub fn data(mut self, data: Rc<Value>) -> Self {
        self.set_data(data);
        self
    }

    pub fn set_data(&mut self, data: Rc<Value>) {
        self.data = data;
    }

    pub fn onselect(mut self, cb: impl IntoEventCallback<Option<Key>>) -> Self {
        self.onselect = cb.into_event_callback();
        self
    }

    pub fn onrowclick(mut self, cb: impl IntoEventCallback<Key>) -> Self {
        self.onrowclick = cb.into_event_callback();
        self
    }

    pub fn onrowdblclick(mut self, cb: impl IntoEventCallback<Key>) -> Self {
        self.onrowdblclick = cb.into_event_callback();
        self
    }

    pub fn with_row(mut self, row: KVGridRow) -> Self {
        self.add_row(row);
        self
    }

    pub fn add_row(&mut self, row: KVGridRow) {
        let name = row.name.clone();
        self.rows.insert(name, row);
    }

    pub fn rows(mut self, rows: Vec<KVGridRow>) -> Self {
        self.set_rows(rows);
        self
    }

    pub fn set_rows(&mut self, rows: Vec<KVGridRow>) {
        for row in rows {
            self.rows.insert(row.name.clone(), row);
        }
    }

    pub fn get_header(&self, name: &str) -> Option<&str> {
        self.rows
            .get(name)
            .map(|row| row.header.as_str())
    }

    pub fn get_row(&self, name: &str) -> Option<&KVGridRow> {
        self.rows.get(name)
    }
}

pub struct PwtKVGrid {
    inner_ref: NodeRef,
    selection: Option<Key>,
}

impl Component for PwtKVGrid {
    type Message = Msg;
    type Properties = KVGrid;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            inner_ref: NodeRef::default(),
            selection: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::SelectItem(key) => {
                self.selection = Some(key.clone());

                if let Some(onselect) = &props.onselect {
                    onselect.emit(Some(key.clone()));
                }
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        let props = ctx.props();

        let old_selection = self.selection.clone();

        if let Some(key) = &self.selection {
            let always_showed = props.get_row(key).map(|row| {
                row.required || row.placeholder.is_some()
            }).unwrap_or(false);
            if !always_showed {
                match props.data.as_ref() {
                    Value::Object(map) => {
                        let key_str: &str = &key;
                        if !map.contains_key(key_str) {
                            self.selection = None;
                        }
                    }
                    _ => {
                        self.selection = None;
                    }
                }
            }
        }

        if old_selection != self.selection {
            if let Some(onselect) = &props.onselect {
                onselect.emit(self.selection.clone());
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut visible_rows: Vec<&KVGridRow> = Vec::new();
        let mut has_selected_item = false;

        for row in props.rows.values() {
            let name = row.name.as_str();
            if props.data.get(name).is_some() || row.placeholder.is_some() || row.required {
                visible_rows.push(row);
            }
            if let Some(selection) = &self.selection {
                if &**selection == name {
                    has_selected_item = true;
                }
            }
        }

        let items: Html = visible_rows.iter().enumerate().map(|(i, row)| {
            let name = row.name.as_str();
            let key = Key::from(name);
            let item = match props.data.get(name) {
                None => {
                    if let Some(placeholder) = &row.placeholder {
                        placeholder.to_string().into()
                    } else {
                        Value::Null
                    }
                }
                Some(value) => value.clone(),
            };

            let selected = if let Some(selection) = &self.selection {
                selection == &key
            } else {
                false
            };

            let class = selected.then(|| "selected");

            let tabindex = if has_selected_item  {
                if selected { "0" } else { "-1" }
            } else {
                if i == 0 { "0" } else { "-1" }
            };

            let onclick = ctx.link().callback({
                let key = key.clone();
                let callback = props.onrowclick.clone();
                move |_| {
                    if let Some(callback) = &callback {
                        callback.emit(key.clone());
                    }
                    Msg::SelectItem(key.clone())
                }
            });

            let onkeydown = ctx.link().batch_callback({
                let key = key.clone();
                let onrowclick = props.onrowclick.clone();
                let onrowdblclick = props.onrowdblclick.clone();
                move |event: KeyboardEvent| {
                    match event.key_code() {
                        32 => { // Space
                            if let Some(onrowclick) = &onrowclick {
                                onrowclick.emit(key.clone());
                            }
                            Some(Msg::SelectItem(key.clone()))
                        }
                        13 => { // Return
                            event.stop_propagation();
                            event.prevent_default();
                            if let Some(onrowdblclick) = &onrowdblclick {
                                onrowdblclick.emit(key.clone());
                            }
                            Some(Msg::SelectItem(key.clone()))
                        }
                        _ => None,
                    }
                }
            });

            let text = match &row.renderer {
                Some(renderer) => renderer.apply(&key, &item, &props.data),
                None => render_value(&item),
            };

            let ondblclick = Callback::from({
                let key = key.clone();
                let callback = props.onrowdblclick.clone();
                move |_| {
                    if let Some(callback) = &callback {
                        callback.emit(key.clone());
                    }
                }
            });

            html!{
                <tr {tabindex} {onclick} {ondblclick} {onkeydown} {key} {class}>
                    <td>{&row.header}</td>
                    <td width="100%" style="max-width:0px;">{text}</td>
                    </tr>
            }

        }).collect();


        let inner_ref =  self.inner_ref.clone();
        let onkeydown = Callback::from( move |event: KeyboardEvent| {
            match event.key_code() {
                40 => { // down
                    focus_next_tabable(&inner_ref, false, true);
                }
                38 => { // up
                    focus_next_tabable(&inner_ref, true, true);
                }
                _ => return,
            }
            event.prevent_default();
        });

        html!{
            <table class="pwt-table table-hover table-striped pwt-p-2">
                <tbody ref={self.inner_ref.clone()} {onkeydown}>{items}</tbody>
            </table>
        }
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
