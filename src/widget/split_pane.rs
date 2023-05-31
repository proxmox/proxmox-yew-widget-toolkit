use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew::html::IntoPropValue;

use gloo_events::EventListener;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

use crate::css::{FlexFillFirstChild, Display};
use crate::props::{ContainerBuilder, EventSubscriber, WidgetBuilder};

use super::Container;
use super::dom::element_direction_rtl;

use pwt_macros::widget;

#[derive(Copy, Clone, PartialEq)]
enum PaneSize {
    // Fraction of the parent size (without space used by separators).
    Fraction(f64),
    // Fixed size in pixel
    Pixel(usize),
    // Flex (Note: cannot be used for min/max size)
    Flex(usize),
}

impl PaneSize {

    fn to_css_size(&self, reserve: f64) -> String {
        match self {
            Self::Fraction(ratio) => {
                // Note: compute percentage without space used by separators.
                format!("calc((100% - {reserve}px)*{ratio})")
            }
            Self::Pixel(p) => format!("{p}px"),
            Self::Flex(_f) => unreachable!(), /* we do not support this */
        }
    }

    fn to_css_flex(&self, reserve: f64) -> String {
        match self {
            Self::Fraction(ratio) => {
                // Note: compute percentage without space used by separators.
                format!("flex: 0 0 calc((100% - {reserve}px)*{ratio});")
            }
            Self::Pixel(p) => format!("flex: 0 0 {p}px;"),
            Self::Flex(f) =>  format!("flex: {f} 1 0px;"),
        }
    }

    fn real_size(&self, sizes: &[f64]) -> f64 {
        let width: f64 = sizes.iter().sum();
        match self {
            Self::Fraction(ratio) => width * ratio,
            Self::Pixel(p) => *p as f64,
            Self::Flex(_f) => unreachable!(), /* we do not support this */
        }
    }
}

/// Options for [SplitPane] children.
///
/// This struct is a wrapper for the pane content. It allow to set
/// additional properties:
///
/// - `size`: the initial pane size (Pixels, Fraction or Flex)
///
/// - `min_size`: the minimal pane size (Pixels or Fraction)
///
/// - `max_size`: the maximal pane size (Pixels or Fraction)
#[derive(Clone, PartialEq)]
pub struct Pane {
    // Initial pane size (defaults to Flex(1))
    size: Option<PaneSize>,
    // Minimal pane size
    min_size: Option<PaneSize>,
    // Maximal pane size
    max_size: Option<PaneSize>,

    // Yew node ref.
    node_ref: NodeRef,
    // Pane content
    content: VNode,
}

impl Pane {

    /// Creates a new instance.
    pub fn new(content: impl Into<VNode>) -> Self {
        Self {
            size: Some(PaneSize::Flex(1)),
            min_size: None,
            max_size: None,
            node_ref: NodeRef::default(),
            content: content.into(),
        }
    }

    /// Builder style method to set the initial pane size in pixels.
    pub fn size(mut self, size: impl IntoPropValue<Option<usize>>) -> Self {
        self.set_size(size);
        self
    }

    /// Method to set the initial pane size in pixels.
    pub fn set_size(&mut self, size: impl IntoPropValue<Option<usize>>) {
        self.size = size.into_prop_value().map(|p| PaneSize::Pixel(p));
    }

    /// Builder style method to set the initial pane size as flex.
    pub fn flex(mut self, flex: impl IntoPropValue<Option<usize>>) -> Self {
        self.set_flex(flex);
        self
    }

    /// Method to set the initial pane size as flex.
    pub fn set_flex(&mut self, flex: impl IntoPropValue<Option<usize>>) {
        self.size = flex.into_prop_value().map(|f| PaneSize::Flex(f));
    }

    /// Builder style method to set the initial pane size as fraction.
    pub fn fraction(mut self, fraction: impl IntoPropValue<Option<f64>>) -> Self {
        self.set_fraction(fraction);
        self
    }

    /// Method to set the initial pane size as fraction.
    pub fn set_fraction(&mut self, fraction: impl IntoPropValue<Option<f64>>) {
       self.size = fraction.into_prop_value().map(|f| PaneSize::Fraction(f));
    }

    /// Builder style method to set the minimal pane size in pixels.
    pub fn min_size(mut self, size: impl IntoPropValue<Option<usize>>) -> Self {
        self.set_min_size(size);
        self
    }

    /// Method to set the minimal pane size in pixels.
    pub fn set_min_size(&mut self, size: impl IntoPropValue<Option<usize>>) {
        self.min_size = size.into_prop_value().map(|p| PaneSize::Pixel(p));
    }

    /// Builder style method to set the minimal pane size as fraction.
    pub fn min_fraction(mut self, fraction: impl IntoPropValue<Option<f64>>) -> Self {
        self.set_min_fraction(fraction);
        self
    }

    /// Method to set the minimal pane size as fraction.
    pub fn set_min_fraction(&mut self, fraction: impl IntoPropValue<Option<f64>>) {
       self.min_size = fraction.into_prop_value().map(|f| PaneSize::Fraction(f));
    }

    /// Builder style method to set the maximal pane size in pixels.
    pub fn max_size(mut self, size: impl IntoPropValue<Option<usize>>) -> Self {
        self.set_max_size(size);
        self
    }

    /// Method to set the maximal pane size in pixels.
    pub fn set_max_size(&mut self, size: impl IntoPropValue<Option<usize>>) {
        self.max_size =  size.into_prop_value().map(|p| PaneSize::Pixel(p));
    }

    /// Builder style method to set the maximal pane size as fraction.
    pub fn max_fraction(mut self, fraction: impl IntoPropValue<Option<f64>>) -> Self {
        self.set_max_fraction(fraction);
        self
    }

    /// Method to set the maximal pane size as fraction.
    pub fn set_max_fraction(&mut self, fraction: impl IntoPropValue<Option<f64>>) {
       self.max_size = fraction.into_prop_value().map(|f| PaneSize::Fraction(f));
    }
}

impl<T: Into<VNode>> From<T> for Pane {
    fn from(content: T) -> Self {
        Self::new(content)
    }
}

/// Container where children are separated by a draggable sparator.
#[widget(pwt=crate, comp=PwtSplitPane, @element)]
#[derive(Clone, PartialEq, Properties)]
pub struct SplitPane {
    /// Container children.
    #[prop_or_default]
    pub children: Vec<Pane>,

    /// Vertical flag.
    #[prop_or_default]
    pub vertical: bool,

    /// Resize handle size (defaults to 7 pixels).
    #[prop_or(7)]
    pub handle_size: usize,
}


impl SplitPane {
    /// Creates a new instance
    pub fn new() -> Self {
        yew::props!(Self {})
    }

    /// Builder style method to set the vertical flag.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.set_vertical(vertical);
        self
    }

    /// Method to set the vertical flag.
    pub fn set_vertical(&mut self, vertical: bool) {
        self.vertical = vertical;
    }

    /// Builder style method to add a [Pane].
    pub fn with_child(mut self, child: impl Into<Pane>) -> Self {
        self.add_child(child);
        self
    }

    /// Method to add a [Pane].
    pub fn add_child(&mut self, child: impl Into<Pane>) {
        self.children.push(child.into());
    }

    /// Builder style method to set the size (pixels) of the resize handle.
    pub fn handle_size(mut self, size: usize) -> Self {
        self.set_handle_size(size);
        self
    }

    /// Method to set the size (pixels) of the resize handle.
    pub fn set_handle_size(&mut self, size: usize) {
        self.handle_size = size;
    }
}

#[doc(hidden)]
pub struct PwtSplitPane {
    rtl: Option<bool>,
    sizes: Vec<f64>, // observer pane sizes
    drag_offset: i32,
    pointermove_listener: Option<EventListener>,
    pointerup_listener: Option<EventListener>,
    pointer_id: Option<i32>,
}

pub enum Msg {
    FocusIn,
    Shrink(usize, bool),
    Grow(usize, bool),
    ResetSize,
    StartResize(usize, i32, i32, i32),
    StopResize(i32),
    PointerMove(usize, i32, i32, i32)
}

impl PwtSplitPane {

    fn create_splitter(&self, ctx: &Context<Self>, index: usize, fraction: Option<f64>) -> Html {
        let props = ctx.props();
        let vertical = props.vertical;
        let rtl = self.rtl.unwrap_or(false);

        let onkeydown = Callback::from({
            let link = ctx.link().clone();
            move |event: KeyboardEvent| {
                let key: &str = &event.key();
                match key {
                    "Enter" => {
                        event.stop_propagation();
                        link.send_message(Msg::ResetSize);
                    }
                    "ArrowUp" if vertical => {
                        event.stop_propagation();
                        link.send_message(Msg::Shrink(index, event.shift_key()));
                    }
                    "ArrowDown" if vertical  => {
                        event.stop_propagation();
                        link.send_message(Msg::Grow(index, event.shift_key()));
                    }
                    "ArrowLeft" if !vertical && !rtl  => {
                        event.stop_propagation();
                        link.send_message(Msg::Shrink(index, event.shift_key()));
                    }
                    "ArrowRight" if !vertical && !rtl => {
                        event.stop_propagation();
                        link.send_message(Msg::Grow(index, event.shift_key()));
                    }
                    "ArrowLeft" if !vertical && rtl  => {
                        event.stop_propagation();
                        link.send_message(Msg::Grow(index, event.shift_key()));
                    }
                    "ArrowRight" if !vertical && rtl => {
                        event.stop_propagation();
                        link.send_message(Msg::Shrink(index, event.shift_key()));
                    }
                    _ => {}
                }
            }
        });

        let splitter = Container::new()
            .attribute("tabindex", "0")
            .attribute("role", "separator")
            .attribute("aria-orientation", if props.vertical { "vertical" } else { "horizontal" })
            .attribute("aria-valuenow", fraction.map(|f| format!("{:.0}", f*100.0)))
            .attribute("style", format!("flex: 0 0 {}px;", props.handle_size))
            .class(if props.vertical { "column-split-handle" } else { "row-split-handle" })
            .onkeydown(onkeydown)
            .ondblclick(ctx.link().callback(|_| Msg::ResetSize))
            .onpointerdown(ctx.link().callback(move |event: PointerEvent| {
                Msg::StartResize(index, event.offset_x(), event.offset_y(), event.pointer_id())
            }));

        splitter.into()
    }

    fn create_pane(&self, ctx: &Context<Self>, index: usize, child: &Pane) -> Html {
        let props = ctx.props();

        let handle_size_sum = (props.handle_size as f64) * props.children.len().saturating_sub(1) as f64;

        let size_attr = if props.vertical { "height" } else { "width" };

        let mut style = match child.size {
            Some(size) => size.to_css_flex(handle_size_sum),
            None => String::from("flex: 0 0 auto;")
        };

        style.push_str("overflow:auto;");

        if let Some(size) = self.sizes.get(index) {
            // use flex-grow to set size. If we resize the container,
            // children resize proportional

            let dynamic = match child.size {
                Some(PaneSize::Fraction(_)) | Some(PaneSize::Flex(_)) => true,
                _ => false,
            };

            style = if dynamic {
                format!("overflow:auto;flex: {size} 1 0px;")
            } else {
                format!("overflow:auto;{size_attr}:{size}px;")
            };
        }

        if let Some(min_size) = &child.min_size {
            let min_size = min_size.to_css_size(handle_size_sum);
            style.push_str(&format!("min-{size_attr}: {min_size};"));
        }

        if let Some(max_size) = &child.max_size {
            let max_size = max_size.to_css_size(handle_size_sum);
            style.push_str(&format!("max-{size_attr}: {max_size};"));
        }

        let pane = Container::new()
            .node_ref(child.node_ref.clone())
            .attribute("style", style)
            .class(Display::Flex)
            .class(FlexFillFirstChild)
            .with_child(child.content.clone());

        pane.into()
    }

    fn query_sizes(&self, props: &SplitPane) -> Option<Vec<f64>> {

        let mut sizes = Vec::new();

        for child in props.children.iter() {
            if let Some(el) = child.node_ref.cast::<web_sys::Element>() {
                let rect = el.get_bounding_client_rect();
                if props.vertical {
                    sizes.push(rect.height());
                } else {
                    sizes.push(rect.width());
                }
            } else {
                return None;
            }
        }

        Some(sizes)
    }

    fn try_new_size(&self, pane: &Pane, current_size: f64, new_size: f64) -> f64 {
        let min = pane.min_size.map(|s| s.real_size(&self.sizes)).unwrap_or(0.0);
        let max = pane.max_size.map(|s| s.real_size(&self.sizes)).unwrap_or(f64::MAX);

        let size = new_size.min(max).max(min);
        let diff = size - current_size;

        diff
    }

    fn resize_pane(&mut self, props: &SplitPane, child_index: usize, new_size1: f64) -> bool {
        if self.sizes.len() <= child_index { // fixme
            // should never happen - just to be sure
            return false;
        }

        let pane1 = &props.children[child_index];
        let pane2 = &props.children[child_index + 1];
        let size1 = self.sizes[child_index];
        let size2 = self.sizes[child_index + 1];

        let diff1 = self.try_new_size(pane1, size1, new_size1);
        let new_size2 = size2 - diff1;
        let diff2 = self.try_new_size(pane2, size2, new_size2);

        let diff = if diff1 > 0.0 && diff2 < 0.0 {
            diff1.min(-diff2)
        } else if diff1 < 0.0 && diff2 > 0.0 {
            diff1.max(-diff2)
        } else {
            return false;
        };

        self.sizes[child_index] += diff;
        self.sizes[child_index + 1] -= diff;

        true
    }
}


impl Component for PwtSplitPane {
    type Message = Msg;
    type Properties = SplitPane;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            rtl: None,
            sizes: Vec::new(),
            drag_offset: 0,
            pointermove_listener: None,
            pointerup_listener: None,
            pointer_id: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();

        if let Some(sizes) = self.query_sizes(props) {
            self.sizes = sizes;
        }

        if self.rtl.is_none() {
            self.rtl = element_direction_rtl(&props.std_props.node_ref);
        }

        match msg {
            Msg::FocusIn => {
                self.rtl = element_direction_rtl(&props.std_props.node_ref);
                true
            }
            Msg::PointerMove(child_index, x, y, pointer_id) => {
                if self.pointer_id == Some(pointer_id) {
                    if self.sizes.len() <= child_index { return false; }

                    let pane = &props.children[child_index];

                    if let Some(el) = pane.node_ref.cast::<web_sys::Element>() {
                        let rect = el.get_bounding_client_rect();

                        let new_size = if props.vertical {
                            (y as f64) - rect.y() - (self.drag_offset as f64)
                        } else {
                            let rtl = self.rtl.unwrap_or(false);

                            if rtl {
                                rect.right() - (x as f64) - (self.drag_offset as f64)
                            } else {
                                (x as f64) - rect.x() - (self.drag_offset as f64)
                            }
                        };

                        return self.resize_pane(props, child_index, new_size);
                    }
                }
                true
            }
            Msg::StopResize(pointer_id) => {
                if self.pointer_id == Some(pointer_id) {
                    self.pointerup_listener = None;
                    self.pointermove_listener = None;
                    self.drag_offset = 0;
                    self.pointer_id = None;
                }
                false
            }
            Msg::StartResize(child_index, x, y, pointer_id) => {
                self.drag_offset = if props.vertical { y } else { x };

                self.rtl = element_direction_rtl(&props.std_props.node_ref);

                let window = web_sys::window().unwrap();
                let link = ctx.link();
                let onpointermove = link.callback(move |e: Event| {
                    let event = e.dyn_ref::<web_sys::PointerEvent>().unwrap_throw();
                    Msg::PointerMove(child_index, event.client_x(), event.client_y(), event.pointer_id())
                });
                let pointermove_listener = EventListener::new(
                    &window,
                    "pointermove",
                    move |e| onpointermove.emit(e.clone()),
                );
                self.pointermove_listener = Some(pointermove_listener);

                let onpointerup = link.callback(|e: Event| {
                    let event = e.dyn_ref::<web_sys::PointerEvent>().unwrap_throw();
                    Msg::StopResize(event.pointer_id())
                });
                let pointerup_listener = EventListener::new(
                    &window,
                    "pointerup",
                    move |e| onpointerup.emit(e.clone()),
                );
                self.pointerup_listener = Some(pointerup_listener);
                self.pointer_id = Some(pointer_id);

                false
            }
            Msg::ResetSize => {
                self.sizes = Vec::new();
                true
            }

            Msg::Shrink(child_index, fast) => {
                let amount = if fast { 10.0 } else { 1.0 };
                self.resize_pane(props, child_index, self.sizes[child_index] - amount)
            }
            Msg::Grow(child_index, fast) => {
                let amount = if fast { 10.0 } else { 1.0 };
                self.resize_pane(props, child_index, self.sizes[child_index] + amount)
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut children = Vec::new();

        let width: f64 = self.sizes.iter().sum();
        let mut position = 0f64;
        for (i, child) in props.children.iter().enumerate() {
            if i > 0 {
                let fraction = (width > 0f64).then(|| position/width);
                children.push(self.create_splitter(ctx, i - 1, fraction));
            }
            position += self.sizes.get(i).map(|s| *s).unwrap_or(0.0);
            children.push(self.create_pane(ctx, i, &child))
        }

        let mut container = yew::props!(Container {
            std_props: props.std_props.clone(),
            listeners: props.listeners.clone(),
            children,
        });

        // use existing style attribute
        let attr_map = container.std_props.attributes.get_mut_index_map();
        let mut style = attr_map.remove(&AttrValue::Static("style"))
            .map(|(style, _)| {
                let mut style = style.to_string();
                if !style.ends_with(';') {
                    style.push(';');
                }
                style
            })
            .unwrap_or(String::new());

        // and append our style at the end
        style.push_str(if props.vertical {
            "display:flex;flex-direction:column;align-items:stretch;"
        } else {
            "display:flex;flex-direction:row;align-items:stretch;"
        });

        container
            .attribute("style", style)
            .onfocusin(ctx.link().callback(|_| Msg::FocusIn))
            .into()
    }
}
