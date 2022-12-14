use std::rc::Rc;

use serde_json::Value;
use indexmap::IndexMap;
use derivative::Derivative;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode, Key};

use crate::prelude::*;
use crate::props::{LoadCallback, IntoLoadCallback, SubmitCallback, IntoSubmitCallback};
use crate::state::Loader;
use crate::widget::{Button, Toolbar};
use crate::widget::form::FormContext;
use crate::component::{EditWindow, KVGrid, KVGridRow};

use crate::widget::data_table::{DataTableKeyboardEvent, DataTableMouseEvent};

#[derive(Derivative)]
#[derivative(Clone, PartialEq)]
pub struct RenderObjectGridItemFn(
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    Rc<dyn Fn(&FormContext, &str, &Value, &Value) -> Html>
);

impl RenderObjectGridItemFn {
    /// Creates a new [`RenderObjectGridItemFn`]
    pub fn new(renderer: impl 'static + Fn(&FormContext, &str, &Value, &Value) -> Html) -> Self {
        Self(Rc::new(renderer))
    }
}

#[derive(Clone, PartialEq)]
pub struct ObjectGridRow {
    row: KVGridRow,
    editor: Option<RenderObjectGridItemFn>,
}

impl ObjectGridRow {

    pub fn new(name: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            row: KVGridRow::new(name, header),
            editor: None,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.set_required(required);
        self
    }

    pub fn set_required(&mut self, required: bool) {
        self.row.set_required(required);
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.set_placeholder(placeholder);
        self
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.row.set_placeholder(placeholder);
    }

    pub fn renderer(mut self, renderer: impl 'static + Fn(&str, &Value, &Value) -> Html) -> Self {
        self.set_renderer(renderer);
        self
    }

    pub fn set_renderer(&mut self, renderer: impl 'static + Fn(&str, &Value, &Value) -> Html) {
        self.row.set_renderer(renderer);
    }

    pub fn editor(
        mut self,
        editor: impl 'static + Fn(&FormContext, &str, &Value, &Value) -> Html,
    ) -> Self {
        self.editor = Some(RenderObjectGridItemFn::new(editor));
        self

    }
}

pub enum Msg {
    DataChange,
    Select(Option<Key>),
    Edit,
    Close,
}

#[derive(Properties, PartialEq, Clone)]
pub struct ObjectGrid {
    editable: bool,
    grid: KVGrid,

    loader: Option<LoadCallback<Value>>,
    editors: IndexMap<String, RenderObjectGridItemFn>,

    data: Option<Value>,

    onsubmit: Option<SubmitCallback>,
}

impl Into<VNode> for ObjectGrid {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtObjectGrid>(Rc::new(self), None);
        VNode::from(comp)
    }

}

impl ObjectGrid {

    pub fn new() -> Self {
        Self {
            loader: None,
            grid: KVGrid::new(),
            editors: IndexMap::new(),
            editable: false,

            data: None,
            onsubmit: None,
       }
    }

    pub fn loader(mut self, callback: impl IntoLoadCallback<Value>) -> Self {
        self.loader = callback.into_load_callback();
        self
    }

    pub fn onsubmit(mut self, callback: impl IntoSubmitCallback) -> Self {
        self.onsubmit = callback.into_submit_callback();
        self
    }

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }

    pub fn with_row(mut self, row: ObjectGridRow) -> Self {
        self.add_row(row);
        self
    }

    pub fn add_row(&mut self, row: ObjectGridRow) {
        if let Some(editor) = row.editor {
            let name = row.row.name.clone();
            self.editors.insert(name, editor);
        }
        self.grid.add_row(row.row); // fixme
    }

    pub fn rows(mut self, rows: Vec<ObjectGridRow>) -> Self {
        self.set_rows(rows);
        self
    }

    pub fn set_rows(&mut self, rows: Vec<ObjectGridRow>) {
        for row in rows {
            self.add_row(row);
        }
    }
}

#[doc(hidden)]
pub struct PwtObjectGrid {
    loader: Loader<Value>,
    selection: Option<Key>,
    show_dialog: bool,
}

impl PwtObjectGrid  {

    fn data(&self) -> Value {
        self.loader.with_state(|loader| {
            match &loader.data {
                Some(Ok(data)) => data.as_ref().clone(),
                _ => Value::Null,
            }
        })
    }

    fn edit_dialog(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let name = self.selection.as_ref().unwrap().to_string();
        let row = props.grid.get_row(&name).unwrap();
        let title = &row.header;

        let data = self.data();
        let value = data[&name].clone();

        let editor = props.editors.get(&name).unwrap().clone();

        EditWindow::new(format!("Edit: {}", title))
            .loader(props.loader.clone())
            .ondone(Some(ctx.link().callback(|_| Msg::Close)))
            .renderer(move |form_state| (editor.0)(&form_state, &name, &value, &data))
            .onsubmit(props.onsubmit.clone())
            .into()
    }
}

impl Component for PwtObjectGrid {
    type Message = Msg;
    type Properties = ObjectGrid;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let loader = Loader::new(ctx.link().callback(|_| Msg::DataChange))
            .loader(props.loader.clone());

        loader.load();

        Self {
            loader,
            selection: None,
            show_dialog: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::DataChange => true,
            Msg::Edit => {
                self.show_dialog = true;
                true
            }
            Msg::Close => {
                self.show_dialog = false;
                self.loader.load();
                true
            }
            Msg::Select(opt_key) => {
                self.selection = opt_key;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let main_view = self.loader.render(|data| self.main_view(ctx, data));

        let disable_edit = if let Some(key) = &self.selection {
            let name: &str = &*key;
            !props.editors.contains_key(name)
        } else {
            true
        };


        html!{
            <>
                if props.editable {
                    {self.toolbar(ctx, disable_edit)}
                    if self.show_dialog && !disable_edit {{self.edit_dialog(ctx)}}
                }
                {main_view}
            </>
        }
    }
}

impl PwtObjectGrid {

    fn toolbar(&self, ctx: &Context<Self>, disable_edit: bool) -> Html {
        Toolbar::new()
            .class("pwt-border-bottom")
            .with_child(
                Button::new("Edit")
                    .disabled(disable_edit)
                    .onclick(ctx.link().callback(|_| Msg::Edit))
            )
            .with_flex_spacer()
            .with_child(self.loader.reload_button())
            .into()
    }

    fn main_view(&self, ctx: &Context<Self>, data: Rc<Value>) -> Html {
        ctx.props().grid.clone()
            .data(data)
            .on_select(ctx.link().callback(|key| Msg::Select(key)))
            .on_row_dblclick({
                let link = ctx.link().clone();
                move |event: &mut DataTableMouseEvent| {
                    link.send_message(Msg::Select(Some(event.record_key.clone())));
                    link.send_message(Msg::Edit);
                }
            })
            .on_row_keydown({
                let link = ctx.link().clone();
                move |event: &mut DataTableKeyboardEvent| {
                    if event.key() == " " {
                        link.send_message(Msg::Select(Some(event.record_key.clone())));
                        link.send_message(Msg::Edit);
                    }
                }
            })
            .into()
    }
}
