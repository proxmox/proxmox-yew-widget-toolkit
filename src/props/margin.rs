use super::AsClassesMut;
use yew::html::IntoPropValue;
use yew::Classes;

/// Defines methods to use CSS margin classes.
///
/// The default CSS template defines utility classes for margins.
///
/// - `pwt-m-{S}`: margin on all sides.
/// - `pwt-mx-{S}`: margin on x-axis (start and end).
/// - `pwt-my-{S}`: margin on y-axis (top and bottom).
/// - `pwt-mt-{S}`: margin on top.
/// - `pwt-mb-{S}`: margin on bottom.
/// - `pwt-ms-{S}`: margin at start.
/// - `pwt-me-{S}`: margin at end.
///
/// The template sepcifies those classes for values 0, 1, 2 and 3. The
/// real size is specified inside the CSS and defaults to 0.5em, 1em,
/// 1.5em and 2em.
///
/// This trait get automatically implemented for widgets using the
/// widget macro, and is also implemented on [Classes].
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// Container::new()
///    .margin_x(2)
///    .margin_top(1)
/// # ;
/// ```

pub trait CssMarginBuilder: AsClassesMut + Sized {
    /// Builder style method to add a box margin class.
    fn margin(mut self, margin: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_margin(margin);
        self
    }

    /// Method to add a box margin class.
    fn add_margin(&mut self, margin: impl IntoPropValue<Option<usize>>) {
        if let Some(margin) = margin.into_prop_value() {
            self.as_classes_mut().push(format!("pwt-m-{margin}"));
        }
    }

    /// Builder style method to add a x-axis margin class.
    fn margin_x(mut self, margin: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_margin_x(margin);
        self
    }

    /// Method to add a x-axis margin class.
    fn add_margin_x(&mut self, margin: impl IntoPropValue<Option<usize>>) {
        if let Some(margin) = margin.into_prop_value() {
            self.as_classes_mut().push(format!("pwt-mx-{margin}"));
        }
    }

    /// Builder style method to add a y-axis margin class.
    fn margin_y(mut self, margin: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_margin_y(margin);
        self
    }

    /// Method to add a y-axis margin class.
    fn add_margin_y(&mut self, margin: impl IntoPropValue<Option<usize>>) {
        if let Some(margin) = margin.into_prop_value() {
            self.as_classes_mut().push(format!("pwt-my-{margin}"));
        }
    }

    /// Builder style method to add a top margin class.
    fn margin_top(mut self, margin: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_margin_top(margin);
        self
    }

    /// Method to add a top margin class.
    fn add_margin_top(&mut self, margin: impl IntoPropValue<Option<usize>>) {
        if let Some(margin) = margin.into_prop_value() {
            self.as_classes_mut().push(format!("pwt-mt-{margin}"));
        }
    }

    /// Builder style method to add a bottom margin class.
    fn margin_bottom(mut self, margin: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_margin_bottom(margin);
        self
    }

    /// Method to add a bottom margin class.
    fn add_margin_bottom(&mut self, margin: impl IntoPropValue<Option<usize>>) {
        if let Some(margin) = margin.into_prop_value() {
            self.as_classes_mut().push(format!("pwt-mb-{margin}"));
        }
    }

    /// Builder style method to add a start margin class.
    fn margin_start(mut self, margin: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_margin_start(margin);
        self
    }

    /// Method to add a start margin class.
    fn add_margin_start(&mut self, margin: impl IntoPropValue<Option<usize>>) {
        if let Some(margin) = margin.into_prop_value() {
            self.as_classes_mut().push(format!("pwt-ms-{margin}"));
        }
    }

    /// Builder style method to add an end margin class.
    fn margin_end(mut self, margin: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_margin_end(margin);
        self
    }

    /// Method to add an end margin class.
    fn add_margin_end(&mut self, margin: impl IntoPropValue<Option<usize>>) {
        if let Some(margin) = margin.into_prop_value() {
            self.as_classes_mut().push(format!("pwt-me-{margin}"));
        }
    }
}

impl CssMarginBuilder for Classes {}
