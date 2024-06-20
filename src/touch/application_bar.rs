use std::rc::Rc;

use yew::html::IntoPropValue;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::prelude::*;
use crate::widget::{ActionIcon, Column, Row};

use pwt_macros::builder;

use super::PageController;

/// Material Design application bar.
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct ApplicationBar {
    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// Leading widget placed before the title.
    ///
    /// By default, we add a back button if the context provides a [PageController].
    #[prop_or_default]
    pub leading: Option<Html>,

    /// Application title.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub title: Option<AttrValue>,

    #[prop_or_default]
    /// Actions, displayed right aligned in the header.
    pub actions: Vec<VNode>,

    /// Widget placed at the bottom, usually a [TabBar](crate::widget::TabBar).
    #[prop_or_default]
    pub bottom: Option<Html>,
}

impl ApplicationBar {
    /// Create a new instance.
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    // Builder style method to set the yew `key` property.
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property.
    pub fn set_key(&mut self, key: impl IntoOptionalKey) {
        self.key = key.into_optional_key();
    }

    /// Builder style method to set the leading widget.
    pub fn leading(mut self, leading: impl Into<VNode>) -> Self {
        self.set_leading(leading);
        self
    }

    /// Method to set the leading widget.
    pub fn set_leading(&mut self, leading: impl Into<VNode>) {
        self.leading = Some(leading.into());
    }

    /// Builder style method to add an action.
    pub fn with_action(mut self, action: impl Into<VNode>) -> Self {
        self.add_action(action);
        self
    }

    /// Method to add an action.
    pub fn add_action(&mut self, action: impl Into<VNode>) {
        self.actions.push(action.into());
    }

    /// Builder style method to add multiple actions.
    pub fn actions(mut self, actions: Vec<VNode>) -> Self {
        self.add_actions(actions);
        self
    }

    /// Method to add multiple actions.
    pub fn add_actions(&mut self, actions: Vec<VNode>) {
        self.actions.extend(actions);
    }

    /// Method to set the actions property.
    pub fn set_actions(&mut self, actions: Vec<VNode>) {
        self.actions = actions;
    }

    /// Builder style method to set the bottom widget.
    pub fn bottom(mut self, bottom: impl Into<VNode>) -> Self {
        self.set_bottom(bottom);
        self
    }

    /// Method to set the bottom widget.
    pub fn set_bottom(&mut self, bottom: impl Into<VNode>) {
        self.bottom = Some(bottom.into());
    }
}

#[doc(hidden)]
pub struct PwtApplicationBar {
    page_controller: Option<PageController>,
}

impl Component for PwtApplicationBar {
    type Message = ();
    type Properties = ApplicationBar;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let page_controller = ctx
            .link()
            .context::<PageController>(Callback::from(|_| {}))
            .map(|(c, _)| c);

        Self { page_controller }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let props = ctx.props();

        let mut actions = Row::new().gap(0);

        if !props.actions.is_empty() {
            actions.add_children(props.actions.clone())
        }

        let leading = props.leading.clone().or_else(|| {
            if self.page_controller.is_some() {
                Some(create_back_button(self.page_controller.clone()).into())
            } else {
                None
            }
        });

        let row1 = Row::new()
            .attribute("aria-label", props.title.clone())
            .class("pwt-application-bar-row1")
            .with_optional_child(leading)
            .with_child(html! {
                <span class="pwt-flex-fill pwt-font-headline-small pwt-text-truncate">{props.title.clone()}</span>
            })
            .with_child(actions);

        Column::new()
            .style("z-index", "1") // make shadow (if any) visible
            .attribute("role", "banner")
            .class("pwt-application-bar")
            .with_child(row1)
            .with_optional_child(props.bottom.clone())
            .into()
    }
}

impl Into<VNode> for ApplicationBar {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtApplicationBar>(Rc::new(self), key);
        VNode::from(comp)
    }
}

fn create_back_button(page_controller: Option<PageController>) -> ActionIcon {
    ActionIcon::new("fa fa-lg fa-arrow-left").on_activate({
        let page_controller = page_controller.clone();
        move |_| {
            if let Some(page_controller) = &page_controller {
                page_controller.last_page();
            }
        }
    })
}
