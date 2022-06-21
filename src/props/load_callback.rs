use std::rc::Rc;
use std::future::Future;
use std::pin::Pin;

use anyhow::Error;
use serde::de::DeserializeOwned;

use yew::AttrValue;
use yew::html::IntoPropValue;

/// Load Callback
///
/// Note: There is basically no way to implement PartialEq for
/// closures, so we simply use Rc::ptr_eq(). So do not rely on
/// PartialEq to trigger reloads, because that would reload
/// frequently.
///
/// As workaround, set the "url" property. If set, that url is used to
/// for PartialEq. That way you get a "trackable" [LoadCallback]. You
/// can use that to trigger atomatic reloads on change.

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

/*
impl<T: 'static + DeserializeOwned> IntoLoadCallback<T> for String {
    fn into_load_callback(self) -> Option<LoadCallback<T>> {
        let url = AttrValue::from(self);
        Some(
            LoadCallback::new({
                let url = url.clone();
                move || {
                    let url = url.clone();
                    async move {
                        crate::http_get(&*url, None).await
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

*/

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
