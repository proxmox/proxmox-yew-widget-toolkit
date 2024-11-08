use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use derivative::Derivative;
use futures::future::{abortable, AbortHandle};
use std::future::Future;

/// Run one or more futures, and abort them on drop.
///
/// This can be used by components to spawn async functions.
/// If you drop the pool, all functions are aborted, i.e. any
/// spawned http request will be aborted.
#[derive(Derivative)]
#[derivative(Clone(bound = ""), PartialEq(bound = ""))]
pub struct AsyncPool {
    #[derivative(PartialEq(compare_with = "Rc::ptr_eq"))]
    inner: Rc<AsyncPoolInner>,
}

struct AsyncPoolInner {
    id_counter: AtomicUsize,
    abort_handles: Rc<RefCell<HashMap<usize, AbortHandle>>>,
}

impl AsyncPool {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            inner: Rc::new(AsyncPoolInner {
                id_counter: AtomicUsize::new(0),
                abort_handles: Rc::new(RefCell::new(HashMap::new())),
            }),
        }
    }

    /// Runs a Rust Future on the current thread.
    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        let (future, abort_handle) = abortable(future);
        let abort_handles = Rc::clone(&self.inner.abort_handles);
        let abort_id = self.inner.id_counter.fetch_add(1, Ordering::Relaxed);

        abort_handles.borrow_mut().insert(abort_id, abort_handle);

        wasm_bindgen_futures::spawn_local(async move {
            match future.await {
                Ok(()) => {
                    abort_handles.borrow_mut().remove(&abort_id);
                }
                Err(futures::future::Aborted) => {
                    // this is only tiggered from drop, so there is no
                    // need to do anything
                }
            }
        });
    }

    /// This method asynchronously awaits a Future that returns a message and sends it
    /// using `link.send_message`.
    pub fn send_future<COMP, F>(&self, link: yew::html::Scope<COMP>, future: F)
    where
        COMP: yew::Component,
        F: Future<Output = COMP::Message> + 'static,
    {
        self.spawn(async move {
            let msg = future.await;
            link.send_message(msg);
        });
    }
}

// Note: We implement Drop on the Inner type, so this is
// called when we drop the last clone of the AsyncPool.
impl Drop for AsyncPoolInner {
    fn drop(&mut self) {
        for (_, handle) in self.abort_handles.borrow().iter() {
            handle.abort();
        }
    }
}
