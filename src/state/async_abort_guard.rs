use futures::future::{abortable, AbortHandle, Future};

/// Abort guard for async functions.
pub struct AsyncAbortGuard {
    abort_handle: AbortHandle,
}

impl AsyncAbortGuard {
    /// Runs a Rust Future on the current thread.
    ///
    /// The future is aborted as soon as the returned guard gets dropped.
    pub fn spawn<F>(future: F) -> Self
    where
        F: Future<Output = ()> + 'static,
    {
        let (future, abort_handle) = abortable(future);

        wasm_bindgen_futures::spawn_local(async move {
            match future.await {
                Ok(()) => { /* do nothing */ }
                Err(futures::future::Aborted) => { /* do nothing (maybe we want to log this?) */ }
            }
        });
        AsyncAbortGuard { abort_handle }
    }
}

impl Drop for AsyncAbortGuard {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}
