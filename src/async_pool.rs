use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use futures::future::{abortable, AbortHandle};
use std::future::Future;

/// Run one or more futures, and abort them on drop.
///
/// This can be used by components to spawn async functions.
/// If you drop the pool, all functions are aborted, i.e. any
/// spawned http request will be aborted.
pub struct AsyncPool {
    id_counter: usize,
    abort_handles: Rc<RefCell<HashMap<usize, AbortHandle>>>,
}

impl AsyncPool {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            id_counter: 0,
            abort_handles: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Runs a Rust Future on the current thread.
    pub fn spawn<F>(&mut self, future: F)
    where
        F: Future<Output = ()> + 'static,
    {
        let (future, abort_handle) = abortable(future);
        let abort_handles = Rc::clone(&self.abort_handles);
        self.id_counter += 1;
        let abort_id = self.id_counter;
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
}

impl Drop for AsyncPool {
    fn drop(&mut self) {
        for (_, handle) in &*self.abort_handles.borrow() {
            handle.abort();
        }
    }
}
