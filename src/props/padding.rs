use crate::props::{AsClassesMut, AsCssStylesMut};

use super::PwtSpace;

/// Defines methods to use CSS padding classes.use yew::{html::IntoPropValue, AttrValue};
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
///
///
pub trait CssPaddingBuilder: AsClassesMut + AsCssStylesMut + Sized {
    generate_padding_trait_fn!(add_padding, padding, "padding", "pwt-p-{}");

    generate_padding_trait_fn!(add_padding_y, padding_y, "padding-block", "pwt-py-{}");
    generate_padding_trait_fn!(add_padding_top, padding_top, "padding-top", "pwt-pt-{}");
    generate_padding_trait_fn!(
        add_padding_bottom,
        padding_bottom,
        "padding-bottom",
        "pwt-pb-{}"
    );

    generate_padding_trait_fn!(add_padding_x, padding_x, "padding-inline", "pwt-px-{}");
    generate_padding_trait_fn!(
        add_padding_start,
        padding_start,
        "padding-inline-start",
        "pwt-ps-{}"
    );
    generate_padding_trait_fn!(
        add_padding_end,
        padding_end,
        "padding-inline-end",
        "pwt-pe-{}"
    );
}
