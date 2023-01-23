use crate::state::local_storage;

use anyhow::{bail, Error};

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

impl Default for Theme {
    fn default() -> Self {
        Self {
            mode: ThemeMode::System,
            name: String::from("Proxmox"),
        }
    }
}
