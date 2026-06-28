use std::borrow::Cow;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::VTag;

use pwt_macros::{builder, widget};

use crate::css::ColorScheme;
use crate::props::{ContainerBuilder, CssLength, IntoVTag, WidgetBuilder, WidgetStyleBuilder};
use crate::widget::{Container, Row};

/// One sub-segment inside a [`SegmentedBar`].
///
/// `value` contributes to the bar's total. A segment is rendered as a horizontal slice whose width
/// is `value / sum_of_values` of the available width. `color` picks the [`ColorScheme`] used for
/// the slice's background; `label` is shown in the legend when the bar's `show_legend(true)` flag
/// is set.
#[derive(Clone, PartialEq)]
pub struct Segment {
    value: f64,
    color: ColorScheme,
    label: Option<AttrValue>,
}

impl Segment {
    /// Create a new segment with a value and the default color scheme.
    pub fn new(value: f64) -> Self {
        Self {
            value,
            color: ColorScheme::Primary,
            label: None,
        }
    }

    /// Set the color scheme for the slice background.
    pub fn color(mut self, color: ColorScheme) -> Self {
        self.color = color;
        self
    }

    /// Set the legend label for the segment.
    pub fn label(mut self, label: impl Into<AttrValue>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// The segment's value contribution.
    pub fn get_value(&self) -> f64 {
        self.value
    }

    /// The segment's color scheme.
    pub fn get_color(&self) -> ColorScheme {
        self.color
    }

    /// The segment's legend label, if any.
    pub fn get_label(&self) -> Option<&AttrValue> {
        self.label.as_ref()
    }
}

/// Horizontal stacked bar that breaks one quantity into named sub-buckets.
///
/// Use when the data is a whole made up of parts that add up: used vs. pending vs. remaining of a
/// quota, succeeded vs. failed vs. pending of a batch, allocated vs. cached vs. free of a memory
/// pool. For single-value gauges with a min/max range and an optimum band, use [`Meter`] instead.
///
/// # Scaling
///
/// By default the bar auto-normalizes: every segment fills `value / sum_of_values` of the track,
/// so segments always cover 100% of the width. Set [`total`](Self::total) to give an explicit
/// denominator instead; segments then fill `value / total` and the unfilled remainder shows the
/// background track. If the segment sum exceeds `total`, all segments are scaled down
/// proportionally so the bar caps at 100%.
///
/// # Examples
///
/// Auto-normalized (parts of a known-complete whole):
///
/// ```rust
/// # use pwt::prelude::*;
/// # use pwt::css::ColorScheme;
/// # use pwt::widget::{SegmentedBar, Segment};
/// SegmentedBar::new()
///     .segment(Segment::new(208.0).color(ColorScheme::Neutral).label("Used"))
///     .segment(Segment::new(0.0).color(ColorScheme::Warning).label("Pending"))
///     .segment(Segment::new(390.0).color(ColorScheme::Success).label("Remaining"))
///     .show_legend(true)
/// # ;
/// ```
///
/// Explicit total (consumption against a quota where the unfilled remainder is meaningful):
///
/// ```rust
/// # use pwt::prelude::*;
/// # use pwt::css::ColorScheme;
/// # use pwt::widget::{SegmentedBar, Segment};
/// SegmentedBar::new()
///     .total(40.0)
///     .segment(Segment::new(8.0).color(ColorScheme::Neutral).label("Used"))
///     .segment(Segment::new(2.0).color(ColorScheme::Warning).label("Pending"))
/// # ;
/// ```
///
/// [`Meter`]: crate::widget::Meter
#[widget(pwt=crate, @element)]
#[builder]
#[derive(Default, Clone, PartialEq, Properties)]
pub struct SegmentedBar {
    /// The list of segments to stack from left to right.
    #[prop_or_default]
    pub segments: Vec<Segment>,

    /// Explicit denominator for segment widths.
    ///
    /// When set, each segment fills `value / total` of the track and the rest stays empty.
    /// When unset, segments auto-normalize to fill the bar (`value / sum_of_values`).
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or_default]
    pub total: Option<f64>,

    /// Render a legend underneath the bar with color swatches and
    /// segment labels. Defaults to false.
    #[builder]
    #[prop_or_default]
    pub show_legend: bool,
}

impl SegmentedBar {
    /// Create a new instance with no segments.
    pub fn new() -> Self {
        yew::props! { Self {} }
    }

    /// Append a segment.
    pub fn segment(mut self, segment: Segment) -> Self {
        self.segments.push(segment);
        self
    }

    /// Append multiple segments at once.
    pub fn segments(mut self, segments: impl IntoIterator<Item = Segment>) -> Self {
        self.segments.extend(segments);
        self
    }
}

impl IntoVTag for SegmentedBar {
    fn into_vtag_with_ref(self, node_ref: NodeRef) -> VTag {
        let sum: f64 = self.segments.iter().map(|s| s.value.max(0.0)).sum();
        // Pick a denominator: explicit total wins, but if the segments
        // overflow it we fall back to the sum so the bar caps at 100%
        // and segments stay proportional to each other. When neither
        // total nor sum is positive, use 1.0 so the empty track still
        // renders without collapsing the surrounding layout.
        let total = match self.total {
            Some(t) if t > 0.0 && sum <= t => t,
            _ if sum > 0.0 => sum,
            _ => 1.0,
        };

        let track_children: Vec<Html> = self
            .segments
            .iter()
            .map(|seg| {
                let fraction = seg.value.max(0.0) / total;
                let mut chip = Container::new()
                    .class("pwt-segmented-bar-segment")
                    .class(seg.color)
                    .width(CssLength::Fraction(fraction as f32));
                if let Some(label) = &seg.label {
                    chip = chip.attribute("title", label.clone());
                }
                chip.into()
            })
            .collect();

        let track: Html = Container::new()
            .class("pwt-segmented-bar-track")
            .with_child(
                Row::new()
                    .class("pwt-segmented-bar-row")
                    .children(track_children),
            )
            .into();

        let mut children: Vec<Html> = vec![track];

        if self.show_legend {
            let legend_children: Vec<Html> = self
                .segments
                .iter()
                .map(|seg| -> Html {
                    let swatch: Html = Container::new()
                        .class("pwt-segmented-bar-legend-swatch")
                        .class(seg.color)
                        .into();
                    let text = seg.label.clone().unwrap_or_else(|| "".into());
                    Container::new()
                        .class("pwt-segmented-bar-legend-item")
                        .with_child(swatch)
                        .with_child(Container::new().with_child(text))
                        .into()
                })
                .collect();
            children.push(
                Container::new()
                    .class("pwt-segmented-bar-legend")
                    .children(legend_children)
                    .into(),
            );
        }

        self.std_props.into_vtag(
            Cow::Borrowed("div"),
            node_ref,
            Some("pwt-segmented-bar"),
            None,
            Some(children),
        )
    }
}
