use std::marker::PhantomData;
use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::props::WidgetBuilder;

use crate::dom::DomVisibilityObserver;

/// Observe visibility changes.
///
/// This is a wrapper around another widget, which sets up a [DomVisibilityObserver]
/// to track visibility changes.
///
/// ```
/// # use pwt::widget::{Panel, VisibilityObserver};
/// # fn test_visibility_observer() {
/// VisibilityObserver::new(
///     Panel::new().title("My Panel"),
///     |visible| {
///         log::info!("Panel visibility changed: {visible}");
///     }
/// );
/// # }
/// ```
#[derive(Clone, PartialEq, Properties)]
pub struct VisibilityObserver<W: WidgetBuilder + PartialEq + Clone> {
    content: W,
    on_visibility_change: Callback<bool>,
}

impl<W: WidgetBuilder + PartialEq + Clone + 'static> VisibilityObserver<W> {
    /// Creates a new instance.
    pub fn new(content: W, on_visibility_change: impl Into<Callback<bool>>) -> Self {
        yew::props!(Self {
            content,
            on_visibility_change: on_visibility_change.into(),
        })
    }
}

#[doc(hidden)]
pub struct PwtVisibilityObserver<W> {
    observer: Option<DomVisibilityObserver>,
    _phantom: PhantomData<W>,
}

pub enum Msg {
    VisibilityChange(bool),
}

impl<W: WidgetBuilder + PartialEq + Clone + 'static> Component for PwtVisibilityObserver<W> {
    type Message = Msg;
    type Properties = VisibilityObserver<W>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            observer: None,
            _phantom: PhantomData,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let widget = props.content.clone();
        widget.into()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            Msg::VisibilityChange(visible) => {
                props.on_visibility_change.emit(visible);
                false
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let props = ctx.props();
            let node_ref = props.content.as_std_props().node_ref.clone();
            let el = node_ref.cast::<web_sys::Element>().unwrap();
            let link = ctx.link().clone();
            let observer = DomVisibilityObserver::new(&el, move |visible| {
                link.send_message(Msg::VisibilityChange(visible));
            });
            self.observer = Some(observer);
        }
    }
}

impl<W: WidgetBuilder + PartialEq + Clone + 'static> Into<VNode> for VisibilityObserver<W> {
    fn into(self) -> VNode {
        let key = self.content.as_std_props().key.clone();
        let comp = VComp::new::<PwtVisibilityObserver<W>>(Rc::new(self), key);
        VNode::from(comp)
    }
}
