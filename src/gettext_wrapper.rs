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

pub fn init_i18n_from_url(url: &str, on_load: impl IntoEventCallback<()>) {
    let url = url.to_string();
    let on_load = on_load.into_event_callback();
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(err) = fetch_catalog(&url).await {
            log::error!("Catalog load error: {err}");
        } else {
            log::info!("I18N Catalog initialized");
        }
        if let Some(on_load) = &on_load {
            on_load.emit(());
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

    let redraw = use_state(|| 0);

    if *redraw == 0 {
        let redraw = redraw.clone();
        crate::init_i18n_from_url(url, move |_| {
            redraw.set(1); // trigger redraw
        });
    }

    *redraw == 1
}
