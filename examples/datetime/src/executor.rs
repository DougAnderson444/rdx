use std::{
    future::Future,
    mem,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
};

use crate::bindings::wasi::io::{self, poll::Pollable};

const READ_SIZE: u64 = 16 * 1024;

static WAKERS: Mutex<Vec<(Pollable, Waker)>> = Mutex::new(Vec::new());

pub fn run<T>(future: impl Future<Output = T>) -> T {
    futures::pin_mut!(future);

    struct DummyWaker;

    impl Wake for DummyWaker {
        fn wake(self: Arc<Self>) {}
    }

    let waker = Arc::new(DummyWaker).into();

    loop {
        match future.as_mut().poll(&mut Context::from_waker(&waker)) {
            Poll::Pending => {
                let mut new_wakers = Vec::new();

                let wakers = mem::take::<Vec<_>>(&mut WAKERS.lock().unwrap());

                assert!(!wakers.is_empty());

                let pollables = wakers
                    .iter()
                    .map(|(pollable, _)| pollable)
                    .collect::<Vec<_>>();

                let mut ready = vec![false; wakers.len()];

                for index in io::poll::poll(&pollables) {
                    ready[usize::try_from(index).unwrap()] = true;
                }

                for (ready, (pollable, waker)) in ready.into_iter().zip(wakers) {
                    if ready {
                        waker.wake()
                    } else {
                        new_wakers.push((pollable, waker));
                    }
                }

                *WAKERS.lock().unwrap() = new_wakers;
            }
            Poll::Ready(result) => break result,
        }
    }
}
