use std::rc::Rc;
use std::cell::RefCell;

use anyhow::{bail, Error};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::MediaQueryList;

use yew::prelude::*;

use crate::state::local_storage;


/// Theme mode - dark, light or auto (use system settings).
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ThemeMode {
    System,
    Dark,
    Light,
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ThemeMode::System => "auto",
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
        })
    }
}

impl TryFrom<&str> for ThemeMode {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            "auto" => ThemeMode::System,
            _ => bail!("'{}' is not a valid theme", value),
        })
    }
}

static mut DEFAULT_THEME_NAME: &'static str = "Material";

pub fn set_default_theme_name(name: &'static str) {
    unsafe {
        DEFAULT_THEME_NAME = name;
    }
}

pub fn get_default_theme_name() -> &'static str {
    unsafe {
        DEFAULT_THEME_NAME
    }
}

/// Theme. Combines a theme name with a theme mode ([ThemeMode])
///
/// This struct implements methods to load and store the current theme
/// settings in the local browser store. The theme name and theme mode
/// can be stored and changed separately, and we emit a custom
/// [web_sys::Event] called `pwt-theme-changed`. So it is possible to
/// observe changes by adding an event listener to the document.
#[derive(PartialEq, Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
    pub name: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            mode: ThemeMode::System,
            name: String::from(get_default_theme_name()),
        }
    }
}

fn emit_theme_changed_event() {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            let event = web_sys::Event::new("pwt-theme-changed").unwrap();
            let _ = document.dispatch_event(&event);
        }
    }
}

impl Theme {
    /// Load the current theme settings.
    pub fn load() -> Self {
        let mut theme = Theme::default();
        let store = match local_storage() {
            Some(store) => store,
            None => return theme,
        };

        if let Ok(Some(mode)) = store.get_item("ThemeMode") {
            if let Ok(mode) = ThemeMode::try_from(mode.as_str()) {
                theme.mode = mode;
            }
        }

        if let Ok(Some(name)) = store.get_item("ThemeName") {
            theme.name = name;
        }

        theme
    }

    /// Load theme, but resctict possible values.
    ///
    /// If the loaded value isn't in the list, we simply return the first
    ///  value from the list.
    pub fn load_filtered(themes: &[&str]) -> Self {
        let mut theme = Self::load();

        let name = &theme.name;
        if themes.iter().find(|t| *t == name).is_some() {
            return theme;
        }

        theme.name = themes.get(0).unwrap_or(&get_default_theme_name()).to_string();

        theme
    }

    /// Store the theme mode and emit the `pwt-theme-changed` event.
    pub fn store_theme_mode(mode: ThemeMode) -> Result<(), Error> {
        if let Some(store) = local_storage() {
            if let Err(_) = store.set_item("ThemeMode", &mode.to_string()) {
                bail!("store: set_item failed");
            }
        } else {
            bail!("no storage");
        }

        emit_theme_changed_event();

        Ok(())
    }

    /// Store the theme name and emit the `pwt-theme-changed` event.
    pub fn store_theme_name(name: &str) -> Result<(), Error> {
        if let Some(store) = local_storage() {
            if let Err(_) = store.set_item("ThemeName", name) {
                bail!("store: set_item failed");
            }
        } else {
            bail!("no storage");
        }

        emit_theme_changed_event();

        Ok(())
    }

    /// Generate a CSS file name: `{name}-yew-style-{mode}.css`
    pub fn get_css_filename(&self, prefer_dark_mode: bool) -> String {
        let mode_str = match self.mode {
            ThemeMode::System => match prefer_dark_mode {
                true => "dark",
                false => "light",
            },
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
        };

        format!("{}-yew-style-{}.css", self.name.to_lowercase(), mode_str)
    }
}

fn get_system_prefer_dark_mode() -> bool {
    let window = web_sys::window().unwrap();
    if let Ok(Some(list)) = window.match_media("(prefers-color-scheme: dark)") {
        list.matches()
    } else {
        false
    }
}

fn use_dark_mode(theme: &Theme, system_prefer_dark_mode: bool) -> bool {
    match theme.mode {
        ThemeMode::System => system_prefer_dark_mode,
        ThemeMode::Dark => true,
        ThemeMode::Light => false,
    }
}

/// Listen to theme changes.
///
/// This helper listens to the `pwt-theme-changed` event, and uses a media
/// query to get notified when `prefers-color-scheme` changes.
pub struct ThemeObserver {
    themes: &'static [&'static str],
    media_query: MediaQueryList,
    scheme_changed_closure: Option<Closure<dyn Fn()>>,
    theme_changed_closure: Option<Closure<dyn Fn()>>,
    on_theme_change: Callback<(Theme, bool)>,
    state: Rc<RefCell<(Theme, bool)>>,
}

impl Drop for ThemeObserver {
    fn drop(&mut self) {
        self.remove_theme_changed_listener();
        self.remove_prefers_color_scheme_listener();
    }
}

impl ThemeObserver {

    /// Creates a new listener.
    pub fn new(themes: &'static [&'static str], on_theme_change: Callback<(Theme, bool)>) -> Self {
        let theme = Theme::load_filtered(themes);
        let system_prefer_dark = get_system_prefer_dark_mode();

        let window = web_sys::window().unwrap();
        let media_query = match window.match_media("(prefers-color-scheme: dark)") {
            Ok(Some(media_query)) => media_query,
            _ => panic!("window.match_media() failed!"),
        };

        let use_dark_mode = use_dark_mode(&theme, system_prefer_dark);
        on_theme_change.emit((theme.clone(), use_dark_mode));

        let mut me = Self {
            themes,
            media_query,
            on_theme_change,
            scheme_changed_closure: None,
            theme_changed_closure: None,
            state: Rc::new(RefCell::new((theme, use_dark_mode))),
        };

        me.add_theme_changed_listener();
        me.add_prefers_color_scheme_listener();

        me
    }

    /// Return the current [Theme],
    pub fn theme(&self) -> Theme {
        self.state.borrow().0.clone()
    }

    /// Returns dark mode settings.
    pub fn dark_mode(&self) -> bool {
        self.state.borrow().1
    }

    fn add_theme_changed_listener(&mut self) {
        let theme_changed_closure = Closure::wrap({
            let on_theme_change = self.on_theme_change.clone();
            let state = self.state.clone();
            let themes = self.themes;
            Box::new(move || {
                let theme = Theme::load_filtered(themes);
                let system_prefer_dark = get_system_prefer_dark_mode();
                let use_dark_mode = use_dark_mode(&theme, system_prefer_dark);
                *state.borrow_mut() = (theme.clone(), use_dark_mode);
                on_theme_change.emit((theme, use_dark_mode));
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

    fn add_prefers_color_scheme_listener(&mut self) {
        let scheme_changed_closure = Closure::wrap({
            let on_theme_change = self.on_theme_change.clone();
            let state = self.state.clone();
            let themes = self.themes;
            Box::new(move || {
                let theme = Theme::load_filtered(themes);
                let system_prefer_dark = get_system_prefer_dark_mode();
                let use_dark_mode = use_dark_mode(&theme, system_prefer_dark);
                *state.borrow_mut() = (theme.clone(), use_dark_mode);
                on_theme_change.emit((theme, use_dark_mode));
            }) as Box<dyn Fn()>
        });

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
