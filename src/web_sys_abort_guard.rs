use anyhow::{Context, Error};

/// Abort Guard for web fetch requests
pub struct WebSysAbortGuard {
    controller: web_sys::AbortController,
}

impl WebSysAbortGuard {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            controller: web_sys::AbortController::new()
                .map_err(crate::convert_js_error)
                .context("failed to create AbortController")?,
        })
    }

    pub fn signal(&self) -> web_sys::AbortSignal {
        self.controller.signal()
    }
}

impl Drop for WebSysAbortGuard {
    fn drop(&mut self) {
        self.controller.abort();
    }
}
