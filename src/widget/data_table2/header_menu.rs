use std::rc::Rc;
use std::marker::PhantomData;

use derivative::Derivative;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};
use yew::html::IntoEventCallback;

use crate::prelude::*;
use crate::widget::{Column, Container, Fa};
use crate::widget::focus::{focus_next_tabable, init_roving_tabindex};

use super::IndexedHeader;

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct HeaderMenu<T: 'static> {
    pub key: Option<Key>,

    pub on_sort_change: Option<Callback<bool>>,
    pub on_hide_click: Option<Callback<usize>>,

    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    headers: Rc<Vec<IndexedHeader<T>>>,
    hidden: Vec<bool>,
}

impl<T: 'static> HeaderMenu<T> {

    pub fn new(headers: Rc<Vec<IndexedHeader<T>>>, hidden: &[bool]) -> Self {
        yew::props!(Self {
            headers,
            hidden: Vec::from(hidden),
        })
    }

    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn on_sort_change(mut self, cb: impl IntoEventCallback<bool>) -> Self {
        self.on_sort_change = cb.into_event_callback();
        self
    }

    pub fn on_hide_click(mut self, cb: impl IntoEventCallback<usize>) -> Self {
        self.on_hide_click = cb.into_event_callback();
        self
    }
}

fn headers_to_menu<T: 'static>(
    headers: &[IndexedHeader<T>],
    props: &HeaderMenu<T>,
    indent_level: usize,
    cell_idx: &mut usize,
    menu: &mut Vec<Html>,
) {

    let indent: Html = (0..indent_level)
        .map(|_| html!{ <span class="pwt-ps-4" /> })
        .collect();


    for header in headers {

        let onclick = {
            let on_hide_click = props.on_hide_click.clone();
            let cell_idx = *cell_idx;
            Callback::from(move |_| {
                if let Some(on_hide_click) = &on_hide_click {
                    on_hide_click.emit(cell_idx);
                }
            })
        };

        let onkeydown = {
            let on_hide_click = props.on_hide_click.clone();
            let cell_idx = *cell_idx;
            move |event: KeyboardEvent| {
                match event.key_code() {
                    32 => {
                        if let Some(on_hide_click) = &on_hide_click {
                            on_hide_click.emit(cell_idx);
                        }
                    }
                    _ => {}
                }
            }
        };

        let hidden = props.hidden.get(*cell_idx).map(|h| *h).unwrap_or(false);
        //let hidden = match hidden { true => html!{"X "}, false => html!{"V "} };
        let hidden = match hidden {
            true => html!{<i class="pwt-pe-2 fa fa-square-o"/>},
            false => html!{<i class="pwt-pe-2 fa fa-check-square-o"/>},
        };

        match header {
            IndexedHeader::Single(cell) => {
                let label = html!{<div>{hidden}{indent.clone()}{cell.column.name.clone()}</div>};
                menu.push(html!{<div class="pwt-menu-item" tabindex="-1" {onclick} {onkeydown}>{label}</div>});
                *cell_idx += 1;
            }
            IndexedHeader::Group(group) => {
                let label = html!{
                    <div>
                    {hidden}
                    {indent.clone()}
                    {group.name.clone()}
                    </div>
                };
                menu.push(html!{<div class="pwt-menu-item" tabindex="-1" {onclick} {onkeydown}>{label}</div>});
                *cell_idx += 1;
                headers_to_menu(&group.children, props, indent_level + 1, cell_idx, menu);
            }
        }
    }
}

pub struct PwtHeaderMenu<T: 'static> {
    _phantom: PhantomData<T>,
    inner_ref: NodeRef,
}

impl<T: 'static> Component for PwtHeaderMenu<T> {
    type Message = ();
    type Properties = HeaderMenu<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            _phantom: PhantomData::<T>,
            inner_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let inner_ref =  self.inner_ref.clone();

        let mut list = Column::new()
            .class("pwt-menu")
            .node_ref(inner_ref.clone())
            .onkeydown(move |event: KeyboardEvent| {
                match event.key_code() {
                    40 => {
                        focus_next_tabable(&inner_ref, false, true);
                    }
                    38 => {
                        focus_next_tabable(&inner_ref, true, true);
                    }
                    _ => return,
                }
                event.stop_propagation();
                event.prevent_default();
            })
            .with_child(
                Container::new()
                    .class("pwt-menu-item")
                    .attribute("tabindex", "-1")
                    .with_child(html!{
                        <>
                        {Fa::new("long-arrow-up").class("pwt-pe-2")}
                        {"Sort Ascending"}
                        </>
                    })
                    .onclick({
                        let on_sort_change = props.on_sort_change.clone();
                        move |event: MouseEvent| {
                            event.stop_propagation();
                            if let Some(on_sort_change) = &on_sort_change {
                                on_sort_change.emit(true);
                            }
                        }
                    })
                    .onkeydown({
                        let on_sort_change = props.on_sort_change.clone();
                        move |event: KeyboardEvent| {
                            match event.key_code() {
                                32 => {
                                    if let Some(on_sort_change) = &on_sort_change {
                                        on_sort_change.emit(true);
                                    }
                                }
                                _ => {}
                            }
                        }
                    })
            )
            .with_child(
                Container::new()
                    .class("pwt-menu-item")
                    .attribute("tabindex", "-1")
                    .with_child(html!{
                        <>
                        {Fa::new("long-arrow-down").class("pwt-pe-2")}
                        {"Sort Descending"}
                        </>
                    })
                    .onclick({
                        let on_sort_change = props.on_sort_change.clone();
                        move |event: MouseEvent| {
                            event.stop_propagation();
                            if let Some(on_sort_change) = &on_sort_change {
                                on_sort_change.emit(false);
                            }
                        }
                    })
                    .onkeydown({
                        let on_sort_change = props.on_sort_change.clone();
                        move |event: KeyboardEvent| {
                            match event.key_code() {
                                32 => {
                                    if let Some(on_sort_change) = &on_sort_change {
                                        on_sort_change.emit(false);
                                    }
                                }
                                _ => {}
                            }
                        }
                    })
               )
            .with_child(html!{<hr class="pwt-w-100 pwt-border-bottom"/>});

        let mut menu = Vec::new();
        headers_to_menu(&props.headers, props, 0, &mut 0, &mut menu);

        for item in menu {
            list.add_child(item);
        }


        list.into()
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            init_roving_tabindex(&self.inner_ref);
        }
    }
}

impl<T: 'static> Into<VNode> for HeaderMenu<T> {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtHeaderMenu<T>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
