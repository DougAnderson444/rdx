//! Ported from >https://github.com/bytecodealliance/wasmtime/blob/main/crates/wasi/src/poll.rs> to support async/await
use super::{Resource, ResourceTable};
use anyhow::Result;
use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub type PollableFuture<'a> = Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
pub type MakeFuture = for<'a> fn(&'a mut dyn Any) -> PollableFuture<'a>;

/// A host representation of the `wasi:io/poll.pollable` resource.
///
/// A pollable is not the same thing as a Rust Future: the same pollable may be used to
/// repeatedly check for readiness of a given condition, e.g. if a stream is readable
/// or writable. So, rather than containing a Future, which can only become Ready once, a
/// Pollable contains a way to create a Future in each call to `poll`.
pub struct Pollable {
    pub index: u32,
    pub make_future: MakeFuture,
}

/// A trait used internally within a [`Pollable`] to create a `pollable`
/// resource in `wasi:io/poll`.
///
/// This trait is the internal implementation detail of any pollable resource in
/// this crate's implementation of WASI. The `ready` function is an `async fn`
/// which resolves when the implementation is ready. Using native `async` Rust
/// enables this type's readiness to compose with other types' readiness
/// throughout the WASI implementation.
///
/// This trait is used in conjunction with [`subscribe`] to create a `pollable`
/// resource.
///
/// # Example
///
/// This is a simple example of creating a `Pollable` resource from a few
/// parameters.
///
/// ```ignore
/// use tokio::time::{self, Duration, Instant};
/// use wasmtime_wasi::{WasiView, Subscribe, subscribe, Pollable, async_trait};
/// use wasmtime::component::Resource;
/// use wasmtime::Result;
///
/// fn sleep(cx: &mut dyn WasiView, dur: Duration) -> Result<Resource<Pollable>> {
///     let end = Instant::now() + dur;
///     let sleep = MySleep { end };
///     let sleep_resource = cx.table().push(sleep)?;
///     subscribe(cx.table(), sleep_resource)
/// }
///
/// struct MySleep {
///     end: Instant,
/// }
///
/// #[async_trait]
/// impl Subscribe for MySleep {
///     async fn ready(&mut self) {
///         tokio::time::sleep_until(self.end).await;
///     }
/// }
/// ```
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

/// Creates a `pollable` resource which is subscribed to the provided
/// `resource`.
///
/// If `resource` is an owned resource then it will be deleted when the returned
/// resource is deleted. Otherwise the returned resource is considered a "child"
/// of the given `resource` which means that the given resource cannot be
/// deleted while the `pollable` is still alive.
pub fn subscribe<T>(
    table: Arc<Mutex<ResourceTable>>,
    resource: Resource<T>,
) -> Result<Resource<Pollable>>
where
    T: Subscribe,
{
    fn make_future<T>(stream: &mut dyn Any) -> PollableFuture<'_>
    where
        T: Subscribe,
    {
        stream.downcast_mut::<T>().unwrap().ready()
    }

    let pollable = Pollable {
        index: resource.rep(),
        make_future: make_future::<T>,
    };

    Ok(table.lock().unwrap().push_child(pollable, &resource)?)
}
