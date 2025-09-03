use std::marker::PhantomData;
use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::props::{IntoVTag, WidgetBuilder};

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
pub struct VisibilityObserver<W: WidgetBuilder + IntoVTag + PartialEq + Clone> {
    content: W,
    on_visibility_change: Callback<bool>,
}

impl<W: WidgetBuilder + IntoVTag + PartialEq + Clone + 'static> VisibilityObserver<W> {
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
    node_ref: NodeRef,
    observer: Option<DomVisibilityObserver>,
    _phantom: PhantomData<W>,
}

pub enum Msg {
    VisibilityChange(bool),
}

impl<W: WidgetBuilder + IntoVTag + PartialEq + Clone + 'static> Component
    for PwtVisibilityObserver<W>
{
    type Message = Msg;
    type Properties = VisibilityObserver<W>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
            observer: None,
            _phantom: PhantomData,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        props
            .content
            .clone()
            .into_html_with_ref(self.node_ref.clone())
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
            let node_ref = self.node_ref.clone();
            let el = node_ref.cast::<web_sys::Element>().unwrap();
            let link = ctx.link().clone();
            let observer = DomVisibilityObserver::new(&el, move |visible| {
                link.send_message(Msg::VisibilityChange(visible));
            });
            self.observer = Some(observer);
        }
    }
}

impl<W: WidgetBuilder + IntoVTag + PartialEq + Clone + 'static> From<VisibilityObserver<W>>
    for VNode
{
    fn from(val: VisibilityObserver<W>) -> Self {
        let key = val.content.as_std_props().key.clone();
        let comp = VComp::new::<PwtVisibilityObserver<W>>(Rc::new(val), key);
        VNode::from(comp)
    }
}
