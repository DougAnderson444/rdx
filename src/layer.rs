pub mod poll;
use poll::{subscribe, MakeFuture, PollableFuture, Subscribe};

mod resource;
pub use resource::Resource;
pub use resource_table::ResourceTable;

mod noop_waker;
pub use noop_waker::noop_waker;
use send_wrapper::SendWrapper;

pub mod resource_table;

use std::any::Any;
use std::cell::RefMut;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::pin::Pin;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
pub use std::time::{Duration, Instant, SystemTime};
#[cfg(target_arch = "wasm32")]
pub use web_time::{Duration, Instant, SystemTime};

pub use anyhow::bail;
use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll};

pub use rhai;

pub use poll::Pollable;
pub use wasm_component_layer::{
    AsContext as _, AsContextMut as _, Component, Engine, Func, FuncType, Instance, Linker, List,
    ListType, RecordType, ResourceOwn, ResourceType, Store, Value, ValueType,
};

#[cfg(not(target_arch = "wasm32"))]
pub use wasmtime_runtime_layer as runtime_layer;

#[cfg(target_arch = "wasm32")]
pub use js_wasm_runtime_layer as runtime_layer;

pub use crate::Error;
use parking_lot::{lock_api::MutexGuard, RawMutex};

/// Immutable reference to the [rhai::Scope]
pub enum ScopeRef {
    Borrowed(Arc<parking_lot::Mutex<rhai::Scope<'static>>>),
    Refcell(Rc<RefCell<rhai::Scope<'static>>>),
}

#[cfg(not(target_arch = "wasm32"))]
impl Deref for ScopeRef {
    type Target = Arc<parking_lot::Mutex<rhai::Scope<'static>>>;

    fn deref(&self) -> &Self::Target {
        match self {
            ScopeRef::Borrowed(scope) => scope,
            ScopeRef::Refcell(_ref_scope) => {
                unreachable!()
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Deref for ScopeRef {
    type Target = Rc<RefCell<rhai::Scope<'static>>>;

    fn deref(&self) -> &Self::Target {
        match self {
            ScopeRef::Borrowed(_) => {
                unreachable!()
            }
            ScopeRef::Refcell(ref_scope) => ref_scope,
        }
    }
}

pub enum ScopeRefMut<'a> {
    Borrowed(MutexGuard<'a, RawMutex, rhai::Scope<'static>>),
    Refcell(RefMut<'a, rhai::Scope<'static>>),
}

impl Deref for ScopeRefMut<'_> {
    type Target = rhai::Scope<'static>;

    fn deref(&self) -> &Self::Target {
        match self {
            ScopeRefMut::Borrowed(scope) => scope,
            ScopeRefMut::Refcell(ref_scope) => ref_scope,
        }
    }
}

impl DerefMut for ScopeRefMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ScopeRefMut::Borrowed(scope) => scope,
            ScopeRefMut::Refcell(ref_scope) => ref_scope,
        }
    }
}
pub trait Inner {
    /// Update the state with the given key and value
    fn update(&mut self, key: &str, value: impl Into<rhai::Dynamic> + Clone);

    /// Return the [rhai::Scope]
    fn scope(&self) -> ScopeRef;

    /// Return the mutable [rhai::Scope]
    fn scope_mut(&mut self) -> ScopeRefMut; // &mut rhai::Scope<'static>;

    /// Consumes [Inner] to yield Owned Scope
    fn into_scope(self) -> rhai::Scope<'static>;
}

/// The sleep resource
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Sleep {
    end: Instant,
}

#[async_trait::async_trait]
impl poll::Subscribe for Sleep {
    async fn ready(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            tracing::info!(
                "READY: Sleeping until {:?} at {:?}",
                self.end,
                Instant::now()
            );
            tokio::time::sleep_until(self.end.into()).await;
            tracing::info!("Woke up at {:?}", Instant::now());
        }

        #[cfg(target_arch = "wasm32")]
        {
            tracing::info!(
                "READY Sleeping until {:?} at {:?}",
                self.end,
                Instant::now()
            );
            send_wrapper::SendWrapper::new(async move {
                if let Err(e) = js_sleep(self.end.elapsed().as_millis() as i32).await {
                    tracing::error!("Error sleeping: {:?}", e);
                }
                tracing::info!("READY Woke up at {:?}", Instant::now());
            })
            .await;
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub async fn js_sleep(millis: i32) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let promise = web_sys::js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, millis)
            .unwrap();
    });

    wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(())
}

enum Deadline {
    Past,
    Instant(Instant),
    Never,
}

#[async_trait::async_trait]
impl Subscribe for Deadline {
    async fn ready(&mut self) {
        match self {
            Deadline::Past => {}
            Deadline::Instant(instant) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    tracing::info!(
                        "Deadline: Sleeping until {:?} at {:?}",
                        instant,
                        Instant::now()
                    );
                    tokio::time::sleep_until((*instant).into()).await;
                }
                #[cfg(target_arch = "wasm32")]
                {
                    tracing::info!(
                        "Deadline: Sleeping until {:?} at {:?}",
                        instant,
                        Instant::now()
                    );
                    send_wrapper::SendWrapper::new(async move {
                        if let Err(e) = js_sleep(instant.elapsed().as_millis() as i32).await {
                            tracing::error!("Error sleeping: {:?}", e);
                        }
                    })
                    .await;
                }
            }
            Deadline::Never => std::future::pending().await,
        }
    }
}

fn subscribe_to_duration(
    table: Arc<Mutex<ResourceTable>>,
    duration: Duration,
) -> anyhow::Result<Resource<Pollable>> {
    let sleep = if duration.is_zero() {
        tracing::info!("subscribe_to_duration: Duration is zero, returning Past");
        table.lock().unwrap().push(Deadline::Past)?
    } else if let Some(deadline) = Instant::now().checked_add(duration) {
        tracing::info!("subscribe_to_duration: Duration is not zero, returning Instant");
        // NB: this resource created here is not actually exposed to wasm, it's
        // only an internal implementation detail used to match the signature
        // expected by `subscribe`.
        table.lock().unwrap().push(Deadline::Instant(deadline))?
    } else {
        tracing::info!("subscribe_to_duration: Duration is too far in the future, returning Never");
        // If the user specifies a time so far in the future we can't
        // represent it, wait forever rather than trap.
        table.lock().unwrap().push(Deadline::Never)?
    };
    subscribe(table, sleep)
}

pub fn instantiate_instance<T: Inner + 'static>(
    bytes: &[u8],
    data: T,
) -> (Instance, Store<T, runtime_layer::Engine>) {
    let table = Arc::new(Mutex::new(ResourceTable::new()));

    // Create a new engine for instantiating a component.
    let engine = Engine::new(runtime_layer::Engine::default());

    // Create a store for managing WASM data and any custom user-defined state.
    let mut store = Store::new(&engine, data);

    // Parse the component bytes and load its imports and exports.
    let component = Component::new(&engine, bytes).unwrap();
    // Create a linker that will be used to resolve the component's imports, if any.
    let mut linker = Linker::default();

    // Pollable resource type
    let resource_pollable_ty = ResourceType::new::<Resource<Pollable>>(None);

    // pollable is wasi:io/poll
    let poll_interface = linker
        .define_instance("wasi:io/poll@0.2.2".try_into().unwrap())
        .unwrap();

    poll_interface
        .define_resource("pollable", resource_pollable_ty.clone())
        .unwrap();

    // ready and block are methods on the pollable resource, "[method]pollable.ready" and "[method]pollable.block"
    //ready: func() -> bool;
    let table_clone = table.clone();
    poll_interface
        .define_func(
            "[method]pollable.ready",
            Func::new(
                &mut store,
                FuncType::new(
                    [ValueType::Borrow(resource_pollable_ty.clone())],
                    [ValueType::Bool],
                ),
                move |store, params, results| {
                    tracing::info!("[method]pollable.ready");

                    let Value::Borrow(pollable_resource) = &params[0] else {
                        panic!("Incorrect input type, found {:?}", params[0]);
                    };

                    tracing::info!("Got borrow param pollable {:?}", pollable_resource);

                    let binding = store.as_context();
                    let res_pollable: &Resource<Pollable> =
                        pollable_resource.rep(&binding).map_err(|e| {
                            tracing::error!("Error getting pollable resource: {:?}", e);
                            e
                        })?;

                    tracing::info!("Got pollable resource");

                    // get pollable from table
                    // get inner table
                    let table: &mut ResourceTable = &mut table_clone.lock().unwrap();

                    let pollable = table.get(res_pollable)?;

                    let ready = (pollable.make_future)(table.get_any_mut(pollable.index)?);

                    tracing::info!("Got ready");

                    let mut fut = pin!(ready);
                    let waker = noop_waker();
                    let mut cx = Context::from_waker(&waker);

                    // Poll the future once
                    let poll_result = fut.as_mut().poll(&mut cx);

                    // Check the result
                    let ready = matches!(poll_result, Poll::Ready(()));

                    tracing::info!("[ready] Poll result: {:?}", ready);

                    // if not ready, save the future to the table

                    results[0] = Value::Bool(ready);
                    Ok(())
                },
            ),
        )
        .unwrap();

    poll_interface
        .define_func(
            "[method]pollable.block",
            Func::new(
                &mut store,
                FuncType::new([], []),
                move |_store, _params, _results| {
                    tracing::info!("[method]pollable.block");
                    //todo!();
                    Ok(())
                },
            ),
        )
        .unwrap();

    // poll: func(in: list<borrow<pollable>>) -> list<u32>;
    let table_clone = table.clone();
    poll_interface
        .define_func(
            "poll",
            Func::new(
                &mut store,
                FuncType::new(
                    [ValueType::List(ListType::new(ValueType::Borrow(
                        resource_pollable_ty.clone(),
                    )))],
                    [ValueType::List(ListType::new(ValueType::U32))],
                ),
                move |mut store, params, results| {
                    tracing::info!("[method]pollable.poll");

                    type ReadylistIndex = u32;

                    tracing::debug!("[poll]: convert list to pollables");

                    let pollables = match &params[0] {
                        Value::List(pollables) => pollables,
                        _ => bail!("Incorrect input type"),
                    };

                    tracing::debug!("[poll]: check if pollables is empty");

                    if pollables.is_empty() {
                        bail!("Empty pollables list");
                    }

                    tracing::debug!("[poll]: create table futures");

                    let mut table_futures: HashMap<u32, (MakeFuture, Vec<ReadylistIndex>)> =
                        HashMap::new();

                    for (ix, p) in pollables.iter().enumerate() {
                        let ix: u32 = ix.try_into()?;

                        tracing::debug!("[poll]: get pollable resource");

                        let Value::Borrow(pollable_resource) = p else {
                            bail!("Incorrect input type, found {:?}", p);
                        };

                        let mut binding = store.as_context_mut();
                        let p: &mut Resource<Pollable> = pollable_resource.rep_mut(&mut binding)?;

                        let binding = table_clone.lock().unwrap();
                        let pollable = binding.get(p)?;
                        let (_, list) = table_futures
                            .entry(pollable.index)
                            .or_insert((pollable.make_future, Vec::new()));
                        list.push(ix);
                    }

                    let mut futures: Vec<(PollableFuture<'_>, Vec<ReadylistIndex>)> = Vec::new();

                    let mut binding = table_clone.lock().unwrap();

                    let it = table_futures.into_iter().map(move |(k, v)| {
                        let item = binding
                            .occupied_mut(k)
                            .map(|e| Box::as_mut(&mut e.entry))
                            // Safety: extending the lifetime of the mutable reference.
                            .map(|item| unsafe { &mut *(item as *mut dyn Any) });
                        (item, v)
                    });

                    for (entry, (make_future, readylist_indices)) in it {
                        let entry = entry?;
                        futures.push((make_future(entry), readylist_indices));
                    }

                    struct PollList<'a> {
                        futures: Vec<(PollableFuture<'a>, Vec<ReadylistIndex>)>,
                    }

                    impl Future for PollList<'_> {
                        type Output = Vec<u32>;

                        fn poll(
                            mut self: Pin<&mut Self>,
                            cx: &mut Context<'_>,
                        ) -> Poll<Self::Output> {
                            let mut any_ready = false;
                            let mut results = Vec::new();
                            for (fut, readylist_indicies) in self.futures.iter_mut() {
                                match fut.as_mut().poll(cx) {
                                    Poll::Ready(()) => {
                                        results.extend_from_slice(readylist_indicies);
                                        any_ready = true;
                                    }
                                    Poll::Pending => {}
                                }
                            }
                            if any_ready {
                                Poll::Ready(results)
                            } else {
                                Poll::Pending
                            }
                        }
                    }

                    tracing::debug!("[poll]: return poll list");

                    // We set results[0] to be the sync equivalent to: PollList { futures }.await
                    results[0] = Value::List(List::new(
                        ListType::new(ValueType::U32),
                        futures
                            .into_iter()
                            // only add to the returned list if the future is ready, otherwise skip
                            // the future until next time
                            .filter_map(|(mut fut, readylist_indices)| {
                                let waker = noop_waker();
                                let mut cx = Context::from_waker(&waker);
                                match fut.as_mut().poll(&mut cx) {
                                    Poll::Ready(()) => Some(readylist_indices),
                                    Poll::Pending => None,
                                }
                            })
                            .flatten()
                            .map(Value::U32)
                            .collect::<Vec<_>>(),
                    )?);

                    Ok(())
                },
            ),
        )
        .unwrap();

    let host_interface = linker
        .define_instance("component:plugin/host".try_into().unwrap())
        .unwrap();

    // params is a record with name and value
    let record = RecordType::new(
        None,
        vec![("name", ValueType::String), ("value", ValueType::String)],
    )
    .unwrap();
    let params = ValueType::Record(record);
    let results = [];

    // "log" function using tracing
    host_interface
        .define_func(
            "log",
            Func::new(
                &mut store,
                FuncType::new([ValueType::String], []),
                move |_store, params, _results| {
                    if let Value::String(s) = &params[0] {
                        tracing::info!("{}", s);
                    }
                    Ok(())
                },
            ),
        )
        .unwrap();

    host_interface
        .define_func(
            "emit",
            Func::new(
                &mut store,
                FuncType::new([params], results),
                move |mut store, params, _results| {
                    tracing::info!("Emitting event {:?}", params);
                    if let Value::Record(record) = &params[0] {
                        let name = record.field("name").unwrap();
                        let value = record.field("value").unwrap();

                        if let Value::String(name) = name {
                            if let Value::String(value) = value {
                                tracing::info!("Updating state with {:?} {:?}", name, value);
                                store.data_mut().update(&name, &*value);
                            }
                        }
                    }

                    Ok(())
                },
            ),
        )
        .unwrap();

    // add func get_random
    host_interface
        .define_func(
            "random-byte",
            Func::new(
                &mut store,
                FuncType::new([], [ValueType::U8]),
                move |_store, _params, results| {
                    let random = rand::random::<u8>();
                    results[0] = Value::U8(random);
                    Ok(())
                },
            ),
        )
        .unwrap();

    // now function
    host_interface
        .define_func(
            "now",
            Func::new(
                &mut store,
                FuncType::new([], [ValueType::S64]),
                move |_store, _params, results| {
                    let unix_timestamp = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    results[0] = Value::S64(unix_timestamp);
                    Ok(())
                },
            ),
        )
        .unwrap();

    // sleep takes ms and returns a Pollable resource type
    let table_clone = table.clone();
    host_interface
        .define_func(
            "subscribe-duration",
            Func::new(
                &mut store,
                FuncType::new(
                    [ValueType::U64],
                    [ValueType::Own(resource_pollable_ty.clone())],
                ),
                move |mut store, params, results| {
                    // sleep should take these millis and turn them into pollable
                    // then return the pollable

                    let Value::U64(millis) = params[0] else {
                        panic!("Incorrect input type.")
                    };

                    tracing::info!("Subscribing to duration: {:?}", millis);

                    let resource_pollable =
                        subscribe_to_duration(table_clone.clone(), Duration::from_millis(millis))
                            .map_err(|e| {
                            tracing::error!("Error subscribing to duration: {:?}", e);
                            e
                        })?;

                    tracing::info!("Subscribed to duration");

                    let pollable_resource = ResourceOwn::new(
                        &mut store,
                        resource_pollable,
                        resource_pollable_ty.clone(),
                    )?;

                    results[0] = Value::Own(pollable_resource);
                    Ok(())
                },
            ),
        )
        .unwrap();

    (linker.instantiate(&mut store, &component).unwrap(), store)
}

pub trait Instantiator<T: Inner + Send + Sync>: Send {
    /// returns the Store
    fn store(&self) -> &Store<T, runtime_layer::Engine>;

    /// mut Store
    fn store_mut(&mut self) -> &mut Store<T, runtime_layer::Engine>;

    // call fn
    fn call(&mut self, name: &str, arguments: &[Value]) -> Result<Option<Value>, Error>;
}

/// Plugin struct to store some state
pub struct LayerPlugin<T: Inner + Send + Sync> {
    pub(crate) store: SendWrapper<Store<T, runtime_layer::Engine>>,
    raw_instance: wasm_component_layer::Instance,
}

impl<T: Inner + Send + Sync + 'static> LayerPlugin<T> {
    /// Creates a new plugin instance with the given name and bytes
    pub fn new(bytes: &[u8], data: T) -> Self {
        let (instance, store) = instantiate_instance(bytes, data);

        Self {
            store: SendWrapper::new(store),
            raw_instance: instance,
        }
    }
}

impl<T: Inner + Send + Sync + 'static> Instantiator<T> for LayerPlugin<T> {
    //fn scope(&self) -> &rhai::Scope {
    //    self.store.data().scope().into()
    //}

    fn store(&self) -> &Store<T, runtime_layer::Engine> {
        &self.store
    }

    fn store_mut(&mut self) -> &mut Store<T, runtime_layer::Engine> {
        &mut self.store
    }

    /// Calls the given function name with the given parameters
    fn call(&mut self, name: &str, arguments: &[Value]) -> Result<Option<Value>, Error> {
        tracing::info!("Calling function: {}", name);
        let export_instance = self
            .raw_instance
            .exports()
            .instance(&"component:plugin/run".try_into()?)
            .ok_or(Error::InstanceNotFound)?;

        let func = export_instance
            .func(name)
            .ok_or_else(|| Error::FuncNotFound(name.to_string()))?;

        let func_result_len = func.ty().results().len();
        let mut results = vec![Value::Bool(false); func_result_len];

        func.call(self.store.deref_mut(), arguments, &mut results)
            .map_err(|e| {
                tracing::error!("Error calling function: {:?}", e);
                e
            })?;

        if results.is_empty() {
            Ok(None)
        } else {
            Ok(Some(results.remove(0)))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::*;

    #[derive(Default)]
    struct State {
        count: rhai::Dynamic,
        scope: Arc<parking_lot::Mutex<rhai::Scope<'static>>>,
    }

    impl Inner for State {
        fn update(&mut self, key: &str, value: impl Into<rhai::Dynamic> + Clone) {
            println!("updating {}: {}", key, value.clone().into());
            // set count to value
            // TODO: Chg hard code into rhai scope change
            if key == "count" {
                self.count = value.clone().into();
            }
        }

        fn scope(&self) -> ScopeRef {
            ScopeRef::Borrowed(self.scope.clone())
        }

        fn scope_mut(&mut self) -> ScopeRefMut {
            ScopeRefMut::Borrowed(self.scope.lock())
        }

        fn into_scope(self) -> rhai::Scope<'static> {
            self.scope.lock().clone()
        }
    }

    #[test]
    fn test_instantiate_instance() {
        const WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/debug/counter.wasm");

        let data: State = State {
            count: 0.into(),
            ..Default::default()
        };

        let (instance, mut store) = instantiate_instance(WASM, data);

        // Get the interface that the interface exports.
        let exports = instance.exports();

        // get the "increment" exported function
        let export_instance = exports
            .instance(&"component:plugin/run".try_into().unwrap())
            .unwrap();

        let _funcs = export_instance
            .funcs()
            .map(|f| {
                // print
                println!("Function {:?}", f.0);
            })
            .collect::<Vec<_>>();

        // call the increment function
        let func = export_instance.func("increment").unwrap();

        const CAPACITY: usize = 1;
        let mut results = [Value::Bool(false); CAPACITY];
        func.call(&mut store, &[], &mut results).unwrap();

        // assert results
        assert_eq!(results[0], Value::S32(1));

        // check the store data
        let data = store.data();
        let count = &data.count;
        let count_string = count.to_string();
        let count_i64 = count_string.parse::<i64>().unwrap();
        assert_eq!(count_i64, 1);
    }

    // test Plugin struct
    #[test]
    fn test_plugin() {
        const WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/debug/counter.wasm");

        let data: State = State {
            count: 0.into(),
            ..Default::default()
        };

        let mut plugin = LayerPlugin::new(WASM, data);

        let _ = plugin.call("increment", &[]).unwrap();

        // current
        let result = plugin.call("current", &[]).unwrap();

        assert_eq!(result, Some(Value::S32(1)));
    }

    // test that Sleep can be saved as Any, then downcast back into Sleep
    #[test]
    fn test_sleep_any_rountrip() {
        let sleep = Sleep {
            end: Instant::now() + Duration::from_millis(100),
        };

        let sleep_any = Box::new(sleep) as Box<dyn Any + Send>;

        // assert sleep now has Any type
        assert!(sleep_any.is::<Sleep>());

        let sleep = sleep_any.downcast::<Sleep>().unwrap();

        assert_eq!(sleep.end, sleep.end);
    }

    // test Sleep is Send
    #[test]
    fn test_sleep_send() {
        fn is_send<T: Send>() {}

        is_send::<Sleep>();
    }
}
