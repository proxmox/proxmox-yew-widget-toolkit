use std::marker::PhantomData;

mod map_point;
pub use map_point::{
    MapPointData, PointsRenderArgs, render_info_default, render_point_default,
    render_tooltip_default,
};

mod zoom_info;
use zoom_info::ZoomInfo;

use crate::dom::align::{AlignOptions, align_to, align_to_xy};
use crate::prelude::*;
use crate::touch::{GestureDetector, GestureDragEvent, GesturePhase, GesturePinchZoomEvent};
use crate::widget::canvas::{Canvas, Circle, Group};
use crate::widget::charts::map::zoom_info::ZoomAction;
use crate::widget::{Button, Card, Container, Row, SizeObserver, Tooltip};
use crate::{client_to_svg_coords, css};
use pwt_macros::{builder, widget};

/// x and y coordinates to represent a position
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
}

/// Represents a point on the map.
#[derive(Clone, PartialEq)]
pub struct MapPoint<T> {
    /// The coordinates of the point on a [Map].
    pub coordinates: Coordinates,
    pub data: T,
}

impl<T> MapPoint<T> {
    /// Create a new [MapPoint] with the given data and coordinates.
    pub fn new(coordinates: Coordinates, data: T) -> Self {
        Self { coordinates, data }
    }
}

/// An interactive Map that handles interaction (zooming, panning) and draws
/// [MapPoint]s on top of the given SVG element.
///
/// Can handle touch and mouse input.
#[widget(pwt=crate,comp=MapComp<T>, @element)]
#[builder]
#[derive(Properties, PartialEq, Clone)]
pub struct Map<T: MapPointData + 'static = AttrValue> {
    /// The map as an SVG element.
    map: Html,

    #[prop_or(1000.0)]
    #[builder]
    /// The width of the shown map. Used for coordinates and scaling.
    width: f64,

    #[prop_or(1000.0)]
    #[builder]
    /// The height of the shown map. Used for coordinates and scaling.
    height: f64,

    #[prop_or_default]
    #[builder]
    /// A list of points to highlight on the map.
    points: Vec<MapPoint<T>>,

    #[prop_or(30.0)]
    #[builder]
    /// The maximum zoom level that is allowed.
    max_zoom_level: f64,

    #[prop_or(8.0)]
    #[builder]
    /// The radius for info points.
    info_point_radius: f64,
}

impl<T: MapPointData> Map<T> {
    /// Creates a new interactive map with the given background SVG map element
    pub fn new(map: impl Into<Html>) -> Self {
        yew::props!(Self { map: map.into() })
    }
}

pub enum Msg {
    WheelZoom(ZoomAction, i32, i32),
    ButtonZoom(ZoomAction),
    PinchZoom(GesturePinchZoomEvent),
    Resize(f64, f64),
    Tooltip(Option<(usize, i32, i32)>),
    ToggleInfo(usize),
    CloseInfo,
    Drag(GestureDragEvent),
}

#[derive(PartialEq, Clone)]
// cache to hold the clustering of the points, can change when the zoom or the fit scale changes
struct Cluster {
    center: Coordinates,
    indices: Vec<usize>,
}

pub struct MapComp<T: Clone + PartialEq> {
    zoom: ZoomInfo,
    pinch_start_scale: f64,
    pinch_last_center: Coordinates,
    fit_scale: f64,
    svg_ref: NodeRef,
    tooltip: Option<(usize, i32, i32)>,
    tooltip_ref: NodeRef,
    info_anchor_ref: NodeRef,
    info_ref: NodeRef,
    info_visible: Option<usize>,
    grab_start: Option<(f64, f64)>,
    // set while a drag is in progress so the trailing synthetic click does not dismiss the info card
    dragged: bool,
    clusters: Vec<Cluster>,
    _phantom_data: PhantomData<T>,
}

impl<T: MapPointData + 'static> MapComp<T> {
    fn create_tooltip(&self, args: &PointsRenderArgs<T>) -> Html {
        Container::new()
            .attribute("role", "tooltip")
            .attribute("aria-live", "polite")
            .attribute("data-show", Some(""))
            .class("pwt-tooltip")
            .class("pwt-tooltip-rich")
            .with_child(T::render_tooltip(args))
            .into_html_with_ref(self.tooltip_ref.clone())
    }

    fn create_info(&self, args: &PointsRenderArgs<T>) -> Html {
        Card::new()
            .class("pwt-map-info")
            .with_child(T::render_info(args))
            .into_html_with_ref(self.info_ref.clone())
    }

    fn cluster_points(&mut self, ctx: &Context<Self>) {
        // simple algorithm to find overlapping points and cluster them together

        let points = &ctx.props().points;
        let mut indices: Vec<usize> = (0..ctx.props().points.len()).collect();

        let effective_radius = ctx.props().info_point_radius / self.fit_scale;
        let mut clusters = Vec::new();
        while let Some(index) = indices.pop() {
            let base = &points[index];
            let mut overlapping = Vec::new();
            let mut non_overlapping = Vec::new();
            let base_coordinates = self.zoom.map_point(base.coordinates);

            // accumulate the cluster center while partitioning, starting from the base point
            let mut x_center = base.coordinates.x;
            let mut y_center = base.coordinates.y;
            for compare_index in indices.into_iter() {
                let p = &points[compare_index];
                let point_coordinates = self.zoom.map_point(p.coordinates);
                let dx = base_coordinates.x - point_coordinates.x;
                let dy = base_coordinates.y - point_coordinates.y;
                if dx * dx + dy * dy < (2.0 * effective_radius).powi(2) {
                    x_center += p.coordinates.x;
                    y_center += p.coordinates.y;
                    overlapping.push(compare_index);
                } else {
                    non_overlapping.push(compare_index);
                }
            }
            indices = non_overlapping;
            overlapping.insert(0, index);

            let len = overlapping.len() as f64;
            clusters.push(Cluster {
                center: Coordinates {
                    x: x_center / len,
                    y: y_center / len,
                },
                indices: overlapping,
            });
        }

        if let Some(index) = self.info_visible {
            // keep the open card on the same set of points across re-clustering (the center
            // shifts with zoom, so match on membership), or reset it if that cluster is gone
            let open_indices = &self.clusters[index].indices;
            self.info_visible = clusters
                .iter()
                .position(|cluster| &cluster.indices == open_indices);
        }
        self.clusters = clusters;
    }
}

impl<T: MapPointData + 'static> yew::Component for MapComp<T> {
    type Message = Msg;
    type Properties = Map<T>;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        let mut zoom = ZoomInfo::new(props.width, props.height, props.max_zoom_level);
        zoom.zoom_to_points(
            ctx.props().points.iter().map(|poi| poi.coordinates),
            2.0 * props.info_point_radius,
        );

        let mut this = Self {
            zoom,
            pinch_start_scale: 1.0,
            pinch_last_center: Coordinates { x: 0.0, y: 0.0 },
            svg_ref: NodeRef::default(),
            fit_scale: 1.0,
            tooltip: None,
            tooltip_ref: NodeRef::default(),
            info_anchor_ref: NodeRef::default(),
            info_ref: NodeRef::default(),
            info_visible: None,
            grab_start: None,
            dragged: false,
            clusters: Vec::new(),
            _phantom_data: PhantomData::<T>,
        };

        this.cluster_points(ctx);

        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        let width = props.width;
        let height = props.height;
        match msg {
            Msg::WheelZoom(change, x, y) => {
                let Some(svg) = self.svg_ref.get() else {
                    return false;
                };
                let coords = client_to_svg_coords(&svg, x as f64, y as f64);
                if !self.zoom.update_zoom(change, coords[0], coords[1]) {
                    return false;
                }
                self.cluster_points(ctx);
            }
            Msg::ButtonZoom(change) => {
                if !self.zoom.update_zoom(change, width / 2.0, height / 2.0) {
                    return false;
                }
                self.cluster_points(ctx);
            }
            Msg::PinchZoom(event) => {
                let Some(svg) = self.svg_ref.get() else {
                    return false;
                };
                let coords = client_to_svg_coords(
                    &svg,
                    (event.point0.x + event.point1.x) as f64 / 2.0,
                    (event.point0.y + event.point1.y) as f64 / 2.0,
                );
                let x = coords[0];
                let y = coords[1];
                match event.phase {
                    GesturePhase::Start => {
                        self.pinch_start_scale = self.zoom.get_zoom_level();
                        self.pinch_last_center = Coordinates { x, y };
                    }
                    GesturePhase::Update => {
                        self.zoom
                            .move_pan(x - self.pinch_last_center.x, y - self.pinch_last_center.y);
                        self.pinch_last_center = Coordinates { x, y };
                        if self.zoom.update_zoom(
                            zoom_info::ZoomAction::Scale(self.pinch_start_scale * event.scale),
                            x,
                            y,
                        ) {
                            // re-cluster on scale change, like wheel/button zoom
                            self.cluster_points(ctx);
                        }
                    }
                    GesturePhase::End => return false,
                }
            }
            Msg::Drag(event) => match event.phase {
                GesturePhase::Start => {
                    // GestureDetector only emits drag events past its tap tolerance, so any drag
                    // start means a real pan; remember it to suppress the trailing click below
                    self.dragged = true;
                    self.grab_start = Some((event.x() as f64, event.y() as f64));
                }
                GesturePhase::Update => {
                    if let Some((start_x, start_y)) = self.grab_start {
                        let x = event.x() as f64;
                        let y = event.y() as f64;
                        self.zoom.move_pan(
                            (x - start_x) / self.fit_scale,
                            (y - start_y) / self.fit_scale,
                        );
                        self.grab_start = Some((x, y));
                    }
                }
                GesturePhase::End => {
                    // re-render (fall through to `true`) so the cursor resets from "grabbing"
                    self.grab_start = None;
                }
            },
            Msg::Resize(real_width, real_height) => {
                if real_width > 0.0 && real_height > 0.0 {
                    // use the smaller scale so the whole map fits
                    self.fit_scale = (real_width / width).min(real_height / height);
                    self.cluster_points(ctx);
                }
            }
            Msg::Tooltip(index) => {
                if index.is_none() && self.tooltip.is_none() {
                    return false;
                }
                self.tooltip = index;
            }
            Msg::ToggleInfo(index) => {
                if self.info_visible == Some(index) {
                    self.info_visible = None;
                } else {
                    self.info_visible = Some(index);
                    if let Some(cluster) = self.clusters.get(index) {
                        self.zoom.center_point(cluster.center);
                        self.tooltip = None;
                    }
                }
            }
            Msg::CloseInfo => {
                // a pan emits a trailing click; consume it here instead of dismissing the card
                if self.dragged {
                    self.dragged = false;
                    return false;
                }
                if self.info_visible.is_none() {
                    return false;
                }
                self.info_visible = None;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let link = ctx.link();
        let width = props.width;
        let height = props.height;

        let effective_radius = props.info_point_radius / self.fit_scale;

        let zoom_level = self.zoom.get_zoom_level();
        let is_zoomed = zoom_level != 1.0;
        let fully_zoomed = zoom_level >= props.max_zoom_level;

        let svg = Canvas::new()
            .onclick({
                // clicking the map background (anything but a point, which stops propagation)
                // dismisses an open info card
                let link = link.clone();
                move |_| link.send_message(Msg::CloseInfo)
            })
            .onwheel({
                let link = link.clone();
                move |event: WheelEvent| {
                    // don't scroll the remaining page when scrolling in map
                    event.prevent_default();
                    // ignore delta mode as we zoom in/out in 10% steps later anyway, only the
                    // direction is relevant here
                    let (delta, x, y) = (event.delta_y(), event.client_x(), event.client_y());
                    let action = if delta < 0.0 {
                        ZoomAction::In
                    } else {
                        ZoomAction::Out
                    };
                    link.send_message(Msg::WheelZoom(action, x, y));
                }
            })
            .style(
                "cursor",
                match (is_zoomed, self.grab_start.is_some()) {
                    (true, true) => Some("grabbing"),
                    (true, false) => Some("grab"),
                    (false, _) => None,
                },
            )
            .class("pwt-map")
            .attribute("viewBox", format!("0 0 {width} {height}"))
            .with_child(
                Group::new()
                    .with_child(props.map.clone())
                    .style("transform", self.zoom.get_transform()),
            );

        let mut points = Group::new();
        let mut tooltip = None;
        let mut info = None;

        for (index, cluster) in self.clusters.iter().enumerate() {
            let len = cluster.indices.len() as f64;
            let radius_factor = if len > 1.0 {
                // grow logarithmically but not too big
                1.0 + len.log2() * 0.3
            } else {
                1.0
            };

            let points_of_interest: Vec<_> = cluster
                .indices
                .iter()
                .map(|index| &ctx.props().points[*index])
                .collect();

            let center = self.zoom.map_point(cluster.center);

            // don't render points, tooltips and info boxes that are out of bounds
            if self.zoom.is_out_of_bounds(center) {
                continue;
            }

            let args = PointsRenderArgs {
                points: &points_of_interest,
                center,
                selected: self.info_visible == Some(index),
                suggested_radius: effective_radius * radius_factor,
            };

            let point = T::render_point(&args)
                .onpointermove(link.callback(move |event: PointerEvent| {
                    let x = event.client_x();
                    let y = event.client_y();
                    Msg::Tooltip(Some((index, x, y)))
                }))
                .onpointerleave(link.callback(move |_| Msg::Tooltip(None)))
                .onclick(link.callback(move |event: MouseEvent| {
                    // keep the click from reaching the background handler that closes the card
                    event.stop_propagation();
                    Msg::ToggleInfo(index)
                }));
            if self.info_visible == Some(index) {
                points.add_child(point);
                // ref for info-box
                points.add_child(
                    Circle::new()
                        .cx(center.x)
                        .cy(center.y)
                        .r(0)
                        .into_html_with_ref(self.info_anchor_ref.clone()),
                );
                info = Some(self.create_info(&args));
            } else {
                points.add_child(point);
            }

            match &self.tooltip {
                Some((tooltip_idx, _, _)) if *tooltip_idx == index => {
                    tooltip = Some(self.create_tooltip(&args));
                }
                _ => {}
            }
        }

        Container::new()
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .class(css::Display::Block)
            .with_child(SizeObserver::new(
                Container::new()
                    .class(css::Display::Flex)
                    .class(css::JustifyContent::Center)
                    .width("100%")
                    .height("100%")
                    .with_child(
                        GestureDetector::new(
                            svg.with_child(points)
                                .into_html_with_ref(self.svg_ref.clone()),
                        )
                        .on_drag(link.callback(Msg::Drag))
                        .on_pinch_zoom(link.callback(Msg::PinchZoom)),
                    ),
                {
                    let link = link.clone();
                    move |(_, _, width, height)| {
                        link.send_message(Msg::Resize(width, height));
                    }
                },
            ))
            .with_optional_child(tooltip)
            .with_optional_child(info)
            .with_child(
                Row::new()
                    .gap(1)
                    .class("pwt-map-interaction-panel")
                    .with_child(
                        Tooltip::new(
                            Button::new_icon("fa fa-arrows-alt")
                                .class(css::ColorScheme::Primary)
                                .disabled(!is_zoomed)
                                .on_activate(link.callback(|_| Msg::ButtonZoom(ZoomAction::Reset))),
                        )
                        .tip(tr!("Show whole map")),
                    )
                    .with_child(
                        Tooltip::new(
                            Button::new_icon("fa fa-minus")
                                .class(css::ColorScheme::Primary)
                                .disabled(!is_zoomed)
                                .on_activate(link.callback(|_| Msg::ButtonZoom(ZoomAction::Out))),
                        )
                        .tip(tr!("Zoom out")),
                    )
                    .with_child(
                        Tooltip::new(
                            Button::new_icon("fa fa-plus")
                                .class(css::ColorScheme::Primary)
                                .disabled(fully_zoomed)
                                .on_activate(link.callback(|_| Msg::ButtonZoom(ZoomAction::In))),
                        )
                        .tip(tr!("Zoom in")),
                    ),
            )
            .into()
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        let props = ctx.props();
        let mut need_clustering = false;
        if props.width != old_props.width
            || props.height != old_props.height
            || props.max_zoom_level != old_props.max_zoom_level
        {
            self.zoom = ZoomInfo::new(props.width, props.height, props.max_zoom_level);
            need_clustering = true;
        }
        if props.points != old_props.points {
            need_clustering = true;
            if old_props.points.is_empty() && !props.points.is_empty() {
                self.zoom.zoom_to_points(
                    ctx.props().points.iter().map(|poi| poi.coordinates),
                    2.0 * props.info_point_radius,
                );
            }
        }

        if need_clustering {
            self.cluster_points(ctx);
        }

        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if let &Some((_, x, y)) = &self.tooltip
            && let Some(el) = self.tooltip_ref.get()
        {
            let _ = align_to_xy(
                el,
                (x as f64 + 10.0, y as f64 + 10.0),
                crate::dom::align::Point::TopStart,
            );
        }
        if let (Some(_), Some(anchor), Some(el)) = (
            self.info_visible,
            self.info_anchor_ref.get(),
            self.info_ref.get(),
        ) {
            let _ = align_to(
                anchor,
                el,
                Some(
                    AlignOptions::new(
                        crate::dom::align::Point::Top,
                        crate::dom::align::Point::Bottom,
                        crate::dom::align::GrowDirection::None,
                    )
                    .offset(0.0, ctx.props().info_point_radius * 2.0),
                ),
            );
        }
    }
}
