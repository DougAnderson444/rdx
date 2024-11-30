#[allow(warnings)]
mod bindings;

use bindings::component::plugin::host::{emit, now};
use bindings::component::plugin::types::Event;
use bindings::exports::component::plugin::run::Guest;

struct Component;

impl Guest for Component {
    fn load() -> String {
        // send the variable and it's value to Rhai scope
        emit(&Event {
            name: "datetime".to_string(),
            value: Self::datetime(),
        });

        r#"
        // call the system function `render` on the template with the ctx from scope
            render(`
                <Vertical>
                    <Label>Seconds since unix was invented: {{datetime}}</Label>
                </Vertical>
            `)
        "#
        .to_string()
    }

    fn datetime() -> String {
        let now = now();

        // convert to date time string
        let datetime = now.to_string();

        datetime.to_string()
    }
}

bindings::export!(Component with_types_in bindings);
