use crate::prelude::*;
use crate::widget::canvas::{Circle, Group};
use crate::widget::charts::map::{Coordinates, MapPoint};
use crate::widget::{container::span, Column};

/// Contains the points rendered together on the map due to clustering,
/// and suggested properties such as the radius (calculated from clustering)
/// as well if the cluster is selected and where it's center is.
#[derive(Clone)]
pub struct PointsRenderArgs<'a, T: MapPointData> {
    pub points: &'a [&'a MapPoint<T>],
    pub center: Coordinates,
    pub selected: bool,
    pub suggested_radius: f64,
}

/// The default renderer for a map cluster.
pub fn render_point_default<T: MapPointData>(args: &PointsRenderArgs<T>) -> Group {
    Group::new()
        .class("pwt-map-location")
        .with_child(
            Circle::new()
                .style(
                    "--pwt-location-radius",
                    format!("{:.3}px", args.suggested_radius),
                )
                .cx(args.center.x)
                .cy(args.center.y),
        )
        .with_optional_child(
            args.selected.then_some(
                Circle::new()
                    .class("pwt-map-location-animated")
                    .style(
                        "--pwt-location-radius",
                        format!("{:.3}px", args.suggested_radius),
                    )
                    .cx(args.center.x)
                    .cy(args.center.y),
            ),
        )
}

/// The default info renderer for a map cluster.
pub fn render_info_default<T: MapPointData>(args: &PointsRenderArgs<T>) -> Html {
    Column::new()
        .gap(1)
        .children(
            args.points
                .iter()
                .map(|point| span(point.data.render_title()).into()),
        )
        .into()
}

/// The default tooltip renderer for a map cluster.
pub fn render_tooltip_default<T: MapPointData>(args: &PointsRenderArgs<T>) -> Html {
    let count = args.points.len();
    if count > 3 {
        Column::new()
            .gap(1)
            .with_child(tr!(
                "{0} and {1} more",
                args.points[0].data.render_title(),
                count - 1
            ))
            .into()
    } else {
        Column::new()
            .gap(1)
            .children(
                args.points
                    .iter()
                    .map(|point| span(point.data.render_title()).into()),
            )
            .into()
    }
}

/// Data attached to a [MapPoint], providing the title and the render hooks used to draw a
/// point (or cluster of points), its info box, and its tooltip on a map.
pub trait MapPointData: PartialEq + Clone {
    /// Get the title of the map point
    fn render_title(&self) -> AttrValue;

    /// Render the map icon for a cluster of points.
    ///
    /// Uses [render_point_default] by default.
    fn render_point(args: &PointsRenderArgs<Self>) -> Group {
        render_point_default(args)
    }

    /// Render the info box for a cluster of points.
    ///
    /// Uses [render_info_default] by default.
    fn render_info(args: &PointsRenderArgs<Self>) -> Html {
        render_info_default(args)
    }

    /// Render the tooltip for a cluster of points.
    ///
    /// Uses [render_tooltip_default] by default.
    fn render_tooltip(args: &PointsRenderArgs<Self>) -> Html {
        render_tooltip_default(args)
    }
}

impl MapPointData for AttrValue {
    fn render_title(&self) -> AttrValue {
        self.clone()
    }
}

impl MapPointData for String {
    fn render_title(&self) -> AttrValue {
        self.clone().into()
    }
}
