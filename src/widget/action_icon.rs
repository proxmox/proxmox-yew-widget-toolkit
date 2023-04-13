use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode, Key};
use yew::html::{IntoEventCallback, IntoPropValue};

use crate::prelude::IntoOptionalKey;
use crate::props::{EventSubscriber, WidgetBuilder};
use crate::widget::Container;

/// A clickable icon. Like [Button](super::Button) without any decoration (inline element).
///
/// This component is useful in data tables because it is visually lighter than a button.
#[derive(Properties, PartialEq, Clone)]
pub struct ActionIcon {
    /// Yew component `ref`.
    #[prop_or_default]
    pub node_ref: NodeRef,

    /// Yew `key` property
    pub key: Option<Key>,

    /// CSS class
    #[prop_or_default]
    pub class: Classes,

    /// The CSS icon class
    pub icon_class: Option<Classes>,

    /// Html tabindex (defaults to -1)
    pub tabindex: Option<i32>,

    /// Aria label
    pub aria_label: Option<AttrValue>,

    /// Disable flag
    #[prop_or_default]
    pub disabled: bool,

    /// Activate callback (click, enter, space)
    pub on_activate: Option<Callback<()>>,
}

impl ActionIcon {

    /// Create a new instance.
    pub fn new(icon_class: impl Into<Classes>) -> Self {
        yew::props!(Self { icon_class: icon_class.into() })
    }

    /// Builder style method to set the yew `node_ref`
    pub fn node_ref(mut self, node_ref: NodeRef) -> Self {
        self.set_node_ref(node_ref);
        self
    }

    /// Method to set the yew `node_ref`
    pub fn set_node_ref(&mut self, node_ref: NodeRef) {
        self.node_ref = node_ref;
    }

    /// Builder style method to set the yew `key` property
    pub fn key(mut self, key: impl IntoOptionalKey) -> Self {
        self.set_key(key);
        self
    }

    /// Method to set the yew `key` property
    pub fn set_key(&mut self, key: impl IntoOptionalKey) {
        self.key = key.into_optional_key();
    }

    /// Builder style method to add a html class
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a html class
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class);
    }

    /// Builder style method to set the html tabindex attribute
    pub fn tabindex(mut self, index: impl IntoPropValue<Option<i32>>) -> Self {
        self.set_tabindex(index);
        self
    }

    /// Method to set the html tabindex attribute
    pub fn set_tabindex(&mut self, index: impl IntoPropValue<Option<i32>>) {
        self.tabindex = index.into_prop_value();
    }

    /// Builder style method to set the html aria-label attribute
    pub fn aria_label(mut self, label: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_aria_label(label);
        self
    }

    /// Method to set the html aria-label attribute
    pub fn set_aria_label(&mut self, label: impl IntoPropValue<Option<AttrValue>>) {
        self.aria_label = label.into_prop_value();
    }

    /// Builder style method to set the disabled flag
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.set_disabled(disabled);
        self
    }

    /// Method to set the disabled flag
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Builder style method to set the activate callback.
    pub fn on_activate(mut self, cb: impl IntoEventCallback<()>) -> Self {
        self.on_activate = cb.into_event_callback();
        self
    }
}

#[doc(hidden)]
pub struct PwtActionIcon;

impl Component for PwtActionIcon {
    type Message = ();
    type Properties = ActionIcon;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let disabled = props.disabled;

        let tabindex = match props.tabindex {
            Some(tabindex) => format!("{tabindex}"),
            None => String::from("-1"),
        };

        Container::new()
            .tag("i")
            .node_ref(props.node_ref.clone())
            .attribute("tabindex", (!disabled).then(|| tabindex))
            .attribute("role", "button")
            .attribute("aria-label", props.aria_label.clone())
            .class("pwt-action-icon")
            .class(props.disabled.then(|| "disabled"))
            .class(props.class.clone())
            .class(props.icon_class.clone())
            .onclick({
                let on_activate = props.on_activate.clone();
                move |event: MouseEvent| {
                    event.stop_propagation();
                    if disabled { return; }
                    if let Some(on_activate) = &on_activate {
                        on_activate.emit(());
                    }
                }
            })
            .onkeydown({
                let on_activate = props.on_activate.clone();
                move |event: KeyboardEvent| {
                    match event.key().as_ref() {
                        "Enter" | " " => {
                            event.stop_propagation();
                            if disabled { return; }
                            if let Some(on_activate) = &on_activate {
                                on_activate.emit(());
                            }
                        }
                        _ => {}
                    }
                }
            })
            // suppress double click to avoid confusion when used inside tables/trees
            .ondblclick(move |event: MouseEvent| { event.stop_propagation(); })
            .into()
    }
}

impl Into<VNode> for ActionIcon {
    fn into(self) -> VNode {
        let key = self.key.clone();
        let comp = VComp::new::<PwtActionIcon>(Rc::new(self), key);
        VNode::from(comp)
    }
}
