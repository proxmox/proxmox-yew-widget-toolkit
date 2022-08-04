use std::borrow::Cow;
use yew::prelude::*;
use yew::virtual_dom::{Listeners, VNode, VList, VTag};
use yew::html::IntoPropValue;

use pwt_macros::widget;

use crate::prelude::*;
use crate::widget::Row;

#[widget(@element, @container)]
#[derive(Default, Debug, Clone)]
pub struct Panel {
    pub title: Option<AttrValue>,
    pub tools: Vec<VNode>,
}

impl Panel {

    pub fn new() -> Self {
        Self::default()
            .border(true)
    }

    pub fn title(mut self, title: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_title(title);
        self
    }

    pub fn set_title(&mut self, title: impl IntoPropValue<Option<AttrValue>>) {
        self.title = title.into_prop_value();
    }

    pub fn tool(mut self, tool: impl Into<VNode>) -> Self {
        self.add_tool(tool);
        self
    }

    pub fn add_tool(&mut self, tool: impl Into<VNode>) {
        self.tools.push(tool.into());
    }
}

impl Into<VTag> for Panel {

    fn into(mut self) -> VTag {

        self.add_class("pwt-panel");

        if self.title.is_some() || !self.tools.is_empty() {
            let header = create_panel_title(self.title, self.tools)
                .class("pwt-panel-header");
            self.children.insert(0, header.into());
        }

        let attributes = self.std_props.cumulate_attributes(None::<&str>);

        let listeners = Listeners::Pending(
            self.listeners.listeners.into_boxed_slice()
        );

        let children = VList::with_children(self.children, None);

        VTag::__new_other(
            Cow::Borrowed("div"),
            self.std_props.node_ref,
            self.std_props.key,
            attributes,
            listeners,
            children,
        )
    }
}

pub(crate) fn create_panel_title(title: Option<AttrValue>, tools: Vec<VNode>) -> Row {

    let mut header = Row::new()
        .class("pwt-align-items-center pwt-gap-1");

    if let Some(title) = title {
        header.add_child(html!{
            <div class="pwt-panel-header-text">{title}</div>
        });
    }

    if !tools.is_empty() {
        header.add_flex_spacer();
        header.add_child(VList::with_children(tools, None));
    }

    header
}
