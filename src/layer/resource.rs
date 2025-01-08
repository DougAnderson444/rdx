#[cfg(not(target_arch = "wasm32"))]
use core::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::marker;

pub struct Resource<T> {
    /// The host-defined 32-bit representation of this resource.
    rep: u32,

    /// Dear rust please consider `T` used even though it's not actually used.
    _marker: marker::PhantomData<fn() -> T>,

    #[cfg(not(target_arch = "wasm32"))]
    state: AtomicResourceState,
    #[cfg(target_arch = "wasm32")]
    state: AtomicResourceState,
}

#[cfg(not(target_arch = "wasm32"))]
struct AtomicResourceState(AtomicU64);
#[cfg(target_arch = "wasm32")]
struct AtomicResourceState(u64);

impl<T> Resource<T>
where
    T: 'static,
{
    /// Creates a new owned resource with the `rep` specified.
    ///
    /// The returned value is suitable for passing to a guest as either a
    /// `(borrow $t)` or `(own $t)`.
    pub fn new_own(rep: u32) -> Resource<T> {
        Resource {
            state: AtomicResourceState::NOT_IN_TABLE,
            rep,
            _marker: marker::PhantomData,
        }
    }

    /// Returns the underlying 32-bit representation used to originally create
    /// this resource.
    pub fn rep(&self) -> u32 {
        self.rep
    }
    /// Returns whether this is an owned resource or not.
    ///
    /// Owned resources can be safely destroyed by the embedder at any time, and
    /// borrowed resources have an owner somewhere else on the stack so can only
    /// be accessed, not destroyed.
    pub fn owned(&self) -> bool {
        match self.state.get() {
            ResourceState::Borrow => false,
            ResourceState::Taken | ResourceState::NotInTable | ResourceState::Index(_) => true,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum ResourceState {
    Borrow,
    NotInTable,
    Taken,
    Index(HostResourceIndex),
}

#[cfg(target_arch = "wasm32")]
impl AtomicResourceState {
    const NOT_IN_TABLE: Self = Self(ResourceState::NOT_IN_TABLE);

    /// get
    fn get(&self) -> ResourceState {
        // the u64 equivalent of AtomicU64 load
        ResourceState::decode(self.0)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl AtomicResourceState {
    #[allow(clippy::declare_interior_mutable_const)]
    const NOT_IN_TABLE: Self = Self(AtomicU64::new(ResourceState::NOT_IN_TABLE));

    fn get(&self) -> ResourceState {
        ResourceState::decode(self.0.load(Relaxed))
    }
}

impl ResourceState {
    // See comments on `state` above for info about these values.
    const BORROW: u64 = u64::MAX;
    const NOT_IN_TABLE: u64 = u64::MAX - 1;
    const TAKEN: u64 = u64::MAX - 2;

    fn decode(bits: u64) -> ResourceState {
        match bits {
            Self::BORROW => Self::Borrow,
            Self::NOT_IN_TABLE => Self::NotInTable,
            Self::TAKEN => Self::Taken,
            other => Self::Index(HostResourceIndex(other)),
        }
    }

    // unused
    //fn encode(&self) -> u64 {
    //    match self {
    //        Self::Borrow => Self::BORROW,
    //        Self::NotInTable => Self::NOT_IN_TABLE,
    //        Self::Taken => Self::TAKEN,
    //        Self::Index(index) => index.0,
    //    }
    //}
}

/// Host representation of an index into a table slot.
///
/// This is morally (u32, u32) but is encoded as a 64-bit integer. The low
/// 32-bits are the table index and the upper 32-bits are the generation
/// counter.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct HostResourceIndex(u64);

//impl HostResourceIndex {
//    fn new(idx: u32, gen: u32) -> HostResourceIndex {
//        HostResourceIndex(u64::from(idx) | (u64::from(gen) << 32))
//    }
//
//    fn index(&self) -> u32 {
//        u32::try_from(self.0 & 0xffffffff).unwrap()
//    }
//
//    fn gen(&self) -> u32 {
//        u32::try_from(self.0 >> 32).unwrap()
//    }
//}
