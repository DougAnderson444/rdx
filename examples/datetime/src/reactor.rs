use crate::bindings::wasi::io::poll::poll;

use super::Pollable;

use super::polling::{EventKey, Poller};

use std::collections::HashMap;
use std::future::{self, Future};
use std::pin::Pin;
use std::task::Waker;
use std::task::{Context, Poll};
use std::{cell::RefCell, rc::Rc};

/// Manage async system resources for WASI 0.1
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

    /// Wait for the pollable to resolve.
    pub async fn wait_for(&self, pollable: Pollable) {
        let mut pollable = Some(pollable);
        let mut key = None;

        // This function is the core loop of our function; it will be called
        // multiple times as the future is resolving.
        future::poll_fn(|cx| {
            // Start by taking a lock on the reactor. This is single-threaded
            // and short-lived, so it will never be contended.
            let mut reactor = self.inner.borrow_mut();

            // Schedule interest in the `pollable` on the first iteration. On
            // every iteration, register the waker with the reactor.
            let key = key.get_or_insert_with(|| reactor.poller.insert(pollable.take().unwrap()));
            reactor.wakers.insert(*key, cx.waker().clone());

            // Check whether we're ready or need to keep waiting. If we're
            // ready, we clean up after ourselves.
            if reactor.poller.get(key).unwrap().ready() {
                reactor.poller.remove(*key);
                reactor.wakers.remove(key);
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await
    }

    pub fn await_on(&self, pollable: Pollable) -> impl Future<Output = ()> {
        let inner = self.inner.clone();
        async move {
            let mut pollable = Some(pollable);
            let mut key = None;
            future::poll_fn(|cx| {
                let mut reactor = inner.borrow_mut();
                let key =
                    key.get_or_insert_with(|| reactor.poller.insert(pollable.take().unwrap()));
                reactor.wakers.insert(*key, cx.waker().clone());

                if reactor.poller.get(key).unwrap().ready() {
                    reactor.poller.remove(*key);
                    reactor.wakers.remove(key);
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await
        }
    }

    /// Reactor wrapper for inner poller to block until new events are ready.
    /// Calls the respective wakers once done.
    pub(crate) fn block_until(&self) {
        let mut reactor = self.inner.borrow_mut();
        for key in reactor.poller.block_until() {
            match reactor.wakers.get(&key) {
                Some(waker) => waker.wake_by_ref(),
                None => panic!("tried to wake the waker for non-existent `{key:?}`"),
            }
        }
    }
}

/// Turn a single pollable into a future.
pub struct PollableFuture {
    pollable: Pollable,
}

impl PollableFuture {
    /// Create a new instance of `PollableFuture`
    pub(crate) fn new(pollable: Pollable) -> Self {
        Self { pollable }
    }
}

impl Future for PollableFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.pollable.ready() {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
