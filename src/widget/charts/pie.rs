//! Pie Chart
//!
//! Provides an element that can be used for a 'gauge' by supplying a single value,
//! a 'pie' chart by providing multiple values with an 'thickness_ratio' of 1.0 or
//! a 'donut' chart.
//!
//! Includes the following features
//! * An optional legend
//! * configuring colors
//! * specifying start and end angles
//! * animated highlighting of segments
//! * hiding segments (via clicking on the legend)
//! * rendering tooltips (with an optional custom renderer)
//! * showing a text in the middle (useful for gauges/donut charts)

use std::collections::HashSet;

use yew::html::IntoPropValue;
use yew::prelude::*;

use crate::css;
use crate::dom::align::align_to_xy;
use crate::prelude::*;
use crate::props::{IntoOptionalRenderFn, RenderFn};
use crate::widget::canvas::{Canvas, Circle, Group, Text};
use crate::widget::{Button, Container};

use pwt_macros::{builder, widget};

// default colors
const DEFAULT_COLORS: &[&str] = &[
    "var(--pwt-color-primary)",
    "var(--pwt-color-secondary)",
    "var(--pwt-color-tertiary)",
];

const GAUGE_COLORS: &[(f64, &str)] = &[
    (0.9, "var(--pwt-color-error)"),
    (0.7, "var(--pwt-color-warning)"),
    (0.0, "var(--pwt-color-primary)"),
];

// base size for the viewBox (before recalculating due to the start/end angles)
const SIZE: f64 = 110.0;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
/// The position of the legend relative to a chart.
pub enum LegendPosition {
    Hidden,
    Start,
    End,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq)]
enum ChartVariant {
    Gauge,
    Pie,
}

#[widget(pwt=crate, comp=PwtPieChart, @element)]
#[derive(Properties, Clone, PartialEq)]
#[builder]
/// Pie chart properties
pub struct PieChart {
    #[builder]
    #[prop_or(0.3)]
    /// The ratio of the ring to the inner radius of the pie chart, must be between 0.01 and 1.0.
    ///
    /// 1.0 results in a pie chart, values below in a donut shaped one.
    /// Default is 1.0 for when using [PieChart::pie], 0.3 otherwise.
    thickness_ratio: f64,

    // the values to show. if only one, it's assumed to be between 0.0 and 1.0
    // and the remainder will be calculated. otherwise it's assumed they're relative
    // values
    values: Vec<(AttrValue, f64)>,

    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    /// Text to display inside the chart. Note that the size and relative position of the text does
    /// not change when the ratio changes or when the chart is clipped due to different start/end
    /// angles. Most useful when using default gauge charts (or only slightly changed angles).
    text: Option<AttrValue>,

    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    /// The list of colors to use. Only used when giving multiple values.
    ///
    /// For the 'gauge' type chart, The 'primary', 'warning' and 'error' colors are used.
    /// For the multi-value chart 'primary', 'secondary' and 'tertiary' colors are used.
    ///
    /// If there are not enough colors for the given amount of values, the colors will
    /// be reused and mixed with an increasing amount of the 'surface' color.
    colors: Option<Vec<AttrValue>>,

    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    /// Allows highlighting the separate value segments. Default is true for charts with multiple
    /// values and false for charts with a single value.
    allow_highlight: Option<bool>,

    #[builder_cb(IntoOptionalRenderFn, into_optional_render_fn, (AttrValue, f64, usize))]
    #[prop_or_default]
    /// Tooltip renderer to override the default one. The parameters are the title, the value and
    /// the index into values.
    render_tooltip: Option<RenderFn<(AttrValue, f64, usize)>>,

    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(true)]
    /// Determine if tooltips are shown or not.
    show_tooltip: bool,

    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    /// Determines the position of the legend.
    ///
    /// Defaults:
    /// for gauge type chart: 'Hidden'
    /// for multi-value chart: 'End'
    legend: Option<LegendPosition>,

    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(0.0)]
    /// The starting angle of the chart in degrees, default is 0.0.
    angle_start: f64,

    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(360.0)]
    /// The ending angle of the chart in degrees, default is 360.0,
    angle_end: f64,

    variant: ChartVariant,
}

impl PieChart {
    /// Creates a new chart with a single value from 0.0 to 1.0 (smaller and larger values will
    /// be clamped).
    pub fn gauge(title: impl Into<AttrValue>, value: f64) -> Self {
        yew::props!(Self {
            values: vec![(title.into(), value.clamp(0.0, 1.0))],
            variant: ChartVariant::Gauge,
        })
    }

    /// Creates a new pie chart with multiple relative values.
    pub fn pie(values: Vec<(impl Into<AttrValue>, f64)>) -> Self {
        yew::props!(Self {
            values: values
                .into_iter()
                .map(|(title, value)| (title.into(), value))
                .collect::<Vec<_>>(),
            variant: ChartVariant::Pie,
            thickness_ratio: 1.0,
        })
    }

    /// Creates a new donut chart with multiple relative values.
    pub fn donut(values: Vec<(impl Into<AttrValue>, f64)>) -> Self {
        yew::props!(Self {
            values: values
                .into_iter()
                .map(|(title, value)| (title.into(), value))
                .collect::<Vec<_>>(),
            variant: ChartVariant::Pie,
        })
    }

    fn segment_get_color(&self, index: usize, value: f64) -> String {
        if self.variant == ChartVariant::Gauge {
            for (threshold, color) in GAUGE_COLORS {
                if value >= *threshold {
                    return color.to_string();
                }
            }
        }
        let (base_color, cycle) = match &self.colors {
            Some(colors) => {
                let base = colors[index % colors.len()].to_string();
                let cycle = index / colors.len();
                (base, cycle)
            }
            None => {
                let base = DEFAULT_COLORS[index % DEFAULT_COLORS.len()].to_string();
                let cycle = index / DEFAULT_COLORS.len();
                (base, cycle)
            }
        };
        format!(
            "color-mix(in hsl, {base_color} {}%, var(--pwt-color-surface))",
            100.0 - (20.0 * cycle as f64)
        )
    }

    fn effective_legend_position(&self) -> LegendPosition {
        self.legend.unwrap_or(match self.variant {
            ChartVariant::Gauge => LegendPosition::Hidden,
            ChartVariant::Pie => LegendPosition::End,
        })
    }
}

struct Segment {
    dasharray: String,
    offset: f64,
    highlight_offset: (f64, f64),
    tooltip: Html,
    color: String,
    allow_highlight: bool,
}

pub enum Msg {
    Highlight(Option<usize>),
    MouseOver(Option<(i32, i32)>),
    ToggleValue(usize),
}

pub struct PwtPieChart {
    radius: f32,
    stroke_width: f32,
    start: f64,
    max_len: f64,
    circumference: f64,
    highlight: Option<usize>,
    mouse_pos: Option<(f64, f64)>,
    tooltip_ref: NodeRef,
    svg_ref: NodeRef,
    hidden: HashSet<usize>,
    segments: Vec<Segment>,
    bottom_diff: f64,
    left_diff: f64,
    right_diff: f64,
}

impl PwtPieChart {
    fn new(ctx: &Context<Self>) -> Self {
        let mut this = Self {
            radius: 0.0,
            stroke_width: 0.0,
            start: 0.0,
            max_len: 0.0,
            circumference: 0.0,
            highlight: None,
            mouse_pos: None,
            tooltip_ref: Default::default(),
            svg_ref: Default::default(),
            hidden: HashSet::new(),
            segments: Vec::new(),
            bottom_diff: 0.0,
            left_diff: 0.0,
            right_diff: 0.0,
        };
        this.recalculate_draw_values(ctx);
        this.recalculate_segments(ctx);
        this
    }

    fn recalculate_draw_values(&mut self, ctx: &Context<Self>) {
        let props = ctx.props();
        let ratio = props.thickness_ratio.clamp(0.01, 1.0) as f32;
        self.stroke_width = ratio * 50.0;
        self.radius = 50.0 - (self.stroke_width / 2.0);
        self.circumference = self.radius as f64 * std::f64::consts::TAU;
        self.max_len = self.circumference * ((props.angle_end - props.angle_start) / 360.0);
        self.start = self.circumference * (props.angle_start / 360.0);
        self.left_diff = 0.0;
        self.right_diff = 0.0;
        self.bottom_diff = 0.0;

        let outer_radius = (self.radius + self.stroke_width / 2.0) as f64;
        let inner_radius = outer_radius - self.stroke_width as f64;
        if props.angle_start > 0.0 && props.angle_end < 360.0 {
            let start_cos = props.angle_start.to_radians().cos();
            let end_cos = props.angle_end.to_radians().cos();
            let cos = start_cos.max(end_cos);

            let bottom_radius = if props.angle_start > 90.0 && props.angle_end < 270.0 {
                inner_radius
            } else {
                outer_radius
            };

            self.bottom_diff = outer_radius - (cos * bottom_radius);
        }

        if props.angle_start > 90.0 {
            let sin = props.angle_start.to_radians().sin();
            self.left_diff = outer_radius - sin * outer_radius;
        }

        if props.angle_end < 270.0 {
            let sin = props.angle_end.to_radians().sin();
            self.right_diff = outer_radius + sin * outer_radius;
        }
    }

    fn recalculate_segments(&mut self, ctx: &Context<Self>) {
        let props = ctx.props();
        let mut segments = Vec::with_capacity(props.values.len());

        let (sum, allow_highlight) = match props.variant {
            ChartVariant::Gauge => (1.0, false),
            ChartVariant::Pie => {
                let sum = props
                    .values
                    .iter()
                    .enumerate()
                    .filter_map(|(index, (_, value))| {
                        (!self.hidden.contains(&index)).then_some(value)
                    })
                    .sum();

                (sum, props.allow_highlight.unwrap_or(true))
            }
        };

        let mut last_value = 0.0;
        for (index, (title, value)) in props.values.iter().enumerate() {
            let is_visible = !self.hidden.contains(&index);

            let highlight_offset = if is_visible {
                let percent = (last_value + value * 0.5) / sum;
                let angle = ((props.angle_end - props.angle_start) * percent + props.angle_start)
                    .to_radians();
                (4.0 * angle.cos(), 4.0 * angle.sin())
            } else {
                (0.0, 0.0)
            };
            // even if this segment is not visible, add the circle but with size 0 so the animation
            // has a start point and the segments don't overlap
            let offset = -(self.start + last_value / sum * self.max_len);
            let length = if is_visible {
                // adding 0.01 here to reduce the seams without changing the size too much
                value / sum * self.max_len + 0.01
            } else {
                0.0
            };
            let remainder = self.circumference;
            let dasharray = format!("{length} {remainder}",);

            if is_visible {
                last_value += value;
            }

            let tooltip = if let Some(renderer) = &props.render_tooltip {
                renderer.apply(&(title.clone(), *value, index))
            } else {
                let pct = (value / sum * 100.0).round();
                format!("{}: {} ({pct}%)", title, value).into()
            };

            let color = props.segment_get_color(index, *value);

            segments.push(Segment {
                dasharray,
                offset,
                highlight_offset,
                tooltip,
                color,
                allow_highlight,
            });
        }

        self.segments = segments;
    }

    fn render_legend(&self, ctx: &Context<Self>) -> Option<Container> {
        let props = ctx.props();
        let direction = match props.effective_legend_position() {
            LegendPosition::Hidden => return None,
            LegendPosition::Start | LegendPosition::End => css::FlexDirection::Column,
            LegendPosition::Top | LegendPosition::Bottom => css::FlexDirection::Row,
        };
        let legend = Container::new()
            .class("pwt-gap-1")
            .class(css::Display::Flex)
            .class(direction)
            .class(css::AlignItems::Stretch)
            .children(self.segments.iter().enumerate().map(|(index, segment)| {
                let visible = !self.hidden.contains(&index);
                let icon_class = classes!(
                    "fa",
                    "fa-circle",
                    "pwt-legend-color",
                    (!visible).then_some("pwt-opacity-25")
                );
                Button::new(props.values[index].0.to_string())
                    .style("--pwt-legend-color", segment.color.clone())
                    .icon_class(icon_class)
                    .on_activate(ctx.link().callback(move |_| Msg::ToggleValue(index)))
                    .onpointerenter(ctx.link().callback(move |_| Msg::Highlight(Some(index))))
                    .onpointerleave(ctx.link().callback(move |_| Msg::Highlight(None)))
                    .into()
            }));
        Some(legend)
    }

    fn render_tooltip(&self, ctx: &Context<Self>) -> Option<Html> {
        let props = ctx.props();
        match (props.show_tooltip, self.mouse_pos, self.highlight) {
            (true, Some(_), Some(index)) => Some(
                Container::new()
                    .attribute("role", "tooltip")
                    .attribute("aria-live", "polite")
                    .attribute("data-show", Some(""))
                    .class("pwt-tooltip")
                    .class("pwt-tooltip-rich")
                    .with_child(self.segments[index].tooltip.clone())
                    .into_html_with_ref(self.tooltip_ref.clone()),
            ),
            _ => None,
        }
    }
}

impl Component for PwtPieChart {
    type Message = Msg;
    type Properties = PieChart;

    fn create(ctx: &Context<Self>) -> Self {
        Self::new(ctx)
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        self.recalculate_draw_values(ctx);
        if ctx.props().values != old_props.values {
            self.hidden = HashSet::new();
            self.mouse_pos = None;
            self.highlight = None;
        }
        self.recalculate_segments(ctx);
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Highlight(highlight) => self.highlight = highlight,
            Msg::MouseOver(pos) => {
                if let Some((x, y)) = pos {
                    self.mouse_pos = Some((x as f64, y as f64));
                } else {
                    self.mouse_pos = None;
                }
            }
            Msg::ToggleValue(index) => {
                if self.hidden.contains(&index) {
                    self.hidden.remove(&index);
                    self.recalculate_segments(ctx);
                } else if self.hidden.len() + 1 < ctx.props().values.len() {
                    self.hidden.insert(index);
                    self.recalculate_segments(ctx);
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut group = Group::new().style("transform", "rotate(90deg)");

        // background for the 'gauge' type chart
        if props.variant == ChartVariant::Gauge {
            group.add_child(
                Circle::new()
                    .fill("none")
                    .r(self.radius)
                    .stroke("var(--pwt-color-surface)")
                    .stroke_width(self.stroke_width)
                    .style(
                        "stroke-dasharray",
                        format!("{} {}", self.max_len, self.circumference),
                    )
                    .style("stroke-dashoffset", (-self.start).to_string())
                    .style("transition", "0.3s"),
            );
        };

        for (index, segment) in self.segments.iter().enumerate() {
            let (x_off, y_off) = match self.highlight {
                Some(idx) if idx == index && segment.allow_highlight => segment.highlight_offset,
                _ => (0.0, 0.0),
            };

            group.add_child(
                Circle::new()
                    .fill("none")
                    .r(self.radius)
                    .stroke(segment.color.to_string())
                    .stroke_width(self.stroke_width)
                    .style("stroke-dasharray", segment.dasharray.clone())
                    .style("stroke-dashoffset", segment.offset.to_string())
                    .style("transition", "0.3s")
                    .style("transform", format!("translate({x_off}px, {y_off}px)"))
                    .onpointerenter(ctx.link().callback(move |_| Msg::Highlight(Some(index))))
                    .onpointerleave(ctx.link().callback(|_| Msg::Highlight(None))),
            );

            // invisible copy of the original position so we don't flicker between positions when
            // the mouse is positioned on parts of the segment that moves out of the way
            group.add_child(
                Circle::new()
                    .fill("none")
                    .r(self.radius)
                    .stroke("transparent")
                    .stroke_width(self.stroke_width)
                    .style("stroke-dasharray", segment.dasharray.clone())
                    .style("stroke-dashoffset", segment.offset.to_string())
                    .onpointerenter(ctx.link().callback(move |_| Msg::Highlight(Some(index))))
                    .onpointerleave(ctx.link().callback(|_| Msg::Highlight(None))),
            );
        }

        let width = SIZE - (self.left_diff + self.right_diff);
        let height = SIZE - self.bottom_diff;
        let mut canvas = Canvas::new()
            .onpointermove(ctx.link().callback(|event: PointerEvent| {
                Msg::MouseOver(Some((event.client_x(), event.client_y())))
            }))
            .onpointerleave(ctx.link().callback(|_| Msg::MouseOver(None)))
            .attribute(
                "viewBox",
                format!(
                    "{} {} {} {}",
                    -(width - self.left_diff + self.right_diff) / 2.0,
                    -(height + self.bottom_diff) / 2.0,
                    width,
                    height
                ),
            )
            .with_child(group);

        canvas.add_optional_child(props.text.as_ref().map(|text| {
            Text::new(text.to_string())
                .dy(0)
                .attribute("text-anchor", "middle")
                .attribute("alignment-baseline", "central")
        }));

        Container::new()
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .class(css::Display::Flex)
            .class(css::AlignItems::Center)
            .class(match props.effective_legend_position() {
                LegendPosition::Hidden => css::FlexDirection::Column,
                LegendPosition::Bottom => css::FlexDirection::Column,
                LegendPosition::Top => css::FlexDirection::ColumnReverse,
                LegendPosition::End => css::FlexDirection::Row,
                LegendPosition::Start => css::FlexDirection::RowReverse,
            })
            .with_child(canvas.into_html_with_ref(self.svg_ref.clone()))
            .with_optional_child(self.render_legend(ctx))
            .with_optional_child(self.render_tooltip(ctx))
            .into()
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let Some((x, y)) = self.mouse_pos {
            if let Some(tooltip_ref) = self.tooltip_ref.get() {
                let _ = align_to_xy(
                    tooltip_ref,
                    (x + 20.0, y + 20.0),
                    crate::dom::align::Point::TopStart,
                );
            }
        }
    }
}
