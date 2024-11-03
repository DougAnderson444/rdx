#[allow(warnings)]
mod bindings;

use std::cell::RefCell;

use bindings::component::plugin::types::Event;
use bindings::emit;
use bindings::exports::component::plugin::provider;
use bindings::Guest;

pub struct Counter {
    count: RefCell<i32>,
}

bindings::export!(Counter with_types_in bindings);

impl provider::Guest for Counter {
    type Counter = Self;
}

impl Guest for Counter {
    /// Say hello!
    fn load() -> String {
        r#"
            <Vertical>
                <Button on_click="increment()">Increment</Button>
                <Button on_click="decrement()">Decrement</Button>
                <Label>{count}</Label>
            </Vertical>
        "#
        .to_string()
    }
}

impl provider::GuestCounter for Counter {
    /// Create a new counter.
    fn new() -> Self {
        Counter {
            count: RefCell::new(0),
        }
    }

    fn increment(&self) -> i32 {
        // self.count + 1
        let mut count = self.count.borrow_mut();
        *count += 1;
        emit(Event { count: *count });
        *count
    }

    fn decrement(&self) -> i32 {
        let mut count = self.count.borrow_mut();
        *count -= 1;
        emit(Event { count: *count });
        *count
    }
}
