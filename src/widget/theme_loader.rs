use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::MediaQueryList;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::state::Theme;

#[derive(Properties, Clone, PartialEq)]
pub struct ThemeLoader {
    body: VNode,
}

impl ThemeLoader {

    pub fn new(body: impl Into<VNode>) -> Self {
        Self { body: body.into() }
    }
}

pub struct PwtThemeLoader {
    loaded: bool,
    theme: Theme,
    theme_css: String,
    media_query: MediaQueryList,
    scheme_changed_closure: Option<Closure::<dyn Fn()>>,
    theme_changed_closure: Option<Closure::<dyn Fn()>>,
    system_prefer_dark: bool,
}

impl PwtThemeLoader {

    fn update_theme(&mut self, theme: Theme, prefer_dark_mode: bool) -> bool {
        self.theme = theme;
        self.system_prefer_dark = prefer_dark_mode;

        let new_css = self.theme.get_css_filename(self.system_prefer_dark).to_string();
        if self.theme_css != new_css {
            self.theme_css = new_css;
            self.loaded = false;
            true
        } else {
            false
        }
    }

    fn add_theme_changed_listener(&mut self, ctx: &Context<Self>) {
        let theme_changed_closure = Closure::wrap({
            let link = ctx.link().clone();
            Box::new(move || {
                link.send_message(Msg::ThemeChanged);
            }) as Box<dyn Fn()>
        });

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let _ = document.add_event_listener_with_callback(
            "pwt-theme-changed",
            theme_changed_closure.as_ref().unchecked_ref(),
        );
        self.theme_changed_closure = Some(theme_changed_closure); // keep alive
    }

    fn remove_theme_changed_listener(&mut self) {
        let theme_changed_closure = match self.theme_changed_closure.take() {
            Some(closure) => closure,
            None => return,
        };

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let _ = document.remove_event_listener_with_callback(
            "pwt-theme-changed",
            theme_changed_closure.as_ref().unchecked_ref(),
        );
    }

    fn add_prefers_color_scheme_listener(&mut self, ctx: &Context<Self>) {
        let scheme_changed_closure = Closure::wrap({
            let link = ctx.link().clone();
            Box::new(move || {
                link.send_message(Msg::SchemeChanged);
            }) as Box<dyn Fn()>
        });

        let dark = self.media_query.matches();
        if self.system_prefer_dark != dark {
            ctx.link().send_message(Msg::SchemeChanged);
        }
        let _ = self.media_query.add_event_listener_with_callback(
            "change",
            scheme_changed_closure.as_ref().unchecked_ref(),
        );

        self.scheme_changed_closure = Some(scheme_changed_closure);
    }

    fn remove_prefers_color_scheme_listener(&mut self) {
        let scheme_changed_closure = match self.scheme_changed_closure.take() {
            Some(closure) => closure,
            None => return,
        };

        let _ = self.media_query.remove_event_listener_with_callback(
            "change",
            scheme_changed_closure.as_ref().unchecked_ref(),
        );
    }
}

pub enum Msg {
    Loaded,
    SchemeChanged,
    ThemeChanged,
}

fn get_system_prefer_dark_mode() -> bool {
    let window = web_sys::window().unwrap();
    if let Ok(Some(list)) = window.match_media("(prefers-color-scheme: dark)") {
        list.matches()
    } else {
        false
    }
}

impl Component for PwtThemeLoader {
    type Message = Msg;
    type Properties = ThemeLoader;

    fn create(_ctx: &Context<Self>) -> Self {
        let theme = Theme::load().unwrap_or(Theme::default());
        let system_prefer_dark = get_system_prefer_dark_mode();

        let window = web_sys::window().unwrap();
        let media_query = match window.match_media("(prefers-color-scheme: dark)") {
            Ok(Some(media_query)) => media_query,
            _ => panic!("window.match_media() failed!"),
        };

        Self {
            loaded: false,
            theme,
            theme_css: theme.get_css_filename(system_prefer_dark).to_string(),
            scheme_changed_closure: None,
            media_query,
            theme_changed_closure: None,
            system_prefer_dark,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded => {
                self.loaded = true;
                true
            }
            Msg::ThemeChanged => {
                let theme = Theme::load().unwrap_or(Theme::default());
                self.update_theme(theme, self.system_prefer_dark)
            }
            Msg::SchemeChanged => {
                let system_prefer_dark = get_system_prefer_dark_mode();
                self.update_theme(self.theme, system_prefer_dark)
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let onload = ctx.link().callback(|_| Msg::Loaded);

        // Note: Try to keep the VDOM, so just set display on the content
        let style = if self.loaded { "display: contents;" } else { "display: none" };

        html! {
            <>
                // Important: use href as Key, to create a new DOM
                // element for each href, and thus get an load event
                // for each href.
                <link key={Key::from(self.theme_css.clone())} {onload} href={self.theme_css.clone()} rel="stylesheet"/>
                <div key="__theme-loader-content__" {style}>{props.body.clone()}</div>
            </>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.add_theme_changed_listener(ctx);
            self.add_prefers_color_scheme_listener(ctx);
        }
    }

    // Note: loader is likely always there, so this in never called
    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.remove_theme_changed_listener();
        self.remove_prefers_color_scheme_listener();
    }
}

impl Into<VNode> for ThemeLoader {
    fn into(self) -> VNode {
        let comp = VComp::new::<PwtThemeLoader>(Rc::new(self), None);
        VNode::from(comp)
    }
}
