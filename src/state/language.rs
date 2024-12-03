use std::sync::OnceLock;
use yew::virtual_dom::Key;
use yew::Callback;

use crate::gettext;
use crate::props::ExtractPrimaryKey;

use super::{PersistentState, SharedState, SharedStateObserver};

#[derive(Clone, PartialEq, Debug)]
pub struct LanguageInfo {
    pub lang: String,            // id (de, en, ...)
    pub text: String,            // Language name (native).
    pub english_text: String,    // English language name.
    pub translated_text: String, // Translated language name.
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
        }
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

    let list = list
        .into_iter()
        .map(|mut info| {
            info.translated_text = gettext(&info.english_text);
            info
        })
        .collect();

    list
}

static mut LANGUAGE: Option<SharedState<PersistentState<String>>> = None;

fn get_state() -> SharedState<PersistentState<String>> {
    unsafe {
        if LANGUAGE.is_none() {
            let state = PersistentState::<String>::new("Language");

            LANGUAGE = Some(SharedState::new(state));
        }
        LANGUAGE.clone().unwrap()
    }
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
        let state = get_state();
        let lang = (***state.read()).to_string();
        lang
    }

    /// Update language.
    pub fn store(lang: impl Into<String>) {
        let lang = lang.into();

        let state = get_state();

        if lang == ***state.read() {
            return; // nothing changed
        }
        state.write().update(lang);
    }
}

/// Listen to language changes.
pub struct LanguageObserver {
    _observer: SharedStateObserver<PersistentState<String>>,
}

impl LanguageObserver {
    pub fn new(on_change: Callback<String>) -> Self {
        let state = get_state();
        let _observer = state.add_listener(move |state: SharedState<PersistentState<String>>| {
            let lang = (***state.read()).clone();
            on_change.emit(lang);
        });
        Self { _observer }
    }
}
