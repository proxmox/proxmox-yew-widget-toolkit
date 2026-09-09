use futures::future::{AbortHandle, Future, abortable};

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
        let (guard, future) = Self::new(future);
        wasm_bindgen_futures::spawn_local(future);
        guard
    }

    /// Wrap a future and return its abort guard without scheduling it.
    pub(crate) fn new<F>(future: F) -> (Self, impl Future<Output = ()>)
    where
        F: Future<Output = ()>,
    {
        let (future, abort_handle) = abortable(future);
        let future = async move {
            let _ = future.await;
        };
        (Self { abort_handle }, future)
    }
}

impl Drop for AsyncAbortGuard {
    fn drop(&mut self) {
        self.abort_handle.abort();
    }
}
