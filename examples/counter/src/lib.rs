#![recursion_limit = "512"]
#[allow(warnings)]
mod bindings;

use html_to_egui::{Action, Button, DivSelectors, Division, Handler, Paragraph};
use std::sync::{LazyLock, Mutex};

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
        let increment_button = Button::new_with_func(
            Action::OnClick,
            // the function name must match the wasm function name in this file
            // converted into kebab-case (so my_function becomes my-function)
            Handler::builder()
                .named("increment-count".to_string())
                .build(),
        )
        .text("Increment")
        .build();

        let decrement_button = Button::new_with_func(
            Action::OnClick,
            Handler::builder().named("decrement".to_string()).build(),
        )
        .text("Decrement")
        .build();

        let no_def_count_para = Paragraph::builder()
            .text("Click to Start counting!")
            .build();

        let def_count_para = Paragraph::builder().text("Count is: {{count}}").build();

        let def_count = Division::builder()
            .push(increment_button.clone())
            .push(decrement_button.clone())
            .push(def_count_para)
            .class(DivSelectors::FlexRow)
            .build()
            .to_string();

        let no_def_count = Division::builder()
            .push(increment_button)
            .push(decrement_button)
            .push(no_def_count_para)
            .build()
            .to_string();

        format!(
            r#"
            // call the system function `render` on the template with the ctx from scope
            
            // wasm functions are bound to the rhai script on load?
            // let count = current(); // TODO: register all exported functions with rhai engine
            // let count = 0;

            if !is_def_var("count") || count == "0" {{

                render(`{no_def_count}`)

            }} else {{

                render(`{def_count}`)

            }}
        "#
        )
    }

    /// Increment the count
    fn increment_count() -> i32 {
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

#[cfg(test)]
mod tests {

    #[test]
    fn test_string_interpolation() {
        let def_count = "a {{test}} string";

        let test = format!(
            r#"
        // call the system function `render` on the template with the ctx from scope
        
        // wasm functions are bound to the rhai script on load?
        // let count = current(); // TODO: register all exported functions with rhai engine
        // let count = 0;

        if !is_def_var("count") || count == "0" {{

            render(`{def_count}`)

        }} else {{

            render(`
                <div class="flex flex-row">
                    <button data-on-click="increment()">Increment</button>
                    <button data-on-click="decrement()">Decrement</button>
                    <!-- inline template vars need double double {{}}'s -->
                    <span>Count is: {{{{count}}}}</span>
                </div>
            `)

        }}
        "#
        );

        eprintln!("{}", test);
    }
}
