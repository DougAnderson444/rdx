use super::log;

use futures::{future, Future};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Wake, Waker};
use std::{mem, ptr};

use crate::bindings::wasi::io;
use crate::bindings::wasi::io::poll::poll;
use crate::polling::{EventKey, Poller};

static WAKERS: Mutex<Vec<(io::poll::Pollable, Waker)>> = Mutex::new(Vec::new());

///// A simple function that creates a future that will resolve when the `pollable`
///// is ready if it isn't already.
//pub fn simple(pollable: io::poll::Pollable) -> impl Future<Output = ()> {
//    //let mut pollable = Some(pollable);
//
//    future::poll_fn(move |cx| {
//        if pollable.ready() {
//            Poll::Ready(())
//        } else {
//            WAKERS.lock().unwrap().push((pollable, cx.waker().clone()));
//            Poll::Pending
//        }
//    })
//}

#[derive(Debug, Clone)]
pub struct Reactor {
    inner: Rc<RefCell<InnerReactor>>,
}

/// The private, internal `Reactor` implementation - factored out so we can take
/// a lock of the whole.
#[derive(Debug)]
struct InnerReactor {
    poller: Poller,
    wakers: HashMap<EventKey, Waker>,
}

impl Reactor {
    /// Create a new instance of `Reactor`
    pub(crate) fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(InnerReactor {
                poller: Poller::new(),
                wakers: HashMap::new(),
            })),
        }
    }

    pub fn wait_for(&self, pollable: io::poll::Pollable) -> impl Future<Output = ()> + use<'_> {
        log("Creating future");

        let mut pollable = Some(pollable);
        let mut key = None;

        future::poll_fn(move |cx| {
            log("wait_for poll_fn");
            // Start by taking a lock on the reactor. This is single-threaded
            // and short-lived, so it will never be contended.
            let mut reactor = self.inner.borrow_mut();

            // Schedule interest in the `pollable` on the first iteration. On
            // every iteration, register the waker with the reactor.
            let key = key.get_or_insert_with(|| reactor.poller.insert(pollable.take().unwrap()));
            reactor.wakers.insert(*key, cx.waker().clone());

            // Check whether we're ready or need to keep waiting. If we're
            // ready, we clean up after ourselves.
            log(&format!(
                "Checking if ready(), wakers: {:?} pollers {:?}",
                reactor.wakers.len(),
                reactor.poller.targets.len()
            ));
            if reactor.poller.get(key).unwrap().ready() {
                reactor.poller.remove(*key);
                reactor.wakers.remove(key);
                Poll::Ready(())
            } else {
                log("wait_for returns Poll::Pending");
                Poll::Pending
            }
        })
    }

    /// Current targets from Poller
    pub fn ready_indexes(&self) -> Vec<EventKey> {
        let mut reactor = self.inner.borrow_mut();
        reactor.poller.ready_indexes()
    }
}

pub struct Executor {
    reactor: Reactor,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    /// Create a new instance of `Executor`
    pub fn new() -> Self {
        Self {
            reactor: Reactor::new(),
        }
    }

    pub fn spawn<T, F, Fut>(&mut self, f: F) -> T
    where
        F: FnOnce(Reactor) -> Fut,
        Fut: Future<Output = T>,
    {
        // Construct the reactor
        let reactor = Reactor::new();

        let future = f(reactor.clone());

        log("pinning future");

        futures::pin_mut!(future);

        struct DummyWaker;

        impl Wake for DummyWaker {
            fn wake(self: Arc<Self>) {}
        }

        let waker = Arc::new(DummyWaker).into();

        loop {
            let cx = &mut Context::from_waker(&waker);
            log(&format!("[loop] star - waker {:?}", cx));

            match future.as_mut().poll(cx) {
                Poll::Pending => {
                    log("[loop] Poll::pending");

                    self.reactor.ready_indexes().iter().for_each(|key| {
                        log(&format!("[loop] wake {:?}", key));
                        if let Some(waker) = self.reactor.inner.borrow().wakers.get(key) {
                            waker.wake_by_ref();
                        }
                    });

                    log("[executor]: done polling");
                }
                Poll::Ready(result) => break result,
            }
            log("[loop] end");
        }
    }
}

fn noop_waker() -> Waker {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(|_| RAW, |_| {}, |_| {}, |_| {});
    const RAW: RawWaker = RawWaker::new(ptr::null(), &VTABLE);
    unsafe { Waker::from_raw(RAW) }
}
