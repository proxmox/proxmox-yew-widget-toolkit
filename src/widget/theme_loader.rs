use std::rc::Rc;

use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::state::{Theme, ThemeObserver};

#[derive(Properties, Clone, PartialEq)]
pub struct ThemeLoader {
    body: VNode,
    themes: &'static [&'static str]
}

impl ThemeLoader {
    pub fn new(themes: &'static [&'static str], body: impl Into<VNode>) -> Self {
        Self { themes, body: body.into() }
    }
}

enum LoadState {
    Initial,
    Loading,
    Loaded,
}

pub struct PwtThemeLoader {
    loadstate: LoadState,
    theme_css: String,
    new_theme_css: Option<String>,
    theme_observer: ThemeObserver,
}

impl PwtThemeLoader {
    fn update_theme(&mut self, theme: Theme, dark_mode: bool, loaded: bool) -> bool {
        let new_css = theme
            .get_css_filename(dark_mode)
            .to_string();

        if self.theme_css != new_css && self.new_theme_css.is_none() {
            self.new_theme_css = Some(new_css);
            self.loadstate = LoadState::Loading;
            true
        } else if self.new_theme_css.is_some() && loaded {
            self.theme_css = self.new_theme_css.take().unwrap();
            self.loadstate = LoadState::Loaded;
            true
        } else if loaded {
            self.loadstate = LoadState::Loaded;
            true
        } else {
            false
        }
    }
}

pub enum Msg {
    Loaded,
    ThemeChanged((Theme, bool)),
}

impl Component for PwtThemeLoader {
    type Message = Msg;
    type Properties = ThemeLoader;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();

        let theme_observer = ThemeObserver::new(
            props.themes,
            ctx.link().callback(Msg::ThemeChanged)
        );

        let theme = theme_observer.theme();
        let dark_mode =  theme_observer.dark_mode();

        Self {
            theme_observer,
            loadstate: LoadState::Initial,
            theme_css: theme.get_css_filename(dark_mode).to_string(),
            new_theme_css: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded => {
                self.update_theme(
                    self.theme_observer.theme(),
                    self.theme_observer.dark_mode(),
                    true,
                )
            }
            Msg::ThemeChanged((theme, dark_mode)) => {
                self.update_theme(theme, dark_mode, false)
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let onload = ctx.link().callback(|_| Msg::Loaded);

        // Note: Try to keep the VDOM, so just set display on the content
        let style = match &self.loadstate {
            LoadState::Loading | LoadState::Loaded => "display: contents;",
            LoadState::Initial => "display: none;",
        };

        html! {
            <>
                // Important: use href as Key, to create a new DOM
                // element for each href, and thus get an load event
                // for each href.
                if let Some(theme) = &self.new_theme_css {
                    <link key={Key::from(self.theme_css.clone())} href={self.theme_css.clone()} rel="stylesheet"/>
                    <link key={Key::from(theme.clone())} {onload} href={theme.clone()} rel="stylesheet"/>
                } else {
                    <link key={Key::from(self.theme_css.clone())} {onload} href={self.theme_css.clone()} rel="stylesheet"/>
                }
                <div key="__theme-loader-content__" {style}>{props.body.clone()}</div>
            </>
        }
    }
}

impl Into<VNode> for ThemeLoader {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtThemeLoader>(Rc::new(self), None);
        VNode::from(comp)
    }
}
