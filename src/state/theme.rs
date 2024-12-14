use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{bail, Error};

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::MediaQueryList;

use yew::prelude::*;

use crate::dom::get_system_prefer_dark_mode;
use crate::state::local_storage;

/// Theme mode - dark, light or auto (use system settings).
#[derive(PartialEq, Debug, Default, Clone, Copy)]
pub enum ThemeMode {
    #[default]
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
            _ => bail!("'{}' is not a valid theme mode", value),
        })
    }
}

/// Theme density
#[derive(PartialEq, Debug, Default, Clone, Copy)]
pub enum ThemeDensity {
    /// Use default spacing and font size from theme.
    #[default]
    Preset,
    /// High density theme with narrow spacing and smaller font.
    Compact,
    /// Medium spacing, suitable for desktop application.
    Medium,
    /// Large, relaxed spacing and higher font size, suitable for high DPI and touch devices.
    Relaxed,
}

impl std::fmt::Display for ThemeDensity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ThemeDensity::Compact => "Compact",
            ThemeDensity::Medium => "Medium",
            ThemeDensity::Relaxed => "Relaxed",
            ThemeDensity::Preset => "Preset",
        })
    }
}

impl TryFrom<&str> for ThemeDensity {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "Compact" => ThemeDensity::Compact,
            "Medium" => ThemeDensity::Medium,
            "Relaxed" => ThemeDensity::Relaxed,
            // NOTE: the three matches (High, Touch, Auto) below are for migrating due to a rename
            // of the options. They could be removed with some future (stable) release.
            "High" => ThemeDensity::Compact,
            "Touch" => ThemeDensity::Relaxed,
            "Auto" => ThemeDensity::Preset,
            "" | "Preset" => ThemeDensity::Preset,
            _ => bail!("'{value}' is not a valid theme density"),
        })
    }
}

static mut AVAILABLE_THEMES: &[&str] = &["Material"];

pub fn set_available_themes(themes: &'static [&'static str]) {
    unsafe {
        AVAILABLE_THEMES = themes;
    }
}

pub fn get_available_themes() -> &'static [&'static str] {
    unsafe { AVAILABLE_THEMES }
}

fn get_default_theme_name() -> String {
    get_available_themes()
        .get(0)
        .unwrap_or(&"Material")
        .to_string()
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
    pub density: ThemeDensity,
    pub name: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            mode: ThemeMode::default(),
            density: ThemeDensity::Preset, // use default from css
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
    // Load the current theme settings from local storage.
    fn load_from_storage() -> Self {
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

        if let Ok(Some(density)) = store.get_item("ThemeDensity") {
            if let Ok(density) = ThemeDensity::try_from(density.as_str()) {
                theme.density = density;
            }
        }

        if let Ok(Some(name)) = store.get_item("ThemeName") {
            theme.name = name;
        }

        theme
    }

    /// Load theme.
    ///
    /// Theme names are restricted by the list of available themes (see [set_available_themes]).
    /// If the loaded value isn't in the list, we simply return the first
    /// value from the list.
    ///
    /// # Note
    ///
    /// Theme name comparison is case-insensitive.
    pub fn load() -> Self {
        let mut theme = Self::load_from_storage();

        let name = theme.name.to_lowercase();
        let themes = get_available_themes();

        if themes.iter().find(|t| t.to_lowercase() == name).is_some() {
            return theme;
        }

        theme.name = get_default_theme_name();

        theme
    }

    /// Store the theme mode and emit the `pwt-theme-changed` event.
    pub fn store_theme_mode(mode: ThemeMode) -> Result<(), Error> {
        if let Some(store) = local_storage() {
            if let Err(_) = store.set_item("ThemeMode", &mode.to_string()) {
                bail!("store_them_mode: set_item failed");
            }
        } else {
            bail!("no storage");
        }

        emit_theme_changed_event();

        Ok(())
    }

    /// Store the theme density and emit the `pwt-theme-changed` event.
    pub fn store_theme_density(density: ThemeDensity) -> Result<(), Error> {
        if let Some(store) = local_storage() {
            if let Err(_) = store.set_item("ThemeDensity", &density.to_string()) {
                bail!("store_theme_density: set_item failed");
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
                bail!("store_theme_name: set_item failed");
            }
        } else {
            bail!("no storage");
        }

        emit_theme_changed_event();

        Ok(())
    }

    /// Generate a CSS file name: `{name}-yew-style-{mode}.css`
    pub fn get_css_filename(&self, prefix: &str) -> String {
        format!("{}{}-yew-style.css", prefix, self.name.to_lowercase(),)
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
    pub fn new(on_theme_change: Callback<(Theme, bool)>) -> Self {
        let theme = Theme::load();
        let system_prefer_dark = get_system_prefer_dark_mode();

        let window = web_sys::window().unwrap();
        let media_query = match window.match_media("(prefers-color-scheme: dark)") {
            Ok(Some(media_query)) => media_query,
            _ => panic!("window.match_media() failed!"),
        };

        let use_dark_mode = use_dark_mode(&theme, system_prefer_dark);
        on_theme_change.emit((theme.clone(), use_dark_mode));

        let mut me = Self {
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
            Box::new(move || {
                let theme = Theme::load();
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
            Box::new(move || {
                let theme = Theme::load();
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
