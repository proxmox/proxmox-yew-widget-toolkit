use std::rc::Rc;

use yew::prelude::*;
use yew::html::IntoEventCallback;
use yew::virtual_dom::{Key, VNode};

use crate::widget::DataTableColumn;
use crate::widget::focus::focus_next_tabable;
use crate::props::ExtractKeyFn;

#[derive(Properties, Clone)]
pub struct GridPicker<T>
where
    T: 'static,
{
    #[prop_or_default]
    node_ref: NodeRef,

    pub columns: Vec<DataTableColumn<T>>,
    #[prop_or_default]
    pub items: Rc<Vec<T>>,

    pub selection: Option<usize>, // todo: multiselect??

    pub extract_key: Option<ExtractKeyFn<T>>,

    pub onselect: Option<Callback<Key>>,

    #[prop_or(true)]
    pub show_header: bool,
}

// Avoid "T: PartialEq" by manual implementation
impl<T> PartialEq for GridPicker<T> {
    fn eq(&self, other: &Self) -> bool {
        self.columns == other.columns &&
            Rc::ptr_eq(&self.items, &other.items) &&
            self.node_ref == other.node_ref &&
            self.selection == other.selection &&
            self.onselect == other.onselect &&
            self.show_header == other.show_header
    }
}

impl<T> GridPicker<T> {

    pub fn new(columns: Vec<DataTableColumn<T>>) -> Self {
        yew::props!(GridPicker<T> { columns })
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

    pub fn show_header(mut self, show_header: bool) -> Self {
        self.show_header = show_header;
        self
    }

    pub fn html(&self) -> Html {

        let headers: Html = self.columns.iter().map(|column| {
            html!{<th>{column.name.clone()}</th>}
        }).collect();

        let is_list = self.columns.len() == 1; // Simple listbox or grid ?

        let options: Html = self.items.iter().enumerate().map(|(n, item)| {
            let selected = self.selection.map(|sel| sel == n).unwrap_or(false);
            let tabindex = if selected { "0" } else { "-1" };
            let class = classes!(selected.then(|| "selected"));
            let cell_class = String::from("pwt-text-truncate");

            let cells: Html = self.columns.iter().enumerate().map(|(n, column)| {
                let item_style = format!("justify-content:{}; grid-column:{};", column.justify, n+1);
                let class = if selected { Some("selected") } else {None };

                html!{
                    <td {class} style={item_style} aria-hidden={is_list.then(|| "true")}><div class={&cell_class}>{ column.render.apply(item) }</div></td>
                }
            }).collect();

            let key = match &self.extract_key {
                None => Key::from(n),
                Some(extract_fn) => extract_fn.apply(item),
            };

            let onclick = Callback::from({
                let onselect = self.onselect.clone();
                let value = key.clone();
                move |_| {
                    if let Some(onselect) = &onselect {
                        onselect.emit(value.clone());
                    }
                }
            });

            let onkeydown = Callback::from({
                let onselect = self.onselect.clone();
                let value = key.clone();
                move |event: KeyboardEvent| {
                    match event.key_code() {
                        32 | 13 => { // space | enter
                            if let Some(onselect) = &onselect {
                                onselect.emit(value.clone());
                            } else {
                                return;
                            }
                        }
                        _ => return,
                    }
                    event.prevent_default();
                }
            });

            if is_list {
                html!{
                    <tr role="option" aria-label={(*key).to_string()} {tabindex} {onclick} {onkeydown} {class}>{cells}</tr>
                }
            } else {
                html!{
                    <tr role="row" {tabindex} {class} {onclick} {onkeydown} aria-selected={selected.then(|| "true")}>{cells}</tr>
                }
            }
        }).collect();

        let onkeydown = Callback::from({
            let node_ref = self.node_ref.clone();
            move |event: KeyboardEvent| {
                match event.key_code() {
                    40 => { // down
                        focus_next_tabable(&node_ref, false, false);
                    }
                    38 => { // up
                        focus_next_tabable(&node_ref, true, false);
                    }
                    _ => return,
                }
                event.prevent_default();
            }
        });

        html! {
            <table role={if is_list { "listbox" } else {"grid"}} ref={self.node_ref.clone()} class="pwt-table pwt-fit table-hover table-striped pwt-border" {onkeydown}>
                if self.show_header { <thead><tr>{headers}</tr></thead> }
                <tbody>{options}</tbody>
            </table>
        }
    }
}

impl<T> Into<VNode> for GridPicker<T> {
    fn into(self) -> VNode {
        self.html()
    }
}
