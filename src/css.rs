//! Wrapper types for CSS properties.
//!
//! This module provides wrapper types for common CSS
//! properties. Using static rust types makes it possible to check
//! correctness at compile time.
//!
//! For common properties, our CSS template contains utility classes
//! to set properties. Corresponding types implements
//! [Into]<[Classes]>, so that you can use the wrapper type when
//! specifying a class name, i.e:
//!
//! ```
//! # use pwt::prelude::*;
//! # use pwt::widget::Container;
//! use pwt::css::*;
//! Container::new()
//!    .class(FlexDirection::Row)
//! # ;
//! ```
//!
//! Please note the special traits [CssMarginBuilder], [CssPaddingBuilder] and
//! [CssBorderBuilder] which provides convienient builder methods to set margin,
//! padding and borders.
//!
//! ```
//! use pwt::prelude::*;
//! use pwt::widget::Container;
//! Container::new()
//!     .margin_top(2)
//!     .padding(1)
//!     .border(true)
//! # ;
//! ```
//!

use yew::Classes;

#[cfg(doc)]
use crate::prelude::*;

/// Wrapper type to specify CSS property `flex-direction`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(FlexDirection::Row)
/// # ;
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl From<FlexDirection> for Classes {
    fn from(value: FlexDirection) -> Self {
        match value {
            FlexDirection::Row => "pwt-flex-direction-row".into(),
            FlexDirection::RowReverse => "pwt-flex-direction-row-reverse".into(),
            FlexDirection::Column => "pwt-flex-direction-column".into(),
            FlexDirection::ColumnReverse => "pwt-flex-direction-column-reverse".into(),
        }
    }
}

/// Wrapper type to specify CSS property `flex-wrap`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(FlexWrap::WrapReverse)
/// # ;
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FlexWrap {
    Wrap,
    WrapReverse,
    NoWrap,
}

impl From<FlexWrap> for Classes {
    fn from(value: FlexWrap) -> Self {
        match value {
            FlexWrap::Wrap => "pwt-flex-wrap".into(),
            FlexWrap::NoWrap => "pwt-flex-nowrap".into(),
            FlexWrap::WrapReverse => "pwt-flex-wrap-reverse".into(),
        }
    }
}

/// Wrapper type to specify CSS property `justify-content`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(JustifyContent::Center)
/// # ;
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum JustifyContent {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Left,
    Right,
    Normal,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
}

impl From<JustifyContent> for Classes {
    fn from(value: JustifyContent) -> Self {
        match value {
            JustifyContent::Start => "pwt-justify-content-start".into(),
            JustifyContent::End => "pwt-justify-content-end".into(),
            JustifyContent::FlexStart => "pwt-justify-content-flex-start".into(),
            JustifyContent::FlexEnd => "pwt-justify-content-flex-end".into(),
            JustifyContent::Center => "pwt-justify-content-center".into(),
            JustifyContent::Left => "pwt-justify-content-left".into(),
            JustifyContent::Right => "pwt-justify-content-right".into(),
            JustifyContent::Normal => "pwt-justify-content-normal".into(),
            JustifyContent::SpaceBetween => "pwt-justify-content-space-between".into(),
            JustifyContent::SpaceAround => "pwt-justify-content-space-around".into(),
            JustifyContent::SpaceEvenly => "pwt-justify-content-space-evenly".into(),
            JustifyContent::Stretch => "pwt-justify-content-stretch".into(),
        }
    }
}

/// Wrapper type to specify CSS property `align-items`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(AlignItems::Center)
/// # ;
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AlignItems {
    Normal,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

impl From<AlignItems> for Classes {
    fn from(value: AlignItems) -> Self {
        match value {
            AlignItems::Normal => "pwt-align-items-normal".into(),
            AlignItems::Start => "pwt-align-items-start".into(),
            AlignItems::End => "pwt-align-items-end".into(),
            AlignItems::FlexStart => "pwt-align-items-flex-start".into(),
            AlignItems::FlexEnd => "pwt-align-items-flex-end".into(),
            AlignItems::Center => "pwt-align-items-center".into(),
            AlignItems::Baseline => "pwt-align-items-baseline".into(),
            AlignItems::Stretch => "pwt-align-items-stretch".into(),
        }
    }
}

/// Wrapper type to specify CSS property `align-self`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(AlignSelf::Center)
/// # ;
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AlignSelf {
    Auto,
    Normal,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

impl From<AlignSelf> for Classes {
    fn from(value: AlignSelf) -> Self {
        match value {
            AlignSelf::Auto => "pwt-align-self-auto".into(),
            AlignSelf::Normal => "pwt-align-self-normal".into(),
            AlignSelf::FlexStart => "pwt-align-self-flex-start".into(),
            AlignSelf::FlexEnd => "pwt-align-self-flex-end".into(),
            AlignSelf::Center => "pwt-align-self-center".into(),
            AlignSelf::Baseline => "pwt-align-self-baseline".into(),
            AlignSelf::Stretch => "pwt-align-self-stretch".into(),
        }
    }
}

/// Wrapper type to specify CSS property `user-select`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(UserSelect::None)
/// # ;
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UserSelect {
    Auto,
    None,
    Text,
    All,
}

impl From<UserSelect> for Classes {
    fn from(value: UserSelect) -> Self {
        match value {
            UserSelect::Auto => "pwt-use-select-auto".into(),
            UserSelect::None => "pwt-use-select-none".into(),
            UserSelect::Text => "pwt-use-select-text".into(),
            UserSelect::All => "pwt-use-select-all".into(),
        }
    }
}

/// Wrapper type to specify CSS property `text-align`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(TextAlign::Justify)
/// # ;
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TextAlign {
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
    JustifyAll,
}

impl From<TextAlign> for Classes {
    fn from(value: TextAlign) -> Self {
        match value {
            TextAlign::Start => "pwt-text-align-start".into(),
            TextAlign::End => "pwt-text-align-end".into(),
            TextAlign::Left => "pwt-text-align-left".into(),
            TextAlign::Right => "pwt-text-align-right".into(),
            TextAlign::Center => "pwt-text-align-center".into(),
            TextAlign::Justify => "pwt-text-align-justify".into(),
            TextAlign::JustifyAll => "pwt-text-align-justify-all".into(),
        }
    }
}

/// Wrapper type to specify CSS property `display`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(Display::InlineBlock)
/// # ;
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Display {
    None,
    Inline,
    InlineBlock,
    Block,
    Grid,
    Table,
    TableRow,
    TableCell,
    Flex,
    InlineFlex,
}

impl From<Display> for Classes {
    fn from(value: Display) -> Self {
        match value {
            Display::None => "pwt-d-none".into(),
            Display::Inline => "pwt-d-inline".into(),
            Display::InlineBlock => "pwt-d-inline-block".into(),
            Display::Block => "pwt-d-block".into(),
            Display::Grid => "pwt-d-grid".into(),
            Display::Table => "pwt-d-table".into(),
            Display::TableRow => "pwt-d-table-row".into(),
            Display::TableCell => "pwt-d-table-cell".into(),
            Display::Flex => "pwt-d-flex".into(),
            Display::InlineFlex => "pwt-d-inline-flex".into(),
        }
    }
}

/// Wrapper type to specify CSS property `overflow`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(Overflow::Hidden)
/// # ;
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

impl From<Overflow> for Classes {
    fn from(value: Overflow) -> Self {
        match value {
            Overflow::Visible => "pwt-overflow-visible".into(),
            Overflow::Hidden => "pwt-overflow-hidden".into(),
            Overflow::Scroll => "pwt-overflow-scroll".into(),
            Overflow::Auto => "pwt-overflow-auto".into(),
        }
    }
}

/// Wrapper type to specify CSS property `overflow-x`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OverflowX {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

impl From<OverflowX> for Classes {
    fn from(value: OverflowX) -> Self {
        match value {
            OverflowX::Visible => "pwt-overflow-x-visible".into(),
            OverflowX::Hidden => "pwt-overflow-x-hidden".into(),
            OverflowX::Scroll => "pwt-overflow-x-scroll".into(),
            OverflowX::Auto => "pwt-overflow-x-auto".into(),
        }
    }
}

/// Wrapper type to specify CSS property `overflow-y`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum OverflowY {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

impl From<OverflowY> for Classes {
    fn from(value: OverflowY) -> Self {
        match value {
            OverflowY::Visible => "pwt-overflow-y-visible".into(),
            OverflowY::Hidden => "pwt-overflow-y-hidden".into(),
            OverflowY::Scroll => "pwt-overflow-y-scroll".into(),
            OverflowY::Auto => "pwt-overflow-y-auto".into(),
        }
    }
}

/// Wrapper type to specify CSS property `white-space`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(WhiteSpace::Pre)
/// # ;
/// ```
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum WhiteSpace {
    Normal,
    Nowrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

impl From<WhiteSpace> for Classes {
    fn from(value: WhiteSpace) -> Self {
        match value {
            WhiteSpace::Normal => "pwt-white-space-normal".into(),
            WhiteSpace::Nowrap => "pwt-white-space-nowrap".into(),
            WhiteSpace::Pre => "pwt-white-space-pre".into(),
            WhiteSpace::PreWrap => "pwt-white-space-pre-wrap".into(),
            WhiteSpace::PreLine => "pwt-white-space-pre-line".into(),
            WhiteSpace::BreakSpaces => "pwt-white-space-break-spaces".into(),
        }
    }
}

/// CSS utility type to truncate text with elipsis.
///
/// `overflow: hidden`, `text-overflow: elipsis`, `white-space: nowrap`
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(TextTruncate)
/// # ;
/// ```
pub struct TextTruncate;
impl From<TextTruncate> for Classes {
    fn from(_value: TextTruncate) -> Self {
        "pwt-text-truncate".into()
    }
}

/// CSS utility type to fit into parent box
///
/// `width: 100%`, `height: 100%`, `overflow: auto`, `box-sizing: border-box`.
///
/// Please note that this only works if the size of the parent is
/// known.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(Fit)
/// # ;
/// ```
pub struct Fit;
impl From<Fit> for Classes {
    fn from(_value: Fit) -> Self {
        "pwt-fit".into()
    }
}

/// CSS utility type to set "flex: 1 1 auto;" to the first child of a container.
///
/// This is useful for placing a child inside the parent box. Please
/// note that this always works even if the parent has no width/height
/// set ("pwt-fit" wont work in that case).
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// # fn dummy(your_child: Html) {
/// Container::new() // parent
///    .class(Display::Flex)
///    .class(FlexFillFirstChild)
///    .with_child(your_child)
/// # ;}
/// ```
pub struct FlexFillFirstChild;
impl From<FlexFillFirstChild> for Classes {
    fn from(_value: FlexFillFirstChild) -> Self {
        "pwt-flex-fill-first-child".into()
    }
}

/// CSS utility type to fit into viewport (use all visible space)
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(Viewport)
/// # ;
/// ```
pub struct Viewport;
impl From<Viewport> for Classes {
    fn from(_value: Viewport) -> Self {
        "pwt-viewport".into()
    }
}

/// CSS utility type to set the flex property.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(Flex::Auto)
/// # ;
/// ```
pub enum Flex {
    Fill,
    Auto,
    Initial,
    None,
}

impl From<Flex> for Classes {
    fn from(value: Flex) -> Self {
        match value {
            Flex::Fill => "pwt-flex-fill".into(),
            Flex::Auto => "pwt-flex-auto".into(),
            Flex::Initial => "pwt-flex-initial".into(),
            Flex::None => "pwt-flex-none".into(),
        }
    }
}

/// Wrapper type to specify CSS color scheme class.
///
/// Color schemes defines color/background-color combinations.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(ColorScheme::Primary)
/// # ;
/// ```
/// You can also have custom color schemes like this:
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(ColorScheme::Custom("foo"))
/// # ;
/// ```
///
/// Where the relevant css code must look like this:
/// ```ignore
/// .pwt-scheme-foo {
///    --pwt-color: #ff00ff;
///    --pwt-color-background: #00ff00;
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorScheme {
    Primary,
    PrimaryContainer,
    Secondary,
    SecondaryContainer,
    Tertiary,
    TertiaryContainer,
    Success,
    SuccessContainer,
    Error,
    ErrorContainer,
    Warning,
    WarningContainer,
    Neutral,
    Surface,
    InverseSurface,
    NeutralAlt,
    DarkSurface,
    Custom(&'static str),
}

impl From<ColorScheme> for Classes {
    fn from(value: ColorScheme) -> Self {
        match value {
            ColorScheme::Primary => "pwt-scheme-primary".into(),
            ColorScheme::PrimaryContainer => "pwt-scheme-primary-container".into(),
            ColorScheme::Secondary => "pwt-scheme-secondary".into(),
            ColorScheme::SecondaryContainer => "pwt-scheme-secondary-container".into(),
            ColorScheme::Tertiary => "pwt-scheme-tertiary".into(),
            ColorScheme::TertiaryContainer => "pwt-scheme-tertiary-container".into(),
            ColorScheme::Success => "pwt-scheme-success".into(),
            ColorScheme::SuccessContainer => "pwt-scheme-success-container".into(),
            ColorScheme::Error => "pwt-scheme-error".into(),
            ColorScheme::ErrorContainer => "pwt-scheme-error-container".into(),
            ColorScheme::Warning => "pwt-scheme-warning".into(),
            ColorScheme::WarningContainer => "pwt-scheme-warning-container".into(),
            ColorScheme::Neutral => "pwt-scheme-neutral".into(),
            ColorScheme::Surface => "pwt-scheme-surface".into(),
            ColorScheme::InverseSurface => "pwt-scheme-inverse-surface".into(),
            ColorScheme::NeutralAlt => "pwt-scheme-neutral-alt".into(),
            ColorScheme::DarkSurface => "pwt-scheme-dark-surface".into(),
            ColorScheme::Custom(value) => format!("pwt-scheme-{value}").into(),
        }
    }
}

/// CSS utility type to use shadow classes
///
/// pwt-shadow0 - pwt-shadow5
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(Shadow(0))
/// # ;
/// ```
pub struct Shadow(pub u8);
impl From<Shadow> for Classes {
    fn from(value: Shadow) -> Self {
        format!("pwt-shadow{}", value.0).into()
    }
}

/// CSS utility type to use the pwt-flex-fit class
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(FlexFit)
/// # ;
/// ```
///
/// This is equivalent to:
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::css::*;
/// Container::new()
///    .class(Flex::Fill)
///    .class(Overflow::Auto)
/// # ;
/// ```
///
pub struct FlexFit;
impl From<FlexFit> for Classes {
    fn from(_value: FlexFit) -> Self {
        "pwt-flex-fit".into()
    }
}

/// Wrapper type to specify CSS font color class.
///
/// Very useful in combination with icons.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// # use pwt::widget::Fa;
/// use pwt::css::*;
/// Fa::new("check")
///    .class(FontColor::Success)
/// # ;
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FontColor {
    Primary,
    PrimaryContainer,
    Secondary,
    SecondaryContainer,
    Tertiary,
    TertiaryContainer,
    Success,
    SuccessContainer,
    Error,
    ErrorContainer,
    Warning,
    WarningContainer,
    Neutral,
    Surface,
    InverseSurface,
    NeutralAlt,
    DarkSurface,
}

impl From<FontColor> for Classes {
    fn from(value: FontColor) -> Self {
        match value {
            FontColor::Primary => "pwt-color-primary".into(),
            FontColor::PrimaryContainer => "pwt-color-primary-container".into(),
            FontColor::Secondary => "pwt-color-secondary".into(),
            FontColor::SecondaryContainer => "pwt-color-secondary-container".into(),
            FontColor::Tertiary => "pwt-color-tertiary".into(),
            FontColor::TertiaryContainer => "pwt-color-tertiary-container".into(),
            FontColor::Success => "pwt-color-success".into(),
            FontColor::SuccessContainer => "pwt-color-success-container".into(),
            FontColor::Error => "pwt-color-error".into(),
            FontColor::ErrorContainer => "pwt-color-error-container".into(),
            FontColor::Warning => "pwt-color-warning".into(),
            FontColor::WarningContainer => "pwt-color-warning-container".into(),
            FontColor::Neutral => "pwt-color-neutral".into(),
            FontColor::Surface => "pwt-color-surface".into(),
            FontColor::InverseSurface => "pwt-color-inverse-surface".into(),
            FontColor::NeutralAlt => "pwt-color-neutral-alt".into(),
            FontColor::DarkSurface => "pwt-color-dark-surface".into(),
        }
    }
}

/// Wrapper type to specify CSS opacity class.
///
/// Very useful in combination with icons.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// # use pwt::widget::Fa;
/// use pwt::css::*;
/// Fa::new("check")
///    .class(Opacity::Quarter)
/// # ;
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Opacity {
    Zero,
    Quarter,
    Half,
    ThreeQuarters,
    Full,
}

impl From<Opacity> for Classes {
    fn from(value: Opacity) -> Self {
        match value {
            Opacity::Zero => "pwt-opacity-0".into(),
            Opacity::Quarter => "pwt-opacity-25".into(),
            Opacity::Half => "pwt-opacity-50".into(),
            Opacity::ThreeQuarters => "pwt-opacity-75".into(),
            Opacity::Full => "pwt-opacity-100".into(),
        }
    }
}
