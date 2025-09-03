use std::marker::PhantomData;
use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{VComp, VNode};

use crate::props::{IntoVTag, WidgetBuilder};

use crate::dom::{DomSizeObserver, IntoSizeCallback, SizeCallback};

/// Observe size changes.
///
/// This is a wrapper around another widget, which sets up a [DomSizeObserver]
/// to track size changes.
///
/// Only work for widgets implementing IntoVTag (because we need to assign a NodeRef)
///
/// ```
/// # use pwt::widget::{Panel, SizeObserver};
/// # fn test_size_observer() {
/// SizeObserver::new(
///     Panel::new().title("My Panel"),
///     |(width, height)| {
///         log::info!("Panel size changed: {width}x{height}");
///     }
/// );
/// # }
/// ```
#[derive(Clone, PartialEq, Properties)]
pub struct SizeObserver<W: WidgetBuilder + IntoVTag + PartialEq + Clone> {
    content: W,
    on_resize: SizeCallback,
}

impl<W: WidgetBuilder + IntoVTag + PartialEq + Clone + 'static> SizeObserver<W> {
    /// Creates a new instance.
    pub fn new<X>(content: W, on_resize: impl IntoSizeCallback<X>) -> Self {
        yew::props!(Self {
            content,
            on_resize: on_resize.into_size_cb()
        })
    }
}

#[doc(hidden)]
pub struct PwtSizeObserver<W> {
    node_ref: NodeRef,
    observer: Option<DomSizeObserver>,
    _phantom: PhantomData<W>,
}

pub enum Msg {
    Resize((f64, f64, f64, f64)),
}

impl<W: WidgetBuilder + IntoVTag + PartialEq + Clone + 'static> Component for PwtSizeObserver<W> {
    type Message = Msg;
    type Properties = SizeObserver<W>;

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
            Msg::Resize(args) => {
                match &props.on_resize {
                    SizeCallback::Normal(cb) => cb.emit((args.0, args.1)),
                    SizeCallback::ClientRect(cb) => cb.emit(args),
                }
                false
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let node_ref = self.node_ref.clone();
            let el = node_ref.cast::<web_sys::Element>().unwrap();
            let link = ctx.link().clone();
            let size_observer =
                DomSizeObserver::new(&el, move |(width, height, client_width, client_height)| {
                    link.send_message(Msg::Resize((width, height, client_width, client_height)));
                });
            self.observer = Some(size_observer);
        }
    }
}

impl<W: WidgetBuilder + IntoVTag + PartialEq + Clone + 'static> From<SizeObserver<W>> for VNode {
    fn from(val: SizeObserver<W>) -> Self {
        let key = val.content.as_std_props().key.clone();
        let comp = VComp::new::<PwtSizeObserver<W>>(Rc::new(val), key);
        VNode::from(comp)
    }
}
