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
