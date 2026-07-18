use std::rc::Rc;

use yew::html::IntoEventCallback;
use yew::html::IntoPropValue;
use yew::virtual_dom::{Key, VComp, VNode};

use pwt_macros::builder;

use crate::css::FontStyle;
use crate::dom::ViewportQuery;
use crate::prelude::*;
use crate::props::{AsCssStylesMut, CssStyles};
use crate::widget::{Button, Column, Container, Dialog, Row};

use super::{SideDialog, SideDialogController, SideDialogLocation};

/// Default media query selecting the wide (centered [Dialog]) layout. Below it the dialog becomes a
/// slide-up bottom sheet. Matches the breakpoint the other adaptive widgets use.
const DEFAULT_WIDE_QUERY: &str = "(min-width: 768px)";

/// A modal that adapts its shell to the viewport.
///
/// On a wide viewport it is a centered, optionally draggable [Dialog]. On a narrow, touch-first one
/// it slides up from the bottom as a [SideDialog] sheet with a drag handle and a header carrying the
/// title and a close button. The same `children` render inside either shell, so a dialog body -
/// including its own hint, error and action rows - keeps its layout across the swap. The active
/// shell switches at runtime when the viewport crosses the breakpoint.
///
/// It mirrors the common [Dialog] builders (`title`, `on_close`, `draggable`, `auto_center`, the
/// [WidgetStyleBuilder] sizing surface) so an existing `Dialog::new(...)` call site can adopt the
/// adaptive behavior by swapping the constructor.
///
/// ```
/// # use pwt::prelude::*;
/// # use pwt::touch::AdaptiveDialog;
/// # fn test(body: Html, on_close: Callback<()>) -> Html {
/// AdaptiveDialog::new("Add a break")
///     .on_close(on_close)
///     .with_child(body)
///     .into()
/// # }
/// ```
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct AdaptiveDialog {
    /// The yew component key.
    #[prop_or_default]
    pub key: Option<Key>,

    /// Dialog title. Rendered in the [Dialog] title bar on wide viewports and in the sheet header on
    /// narrow ones.
    #[prop_or_default]
    pub title: AttrValue,

    /// Dialog body.
    #[prop_or_default]
    pub children: Vec<VNode>,

    /// Close/abort callback. Fires when the [Dialog] is closed or the bottom sheet is dismissed
    /// (backdrop tap, swipe, the header close button or Escape).
    #[builder_cb(IntoEventCallback, into_event_callback, ())]
    #[prop_or_default]
    pub on_close: Option<Callback<()>>,

    /// Whether the wide [Dialog] can be dragged by its title bar. No effect on the bottom sheet.
    #[prop_or(true)]
    #[builder]
    pub draggable: bool,

    /// Whether the wide [Dialog] can be resized. No effect on the bottom sheet.
    #[prop_or_default]
    #[builder]
    pub resizable: bool,

    /// Whether the wide [Dialog] auto-centers on open. No effect on the bottom sheet.
    #[prop_or(true)]
    #[builder]
    pub auto_center: bool,

    /// Media query selecting the wide (centered [Dialog]) layout; below it the dialog becomes a
    /// bottom sheet. Defaults to `(min-width: 768px)`.
    #[builder(IntoPropValue, into_prop_value)]
    #[prop_or(AttrValue::Static(DEFAULT_WIDE_QUERY))]
    pub wide_query: AttrValue,

    /// CSS styles applied to the wide [Dialog] window (for example a `max-width`). The bottom sheet
    /// sizes itself to the viewport instead.
    #[prop_or_default]
    pub styles: CssStyles,
}

impl AsCssStylesMut for AdaptiveDialog {
    fn as_css_styles_mut(&mut self) -> &mut CssStyles {
        &mut self.styles
    }
}

impl WidgetStyleBuilder for AdaptiveDialog {}

impl ContainerBuilder for AdaptiveDialog {
    fn as_children_mut(&mut self) -> &mut Vec<VNode> {
        &mut self.children
    }
}

impl Default for AdaptiveDialog {
    fn default() -> Self {
        Self::new(AttrValue::Static(""))
    }
}

impl AdaptiveDialog {
    /// Create a new instance with the given title.
    pub fn new(title: impl Into<AttrValue>) -> Self {
        yew::props!(Self {
            title: title.into(),
        })
    }
}

#[doc(hidden)]
pub enum Msg {
    ViewportChanged(bool),
}

#[doc(hidden)]
pub struct PwtAdaptiveDialog {
    is_wide: bool,
    viewport: Option<ViewportQuery>,
    side_controller: SideDialogController,
}

impl PwtAdaptiveDialog {
    fn eval_viewport(ctx: &Context<Self>) -> (bool, Option<ViewportQuery>) {
        ViewportQuery::subscribe(
            ctx.props().wide_query.as_str(),
            ctx.link().callback(Msg::ViewportChanged),
        )
    }
}

impl Component for PwtAdaptiveDialog {
    type Message = Msg;
    type Properties = AdaptiveDialog;

    fn create(ctx: &Context<Self>) -> Self {
        let (is_wide, viewport) = Self::eval_viewport(ctx);
        Self {
            is_wide,
            viewport,
            side_controller: SideDialogController::new(),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        if ctx.props().wide_query != old_props.wide_query {
            (self.is_wide, self.viewport) = Self::eval_viewport(ctx);
        }
        true
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ViewportChanged(is_wide) => {
                if self.is_wide == is_wide {
                    return false;
                }
                self.is_wide = is_wide;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        if self.is_wide {
            return Dialog::new(props.title.clone())
                .on_close(props.on_close.clone())
                .draggable(props.draggable)
                .resizable(props.resizable)
                .auto_center(props.auto_center)
                .styles(props.styles.clone())
                .children(props.children.clone())
                .into();
        }

        // Narrow: a bottom sheet. The Bottom slider has no height cap of its own, so pin it to a
        // column filling the sheet and cap it below the viewport; the body between the header and
        // its own action row scrolls.
        let controller = self.side_controller.clone();
        // mirror Dialog, which only adds its close tool when a handler exists: SideDialog's close
        // path is a no-op without one, so an unconditional button would be a dead affordance
        let close_button = props.on_close.is_some().then(|| {
            Button::new_icon("fa fa-times")
                .attribute("aria-label", tr!("Close"))
                .onclick(move |_| controller.close_dialog())
        });
        let header = Row::new()
            .class("pwt-align-items-center")
            .class("pwt-bg-color-surface")
            .padding(2)
            .gap(2)
            .with_child(
                Container::from_tag("span")
                    .class(FontStyle::TitleLarge)
                    .with_child(props.title.to_string()),
            )
            .with_flex_spacer()
            .with_optional_child(close_button);

        SideDialog::new()
            .location(SideDialogLocation::Bottom)
            .controller(self.side_controller.clone())
            .on_close(props.on_close.clone())
            .style("flex-direction", "column")
            .style("max-height", "90dvh")
            .with_child(
                Column::new()
                    .class("pwt-flex-fit")
                    .class("pwt-adaptive-dialog-sheet")
                    .with_child(header)
                    .children(props.children.clone()),
            )
            .into()
    }
}

impl From<AdaptiveDialog> for VNode {
    fn from(val: AdaptiveDialog) -> Self {
        let key = val.key.clone();
        let comp = VComp::new::<PwtAdaptiveDialog>(Rc::new(val), key);
        VNode::from(comp)
    }
}
