use gettext::Catalog;

use yew::prelude::*;
use yew::html::IntoEventCallback;

static mut CATALOG: Option<Catalog> = None;

pub fn init_i18n(catalog: Catalog) {
    unsafe {
        CATALOG = Some(catalog);
    }
}

pub fn init_i18n_from_blob(blob: Vec<u8>) -> Result<(), String> {
    let catalog = Catalog::parse(&mut &blob[..]).map_err(|err| err.to_string())?;
    unsafe {
        CATALOG = Some(catalog);
    }
    Ok(())
}

pub fn init_i18n_from_url(url: &str, on_load: impl IntoEventCallback<String>) {
    let url = url.to_string();
    let on_load = on_load.into_event_callback();
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(err) = fetch_catalog(&url).await {
            log::error!("Catalog load error: {err}");
        } else {
            log::info!("I18N Catalog initialized");
        }
        if let Some(on_load) = &on_load {
            on_load.emit(url.clone());
        }
    });
}

pub fn gettext(msg_id: &str) -> String {
    let catalog = unsafe {
        match CATALOG.as_ref() {
            None => return msg_id.to_string(),
            Some(catalog) => catalog,
        }
    };
    catalog.gettext(msg_id).to_string()
}

pub fn pgettext(msg_context: &str, msg_id: &str) -> String {
    let catalog = unsafe {
        match CATALOG.as_ref() {
            None => return msg_id.to_string(),
            Some(catalog) => catalog,
        }
    };
    catalog.pgettext(msg_context, msg_id).to_string()
}

pub fn ngettext(msg_id: &str, msg_id_plural: &str, n: u64) -> String {
    let catalog = unsafe {
        match CATALOG.as_ref() {
            None => return msg_id.to_string(),
            Some(catalog) => catalog,
        }
    };
    catalog.ngettext(msg_id, msg_id_plural, n).to_string()
}

pub fn npgettext(msg_context: &str, msg_id: &str, msg_id_plural: &str, n: u64) -> String {
    let catalog = unsafe {
        match CATALOG.as_ref() {
            None => return msg_id.to_string(),
            Some(catalog) => catalog,
        }
    };
    catalog
        .npgettext(msg_context, msg_id, msg_id_plural, n)
        .to_string()
}

fn convert_js_error(js_err: ::wasm_bindgen::JsValue) -> String {
    if let Ok(error) = ::wasm_bindgen::JsCast::dyn_into::<js_sys::Error>(js_err) {
        format!("{}", error.message())
    } else {
        format!("unknown js error: error is no ERROR object")
    }
}

async fn fetch_catalog(url: &str) -> Result<(), String> {
    let mut init = web_sys::RequestInit::new();
    init.method("GET");

    let request =
        web_sys::Request::new_with_str_and_init(url, &init).map_err(|err| convert_js_error(err))?;

    let window = web_sys::window().ok_or_else(|| format!("unable to get window object"))?;
    let promise = window.fetch_with_request(&request);

    let js_resp = wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(|err| convert_js_error(err))?;

    let response: web_sys::Response = js_resp.into();
    let status = response.status();

    if !(status >= 200 && status < 300) {
        return Err(format!(
            "Catalog download failed -g ot HTTP status {}",
            status
        ));
    }
    let promise = response
        .array_buffer()
        .map_err(|err| convert_js_error(err))?;

    let js_fut = wasm_bindgen_futures::JsFuture::from(promise);
    let body = js_fut.await.map_err(|err| convert_js_error(err))?;
    let body = js_sys::Uint8Array::new(&body).to_vec();

    init_i18n_from_blob(body)?;

    Ok(())
}


/// Hook to download a catalog and initialize I18N with functional components.
///
/// This hook returns when the catalog is loaded.
#[hook]
pub fn use_catalog(url: &str) -> bool {

    #[derive(Clone, PartialEq)]
    enum LoadState { Idle, Loading, LoadFinished(String) }

    use std::cell::RefCell;

    thread_local!{
        static LAST_URL: RefCell<String> = RefCell::new(String::new());
    }

    let state = use_state(|| LoadState::Idle);

    match &*state {
        LoadState::Idle => {
            let last_url = LAST_URL.with(|c| c.borrow().clone());
            if &last_url != url {
                state.set(LoadState::Loading);
                let state = state.clone();
                crate::init_i18n_from_url(&url, move |url| {
                    state.set(LoadState::LoadFinished(url));
                });
            }
        }
        LoadState::Loading => { /* wait until loaded */ }
        LoadState::LoadFinished(loaded_url) => {
            let loaded_url = loaded_url.clone();
            LAST_URL.with(move |c| *c.borrow_mut() = loaded_url);
            state.set(LoadState::Idle);
        }
    }

    !matches!(*state, LoadState::LoadFinished(_))
}

/// This is an implementation detail for replacing arguments in the gettext macros.
/// Don't call this directly.
#[allow(dead_code)]
#[doc(hidden)]
pub fn freeformat<T: std::fmt::Display>(msgstr: &str, pat: &str, arg: T) -> String {
    match msgstr.split_once(pat) {
        Some((pre, suf)) => format!("{}{}{}", pre, arg, suf),
        None => {
            debug_assert!(false, "There are more arguments than format directives");
            msgstr.to_string()
        }
    }
}

/// This is an implementation detail for replacing arguments in the gettext macros.
/// Don't call this directly.
#[macro_export]
#[doc(hidden)]
macro_rules! rt_format {
	( $msgstr:expr, $pat:expr, ) => {{
        debug_assert!(
            !$msgstr.contains("{}"),
            "There are fewer arguments than format directives"
        );
		$msgstr
	}};
	( $msgstr:expr, $pat:expr, $arg:expr $(, $rest: expr)* ) => {{
		$crate::rt_format!(
            $crate::gettext_wrapper::freeformat(&$msgstr, $pat, $arg),
            $pat,
            $($rest),*
        )
	}};
}

/// Like [`gettext`], but allows for formatting.
///
/// It calls [`gettext`] on `msgid`, and then replaces each occurrence of `{}` with the next value
/// out of `args`.
///
/// # Panics
///
/// If compiled with debug assertions enabled (as in "dev" profile),
/// will panic if the number of arguments doesn't match the number of format directives.
///
/// [`gettext`]: gettext()
#[macro_export]
macro_rules! tr {
    ( $msgid:expr $(,)? ) => {
        $crate::gettext($msgid)
    };
    ( $msgid:expr, $($args:expr),+ $(,)? ) => {{
        $crate::rt_format!(
            $crate::gettext($msgid),
            "{}",
            $($args),+
        )
    }};
}

/// Like [`ngettext`], but allows for formatting.
///
/// It calls [`ngettext`] on `msgid`, `msgid_plural`, and `n`, and then replaces each occurrence of
/// `{}` with the next value out of `args`, and `{n}` with `n`.
///
/// # Panics
///
/// If compiled with debug assertions enabled (as in "dev" profile),
/// will panic if the number of arguments doesn't match the number of format directives.
#[macro_export]
macro_rules! ngettext {
    ( $msgid:expr, $msgid_plural:expr, $n:expr $(,)? ) => {{
        let mut msgstr = $crate::ngettext($msgid, $msgid_plural, $n);
        while msgstr.contains("{n}") {
            msgstr = $crate::rt_format!(&msgstr, "{n}", $n);
        }
        msgstr
    }};
    ( $msgid:expr, $msgid_plural:expr, $n:expr, $($args:expr),+ $(,)? ) => {{
        let mut msgstr = $crate::ngettext($msgid, $msgid_plural, $n);
        while msgstr.contains("{n}") {
            msgstr = $crate::rt_format!(&msgstr, "{n}", $n);
        }
        $crate::rt_format!(msgstr, "{}", $($args),+)
    }};
}

/// Like [`pgettext`], but allows for formatting.
///
/// It calls [`pgettext`] on `msgctxt` and `msgid`, and then replaces each occurrence of `{}` with
/// the next value out of `args`.
///
/// # Panics
///
/// If compiled with debug assertions enabled (as in "dev" profile),
/// will panic if the number of arguments doesn't match the number of format directives.
#[macro_export]
macro_rules! pgettext {
    ( $msgctxt:expr, $msgid:expr $(,)? ) => {
        $crate::pgettext($msgctxt, $msgid)
    };
    ( $msgctxt:expr, $msgid:expr, $($args:expr),+ $(,)? ) => {{
        $crate::rt_format!(
            $crate::pgettext($msgctxt, $msgid),
            "{}",
            $($args),+
        )
    }};
}

/// Like [`npgettext`], but allows for formatting.
///
/// It calls [`npgettext`] on `msgctxt`, `msgid`, `msgid_plural`, and `n`, and then replaces each
/// occurrence of `{}` with the next value out of `args`, and `{n}` with `n`.
///
/// # Panics
///
/// If compiled with debug assertions enabled (as in "dev" profile),
/// will panic if the number of arguments doesn't match the number of format directives.
#[macro_export]
macro_rules! npgettext {
    ( $msgctxt:expr, $msgid:expr, $msgid_plural:expr, $n:expr $(,)? ) => {{
        let mut msgstr = $crate::npgettext($msgctxt, $msgid, $msgid_plural, $n);
        while msgstr.contains("{n}") {
            msgstr = $crate::rt_format!(&msgstr, "{n}", $n);
        }
        msgstr
    }};
    ( $msgctxt:expr, $msgid:expr, $msgid_plural:expr, $n:expr, $($args:expr),+ $(,)? ) => {{
        let mut msgstr = $crate::npgettext($msgctxt, $msgid, $msgid_plural, $n);
        while msgstr.contains("{n}") {
            msgstr = $crate::rt_format!(&msgstr, "{n}", $n);
        }
        $crate::rt_format!(msgstr, "{}", $($args),+)
    }};
}
