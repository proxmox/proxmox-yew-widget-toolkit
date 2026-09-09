use std::cell::RefCell;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::channel::oneshot;
use futures::task::noop_waker;

use super::*;

type Task = Pin<Box<dyn Future<Output = ()>>>;
type Snapshot = (bool, Option<Result<u32, String>>);
type Sender = oneshot::Sender<Result<u32, Error>>;
type Requests = Rc<RefCell<VecDeque<Sender>>>;

fn poll(task: &mut Task) -> Poll<()> {
    let waker = noop_waker();
    task.as_mut().poll(&mut Context::from_waker(&waker))
}

fn configure(loader: &mut Loader<u32>) -> Requests {
    let requests = Rc::new(RefCell::new(VecDeque::new()));
    loader.set_loader({
        let requests = Rc::clone(&requests);
        move || {
            let (sender, receiver) = oneshot::channel();
            requests.borrow_mut().push_back(sender);
            async move { receiver.await.unwrap() }
        }
    });
    requests
}

fn start(loader: &Loader<u32>) -> Task {
    Box::pin(loader.start_load().unwrap())
}

fn pending_request(loader: &Loader<u32>, requests: &Requests) -> (Sender, Task) {
    let mut task = start(loader);
    assert!(poll(&mut task).is_pending());
    let sender = requests.borrow_mut().pop_front().unwrap();
    (sender, task)
}

fn snapshot(loader: &Loader<u32>) -> Snapshot {
    let data = loader.read().data.as_ref().map(|result| match result {
        Ok(data) => Ok(**data),
        Err(err) => Err(err.to_string()),
    });
    (loader.loading(), data)
}

fn record(events: &Rc<RefCell<Vec<Snapshot>>>) -> Callback<Loader<u32>> {
    let events = Rc::clone(events);
    Callback::from(move |loader| events.borrow_mut().push(snapshot(&loader)))
}

#[test]
fn missing_or_cleared_callback_does_not_replace_request() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut loader = Loader::<u32>::new().on_change(record(&events));
    assert!(loader.start_load().is_none());
    assert_eq!(snapshot(&loader), (false, None));
    assert!(events.borrow().is_empty());

    let requests = configure(&mut loader);
    let mut task = start(&loader);
    loader.set_loader(None::<LoadCallback<u32>>);
    assert!(loader.start_load().is_none());
    assert_eq!(snapshot(&loader), (true, None));
    assert!(events.borrow().is_empty());
    assert!(poll(&mut task).is_pending());
    let sender = requests.borrow_mut().pop_front().unwrap();
    sender.send(Ok(1)).unwrap();
    assert!(poll(&mut task).is_ready());
    assert_eq!(snapshot(&loader), (false, Some(Ok(1))));
    assert_eq!(*events.borrow(), vec![(false, Some(Ok(1)))]);
}

#[test]
fn sequential_loads_reuse_callback_and_notify_only_on_completion() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut loader = Loader::new().on_change(record(&events));
    let requests = configure(&mut loader);
    let mut expected = Vec::new();
    for value in 1..=3 {
        let (sender, mut task) = pending_request(&loader, &requests);
        assert_eq!(
            snapshot(&loader),
            (true, (value > 1).then_some(Ok(value - 1)))
        );
        assert_eq!(*events.borrow(), expected);
        sender.send(Ok(value)).unwrap();
        assert!(poll(&mut task).is_ready());
        assert_eq!(snapshot(&loader), (false, Some(Ok(value))));
        assert!(loader.has_valid_data());
        expected.push((false, Some(Ok(value))));
        assert_eq!(*events.borrow(), expected);
    }
}

#[test]
fn replacement_retires_old_request_in_either_poll_order() {
    for retire_old_first in [true, false] {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut loader = Loader::new().on_change(record(&events));
        let requests = configure(&mut loader);
        let (old_sender, mut old_task) = pending_request(&loader, &requests);
        let (sender, mut task) = pending_request(&loader, &requests);
        assert_eq!(snapshot(&loader), (true, None));
        assert!(events.borrow().is_empty());

        let old_sender = if retire_old_first {
            // A result can become ready after cancellation but before the executor observes it.
            old_sender.send(Ok(1)).unwrap();
            assert!(poll(&mut old_task).is_ready());
            assert_eq!(snapshot(&loader), (true, None));
            assert!(events.borrow().is_empty());
            None
        } else {
            Some(old_sender)
        };

        sender.send(Ok(2)).unwrap();
        assert!(poll(&mut task).is_ready());
        assert_eq!(snapshot(&loader), (false, Some(Ok(2))));
        assert_eq!(*events.borrow(), vec![(false, Some(Ok(2)))]);
        if let Some(old_sender) = old_sender {
            old_sender.send(Err(Error::msg("superseded"))).unwrap();
            assert!(poll(&mut old_task).is_ready());
            assert_eq!(snapshot(&loader), (false, Some(Ok(2))));
            assert_eq!(*events.borrow(), vec![(false, Some(Ok(2)))]);
        }
    }
}

#[test]
fn queued_replacements_invoke_only_current_callback() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut loader = Loader::new().on_change(record(&events));
    let requests = configure(&mut loader);
    let mut tasks: Vec<_> = (0..3).map(|_| start(&loader)).collect();
    assert_eq!(snapshot(&loader), (true, None));
    assert!(requests.borrow().is_empty());
    assert!(events.borrow().is_empty());

    let mut task = tasks.pop().unwrap();
    assert!(poll(&mut task).is_pending());
    assert_eq!(requests.borrow().len(), 1);
    let sender = requests.borrow_mut().pop_front().unwrap();
    sender.send(Ok(3)).unwrap();
    assert!(poll(&mut task).is_ready());
    for mut task in tasks {
        assert!(poll(&mut task).is_ready());
    }
    assert!(requests.borrow().is_empty());
    assert_eq!(snapshot(&loader), (false, Some(Ok(3))));
    assert_eq!(*events.borrow(), vec![(false, Some(Ok(3)))]);
}

#[test]
fn abort_retires_queued_and_pending_requests_without_changing_data() {
    for poll_first in [false, true] {
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut loader = Loader::new();
        loader.write().data = Some(Ok(Rc::new(7)));
        let requests = configure(&mut loader);
        let _observer = loader.add_listener(record(&events));
        let mut task = start(&loader);
        assert_eq!(snapshot(&loader), (true, Some(Ok(7))));
        assert!(requests.borrow().is_empty());
        let sender = if poll_first {
            assert!(poll(&mut task).is_pending());
            Some(requests.borrow_mut().pop_front().unwrap())
        } else {
            None
        };
        loader.abort();
        assert_eq!(snapshot(&loader), (false, Some(Ok(7))));
        assert_eq!(*events.borrow(), vec![(false, Some(Ok(7)))]);
        assert!(poll(&mut task).is_ready());
        if let Some(sender) = sender {
            assert!(sender.is_canceled());
        }
        assert!(requests.borrow().is_empty());

        loader.abort();
        assert_eq!(*events.borrow(), vec![(false, Some(Ok(7))); 2]);
    }
}

#[test]
fn load_after_abort_and_failure_recovers() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut loader = Loader::new().on_change(record(&events));
    let requests = configure(&mut loader);
    let mut canceled = start(&loader);
    loader.abort();
    assert!(poll(&mut canceled).is_ready());
    assert!(requests.borrow().is_empty());
    assert_eq!(*events.borrow(), vec![(false, None)]);

    let (sender, mut task) = pending_request(&loader, &requests);
    sender.send(Err(Error::msg("failure"))).unwrap();
    assert!(poll(&mut task).is_ready());
    assert_eq!(snapshot(&loader), (false, Some(Err("failure".into()))));
    assert!(!loader.has_valid_data());
    let mut expected = vec![(false, None), (false, Some(Err("failure".into())))];
    assert_eq!(*events.borrow(), expected);

    let (sender, mut task) = pending_request(&loader, &requests);
    assert_eq!(snapshot(&loader), (true, Some(Err("failure".into()))));
    assert_eq!(*events.borrow(), expected);
    sender.send(Ok(3)).unwrap();
    assert!(poll(&mut task).is_ready());
    assert_eq!(snapshot(&loader), (false, Some(Ok(3))));
    assert!(loader.has_valid_data());
    expected.push((false, Some(Ok(3))));
    assert_eq!(*events.borrow(), expected);
}

#[test]
fn failed_refresh_replaces_previous_data_with_error() {
    let mut loader = Loader::new();
    let requests = configure(&mut loader);
    let (sender, mut task) = pending_request(&loader, &requests);
    sender.send(Ok(1)).unwrap();
    assert!(poll(&mut task).is_ready());

    let (sender, mut task) = pending_request(&loader, &requests);
    assert_eq!(snapshot(&loader), (true, Some(Ok(1))));
    sender.send(Err(Error::msg("failure"))).unwrap();
    assert!(poll(&mut task).is_ready());
    assert_eq!(snapshot(&loader), (false, Some(Err("failure".into()))));
    assert!(!loader.has_valid_data());
}

#[test]
fn dropping_last_owner_cancels_queued_and_pending_requests() {
    for (with_on_change, with_observer) in
        [(false, false), (false, true), (true, false), (true, true)]
    {
        for poll_first in [false, true] {
            let events = Rc::new(RefCell::new(Vec::new()));
            let mut loader = Loader::new();
            if with_on_change {
                loader = loader.on_change(record(&events));
            }
            let observer = with_observer.then(|| loader.add_listener(record(&events)));
            let requests = configure(&mut loader);
            let mut task = start(&loader);
            assert!(requests.borrow().is_empty());
            let sender = if poll_first {
                assert!(poll(&mut task).is_pending());
                Some(requests.borrow_mut().pop_front().unwrap())
            } else {
                None
            };
            drop(loader);
            assert!(events.borrow().is_empty());
            assert!(
                poll(&mut task).is_ready(),
                "on_change={with_on_change}, observer={with_observer}, polled={poll_first}"
            );
            if let Some(sender) = sender {
                assert!(sender.is_canceled());
            }
            assert!(requests.borrow().is_empty());
            assert!(events.borrow().is_empty());
            drop(observer);
        }
    }
}

#[test]
fn dropping_one_clone_keeps_request_and_listener_alive() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut loader = Loader::new().on_change(record(&events));
    let requests = configure(&mut loader);
    let clone = loader.clone();
    let (sender, mut task) = pending_request(&loader, &requests);
    drop(loader);
    assert!(clone.loading());
    assert!(poll(&mut task).is_pending());
    sender.send(Ok(1)).unwrap();
    assert!(poll(&mut task).is_ready());
    assert_eq!(snapshot(&clone), (false, Some(Ok(1))));
    assert_eq!(*events.borrow(), vec![(false, Some(Ok(1)))]);
}

#[test]
fn replacing_listener_does_not_retain_previous_registration() {
    let first = Rc::new(RefCell::new(Vec::new()));
    let second = Rc::new(RefCell::new(Vec::new()));
    let mut loader = Loader::new().on_change(record(&first));
    let clone = loader.clone();
    loader = loader.on_change(record(&second));
    loader.abort();
    assert_eq!(*first.borrow(), vec![(false, None)]);
    assert_eq!(*second.borrow(), vec![(false, None)]);
    drop(clone);
    loader.abort();
    assert_eq!(*first.borrow(), vec![(false, None)]);
    assert_eq!(*second.borrow(), vec![(false, None); 2]);
    loader = loader.on_change(None::<Callback<Loader<u32>>>);
    loader.abort();
    assert_eq!(*second.borrow(), vec![(false, None); 2]);
}

#[test]
fn callback_handle_owns_request_without_retaining_registration() {
    let received = Rc::new(RefCell::new(Vec::new()));
    let mut loader = Loader::<u32>::new().on_change({
        let received = Rc::clone(&received);
        move |loader| received.borrow_mut().push(loader)
    });
    let requests = configure(&mut loader);
    loader.abort();
    let callback_loader = received.borrow_mut().pop().unwrap();
    let (sender, mut task) = pending_request(&loader, &requests);
    drop(loader);
    assert!(callback_loader.loading());
    assert!(poll(&mut task).is_pending());
    sender.send(Ok(1)).unwrap();
    assert!(poll(&mut task).is_ready());
    assert_eq!(snapshot(&callback_loader), (false, Some(Ok(1))));
    assert!(received.borrow().is_empty());
}

#[test]
fn dropping_observer_can_release_last_captured_owner() {
    for keep_owner in [false, true] {
        for with_request in [false, true] {
            let events = Rc::new(RefCell::new(Vec::new()));
            let mut loader = Loader::<u32>::new();
            let requests = configure(&mut loader);
            let _observer = loader.add_listener(record(&events));
            let captured = loader.clone();
            let observer = loader.add_listener(move |_| {
                let _ = captured.loading();
            });
            let mut pending = with_request.then(|| pending_request(&loader, &requests));
            let owner = keep_owner.then_some(loader);
            drop(observer);
            if let Some(owner) = &owner {
                assert_eq!(snapshot(owner), (with_request, None));
                if let Some((_, task)) = &mut pending {
                    assert!(poll(task).is_pending());
                }
            }
            drop(owner);
            if let Some((sender, mut task)) = pending {
                assert!(poll(&mut task).is_ready());
                assert!(sender.is_canceled());
            }
            assert!(events.borrow().is_empty());
        }
    }
}

#[test]
fn reentrant_replacement_cannot_publish_or_abort_new_request() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut loader = Loader::new().on_change(record(&events));
    let owner = Rc::new(loader.clone());
    let weak_owner = Rc::downgrade(&owner);
    let tasks = Rc::new(RefCell::new(Vec::<Task>::new()));
    loader.set_loader({
        let tasks = Rc::clone(&tasks);
        move || {
            let mut loader = weak_owner.upgrade().unwrap().as_ref().clone();
            let tasks = Rc::clone(&tasks);
            async move {
                loader.set_loader(|| async { Ok(2) });
                tasks.borrow_mut().push(start(&loader));
                Ok(1)
            }
        }
    });
    let mut task = start(&loader);
    assert!(poll(&mut task).is_ready());
    assert_eq!(snapshot(&loader), (true, None));
    assert!(events.borrow().is_empty());
    assert!(poll(&mut tasks.borrow_mut().pop().unwrap()).is_ready());
    assert_eq!(snapshot(&loader), (false, Some(Ok(2))));
    assert_eq!(*events.borrow(), vec![(false, Some(Ok(2)))]);
}

#[test]
fn reentrant_abort_cannot_publish_result() {
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut loader = Loader::new().on_change(record(&events));
    let owner = Rc::new(loader.clone());
    let weak_owner = Rc::downgrade(&owner);
    loader.set_loader(move || {
        let mut loader = weak_owner.upgrade().unwrap().as_ref().clone();
        async move {
            loader.abort();
            Ok(1)
        }
    });
    let mut task = start(&loader);
    assert!(poll(&mut task).is_ready());
    assert_eq!(snapshot(&loader), (false, None));
    assert_eq!(*events.borrow(), vec![(false, None)]);
}

#[test]
fn dropping_last_owner_during_final_poll_cannot_publish_result() {
    let mut loader = Loader::<u32>::new();
    let owner = Rc::new(RefCell::new(None));
    loader.set_loader({
        let owner = Rc::clone(&owner);
        move || {
            let owner = Rc::clone(&owner);
            async move {
                owner.borrow_mut().take();
                Ok(1)
            }
        }
    });
    // Retain storage, not an owner, to observe whether completion publishes after teardown.
    let state = loader.inner.state.clone();
    let inner = Rc::downgrade(&loader.inner);
    let mut task = start(&loader);
    *owner.borrow_mut() = Some(loader);
    assert!(poll(&mut task).is_ready());
    assert!(inner.upgrade().is_none());
    assert!(state.read().data.is_none());
}
