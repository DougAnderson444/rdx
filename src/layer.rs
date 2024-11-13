use wasm_component_layer::{
    Component, Engine, Func, FuncType, Instance, Linker, RecordType, Store, Value, ValueType,
};

#[cfg(not(target_arch = "wasm32"))]
pub use wasmtime_runtime_layer as runtime_layer;

#[cfg(target_arch = "wasm32")]
use js_wasm_runtime_layer as runtime_layer;

use crate::Error;

pub trait Inner {
    /// Update the state with the given key and value
    fn update(&mut self, key: &str, value: impl Into<rhai::Dynamic> + Copy);
}

// pub struct Layer<T: Inner> {
//     linker: Linker,
//     engine: Engine<runtime_layer::Engine>,
//     store: Store<T, runtime_layer::Engine>,
// }
//
// impl<T: Inner> Layer<T> {
//     pub fn new(data: T) -> Self {
//         let engine = Engine::new(runtime_layer::Engine::default());
//         let store = Store::new(&engine, data);
//         Self {
//             linker: Linker::default(),
//             engine,
//             store,
//         }
//     }
//
//     /// Adds the import to the linker
//     pub fn add_to_linker(&mut self, interface: &str, name: &str) {
//         let host_interface = self
//             .linker
//             .define_instance(interface.try_into().unwrap())
//             .unwrap();
//
//         // params is a record with name and value
//         let record = RecordType::new(
//             Some(TypeIdentifier::new(
//                 "event",
//                 Some(InterfaceIdentifier::new(
//                     "component:plugin".try_into().unwrap(),
//                     "types",
//                 )),
//             )),
//             vec![("name", ValueType::String), ("value", ValueType::String)],
//         )
//         .unwrap();
//
//         tracing::info!("Record {:?}", record);
//
//         let params = ValueType::Record(record);
//         let results = [];
//
//         host_interface
//             .define_func(
//                 name,
//                 Func::new(
//                     &mut self.store,
//                     FuncType::new([params], results),
//                     move |mut store, params, _results| {
//                         if let Value::String(name) = &params[0] {
//                             if let Value::String(value) = &params[1] {
//                                 store.data_mut().update(name, &**value);
//                             }
//                         }
//
//                         Ok(())
//                     },
//                 ),
//             )
//             .unwrap();
//     }
//
//     pub fn instantiate(&mut self, component: Component) -> Instance {
//         self.linker
//             .instantiate(&mut self.store, &component)
//             .unwrap()
//     }
// }

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
                    // [Record(Record { fields: [("name", String("count")), ("value", String("1"))], ty: RecordType { fields: [(0, "name", String), (1, "value", String)], name: Some(component:plugin/types.event) } })]
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

    (linker.instantiate(&mut store, &component).unwrap(), store)
}

/// Plugin struct to store some state
pub struct LayerPlugin<T: Inner> {
    store: Store<T, runtime_layer::Engine>,
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
    }

    impl Inner for State {
        fn update(&mut self, key: &str, value: impl Into<rhai::Dynamic> + Copy) {
            println!("updating {}: {}", key, value.into());
            // set count to value
            if key == "count" {
                self.count = value.into();
            }
        }
    }

    #[test]
    fn test_instantiate_instance() {
        const WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/debug/counter.wasm");

        let data = State { count: 0.into() };

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
        let count_i64 = count.as_int().unwrap();
        assert_eq!(count_i64, 1);
    }

    // test Plugin struct
    #[test]
    fn test_plugin() {
        const WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/debug/counter.wasm");

        let data = State { count: 0.into() };

        let mut plugin = LayerPlugin::new(WASM, data);

        let _ = plugin.call("increment", &[]).unwrap();

        // current
        let result = plugin.call("current", &[]).unwrap();

        assert_eq!(result, Value::S32(1));
    }
}
