use yew::Callback;

use super::{PersistentState, SharedState, SharedStateObserver};

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
