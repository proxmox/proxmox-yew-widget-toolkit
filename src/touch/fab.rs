use yew::html::{IntoEventCallback, IntoPropValue};
use yew::prelude::*;

use crate::props::{EventSubscriber, WidgetBuilder};
use crate::widget::Button;

use pwt_macros::{builder, widget};

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum FabSize {
    Small,
    #[default]
    Standard,
    Large,
}

/// Favorite action button.
#[widget(pwt=crate, comp=PwtFab, @element)]
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct Fab {
    /// The size of the FAB
    #[prop_or_default]
    #[builder]
    pub size: FabSize,

    /// Icon (CSS class).
    pub icon_class: Classes,

    /// Optional Button text (for small buttons)
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub text: Option<AttrValue>,

    /// Click callback
    #[prop_or_default]
    #[builder_cb(IntoEventCallback, into_event_callback, MouseEvent)]
    pub on_activate: Option<Callback<MouseEvent>>,
}

impl Fab {
    /// Create a new instance.
    pub fn new(icon_class: impl Into<Classes>) -> Self {
        yew::props!(Self {
            icon_class: icon_class.into(),
        })
    }
}

#[doc(hidden)]
pub struct PwtFab {}

impl Component for PwtFab {
    type Message = ();
    type Properties = Fab;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();

        let mut icon_class = props.icon_class.clone();
        icon_class.push("pwt-fab-icon");

        let mut class = classes!("pwt-fab");

        match props.size {
            FabSize::Small => {
                class.push("pwt-fab-small");
            }
            FabSize::Standard => {}
            FabSize::Large => {
                class.push("pwt-fab-large");
            }
        }

        if props.text.is_some() {
            class.push("pwt-fab-extended");
        }

        Button::new(&props.text)
            .icon_class(icon_class)
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .class(class)
            .on_activate(Callback::from({
                let on_activate = props.on_activate.clone();
                move |event: MouseEvent| {
                    if let Some(on_activate) = &on_activate {
                        on_activate.emit(event);
                    }
                }
            }))
            .into()
    }
}
