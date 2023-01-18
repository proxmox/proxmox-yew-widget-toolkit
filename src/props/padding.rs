use yew::Classes;
use yew::html::IntoPropValue;
use super:: AsClassesMut;

/// Defines methods to use CSS padding classes.
///
/// The default CSS template defines utility classes for paddings.
///
/// - `pwt-p-{S}`: padding on all sides.
/// - `pwt-px-{S}`: padding on x-axis (start and end).
/// - `pwt-py-{S}`: padding on y-axis (top and bottom).
/// - `pwt-pt-{S}`: padding on top.
/// - `pwt-pb-{S}`: padding on bottom.
/// - `pwt-ps-{S}`: padding at start.
/// - `pwt-pe-{S}`: padding at end.
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
///    .padding_x(2)
///    .padding_top(1)
/// # ;
/// ```

pub trait CssPaddingBuilder: AsClassesMut + Sized {

    /// Builder style method to add a box padding class.
    fn padding(mut self, padding: impl IntoPropValue<Option<usize>>) -> Self {
        self.add_padding(padding);
        self
    }

    /// Method to add a box padding class.
    fn add_padding(&mut self, padding: impl IntoPropValue<Option<usize>>) {
        if let Some(padding) = padding.into_prop_value() {
            self.as_classes_mut().push(format!("pwt-p-{padding}"));
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
            self.as_classes_mut().push(format!("pwt-px-{padding}"));
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
            self.as_classes_mut().push(format!("pwt-py-{padding}"));
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
            self.as_classes_mut().push(format!("pwt-pt-{padding}"));
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
            self.as_classes_mut().push(format!("pwt-pb-{padding}"));
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
            self.as_classes_mut().push(format!("pwt-ps-{padding}"));
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
            self.as_classes_mut().push(format!("pwt-pe-{padding}"));
        }
    }
}

impl CssPaddingBuilder for Classes {}
