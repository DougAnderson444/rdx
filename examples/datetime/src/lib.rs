#[allow(warnings)]
mod bindings;

use bindings::component::plugin::host::{emit, now};
use bindings::component::plugin::types::Event;
use bindings::exports::component::plugin::run::Guest;

struct Component;

impl Guest for Component {
    fn load() -> String {
        let now = Self::datetime();

        // send the variable and it's value to Rhai scope
        emit(&Event {
            name: "datetime".to_string(),
            value: now,
        });

        r#"
        // call the system function `render` on the template with the ctx from scope
            render(ctx, `
                <Vertical>
                    <Label>Date Time is {{datetime}}</Label>
                </Vertical>
            `)
        "#
        .to_string()
    }

    fn datetime() -> String {
        let now = now();

        // convert to date time string
        let datetime = chrono::DateTime::from_timestamp(now, 0).unwrap();

        datetime.to_string()
    }
}

bindings::export!(Component with_types_in bindings);
