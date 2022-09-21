use std::rc::Rc;
use std::marker::PhantomData;

use derivative::Derivative;
use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};
use yew::html::IntoEventCallback;

use crate::prelude::*;
use crate::widget::{Column, Container};

use super::Header;

#[derive(Properties)]
#[derive(Derivative)]
#[derivative(Clone(bound=""), PartialEq(bound=""))]
pub struct HeaderMenu<T: 'static> {

    pub on_sort_change: Option<Callback<bool>>,

    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
    headers: Rc<Vec<Header<T>>>,
}

impl<T: 'static> HeaderMenu<T> {

    pub fn new(headers: Rc<Vec<Header<T>>>) -> Self {
        yew::props!(Self { headers })
    }

    pub fn on_sort_change(mut self, cb: impl IntoEventCallback<bool>) -> Self {
        self.on_sort_change = cb.into_event_callback();
        self
    }

}

fn headers_to_menu<T: 'static>(
    headers: &[Header<T>],
    indent_level: usize,
    menu: &mut Vec<Html>,
) {

    let indent: Html = (0..indent_level)
        .map(|_| html!{ <span class="pwt-ps-4" /> })
        .collect();
    
    for header in headers { 
        match header {
            Header::Single(column) => {
                let label = html!{<>{indent.clone()}{column.name.clone()}</>};
                menu.push(label);
            }
            Header::Group(group) => {
                let label = html!{
                    <>
                    {indent.clone()}
                    {group.content.clone().unwrap_or(html!{})}
                    </>
                };
                menu.push(label);
                headers_to_menu(&group.children, indent_level + 1, menu);
            }
        }
    }
}

pub struct PwtHeaderMenu<T: 'static> {
    _phantom: PhantomData<T>,
}

impl<T: 'static> Component for PwtHeaderMenu<T> {
    type Message = ();
    type Properties = HeaderMenu<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            _phantom: PhantomData::<T>,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut list = Column::new()
            .class("pwt-menu")
            .with_child(html!{<div class="pwt-p-2">{"THIS IS A TEST MENU"}</div>})
            .with_child(
                Container::new()
                    .class("pwt-menu-item")
                    .with_child("Sort Ascending")
                    .onclick({
                        let on_sort_change = props.on_sort_change.clone();
                        move |event: MouseEvent| {
                            event.stop_propagation();
                            if let Some(on_sort_change) = &on_sort_change {
                                on_sort_change.emit(true);
                            }
                        }
                    })
            )
            .with_child(
                Container::new()
                    .class("pwt-menu-item")
                    .with_child("Sort Descending")
                    .onclick({
                        let on_sort_change = props.on_sort_change.clone();
                        move |event: MouseEvent| {
                            event.stop_propagation();
                            if let Some(on_sort_change) = &on_sort_change {
                                on_sort_change.emit(false);
                            }
                        }
                    })
            )
            .with_child(html!{<hr class="pwt-w-100 pwt-border-bottom"/>});

        let mut menu = Vec::new();
        headers_to_menu(&props.headers, 0, &mut menu);
        
        for item in menu {
            list.add_child(html!{<div class="pwt-menu-item">{item}</div>});
        }
        
        
        list.into()
    }
}

impl<T: 'static> Into<VNode> for HeaderMenu<T> {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtHeaderMenu<T>>(Rc::new(self), None);
        VNode::from(comp)
    }
}
