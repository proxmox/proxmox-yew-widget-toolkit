use crate::local_storage;

use anyhow::{bail, Error};

#[derive(PartialEq, Clone, Copy)]
pub enum Theme {
    System,
    Dark,
    Light,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Theme::System => "auto",
            Theme::Dark => "dark",
            Theme::Light => "light",
        })
    }
}

impl TryFrom<&str> for Theme {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            "auto" => Theme::System,
            _ => bail!("'{}' is not a valid theme", value),
        })
    }
}

impl Theme {
    pub fn load() -> Option<Self> {
        local_storage().and_then(|store| {
            let value = store.get_item("Theme").unwrap_or(None);
            match value {
                Some(value) => match Theme::try_from(value.as_str()) {
                    Ok(theme) => Some(theme),
                    Err(_) => None,
                },
                None => None,
            }
        })
    }

    pub fn store(&self) -> Result<(), Error> {
        if let Some(store) = local_storage() {
            if let Err(_) = store.set_item("Theme", &self.to_string()) {
                bail!("store: set_item failed");
            }
        } else {
            bail!("no storage");
        }

        Ok(())
    }

    pub fn get_css_filename(&self) -> &'static str {
        match *self {
            Theme::System => "proxmox-yew-style-auto.css",
            Theme::Dark => "proxmox-yew-style-dark.css",
            Theme::Light => "proxmox-yew-style-light.css",
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}
