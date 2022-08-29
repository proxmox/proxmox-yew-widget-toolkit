use std::rc::Rc;
use std::marker::PhantomData;

use derivative::Derivative;
use yew::prelude::*;
use yew::html::{IntoEventCallback, IntoPropValue};
use yew::virtual_dom::{Key, VComp, VNode};

use crate::widget::DataTableColumn;
use crate::widget::focus::focus_next_tabable;
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

    #[prop_or(true)]
    pub show_header: bool,
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

    pub fn show_header(mut self, show_header: bool) -> Self {
        self.show_header = show_header;
        self
    }
}

#[doc(hidden)]
pub struct PwtGridPicker<T> {
    _phantom: PhantomData<T>,
}

impl<T: 'static> Component for PwtGridPicker<T> {
    type Message = ();
    type Properties = GridPicker<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            _phantom: PhantomData::<T>,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let headers: Html = props.columns.iter().map(|column| {
            html!{<th>{column.name.clone()}</th>}
        }).collect();

        let is_list = props.columns.len() == 1; // Simple listbox or grid ?

        let options: Html = props.items.iter().enumerate().map(|(n, item)| {
            let selected = props.selection.map(|sel| sel == n).unwrap_or(false);
            let tabindex = if selected { "0" } else { "-1" };
            let class = classes!(selected.then(|| "selected"));
            let cell_class = String::from("pwt-text-truncate");

            let cells: Html = props.columns.iter().enumerate().map(|(n, column)| {
                let item_style = format!("justify-content:{}; grid-column:{};", column.justify, n+1);
                let class = if selected { Some("selected") } else {None };

                html!{
                    <td {class} style={item_style} aria-hidden={is_list.then(|| "true")}><div class={&cell_class}>{ column.render.apply(item) }</div></td>
                }
            }).collect();

            let key = match &props.extract_key {
                None => Key::from(n),
                Some(extract_fn) => extract_fn.apply(item),
            };

            let onclick = Callback::from({
                let onselect = props.onselect.clone();
                let value = key.clone();
                move |_| {
                    if let Some(onselect) = &onselect {
                        onselect.emit(value.clone());
                    }
                }
            });

            let onkeydown = Callback::from({
                let onselect = props.onselect.clone();
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
            let node_ref = props.node_ref.clone();
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
            <table role={if is_list { "listbox" } else {"grid"}} ref={props.node_ref.clone()} class="pwt-table pwt-fit table-hover table-striped pwt-border" {onkeydown}>
                if props.show_header { <thead><tr>{headers}</tr></thead> }
                <tbody>{options}</tbody>
            </table>
        }
    }
}

impl<T: 'static> Into<VNode> for GridPicker<T> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtGridPicker<T>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
