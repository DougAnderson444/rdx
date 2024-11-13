#[allow(warnings)]
mod bindings;

use std::sync::LazyLock;
use std::sync::Mutex;

use bindings::component::plugin::host::emit;
use bindings::component::plugin::types::Event;
use bindings::exports::component::plugin::run::Guest;

static COUNT: LazyLock<Mutex<i32>> = LazyLock::new(|| Mutex::new(0i32));
// static COUNTER: LazyLock<Counter> = LazyLock::new(Counter::new);

bindings::export!(Counter with_types_in bindings);

struct Counter;

// struct Counter {
//     count: i32,
// }

// impl Counter {
//     fn new() -> Self {
//         Self { count: 0 }
//     }
// }

impl Guest for Counter {
    /// Say hello!
    fn load() -> String {
        r#"
            <Vertical>
                <Button on_click=increment()>Increment</Button>
                <Button on_click=decrement()>Decrement</Button>
                <Label>Count is: {{count}}</Label>
            </Vertical>
        "#
        .to_string()
    }

    /// Increment the count
    fn increment() -> i32 {
        // let mut count = COUNTER.count;
        // count += 1;
        //
        //
        // count

        let mut count = COUNT.lock().unwrap();
        *count += 1;

        emit(&Event {
            name: "count".to_string(),
            value: (count).to_string(),
        });

        *count
    }

    /// Decrement the count
    fn decrement() -> i32 {
        // let mut count = COUNTER.count;
        // count -= 1;
        // count

        let mut count = COUNT.lock().unwrap();
        *count -= 1;

        emit(&Event {
            name: "count".to_string(),
            value: (count).to_string(),
        });

        *count
    }

    fn current() -> i32 {
        // COUNTER.count

        *COUNT.lock().unwrap()
    }
}
