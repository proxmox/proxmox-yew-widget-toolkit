use std::rc::Rc;

use pwt_macros::builder;
use yew::prelude::*;
use yew::virtual_dom::{Key, VComp, VNode};

use crate::impl_to_html;
use crate::props::{IntoOptionalTextRenderFn, TextRenderFn};
use crate::state::{Theme, ThemeDensity, ThemeObserver};

/// Dynamic theme loader component.
///
/// This widget serves as the root boundary for theming in the application. It manages:
/// - Loading the appropriate CSS file for the active theme.
/// - Applying global CSS classes for density (e.g., `pwt-density-high`) to the document root.
/// - Applying global CSS classes for dark/light mode (e.g., `pwt-dark-mode`) to the document root.
/// - Handling transitions between themes by loading the new CSS before removing the old one to avoid FOUC.
/// - Exposing the application content only after the initial theme is loaded.
///
/// It uses [ThemeObserver] internally to react to system and application-level theme changes, and
/// displays an empty page until the first theme is successfully loaded.
///
/// This is typically the root of the widget tree, and is used by scaffold widgets
/// like [DesktopApp][crate::widget::DesktopApp] and
/// [MaterialApp][crate::touch::MaterialApp].
///
/// Usage:
/// ```rust
/// use pwt::widget::ThemeLoader;
/// use yew::prelude::*;
///
/// #[function_component(App)]
/// fn app() -> Html {
///     ThemeLoader::new(html! {
///         <div>{"My App Content"}</div>
///     })
///     .into()
/// }
/// ```
#[derive(Properties, Clone, PartialEq)]
#[builder]
pub struct ThemeLoader {
    // The application root content.
    body: VNode,

    /// Optional custom builder for the theme CSS path.
    ///
    /// If provided, this function generates the URL for the CSS file based on the theme name.
    ///
    /// Default behavior: `format!("{}-yew-style.css", theme_name.to_lowercase())`
    #[builder_cb(IntoOptionalTextRenderFn, into_optional_text_render_fn, String)]
    #[prop_or_default]
    pub theme_url_builder: Option<TextRenderFn<String>>,
}

impl ThemeLoader {
    /// Create a new instance wrapping the given content.
    pub fn new(body: impl Into<VNode>) -> Self {
        yew::props!(Self { body: body.into() })
    }

    /// Set the dark/light mode class on the document root (`<html>`).
    ///
    /// This removes any existing `pwt-dark-mode` or `pwt-light-mode` classes and adds
    /// the appropriate one based on the `dark` argument.
    ///
    /// # Static Usage
    ///
    /// While [ThemeLoader] calls this automatically, this function is exposed for applications
    /// that might need to manually control the theme class on the document root, primarily
    /// for integration with other frameworks or in scenarios without a [ThemeLoader].
    pub fn set_dark_mode_on_document_root(dark: bool) {
        let root = match get_document_root() {
            Some(root) => root,
            None => return,
        };

        let class_list = root.class_list();

        let _ = class_list.remove_2("pwt-dark-mode", "pwt-light-mode");

        let _ = class_list.add_1(if dark {
            "pwt-dark-mode"
        } else {
            "pwt-light-mode"
        });
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
    gloo_utils::document().document_element()
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
        ThemeDensity::Compact => {
            let _ = class_list.add_1("pwt-density-high");
        }
        ThemeDensity::Relaxed => {
            let _ = class_list.add_1("pwt-density-touch");
        }
        ThemeDensity::Medium => {
            let _ = class_list.add_1("pwt-density-medium");
        }
        ThemeDensity::Preset => { /* do nothing */ }
    };
}
impl PwtThemeLoader {
    fn get_css_filename(props: &ThemeLoader, theme: &Theme) -> String {
        match &props.theme_url_builder {
            Some(theme_url_builder) => theme_url_builder.apply(&theme.name),
            None => format!("{}-yew-style.css", theme.name.to_lowercase()),
        }
    }

    fn update_theme(
        &mut self,
        props: &ThemeLoader,
        theme: &Theme,
        dark_mode: bool,
        loaded: bool,
    ) -> bool {
        let new_css = Self::get_css_filename(props, theme);

        if self.theme_css != new_css && self.new_theme_css.is_none() {
            self.new_theme_css = Some(new_css);
            self.loadstate = LoadState::Loading;
            return true;
        }

        set_css_density(theme.density);
        ThemeLoader::set_dark_mode_on_document_root(dark_mode);

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

        let props = ctx.props();

        Self {
            theme_observer,
            loadstate: LoadState::Initial,
            theme_css: Self::get_css_filename(props, &theme),
            new_theme_css: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        let theme = self.theme_observer.theme();
        match msg {
            Msg::Loaded => self.update_theme(props, &theme, self.theme_observer.dark_mode(), true),
            Msg::ThemeChanged((theme, dark_mode)) => {
                self.update_theme(props, &theme, dark_mode, false)
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

impl From<ThemeLoader> for VNode {
    fn from(val: ThemeLoader) -> Self {
        let comp = VComp::new::<PwtThemeLoader>(Rc::new(val), None);
        VNode::from(comp)
    }
}

impl_to_html!(ThemeLoader);
