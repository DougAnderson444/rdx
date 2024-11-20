use std::any::Any;
use std::future::Future;
use std::pin::Pin;

use wasm_component_layer::ResourceType;

pub type PollableFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
pub type MakeFuture = for<'a> fn(&'a mut dyn Any) -> PollableFuture<'a>;
pub type ClosureFuture = Box<dyn Fn() -> PollableFuture<'static> + Send + 'static>;

#[derive(Debug)]
/// A host representation of the `wasi:io/poll.pollable` resource.
///
/// A pollable is not the same thing as a Rust Future: the same pollable may be used to
/// repeatedly check for readiness of a given condition, e.g. if a stream is readable
/// or writable. So, rather than containing a Future, which can only become Ready once, a
/// Pollable contains a way to create a Future in each call to `poll`.
pub struct Pollable {
    index: u32,
    make_future: MakeFuture,
    //remove_index_on_delete: Option<fn(&mut ResourceTable, u32) -> Result<()>>,
}

impl Pollable {
    /// Create a new Pollable resource.
    pub fn new(index: u32, make_future: MakeFuture) -> Self {
        Self {
            index,
            make_future,
            //remove_index_on_delete: None,
        }
    }

    // / Create a new Pollable resource with a custom remove_index_on_delete function.
    // pub fn new_with_remove_index_on_delete(
    //     index: u32,
    //     make_future: MakeFuture,
    //     remove_index_on_delete: fn(&mut ResourceTable, u32) -> Result<()>,
    // ) -> Self {
    //     Self {
    //         index,
    //         make_future,
    //         remove_index_on_delete: Some(remove_index_on_delete),
    //     }
    // }

    /// Get the index of the Pollable.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Create a new Future for the Pollable.
    pub fn make_future(&mut self) -> PollableFuture<'_> {
        (self.make_future)(self)
    }

    // /// Remove the Pollable from the ResourceTable.
    // pub fn remove_index_on_delete(&mut self, resource_table: &mut ResourceTable) -> Result<()> {
    //     if let Some(remove_index_on_delete) = self.remove_index_on_delete {
    //         remove_index_on_delete(resource_table, self.index)
    //     } else {
    //         Ok(())
    //     }
    // }
}

impl Pollable {
    /// Pollable is ready
    pub fn ready(&self) -> bool {
        todo!()
    }

    /// Pollable is blocking.
    ///
    /// Block returns immediately if the pollable is ready, and otherwise blocks until ready.
    /// This function is equivalent to calling poll.poll on a list containing only this pollable.
    pub fn block(&self) {
        todo!()
    }
}

#[async_trait::async_trait]
pub trait Subscribe: Send + 'static {
    /// An asynchronous function which resolves when this object's readiness
    /// operation is ready.
    ///
    /// This function is invoked as part of `poll` in `wasi:io/poll`. The
    /// meaning of when this function Returns depends on what object this
    /// [`Subscribe`] is attached to. When the returned future resolves then the
    /// corresponding call to `wasi:io/poll` will return.
    ///
    /// Note that this method does not return an error. Returning an error
    /// should be done through accessors on the object that this `pollable` is
    /// connected to. The call to `wasi:io/poll` itself does not return errors,
    /// only a list of ready objects.
    async fn ready(&mut self);
}

pub(crate) fn make_future<'a, T>(stream: &'a mut dyn Any) -> PollableFuture<'a>
where
    T: Subscribe,
{
    stream.downcast_mut::<T>().unwrap().ready()
}

/// The poll function can be called by guest components can submit interest
/// in an operation to the host system. List the `pollables: Vec<Resource<Pollable>>`
/// that the guest is interested in, and the host will return a list of Readylist Index.
pub fn poll(pollables: &[&Pollable]) -> Vec<u32> {
    todo!()
}
