use gettext::Catalog;

use yew::html::IntoEventCallback;

static mut CATALOG: Option<Catalog> = None;

/// Intitialize the global translation catalog.
pub fn init_i18n(catalog: Catalog) {
    unsafe {
        CATALOG = Some(catalog);
    }
}

/// Intitialize the global translation catalog, using a binary blob.
pub fn init_i18n_from_blob(blob: Vec<u8>) -> Result<(), String> {
    let catalog = Catalog::parse(&mut &blob[..]).map_err(|err| err.to_string())?;
    init_i18n(catalog);
    Ok(())
}

/// Intitialize the global translation catalog by downloading data from url.
pub fn init_i18n_from_url(url: &str, on_load: impl IntoEventCallback<String>) {
    let url = url.to_string();
    let on_load = on_load.into_event_callback();
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(err) = fetch_catalog(&url).await {
            log::error!("Catalog load error: {err}");
            init_i18n(Catalog::empty());
        } else {
            log::info!("I18N Catalog initialized");
        }
        if let Some(on_load) = &on_load {
            on_load.emit(url.clone());
        }
    });
}

/// Translate text string using global translation catalog.
///
/// Please use [gettext!](crate::gettext!) to format text with arguments.
pub fn gettext(msg_id: &str) -> String {
    let catalog = unsafe {
        match CATALOG.as_ref() {
            None => return msg_id.to_string(),
            Some(catalog) => catalog,
        }
    };
    catalog.gettext(msg_id).to_string()
}

/// Mark a string as translatable, but do not actually translate.
pub fn gettext_noop(msg_id: &str) -> &str {
    msg_id
}

/// Translate text string using global translation catalog (with context).
///
/// Please use [pgettext!](crate::pgettext!) to format text with arguments.
pub fn pgettext(msg_context: &str, msg_id: &str) -> String {
    let catalog = unsafe {
        match CATALOG.as_ref() {
            None => return msg_id.to_string(),
            Some(catalog) => catalog,
        }
    };
    catalog.pgettext(msg_context, msg_id).to_string()
}

/// Translate text string using global translation catalog (singular/plural).
///
/// Please use [ngettext!](crate::ngettext!) to format text with arguments.
pub fn ngettext(msg_id: &str, msg_id_plural: &str, n: u64) -> String {
    let catalog = unsafe {
        match CATALOG.as_ref() {
            None => {
                return if n == 1 {
                    msg_id.to_string()
                } else {
                    msg_id_plural.to_string()
                }
            }
            Some(catalog) => catalog,
        }
    };
    catalog.ngettext(msg_id, msg_id_plural, n).to_string()
}

/// Translate text string using global translation catalog (singular/plural with context).
///
/// Please use [npgettext!](crate::npgettext!) to format text with arguments.
pub fn npgettext(msg_context: &str, msg_id: &str, msg_id_plural: &str, n: u64) -> String {
    let catalog = unsafe {
        match CATALOG.as_ref() {
            None => {
                return if n == 1 {
                    msg_id.to_string()
                } else {
                    msg_id_plural.to_string()
                }
            }
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
    let abort = crate::props::WebSysAbortGuard::new()
        .map_err(|err| format!("unable to create abort guard: {err}"))?;

    let mut init = web_sys::RequestInit::new();
    init.method("GET");
    init.signal(Some(&abort.signal()));

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gettext_fns() {
        assert_eq!(gettext("foo bar"), "foo bar");

        assert_eq!(ngettext("one", "plural", 0), "plural");
        assert_eq!(ngettext("one", "plural", 1), "one");
        assert_eq!(ngettext("one", "plural", 2), "plural");

        assert_eq!(pgettext("context", "foo bar"), "foo bar");

        assert_eq!(npgettext("context", "one", "plural", 0), "plural");
        assert_eq!(npgettext("context", "one", "plural", 1), "one");
        assert_eq!(npgettext("context", "one", "plural", 2), "plural");
    }
}
