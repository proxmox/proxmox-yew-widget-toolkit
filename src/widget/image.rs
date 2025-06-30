use yew::html::IntoPropValue;
use yew::prelude::*;
use yew::AttrValue;

use crate::props::{EventSubscriber, WidgetBuilder};
use crate::state::Theme;
use crate::state::ThemeObserver;
use crate::widget::Container;

use pwt_macros::{builder, widget};

/// Image
///
/// An image element. Has convenience options to change the image in dark mode.
#[widget(pwt=crate, comp=PwtImage, @element)]
#[derive(Properties, PartialEq, Clone)]
#[builder]
pub struct Image {
    /// The source url for the image.
    pub src: AttrValue,

    /// An alternative source url for the image used in dark mode.
    #[prop_or_default]
    #[builder(IntoPropValue, into_prop_value)]
    pub dark_mode_src: Option<AttrValue>,
}

impl Image {
    /// Create a new [Image] with a src image url.
    pub fn new(src: impl Into<AttrValue>) -> Self {
        yew::props!(Self { src: src.into() })
    }
}

pub enum Msg {
    ThemeChanged((Theme, bool)),
}

#[doc(hidden)]
pub struct PwtImage {
    dark_mode: bool,
    _theme_observer: ThemeObserver,
}

impl Component for PwtImage {
    type Message = Msg;
    type Properties = Image;

    fn create(_ctx: &Context<Self>) -> Self {
        let _theme_observer = ThemeObserver::new(_ctx.link().callback(Msg::ThemeChanged));
        Self {
            dark_mode: _theme_observer.dark_mode(),
            _theme_observer,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ThemeChanged((_theme, dark_mode)) => self.dark_mode = dark_mode,
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let src = match (self.dark_mode, &props.dark_mode_src) {
            (true, Some(src)) => src.clone(),
            _ => props.src.clone(),
        };
        Container::from_tag("img")
            .with_std_props(&props.std_props)
            .listeners(&props.listeners)
            .attribute("src", src)
            .into()
    }
}
