use std::rc::Rc;
use std::marker::PhantomData;

use derivative::Derivative;
use web_sys::HtmlInputElement;

use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::{Key, VComp, VNode};

use crate::prelude::*;
use crate::widget::{get_unique_element_id, Column, Container, DataTableColumn, Row};
use crate::widget::form::Input;
use crate::props::ExtractKeyFn;

#[derive(Derivative, Properties)]
// Note: use derivative to avoid Clone/PartialEq requirement on T
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct GridPicker<T>
where
    T: 'static,
{
    #[prop_or_default]
    node_ref: NodeRef,
    pub key: Option<Key>,

    pub columns: Vec<DataTableColumn<T>>,

    #[prop_or_default]
    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    pub items: Rc<Vec<T>>,

    pub selection: Option<usize>, // todo: multiselect??

    pub extract_key: Option<ExtractKeyFn<T>>,

    pub onselect: Option<Callback<Key>>,

    /// Filter change event.
    ///
    /// Filter change often change the number of displayed items, so
    /// the size of the widget is likely to change. This callback is
    /// useful to reposition the dropdown.
    pub on_filter_change: Option<Callback<()>>,

    #[prop_or(true)]
    pub show_header: bool,

    #[prop_or(true)]
    //#[prop_or(false)]
    pub show_filter: bool,
}

impl<T> GridPicker<T> {

    // Create a new instance.
    pub fn new(columns: Vec<DataTableColumn<T>>) -> Self {
        yew::props!(GridPicker<T> { columns })
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: ::yew::html::NodeRef) -> Self {
        self.set_node_ref(node_ref);
        self
    }

    /// Method to set the yew `node_ref`
    pub fn set_node_ref(&mut self, node_ref: ::yew::html::NodeRef) {
        self.node_ref = node_ref;
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoPropValue<Option<Key>>) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property
    pub fn set_key(&mut self, key: impl IntoPropValue<Option<Key>>) {
        self.key = key.into_prop_value();
    }

    pub fn data(mut self, data: Rc<Vec<T>>) -> Self {
        self.set_data(data);
        self
    }

    pub fn set_data(&mut self, data: Rc<Vec<T>>) {
        self.items = data;
    }

    pub fn extract_key(mut self, extract_fn: impl Into<ExtractKeyFn<T>>) -> Self {
        self.extract_key = Some(extract_fn.into());
        self
    }

    pub fn selection(mut self, selection: Option<usize>) -> Self {
        self.selection = selection;
        self
    }

    pub fn onselect(mut self, cb: impl IntoEventCallback<Key>) -> Self {
        self.onselect = cb.into_event_callback();
        self
    }

    pub fn on_filter_change(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_filter_change = cb.into_event_callback();
        self
    }

    pub fn show_header(mut self, show_header: bool) -> Self {
        self.show_header = show_header;
        self
    }

    pub fn show_filter(mut self, show_filter: bool) -> Self {
        self.set_show_filter(show_filter);
        self
    }

    pub fn set_show_filter(&mut self, show_filter: bool) {
        self.show_filter = show_filter;
    }
}
pub enum Msg {
    CursorDown,
    CursorUp,
    CursorSelect,
    FilterUpdate(String),
}

#[doc(hidden)]
pub struct PwtGridPicker<T> {
    _phantom: PhantomData<T>,
    filter: String,
    // fixme: last_data: Rc<Vec<T>> // track changes
    filtered_data: Vec<usize>,
    cursor: Option<usize>,
    unique_id: String,
}
impl<T: 'static> PwtGridPicker<T> {

    fn update_filter(&mut self, ctx: &Context<Self>, filter: String) {
        let props = ctx.props();
        self.filter = filter;
        if let Some(ref on_filter_change) = props.on_filter_change {
            on_filter_change.emit(());
        }
        self.filtered_data = props.items.iter().enumerate().filter_map(|(n, item)| {
            let key = match &props.extract_key {
                None => Key::from(n),
                Some(extract_fn) => extract_fn.apply(item),
            };

            if !self.filter.is_empty() {
                if !key.to_lowercase().contains(&self.filter) {
                    return None;
                }
            }
            Some(n)
        }).collect();

        self.cursor = None; // fixme
    }

    fn get_unique_item_id(&self, n: usize) -> String {
        format!("{}-item-{}", self.unique_id, n)
    }

    fn cursor_down(&mut self) {
        let len = self.filtered_data.len();
        if len == 0 {
            self.cursor = None;
            return;
        }
        self.cursor = match self.cursor {
            Some(n) => if (n + 1) < len { Some(n + 1) }  else { None },
            None => Some(0),
        };
    }

    fn cursor_up(&mut self) {
        let len = self.filtered_data.len();
        if len == 0 {
            self.cursor = None;
            return;
        }

        self.cursor = match self.cursor {
            Some(n) => if n > 0 { Some(n - 1) } else { None },
            None => Some(len - 1),
        }
    }
}


impl<T: 'static> Component for PwtGridPicker<T> {
    type Message = Msg;
    type Properties = GridPicker<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        Self {
            _phantom: PhantomData::<T>,
            filter: String::new(),
            filtered_data: props.items.iter().enumerate().filter_map(|(n, _)| Some(n)).collect(),
            cursor: None,
            unique_id: get_unique_element_id(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::FilterUpdate(value) => {
                self.update_filter(ctx, value);
                true
            }
            Msg::CursorSelect => {
                let cursor = match self.cursor {
                    Some(n) => n,
                    None => return false, // nothing to do
                };

                let n = self.filtered_data[cursor];

                if let Some(onselect) = &props.onselect {
                    let item = &props.items[n];

                    let key = match &props.extract_key {
                        None => Key::from(n),
                        Some(extract_fn) => extract_fn.apply(item),
                    };

                    onselect.emit(key);
                }
                false
            }
            Msg::CursorDown => {
                self.cursor_down();
                true
            }
            Msg::CursorUp => {
                self.cursor_up();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let headers: Html = props.columns.iter().map(|column| {
            html!{<th>{column.name.clone()}</th>}
        }).collect();

        let is_list = props.columns.len() == 1; // Simple listbox or grid ?

        let mut active_descendant = None;

        let options: Html = self.filtered_data.iter().enumerate().map(|(filtered_n, n)| {
            let n = *n;
            let item = &props.items[n];

            let key = match &props.extract_key {
                None => Key::from(n),
                Some(extract_fn) => extract_fn.apply(item),
            };

            let selected = props.selection.map(|sel| sel == n).unwrap_or(false);
            let is_active = self.cursor.map(|cursor| cursor == filtered_n).unwrap_or(false);

            if is_active {
                active_descendant = Some(self.get_unique_item_id(n));
            }

            let class = classes!(
                selected.then(|| "selected"),
                is_active.then(|| "row-cursor"),
            );
            let cell_class = String::from("pwt-text-truncate");

            let cells = props.columns.iter().enumerate().map(|(n, column)| {
                let item_style = format!("justify-content:{}; grid-column:{};", column.justify, n+1);
                let class = if selected { Some("selected") } else {None };

                html!{
                    <td {class} style={item_style} aria-hidden={is_list.then(|| "true")}><div class={&cell_class}>{ column.render.apply(item) }</div></td>
                }
            });

            // fixme: avoid multiple onclick handlers
            let onclick = Callback::from({
                let onselect = props.onselect.clone();
                let value = key.clone();
                move |_| {
                    if let Some(onselect) = &onselect {
                        onselect.emit(value.clone());
                    }
                }
            });

            let id = self.get_unique_item_id(n);
            let aria_selected = if selected { "true" } else { "false" };

            let mut row = Container::new()
                .tag("tr")
                .class(class)
                .attribute("id", id)
                .attribute("aria-selected", aria_selected)
                .onclick(onclick)
                .children(cells);


            if is_list {
                row.set_attribute("role", "option");
                row.set_attribute("aria-label", (*key).to_string());
            } else {
                row.set_attribute("role", "row");
            }

            row.key(key)
        }).collect();

        let onkeydown = Callback::from({
            let link = ctx.link().clone();
            move |event: KeyboardEvent| {
                match event.key_code() {
                    40 => { // down
                        link.send_message(Msg::CursorDown);
                    }
                    38 => { // up
                        link.send_message(Msg::CursorUp);
                    }
                    9 => { // tab
                        log::info!("TAB");
                        // fixme: impl?
                    }
                    13 => { // RETURN
                        link.send_message(Msg::CursorSelect);
                    }
                    _ => return,
                }
                event.prevent_default();
            }
        });

        let list_id = format!("{}-list", self.unique_id);

        let table = html! {
            <div class="pwt-flex-fill pwt-overflow-auto">
                <table id={list_id.clone()} role={if is_list { "listbox" } else {"grid"}} ref={props.node_ref.clone()} class="pwt-fit pwt-table table-hover table-striped pwt-border">
                if props.show_header { <thead><tr>{headers}</tr></thead> }
                <tbody>
                    {options}
                </tbody>
            </table>
            </div>
        };

        let mut view = Column::new()
            .class("pwt-flex-fill pwt-overflow-auto")
            .onkeydown(onkeydown);

        let filter_invalid = self.filtered_data.is_empty();

        if props.show_filter {
            let filter = Row::new()
                .attribute("role", "combobox")
                .attribute("aria-expanded", "true")
                .attribute("aria-activedescendant", active_descendant.clone())
                .attribute("aria-controls", list_id.clone())
                .attribute("aria-haspopup", if is_list { "listbox" } else {"grid"})
                 .gap(2)
                .class("pwt-p-2 pwt-border-bottom pwt-w-100 pwt-align-items-center")
                .with_child(html!{<label for="testinput">{"Filter"}</label>})
                .with_child(
                    Input::new()
                        .attribute("autocomplete", "off")
                        .class("pwt-input")
                        .class("pwt-w-100")
                        .class(if filter_invalid { "is-invalid" } else { "is-valid" })
                        .attribute("value", self.filter.clone())
                        .attribute("aria-invalid", filter_invalid.then(|| "true"))
                        .oninput(ctx.link().callback(move |event: InputEvent| {
                            let input: HtmlInputElement = event.target_unchecked_into();
                            Msg::FilterUpdate(input.value())
                        }))
                );

            view.add_child(filter);
            view.add_child(table);

        } else {

            view.set_attribute("tabindex", "0");
            view.set_attribute("style", "outline: 0;");

            view.set_attribute("role", "combobox");
            view.set_attribute("aria-expanded", "true");
            view.set_attribute("aria-activedescendant", active_descendant);
            view.set_attribute("aria-controls", list_id.clone());
            view.set_attribute("aria-haspopup", if is_list { "listbox" } else {"grid"});
            view.add_child(table);
        }

        view.add_optional_child(self.filtered_data.is_empty().then(|| html!{
            <div class="pwt-p-2 pwt-flex-fill pwt-overflow-auto">{"no data"}</div>
        }));

        view.into()
    }
}

impl<T: 'static> Into<VNode> for GridPicker<T> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtGridPicker<T>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
