//! Wrapper types for CSS properties.
//!
//! This module provides wrapper types for common CSS
//! properties. Using static rust types makes it possible to check
//! correctness at compile type.

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
    Column,
}

impl From<FlexDirection> for Classes {
    fn from(value: FlexDirection) -> Self {
        match value {
            FlexDirection::Row => "pwt-flex-row".into(),
            FlexDirection::Column => "pwt-flex-column".into(),
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
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl From<JustifyContent> for Classes {
    fn from(value: JustifyContent) -> Self {
        match value {
            JustifyContent::FlexStart => "pwt-justify-content-start".into(),
            JustifyContent::FlexEnd => "pwt-justify-content-end".into(),
            JustifyContent::Center => "pwt-justify-content-center".into(),
            JustifyContent::SpaceBetween => "pwt-justify-content-between".into(),
            JustifyContent::SpaceAround => "pwt-justify-content-around".into(),
            JustifyContent::SpaceEvenly => "pwt-justify-content-evenly".into(),
        }
    }
}
