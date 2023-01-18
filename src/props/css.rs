//! Wrapper types for CSS properties.
//!
//! This module provides wrapper types for common CSS
//! properties. Using static rust types makes it possible to check
//! correctness at compile type.
//!
//! For common properties, our CSS template contains utility classes
//! to set properties. Corresponding types implements
//! [Into]<[Classes]>, so that you can use the wrapper type when
//! specifying a class name, i.e:
//!
//! ```
//! # use pwt::prelude::*;
//! # use pwt::widget::Container;
//! use pwt::props::css::*;
//! Container::new()
//!    .class(FlexDirection::Row)
//! # ;
//! ```


use yew::Classes;

/// Wrapper type to specify CSS property `flex-direction`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::props::css::*;
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
            FlexDirection::Row => "pwt-flex-row".into(),
            FlexDirection::RowReverse => "pwt-flex-row-reverse".into(),
            FlexDirection::Column => "pwt-flex-column".into(),
            FlexDirection::ColumnReverse => "pwt-flex-column-reverse".into(),
        }
    }
}

/// Wrapper type to specify CSS property `flex-wrap`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::props::css::*;
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
/// use pwt::props::css::*;
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
            JustifyContent::SpaceBetween => "pwt-justify-content-between".into(),
            JustifyContent::SpaceAround => "pwt-justify-content-around".into(),
            JustifyContent::SpaceEvenly => "pwt-justify-content-evenly".into(),
            JustifyContent::Stretch => "pwt-justify-content-stretch".into(),
        }
    }
}

/// Wrapper type to specify CSS property `align-items`.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// use pwt::props::css::*;
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
/// use pwt::props::css::*;
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
/// use pwt::props::css::*;
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
/// use pwt::props::css::*;
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
/// use pwt::props::css::*;
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
