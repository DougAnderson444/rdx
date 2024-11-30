use super::Pollable;

use super::polling::{EventKey, Poller};

use std::collections::HashMap;
use std::future::{self, Future};
use std::task::Poll;
use std::task::Waker;
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
    pub fn wait_for(&self, pollable: Pollable) -> impl Future<Output = ()> + '_ {
        let mut pollable = Some(pollable);
        async move {
            let mut key = None;
            future::poll_fn(|cx| {
                let mut reactor = self.inner.borrow_mut();
                if key.is_none() {
                    key = Some(reactor.poller.insert(pollable.take().unwrap()));
                }
                let k = key.unwrap();
                reactor.wakers.insert(k, cx.waker().clone());

                if reactor.poller.get(&k).unwrap().ready() {
                    reactor.poller.remove(k);
                    reactor.wakers.remove(&k);
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

///// Turn a single pollable into a future.
//pub struct PollableFuture {
//    pollable: Pollable,
//}
//
//impl PollableFuture {
//    /// Create a new instance of `PollableFuture`
//    pub(crate) fn new(pollable: Pollable) -> Self {
//        Self { pollable }
//    }
//}
//
//impl Future for PollableFuture {
//    type Output = ();
//
//    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//        if self.pollable.ready() {
//            Poll::Ready(())
//        } else {
//            cx.waker().wake_by_ref();
//            Poll::Pending
//        }
//    }
//}
