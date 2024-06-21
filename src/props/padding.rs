use yew::html::IntoPropValue;

use crate::props::{AsClassesMut, AsCssStylesMut};

/// Defines methods to use CSS padding classes.
///
/// The default CSS template defines utility classes for paddings that rely on
/// a CSS variable that multiplies the base width of the spacer to get
/// consistent spacings.
///
/// - `pwt-p` (`--pwt-padding-factor`): padding on all sides.
/// - `pwt-px` (`--pwt-padding-x-factor`): padding on x-axis (start and end).
/// - `pwt-py` (`--pwt-padding-y-factor`): padding on y-axis (top and bottom).
/// - `pwt-pt` (`--pwt-padding-top-factor`): padding on top.
/// - `pwt-pb` (`--pwt-padding-bottom-factor`): padding on bottom.
/// - `pwt-ps` (`--pwt-padding-start-factor`): padding at start.
/// - `pwt-pe` (`--pwt-padding-end-factor`): padding at end.
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
///    .padding_x(2)
///    .padding_top(1)
/// # ;
/// ```

pub trait CssPaddingBuilder: AsClassesMut + AsCssStylesMut + Sized {
    /// Builder style method to add a box padding class.
    fn padding(mut self, padding: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_padding(padding);
        self
    }

    /// Method to add a box padding class.
    fn add_padding(&mut self, padding: impl IntoPropValue<Option<usize>>) {
        if let Some(padding) = padding.into_prop_value() {
            self.as_classes_mut().push("pwt-p");
            self.as_css_styles_mut()
                .set_style("--pwt-padding-factor", padding.to_string())
        }
    }

    /// Builder style method to add a x-axis padding class.
    fn padding_x(mut self, padding: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_padding_x(padding);
        self
    }

    /// Method to add a x-axis padding class.
    fn add_padding_x(&mut self, padding: impl IntoPropValue<Option<usize>>) {
        if let Some(padding) = padding.into_prop_value() {
            self.as_classes_mut().push("pwt-px");
            self.as_css_styles_mut()
                .set_style("--pwt-padding-x-factor", padding.to_string())
        }
    }

    /// Builder style method to add a y-axis padding class.
    fn padding_y(mut self, padding: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_padding_y(padding);
        self
    }

    /// Method to add a y-axis padding class.
    fn add_padding_y(&mut self, padding: impl IntoPropValue<Option<usize>>) {
        if let Some(padding) = padding.into_prop_value() {
            self.as_classes_mut().push("pwt-py");
            self.as_css_styles_mut()
                .set_style("--pwt-padding-y-factor", padding.to_string())
        }
    }

    /// Builder style method to add a top padding class.
    fn padding_top(mut self, padding: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_padding_top(padding);
        self
    }

    /// Method to add a top padding class.
    fn add_padding_top(&mut self, padding: impl IntoPropValue<Option<usize>>) {
        if let Some(padding) = padding.into_prop_value() {
            self.as_classes_mut().push("pwt-pt");
            self.as_css_styles_mut()
                .set_style("--pwt-padding-top-factor", padding.to_string())
        }
    }

    /// Builder style method to add a bottom padding class.
    fn padding_bottom(mut self, padding: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_padding_bottom(padding);
        self
    }

    /// Method to add a bottom padding class.
    fn add_padding_bottom(&mut self, padding: impl IntoPropValue<Option<usize>>) {
        if let Some(padding) = padding.into_prop_value() {
            self.as_classes_mut().push("pwt-pb");
            self.as_css_styles_mut()
                .set_style("--pwt-padding-bottom-factor", padding.to_string())
        }
    }

    /// Builder style method to add a start padding class.
    fn padding_start(mut self, padding: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_padding_start(padding);
        self
    }

    /// Method to add a start padding class.
    fn add_padding_start(&mut self, padding: impl IntoPropValue<Option<usize>>) {
        if let Some(padding) = padding.into_prop_value() {
            self.as_classes_mut().push("pwt-ps");
            self.as_css_styles_mut()
                .set_style("--pwt-padding-start-factor", padding.to_string())
        }
    }

    /// Builder style method to add an end padding class.
    fn padding_end(mut self, padding: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_padding_end(padding);
        self
    }

    /// Method to add an end padding class.
    fn add_padding_end(&mut self, padding: impl IntoPropValue<Option<usize>>) {
        if let Some(padding) = padding.into_prop_value() {
            self.as_classes_mut().push("pwt-pe");
            self.as_css_styles_mut()
                .set_style("--pwt-padding-end-factor", padding.to_string())
        }
    }
}
