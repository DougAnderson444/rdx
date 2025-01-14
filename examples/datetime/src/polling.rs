use std::ops::Deref;

use slab::Slab;

use super::{poll, Pollable}; // an efficient key-value data structure

#[derive(Debug)]
pub(crate) struct Poller {
    pub(crate) targets: Slab<Pollable>,
}

/// A key representing an entry into the poller.
///
/// Whenever we insert a type into Slab it returns a key of type usize. We define
/// our own EventKey type to wrap the keys for us.
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub(crate) struct EventKey(pub(crate) u32);

impl Deref for EventKey {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// From u32
impl From<u32> for EventKey {
    fn from(key: u32) -> Self {
        Self(key)
    }
}

impl Poller {
    /// Create a new instance of `Poller`
    pub(crate) fn new() -> Self {
        Self {
            targets: Slab::new(),
        }
    }

    /// Insert a new `Pollable` target into `Poller`
    pub(crate) fn insert(&mut self, target: Pollable) -> EventKey {
        let key = self.targets.insert(target);
        EventKey(key as u32)
    }

    /// Get a `Pollable` if it exists.
    pub(crate) fn get(&self, key: &EventKey) -> Option<&Pollable> {
        self.targets.get(key.0 as usize)
    }

    /// Remove an instance of `Pollable` from `Poller`.
    ///
    /// Returns `None` if no entry was found for `key`.
    pub(crate) fn remove(&mut self, key: EventKey) -> Option<Pollable> {
        self.targets.try_remove(key.0 as usize)
    }

    /// Block the current thread until a new event has triggered.
    ///
    /// This will clear the value of `ready_list`.
    pub(crate) fn block_until(&mut self) -> Vec<EventKey> {
        // We're about to wait for a number of pollables. When they wake we get
        // the *indexes* back for the pollables whose events were available - so
        // we need to be able to associate the index with the right waker.

        // We start by iterating over the pollables, and keeping note of which
        // pollable belongs to which waker index
        let mut indexes = Vec::with_capacity(self.targets.len());
        let mut targets = Vec::with_capacity(self.targets.len());
        for (index, target) in self.targets.iter() {
            indexes.push(index);
            targets.push(target);
        }

        // Now that we have that association, we're ready to poll our targets.
        // This will block until an event has completed.
        let ready_indexes = poll(&targets);

        // Once we have the indexes for which pollables are available, we need
        // to convert it back to the right keys for the wakers. Earlier we
        // established a positional index -> waker key relationship, so we can
        // go right ahead and perform a lookup there.
        ready_indexes
            .into_iter()
            .map(|index| EventKey(indexes[index as usize] as u32))
            .collect()
    }
}
