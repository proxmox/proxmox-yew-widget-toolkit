use std::borrow::Cow;

use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::virtual_dom::{Listeners, VList, VNode, VTag};

use crate::css::ColorScheme;
use crate::props::WidgetBuilder;
use crate::widget::Fa;

use pwt_macros::widget;

/// A small rounded status label, sometimes called a badge, chip or tag.
///
/// A compact pill for a status, count or short attribute: a color-scheme tint, an optional leading
/// icon and some text. Use it for things like "Pending", "3 open" or "On leave" beside the item
/// they describe, rather than re-styling a bare span at every call site.
///
/// The tint comes from a [`ColorScheme`], painted the same way a filled button is, so it works
/// across every theme; the `...Container` schemes read best as a badge background. It defaults to a
/// neutral scheme, so a badge always has a legible fill.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::widget::Badge;
/// # use pwt::css::ColorScheme;
/// # fn test() -> Html {
/// Badge::new("Pending")
///     .icon("hourglass-half")
///     .color_scheme(ColorScheme::WarningContainer)
///     .tip("Recorded outside your normal hours, awaiting confirmation")
///     .into()
/// # }
/// ```
#[widget(pwt=crate, comp=PwtBadge, @element)]
#[derive(Properties, PartialEq, Clone)]
pub struct Badge {
    /// The badge content (text or richer nodes).
    #[prop_or_default]
    content: Option<VNode>,

    /// Leading Font-Awesome icon class, without the `fa ` prefix (e.g. `"check"`).
    #[prop_or_default]
    icon_class: Option<AttrValue>,

    /// Color-scheme tint. A `...Container` scheme reads best; defaults to `NeutralAlt` when unset.
    #[prop_or_default]
    color_scheme: Option<ColorScheme>,
}

impl Badge {
    /// Create a new badge with the given content.
    pub fn new(content: impl Into<VNode>) -> Self {
        yew::props!(Self {
            content: Some(content.into()),
        })
    }

    /// Builder style method to set the leading icon (Font-Awesome class without the `fa ` prefix).
    pub fn icon(mut self, icon_class: impl Into<AttrValue>) -> Self {
        self.icon_class = Some(icon_class.into());
        self
    }

    /// Builder style method to set the color-scheme tint.
    pub fn color_scheme(mut self, scheme: ColorScheme) -> Self {
        self.color_scheme = Some(scheme);
        self
    }

    /// Builder style method to set a native tooltip shown on hover.
    pub fn tip(self, tip: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.attribute("title", tip)
    }
}

#[doc(hidden)]
pub struct PwtBadge;

impl Component for PwtBadge {
    type Message = ();
    type Properties = Badge;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        // Always apply a scheme so the pill paints; a neutral one unless the caller set another.
        // The base `.pwt-badge` reads the scheme's --pwt-color-background / --pwt-color, so the
        // tint reaches the pill the same way a filled button's does, in every theme.
        let scheme = props.color_scheme.unwrap_or(ColorScheme::NeutralAlt);
        let class = classes!("pwt-badge", scheme);
        let attributes = props.std_props.cumulate_attributes(Some(class));

        let mut children: Vec<VNode> = Vec::new();
        if let Some(icon) = &props.icon_class {
            children.push(Fa::new(icon).into());
        }
        if let Some(content) = &props.content {
            children.push(content.clone());
        }

        let listeners = Listeners::Pending(props.listeners.listeners.clone().into_boxed_slice());

        VTag::__new_other(
            Cow::Borrowed("span"),
            NodeRef::default(),
            props.std_props.key.clone(),
            attributes,
            listeners,
            VList::with_children(children, None).into(),
        )
        .into()
    }
}
