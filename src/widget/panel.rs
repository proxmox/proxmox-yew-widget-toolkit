use std::borrow::Cow;
use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VNode, VTag};

use pwt_macros::widget;

use crate::prelude::*;
use crate::props::IntoOptionalInlineHtml;
use crate::widget::Row;

/// Container with header and body.
///
/// The header can contain tools, which are widgets displayed on the
/// right side of the header, like a help button.
#[widget(pwt=crate, @element, @container)]
#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Panel {
    /// Optional header text.
    #[prop_or_default]
    pub title: Option<Html>,
    /// Tools, displayed right aligned in the header.
    #[prop_or_default]
    pub tools: Vec<VNode>,
    /// Optional header CSS class.
    #[prop_or_default]
    pub header_class: Classes,
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Panel {
    /// Creates a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the title text.
    pub fn title(mut self, title: impl IntoOptionalInlineHtml) -> Self {
        self.set_title(title);
        self
    }

    /// Method to set the title text.
    pub fn set_title(&mut self, title: impl IntoOptionalInlineHtml) {
        self.title = title.into_optional_inline_html();
    }

    /// Builder style method to add a tool.
    pub fn with_tool(mut self, tool: impl Into<VNode>) -> Self {
        self.add_tool(tool);
        self
    }

    /// Method to add a tool.
    pub fn add_tool(&mut self, tool: impl Into<VNode>) {
        self.tools.push(tool.into());
    }

    /// Builder style method to add a header class.
    pub fn header_class(mut self, class: impl Into<Classes>) -> Self {
        self.add_header_class(class);
        self
    }

    /// Method to add a header class.
    pub fn add_header_class(&mut self, class: impl Into<Classes>) {
        self.header_class.push(class);
    }
}

impl From<Panel> for VTag {
    fn from(mut val: Panel) -> Self {
        val.add_class("pwt-panel");

        if val.title.is_some() || !val.tools.is_empty() {
            let header = create_panel_title(val.title, val.tools)
                .class("pwt-panel-header")
                .class(val.header_class);
            val.children.insert(0, header.into());
        }

        let attributes = val.std_props.cumulate_attributes(None::<&str>);

        let listeners = Listeners::Pending(val.listeners.listeners.into_boxed_slice());

        let children = VList::with_children(val.children, None);

        VTag::__new_other(
            Cow::Borrowed("div"),
            val.std_props.node_ref,
            val.std_props.key,
            attributes,
            listeners,
            children.into(),
        )
    }
}

pub(crate) fn create_panel_title(title: Option<Html>, tools: Vec<VNode>) -> Row {
    let mut header = Row::new()
        .attribute("role", "group")
        .attribute("aria-label", "panel header")
        .class("pwt-align-items-center pwt-gap-1");

    if let Some(title) = title {
        header.add_child(html! {
            <div role="none" class="pwt-panel-header-text">{title}</div>
        });
    }

    if !tools.is_empty() {
        header.add_flex_spacer();
        header.add_child(VList::with_children(tools, None));
    }

    header
}
