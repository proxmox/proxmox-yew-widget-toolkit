use crate::props::{AsClassesMut, AsCssStylesMut};

use super::PwtSpace;

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
    generate_padding_trait_fn!(add_margin, margin, "margin", "pwt-m-{}");
    generate_padding_trait_fn!(add_margin_x, margin_x, "margin-inline", "pwt-mx-{}");
    generate_padding_trait_fn!(
        add_margin_start,
        margin_start,
        "margin-inline-start",
        "pwt-ms-{}"
    );
    generate_padding_trait_fn!(add_margin_end, margin_end, "margin-inline-end", "pwt-me-{}");

    generate_padding_trait_fn!(add_margin_y, margin_y, "margin-block", "pwt-my-{}");
    generate_padding_trait_fn!(add_margin_top, margin_top, "margin-top", "pwt-mt-{}");
    generate_padding_trait_fn!(
        add_margin_bottom,
        margin_bottom,
        "margin-bottom",
        "pwt-mb-{}"
    );
}
