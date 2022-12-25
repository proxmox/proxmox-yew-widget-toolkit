use std::rc::Rc;
use std::future::Future;
use std::pin::Pin;
use std::cell::RefCell;

use anyhow::{bail, format_err, Error};
use serde::de::DeserializeOwned;
use serde_json::Value;

use yew::AttrValue;
use yew::html::IntoPropValue;

/// Load Callback
///
/// There is basically no way to implement [PartialEq] for
/// closures, so we simply use an [Rc] to store the callback and
/// [Rc::ptr_eq] to implement [PartialEq].
///
/// Always store created callbacks inside the component state to avoid
/// unnecessary property change triggers (especially if a compoment
/// automatically triggers reload, because that would reload
/// frequently).
///
/// As workaround, set the "url" property. If set, only that url is
/// compared for [PartialEq]. That way you get a "trackable"
/// [LoadCallback]. You can use that to trigger atomatic reloads on
/// change.

pub struct LoadCallback<T> {
    callback: Rc<dyn Fn() -> Pin<Box<dyn Future<Output=Result<T, Error>>>>>,
    url: Option<AttrValue>, // only used for change tracking
}

impl<T> LoadCallback<T> {

    pub fn new<F, R>(callback: F) -> Self
    where
        F: 'static + Fn() -> R,
        R: 'static + Future<Output = Result<T, Error>>,
    {
        Self {
            url: None,
            callback: Rc::new(move || {
                let future = callback();
                Box::pin(future)
            }),
        }
    }

    pub fn url(mut self, url: impl IntoPropValue<Option<AttrValue>>) -> Self {
        self.set_url(url);
        self
    }

    pub fn set_url(&mut self, url: impl IntoPropValue<Option<AttrValue>>) {
        self.url = url.into_prop_value();
    }

    /// Mark the callback as static (disable change detection).
    ///
    /// Useful for callback which always returns the same data.
    pub fn static_callback(self) -> Self {
        // Simply set a fixed url
        self.url("__static__")
    }

    pub async fn apply(&self) -> Result<T, Error> {
        (self.callback)().await
    }
}

impl<T> Clone for LoadCallback<T> {
    fn clone(&self) -> Self {
        Self { callback: Rc::clone(&self.callback), url: self.url.clone() }
    }
}

impl<T> PartialEq for LoadCallback<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.url.is_some() && other.url.is_some() {
            return self.url == other.url;
        }

        Rc::ptr_eq(&self.callback, &other.callback)
    }
}

/// Helper trait to create optional [LoadCallback] properties.
pub trait IntoLoadCallback<T> {
    fn into_load_callback(self) -> Option<LoadCallback<T>>;
}

impl<T> IntoLoadCallback<T> for LoadCallback<T> {
    fn into_load_callback(self) -> Option<LoadCallback<T>> {
        Some(self)
    }
}

impl<T> IntoLoadCallback<T> for Option<LoadCallback<T>> {
    fn into_load_callback(self) -> Option<LoadCallback<T>> {
        self
    }
}

impl<T, F, R> IntoLoadCallback<T> for F
where
    F: 'static + Fn() -> R,
    R: 'static + Future<Output = Result<T, Error>>
{
    fn into_load_callback(self) -> Option<LoadCallback<T>> {
        Some(LoadCallback::new(self))
    }
}

impl<T: 'static + DeserializeOwned> IntoLoadCallback<T> for &str {
    fn into_load_callback(self) -> Option<LoadCallback<T>> {
        let url = self.to_owned();
        url.into_load_callback()
    }
}

impl<T: 'static + DeserializeOwned> IntoLoadCallback<T> for String {
    fn into_load_callback(self) -> Option<LoadCallback<T>> {
        let url = AttrValue::from(self);
        Some(
            LoadCallback::new({
                let url = url.clone();
                move || {
                    let url = url.clone();
                    let http_get = HTTP_GET.with(|cell| Rc::clone(&cell.borrow()));
                    async move {
                        let value = http_get(url.to_string()).await?;
                        let data = serde_json::from_value(value)?;
                        Ok(data)
                    }
                }
            }).url(url)
        )
    }
}

impl<T: 'static + DeserializeOwned> IntoLoadCallback<T> for Option<String> {
    fn into_load_callback(self) -> Option<LoadCallback<T>> {
        if let Some(url) = self {
            url.into_load_callback()
        } else {
            None
        }
    }
}


impl<T: 'static, F, R, P> IntoLoadCallback<T> for (F, P)
where
    P: Into<AttrValue>,
    F: 'static + Fn(AttrValue) -> R,
    R: 'static + Future<Output = Result<T, Error>>,
{
    fn into_load_callback(self) -> Option<LoadCallback<T>> {
        let (callback, url) = (self.0, self.1.into());
        let callback = {
            let url = url.clone();
            move || callback(url.clone())
        };
        Some(LoadCallback::new(callback).url(url))
    }
}

thread_local!{
    static HTTP_GET: RefCell<Rc<dyn Send + Sync + Fn(String) -> Pin<Box<dyn Future<Output = Result<Value, Error>>>> >> = {
        RefCell::new(Rc::new(|url| Box::pin(http_get(url))))
    };
}

/// Overwrite the HTTP get method used by the [LoadCallback]
///
/// The default method expects a valid json response and simply
/// deserializes the data using serde.
pub fn set_http_get_method<F: 'static +  Future<Output = Result<Value, Error>>>(cb: fn(String) -> F) {
    HTTP_GET.with(|cell| *cell.borrow_mut() = Rc::new(move |url| Box::pin(cb(url))));
}

async fn http_get(url: String) -> Result<Value, Error> {

    let mut init = web_sys::RequestInit::new();
    init.method("GET");

    let js_headers = web_sys::Headers::new()
        .map_err(|err| format_err!("{:?}", err))?;

    js_headers.append("content-type", "application/x-www-form-urlencoded")
        .map_err(|err| format_err!("{:?}", err))?;

    init.headers(&js_headers);
    let js_req = web_sys::Request::new_with_str_and_init(&url, &init)
        .map_err(|err| format_err!("{:?}", err))?;

    let window = web_sys::window()
        .ok_or_else(|| format_err!("unable to get window object"))?;

    let promise = window.fetch_with_request(&js_req);
    let js_fut =  wasm_bindgen_futures::JsFuture::from(promise);
    let js_resp = js_fut.await
        .map_err(|err| format_err!("{:?}", err))?;

    let resp: web_sys::Response = js_resp.into();

    let promise = resp.text()
        .map_err(|err| format_err!("{:?}", err))?;

    let js_fut =  wasm_bindgen_futures::JsFuture::from(promise);
    let body = js_fut.await
        .map_err(|err| format_err!("{:?}", err))?;

    let text = body.as_string()
        .ok_or_else(|| format_err!("Got non-utf8-string response"))?;

    let data = if resp.ok() {
        if text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text)
                .map_err(|err| format_err!("invalid json: {}", err))?
        }
    } else {
        bail!("HTTP status {}: {}", resp.status(), resp.status_text());
    };

    Ok(data)
}
