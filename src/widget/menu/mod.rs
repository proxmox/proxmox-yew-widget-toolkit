use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::prelude::*;
use crate::widget::Column;
use crate::widget::focus::{focus_next_tabable, init_roving_tabindex};

mod menu_item;
pub use menu_item::MenuItem;

/// Menu - A container for [MenuItem]s.
///
/// The container implements a roving focus to allow keyboard
/// navigation.
///
/// It is also possible to add any html as child.
#[derive(Clone, PartialEq, Properties)]
pub struct Menu {
    #[prop_or_default]
    children: Vec<VNode>,

    #[prop_or_default]
    pub class: Classes,
}

impl Menu {

    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to add a html class.
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class.
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }


    /// Builder style method to add a simple line to separate menu items.
    pub fn with_separator(mut self) -> Self {
        self.add_child(html!{<hr/>});
        self
    }
}

impl ContainerBuilder for Menu {
    fn as_children_mut(&mut self) -> &mut Vec<VNode> {
        &mut self.children
    }
}

#[doc(hidden)]
pub struct PwtMenu {
    inner_ref: NodeRef,
}

impl Component for PwtMenu {
    type Message = ();
    type Properties = Menu;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            inner_ref: NodeRef::default(),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let inner_ref =  self.inner_ref.clone();

        Column::new()
            .class("pwt-menu")
            .class(props.class.clone())
            .node_ref(self.inner_ref.clone())
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
            .children(props.children.clone())
            .into()
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            init_roving_tabindex(&self.inner_ref);
        }
    }
}

impl Into<VNode> for Menu {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtMenu>(Rc::new(self), None);
        VNode::from(comp)
    }
}
