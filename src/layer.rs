mod poll;
use poll::subscribe;

pub mod resource_table;
use resource_table::ResourceTable;

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant, SystemTime};
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant, SystemTime};

use anyhow::bail;
use core::future::Future;
use core::pin::pin;
use core::task::{Context, Poll};
pub use poll::Pollable;
use wasm_component_layer::{
    AsContext as _, Component, Engine, Func, FuncType, Instance, Linker, ListType, RecordType,
    ResourceType, Store, Value, ValueType,
};

#[cfg(not(target_arch = "wasm32"))]
pub use wasmtime_runtime_layer as runtime_layer;

#[cfg(target_arch = "wasm32")]
use js_wasm_runtime_layer as runtime_layer;

use crate::Error;

pub trait Inner {
    /// Update the state with the given key and value
    fn update(&mut self, key: &str, value: impl Into<rhai::Dynamic> + Copy);

    /// Return the [Pollable] resource
    fn table(&mut self) -> &mut ResourceTable;
}

/// The sleep resource
pub struct Sleep {
    end: Instant,
}

#[async_trait::async_trait]
impl poll::Subscribe for Sleep {
    async fn ready(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        tokio::time::sleep_until(self.end.into()).await;

        #[cfg(target_arch = "wasm32")]
        {
            send_wrapper::SendWrapper::new(async move {
                js_sleep(self.end.elapsed().as_millis() as i32)
                    .await
                    .unwrap();
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

pub fn instantiate_instance<T: Inner>(
    bytes: &[u8],
    data: T,
) -> (Instance, Store<T, runtime_layer::Engine>) {
    // Create a new engine for instantiating a component.
    let engine = Engine::new(runtime_layer::Engine::default());

    // Create a store for managing WASM data and any custom user-defined state.
    let mut store = Store::new(&engine, data);

    // Parse the component bytes and load its imports and exports.
    let component = Component::new(&engine, bytes).unwrap();
    // Create a linker that will be used to resolve the component's imports, if any.
    let mut linker = Linker::default();

    // Pollable resource type
    let pollable_resource_ty = ResourceType::new::<Pollable>(None);
    let pollable_resource_ty_clone = pollable_resource_ty.clone();

    // pollable is wasi:io/poll
    let poll_interface = linker
        .define_instance("wasi:io/poll@0.2.2".try_into().unwrap())
        .unwrap();

    poll_interface
        .define_resource("pollable", pollable_resource_ty.clone())
        .unwrap();

    // ready and block are methods on the pollable resource, "[method]pollable.ready" and "[method]pollable.block"
    //ready: func() -> bool;
    poll_interface
        .define_func(
            "[method]pollable.ready",
            Func::new(
                &mut store,
                FuncType::new(
                    [ValueType::Borrow(pollable_resource_ty.clone())],
                    [ValueType::Bool],
                ),
                move |mut store, params, results| {
                    let Value::Borrow(res) = &params[0] else {
                        bail!(format!("Incorrect input type, found {:?}", params[0]));
                    };

                    // need to go from Resource to Pollable type
                    let index = {
                        let binding = store.as_context();
                        let pollable = res.rep::<Pollable, _, _>(&binding).unwrap();
                        pollable.index
                    };

                    // this calls .ready() on the inner value (ie. Sleep) which impl Subscribe
                    // We take Pollable, get the Sleep resource from the index,
                    // which was saved under index after .push()?
                    // then get the inner
                    // from the sleep_resource.
                    let ctx = store.as_context();
                    let pollable = res.rep::<&mut Pollable, _, _>(&ctx)?;
                    let ready = (pollable.make_future)(store.data_mut().table().get_any_mut(index));

                    let mut fut = pin!(ready);
                    let waker = async_runtime_unknown::noop_waker();
                    let mut cx = Context::from_waker(&waker);

                    // Poll the future once
                    let poll_result = fut.as_mut().poll(&mut cx);

                    // Check the result
                    let ready = matches!(poll_result, Poll::Ready(()));

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
                move |store, params, results| {
                    todo!();
                    Ok(())
                },
            ),
        )
        .unwrap();

    // poll: func(in: list<borrow<pollable>>) -> list<u32>;
    poll_interface
        .define_func(
            "poll",
            Func::new(
                &mut store,
                FuncType::new(
                    [ValueType::List(ListType::new(ValueType::Borrow(
                        pollable_resource_ty_clone,
                    )))],
                    [ValueType::List(ListType::new(ValueType::U32))],
                ),
                move |store, params, results| {
                    todo!();
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
    host_interface
        .define_func(
            "sleep",
            Func::new(
                &mut store,
                FuncType::new(
                    [ValueType::U64],
                    [ValueType::Own(pollable_resource_ty.clone())],
                ),
                move |mut store, params, results| {
                    // sleep should take these millis and turn them into pollable
                    // then return the pollable

                    let Value::U64(millis) = params[0] else {
                        panic!("Incorrect input type.")
                    };

                    let sleep = Sleep {
                        end: Instant::now() + Duration::from_millis(millis),
                    };

                    let sleep_idx = store.data_mut().table().push(sleep)?;

                    //let table = store.data_mut().table();

                    let pollable_owned = subscribe::<_, Sleep>(&mut store, sleep_idx)?;

                    //let pollable_owned =
                    //    subscribe(&mut store.data_mut().table(), pollable_resource_ty.clone())?;

                    results[0] = Value::Own(pollable_owned);
                    Ok(())
                },
            ),
        )
        .unwrap();

    (linker.instantiate(&mut store, &component).unwrap(), store)
}

/// Plugin struct to store some state
pub struct LayerPlugin<T: Inner> {
    pub(crate) store: Store<T, runtime_layer::Engine>,
    raw_instance: wasm_component_layer::Instance,
}

impl<T: Inner> LayerPlugin<T> {
    /// Creates a new plugin instance with the given name and bytes
    pub fn new(bytes: &[u8], data: T) -> Self {
        let (instance, store) = instantiate_instance(bytes, data);

        Self {
            store,
            raw_instance: instance,
        }
    }

    /// Calls the given function name with the given parameters
    pub fn call(&mut self, name: &str, arguments: &[Value]) -> Result<Value, Error> {
        let export_instance = self
            .raw_instance
            .exports()
            .instance(&"component:plugin/run".try_into()?)
            .ok_or(Error::InstanceNotFound)?;

        let func = export_instance
            .func(name)
            .ok_or_else(|| Error::FuncNotFound(name.to_string()))?;

        const CAPACITY: usize = 1;
        let mut results = [Value::Bool(false); CAPACITY];
        func.call(&mut self.store, arguments, &mut results)?;

        Ok(results[0].clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct State {
        count: rhai::Dynamic,
        pollable: Option<Pollable>,
        table: ResourceTable,
    }

    impl Inner for State {
        fn update(&mut self, key: &str, value: impl Into<rhai::Dynamic> + Copy) {
            println!("updating {}: {}", key, value.into());
            // set count to value
            // TODO: Chg hard code into rhai scope change
            if key == "count" {
                self.count = value.into();
            }
        }

        fn table(&mut self) -> &mut resource_table::ResourceTable {
            &mut self.table
        }
    }

    #[test]
    fn test_instantiate_instance() {
        const WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/debug/counter.wasm");

        let data = State {
            count: 0.into(),
            pollable: None,
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

        let data = State {
            count: 0.into(),
            pollable: None,
            ..Default::default()
        };

        let mut plugin = LayerPlugin::new(WASM, data);

        let _ = plugin.call("increment", &[]).unwrap();

        // current
        let result = plugin.call("current", &[]).unwrap();

        assert_eq!(result, Value::S32(1));
    }
}
