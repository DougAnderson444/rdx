use core::ptr;
use core::task::Waker;
use core::task::{RawWaker, RawWakerVTable};

/// Construct a new no-op waker
// NOTE: we can remove this once <https://github.com/rust-lang/rust/issues/98286> lands
pub fn noop_waker() -> Waker {
    const VTABLE: RawWakerVTable = RawWakerVTable::new(
        // Cloning just returns a new no-op raw waker
        |_| RAW,
        // `wake` does nothing
        |_| {},
        // `wake_by_ref` does nothing
        |_| {},
        // Dropping does nothing as we don't allocate anything
        |_| {},
    );
    const RAW: RawWaker = RawWaker::new(ptr::null(), &VTABLE);

    // SAFETY: all fields are no-ops, so this is safe
    unsafe { Waker::from_raw(RAW) }
}
