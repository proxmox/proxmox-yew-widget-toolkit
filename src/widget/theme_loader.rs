use std::rc::Rc;

use pwt_macros::builder;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::impl_to_html;
use crate::state::{Theme, ThemeDensity, ThemeObserver};

#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct ThemeLoader {
    body: VNode,

    #[prop_or_default]
    #[builder]
    /// The directory prefix for the css files. (E.g. "/css/")
    pub dir_prefix: AttrValue,
}

impl ThemeLoader {
    pub fn new(body: impl Into<VNode>) -> Self {
        yew::props!(Self { body: body.into() })
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

fn get_document_root() -> Option<web_sys::Element> {
    let window = match web_sys::window() {
        Some(window) => window,
        None => return None,
    };

    let document = match window.document() {
        Some(document) => document,
        None => return None,
    };

    document.document_element()
}

fn set_css_density(density: ThemeDensity) {
    let root = match get_document_root() {
        Some(root) => root,
        None => return,
    };

    let class_list = root.class_list();

    let _ = class_list.remove_3(
        "pwt-density-high",
        "pwt-density-medium",
        "pwt-density-touch",
    );

    match density {
        ThemeDensity::High => {
            let _ = class_list.add_1("pwt-density-high");
        }
        ThemeDensity::Touch => {
            let _ = class_list.add_1("pwt-density-touch");
        }
        ThemeDensity::Medium => {
            let _ = class_list.add_1("pwt-density-medium");
        }
        ThemeDensity::Auto => { /* do nothing */ }
    };
}

impl PwtThemeLoader {
    fn update_theme(&mut self, theme: Theme, dark_mode: bool, loaded: bool, prefix: &str) -> bool {
        let new_css = theme.get_css_filename(dark_mode, prefix).to_string();

        if self.theme_css != new_css && self.new_theme_css.is_none() {
            self.new_theme_css = Some(new_css);
            self.loadstate = LoadState::Loading;
            return true;
        }

        set_css_density(theme.density);

        if self.new_theme_css.is_some() && loaded {
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
        let theme_observer = ThemeObserver::new(ctx.link().callback(Msg::ThemeChanged));

        let theme = theme_observer.theme();
        let dark_mode = theme_observer.dark_mode();

        let props = ctx.props();

        Self {
            theme_observer,
            loadstate: LoadState::Initial,
            theme_css: theme
                .get_css_filename(dark_mode, props.dir_prefix.as_str())
                .to_string(),
            new_theme_css: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let prefix = ctx.props().dir_prefix.as_str();
        match msg {
            Msg::Loaded => self.update_theme(
                self.theme_observer.theme(),
                self.theme_observer.dark_mode(),
                true,
                prefix,
            ),
            Msg::ThemeChanged((theme, dark_mode)) => {
                self.update_theme(theme, dark_mode, false, prefix)
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

impl_to_html!(ThemeLoader);
