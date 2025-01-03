use yew::html::IntoPropValue;

use crate::props::{AsClassesMut, AsCssStylesMut};

/// Defines methods to use CSS margin classes.
///
/// The default CSS template defines utility classes for margins that rely on a
/// CSS variable that multiplies with the base width of the spacer to get
/// consistent spacings.
///
/// - `pwt-m` (`--pwt-margin-factor`): margin on all sides
/// - `pwt-mx` (`--pwt-margin-x-factor`): margin on x-axis (start and end).
/// - `pwt-my` (`--pwt-margin-y-factor`): margin on y-axis (top and bottom).
/// - `pwt-mt` (`--pwt-margin-top-factor`): margin on top.
/// - `pwt-mb` (`--pwt-margin-bottom-factor`): margin on bottom.
/// - `pwt-ms` (`--pwt-margin-start-factor`): margin at start.
/// - `pwt-me` (`--pwt-margin-end-factor`): margin at end.
///
/// The template specifies those classes and the code sets the variable via
/// the `style` attribute on the element. The base size is specified inside
/// the CSS.
///
/// This trait get automatically implemented for widgets using the
/// widget macro.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// Container::new()
///    .margin_x(2)
///    .margin_top(1)
/// # ;
/// ```
pub trait CssMarginBuilder: AsClassesMut + AsCssStylesMut + Sized {
    /// Builder style method to add a box margin class.
    fn margin(mut self, margin: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_margin(margin);
        self
    }

    /// Method to add a box margin class.
    fn add_margin(&mut self, margin: impl IntoPropValue<Option<usize>>) {
        if let Some(margin) = margin.into_prop_value() {
            self.as_classes_mut().push("pwt-m");
            self.as_css_styles_mut()
                .set_style("--pwt-margin-factor", margin.to_string())
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
            self.as_classes_mut().push("pwt-mx");
            self.as_css_styles_mut()
                .set_style("--pwt-margin-x-factor", margin.to_string());
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
            self.as_classes_mut().push("pwt-my");
            self.as_css_styles_mut()
                .set_style("--pwt-margin-y-factor", margin.to_string());
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
            self.as_classes_mut().push("pwt-mt");
            self.as_css_styles_mut()
                .set_style("--pwt-margin-top-factor", margin.to_string());
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
            self.as_classes_mut().push("pwt-mb");
            self.as_css_styles_mut()
                .set_style("--pwt-margin-bottom-factor", margin.to_string());
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
            self.as_classes_mut().push("pwt-ms");
            self.as_css_styles_mut()
                .set_style("--pwt-margin-start-factor", margin.to_string());
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
            self.as_classes_mut().push("pwt-me");
            self.as_css_styles_mut()
                .set_style("--pwt-margin-end-factor", margin.to_string());
        }
    }
}
