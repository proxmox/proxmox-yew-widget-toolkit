use std::rc::Rc;

use yew::html::IntoEventCallback;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};
use crate::widget::{Container, Fa};

use super::{SlidableActionMouseEvent, SlidableController};

/// A button which automatically connects to the [SlidableController].
///
/// The [on_activate](Self::on_activate) handler gets an [SlidableActionMouseEvent] argument,
/// which lets you collapse or dismiss the slidable.
#[derive(Properties, Clone, PartialEq)]
pub struct SlidableAction {
    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// The action label.
    pub label: AttrValue,

    /// An optional CSS icon class.
    #[prop_or_default]
    pub icon_class: Option<Classes>,

    #[prop_or_default]
    /// Optional additional CSS classes
    pub class: Classes,

    /// Click callback.
    ///
    /// Emited when the user clicks on the entry.
    #[prop_or_default]
    pub on_activate: Option<Callback<SlidableActionMouseEvent>>,
}

impl SlidableAction {
    pub fn new(label: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            label: label.into()
        })
    }

    /// Builder style method to set the icon CSS class.
    pub fn icon_class(mut self, icon_class: impl Into<Classes>) -> Self {
        self.set_icon_class(icon_class);
        self
    }

    /// Method to set the icon CSS class.
    pub fn set_icon_class(&mut self, icon_class: impl Into<Classes>) {
        self.icon_class = Some(icon_class.into());
    }

    /// Builder style method to set the on_activate callback.
    pub fn on_activate(mut self, cb: impl IntoEventCallback<SlidableActionMouseEvent>) -> Self {
        self.on_activate = cb.into_event_callback();
        self
    }

    /// Builder style method to add a CSS class.
    pub fn class(mut self, class: impl Into<Classes>) -> Self {
        self.add_class(class);
        self
    }

    /// Method to add a CSS class.
    pub fn add_class(&mut self, class: impl Into<Classes>) {
        self.class.push(class.into());
    }
}

pub enum Msg {
    ControllerUpdate(SlidableController),
}

#[doc(hidden)]
pub struct PwtSlidableAction {
    _context_handle: Option<ContextHandle<SlidableController>>,
    controller: Option<SlidableController>,
}

impl Component for PwtSlidableAction {
    type Message = Msg;
    type Properties = SlidableAction;

    fn create(ctx: &Context<Self>) -> Self {
        let (controller, _context_handle) = match ctx
            .link()
            .context::<SlidableController>(ctx.link().callback(Msg::ControllerUpdate))
        {
            Some((controller, handle)) => (Some(controller), Some(handle)),
            None => (None, None),
        };

        Self {
            _context_handle,
            controller,
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ControllerUpdate(controller) => {
                self.controller = Some(controller);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let icon = props.icon_class.clone().map(Fa::from_class);

        let onclick = Callback::from({
            let controller = self.controller.clone();
            let on_activate = props.on_activate.clone();
            move |event: MouseEvent| {
                if let Some(on_activate) = &on_activate {
                    let event = SlidableActionMouseEvent::new(event);
                    on_activate.emit(event.clone());
                    if let Some(controller) = &controller {
                        if event.get_dismiss() {
                            controller.dismiss();
                        } else if !event.get_keep_open() {
                            controller.collapse();
                        }
                    }
                } else {
                    // Always collapse if action is without on_activate callback
                    if let Some(controller) = &controller {
                        controller.collapse();
                    }
                }
            }
        });

        Container::new()
            .class("pwt-slidable-action")
            .class(props.class.clone())
            .with_optional_child(icon)
            .with_child(props.label.clone())
            .onclick(onclick)
            .into()
    }
}

impl From<SlidableAction> for VNode {
    fn from(val: SlidableAction) -> Self {
        let key = val.key.clone();
        let comp = VComp::new::<PwtSlidableAction>(Rc::new(val), key);
        VNode::from(comp)
    }
}
