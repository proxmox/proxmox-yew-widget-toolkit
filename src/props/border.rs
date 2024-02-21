use super::AsClassesMut;
use yew::html::IntoPropValue;
use yew::Classes;

/// Defines methods to use CSS border classes.
///
/// The default CSS template defines utility classes for borders.
///
/// - `pwt-border`: border on all sides.
/// - `pwt-no-border`: no border on all sides.
/// - `pwt-border-top`: border on top.
/// - `pwt-no-border-top`: no border on top.
/// - `pwt-border-bottom`: border on bottom.
/// - `pwt-no border-bottom`: no border on bottom.
/// - `pwt-border-start`: border on the start.
/// - `pwt-no-border-start`: no border on the start.
/// - `pwt-border-end`: border on the end.
/// - `pwt-no-border-end`: border on the end.
/// - `pwt-border-left`: border on the left side.
/// - `pwt-no-border-left`: no border on the left side.
/// - `pwt-border-right`: border on the right side.
/// - `pwt-no-border-right`: border on the right side.
///
///
/// This trait get automatically implemented for widgets using the
/// widget macro, and is also implemented on [Classes].
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Container;
/// Container::new()
///    .border(true)
///    .border_top(false)
/// # ;
/// ```

pub trait CssBorderBuilder: AsClassesMut + Sized {
    /// Builder style method to add a box border class.
    fn border(mut self, border: impl IntoPropValue<Option<bool>>) -> Self {
        self.add_border(border);
        self
    }

    /// Method to add a box border class.
    fn add_border(&mut self, border: impl IntoPropValue<Option<bool>>) {
        let border = border.into_prop_value();
        if let Some(border) = border {
            self.as_classes_mut().push(if border {
                "pwt-border"
            } else {
                "pwt-no-border"
            });
        };
    }

    /// Builder style method to add a top border class.
    fn border_top(mut self, border: impl IntoPropValue<Option<bool>>) -> Self {
        self.add_border_top(border);
        self
    }

    /// Method to add a top border class.
    fn add_border_top(&mut self, border: impl IntoPropValue<Option<bool>>) {
        let border = border.into_prop_value();
        if let Some(border) = border {
            self.as_classes_mut().push(if border {
                "pwt-border-top"
            } else {
                "pwt-no-border-top"
            });
        };
    }

    /// Builder style method to add a bottom border class.
    fn border_bottom(mut self, border: impl IntoPropValue<Option<bool>>) -> Self {
        self.add_border_bottom(border);
        self
    }

    /// Method to add a bottom border class.
    fn add_border_bottom(&mut self, border: impl IntoPropValue<Option<bool>>) {
        let border = border.into_prop_value();
        if let Some(border) = border {
            self.as_classes_mut().push(if border {
                "pwt-border-bottom"
            } else {
                "pwt-no-border-bottom"
            });
        };
    }

    /// Builder style method to add a start border class.
    fn border_start(mut self, border: impl IntoPropValue<Option<bool>>) -> Self {
        self.add_border_start(border);
        self
    }

    /// Method to add a start border class.
    fn add_border_start(&mut self, border: impl IntoPropValue<Option<bool>>) {
        let border = border.into_prop_value();
        if let Some(border) = border {
            self.as_classes_mut().push(if border {
                "pwt-border-start"
            } else {
                "pwt-no-border-start"
            });
        };
    }

    /// Builder style method to add an end border class.
    fn border_end(mut self, border: impl IntoPropValue<Option<bool>>) -> Self {
        self.add_border_end(border);
        self
    }

    /// Method to add an end border class.
    fn add_border_end(&mut self, border: impl IntoPropValue<Option<bool>>) {
        let border = border.into_prop_value();
        if let Some(border) = border {
            self.as_classes_mut().push(if border {
                "pwt-border-end"
            } else {
                "pwt-no-border-end"
            });
        };
    }

    /// Builder style method to add a left border class.
    fn border_left(mut self, border: impl IntoPropValue<Option<bool>>) -> Self {
        self.add_border_left(border);
        self
    }

    /// Method to add a left border class.
    fn add_border_left(&mut self, border: impl IntoPropValue<Option<bool>>) {
        let border = border.into_prop_value();
        if let Some(border) = border {
            self.as_classes_mut().push(if border {
                "pwt-border-left"
            } else {
                "pwt-no-border-left"
            });
        };
    }

    /// Builder style method to add a right border class.
    fn border_right(mut self, border: impl IntoPropValue<Option<bool>>) -> Self {
        self.add_border_right(border);
        self
    }

    /// Method to add a right border class.
    fn add_border_right(&mut self, border: impl IntoPropValue<Option<bool>>) {
        let border = border.into_prop_value();
        if let Some(border) = border {
            self.as_classes_mut().push(if border {
                "pwt-border-right"
            } else {
                "pwt-no-border-right"
            });
        };
    }
}

impl CssBorderBuilder for Classes {}
