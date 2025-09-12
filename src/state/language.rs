use std::sync::OnceLock;

use yew::virtual_dom::Key;
use yew::Callback;

use crate::gettext;
use crate::props::ExtractPrimaryKey;

use super::{PersistentState, SharedState, SharedStateObserver};

/// The text direction, default is LTR
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum TextDirection {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LanguageInfo {
    pub lang: String,             // id (de, en, ...)
    pub text: String,             // Language name (native).
    pub english_text: String,     // English language name.
    pub translated_text: String,  // Translated language name.
    pub direction: TextDirection, // Text direction of the language
}

impl LanguageInfo {
    pub fn new(
        lang: impl Into<String>,
        text: impl Into<String>,
        english_text: impl Into<String>,
    ) -> LanguageInfo {
        let english_text = english_text.into();
        LanguageInfo {
            lang: lang.into(),
            text: text.into(),
            translated_text: gettext(&english_text),
            english_text,
            direction: Default::default(),
        }
    }

    /// Builder style method to set the text direction
    pub fn direction(mut self, direction: TextDirection) -> Self {
        self.direction = direction;
        self
    }
}

impl ExtractPrimaryKey for LanguageInfo {
    fn extract_key(&self) -> yew::virtual_dom::Key {
        Key::from(self.lang.clone())
    }
}

static AVAILABLE_LANGUAGES: OnceLock<Vec<LanguageInfo>> = OnceLock::new();

pub fn set_available_languages(list: Vec<LanguageInfo>) {
    AVAILABLE_LANGUAGES
        .set(list)
        .expect("cannot set language info twice")
}

pub fn get_available_languages() -> Vec<LanguageInfo> {
    let list = AVAILABLE_LANGUAGES
        .get()
        .cloned()
        .expect("cannot access available languages before they've been set");

    list.into_iter()
        .map(|mut info| {
            info.translated_text = gettext(&info.english_text);
            info
        })
        .collect()
}

/// Get [`LanguageInfo`] by its short name.
pub fn get_language_info(lang: &str) -> Option<LanguageInfo> {
    get_available_languages()
        .into_iter()
        .find(|info| info.lang == lang)
}

// this `thread_local!` definition should be fine as this crate is essentially WASM only where
// besides web workers and similar ways to spawn futures, there is only one thread. if this
// assumption changes, this will need to be adapted.
thread_local! {
    static LANGUAGE: SharedState<PersistentState<String>> = SharedState::new(PersistentState::new("Language"));
}

/// Persistent store for language setting.
///
/// This struct implements methods to load and store the current language
/// settings (ISO 639-1 language code) in the local browser store.
///
/// Please use [LanguageObserver] to track changes.
pub struct Language;

impl Language {
    /// Load current language.
    pub fn load() -> String {
        LANGUAGE.with(|s| s.read().to_string())
    }

    /// Update language.
    pub fn store(lang: impl Into<String>) {
        let lang = lang.into();

        if LANGUAGE.with(|s| ***s.read() == lang) {
            return; // nothing changed
        }

        LANGUAGE.with(|s| s.write().update(lang));
    }
}

/// Listen to language changes.
pub struct LanguageObserver {
    _observer: SharedStateObserver<PersistentState<String>>,
}

impl LanguageObserver {
    pub fn new(on_change: Callback<String>) -> Self {
        let _observer = LANGUAGE.with(|state| {
            state.add_listener(move |state: SharedState<PersistentState<String>>| {
                let lang = (***state.read()).clone();
                on_change.emit(lang);
            })
        });
        Self { _observer }
    }
}
