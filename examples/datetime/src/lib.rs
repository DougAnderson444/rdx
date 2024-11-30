mod reactor;
use reactor::Reactor;

mod block_on;
pub use block_on::{block_on, noop_waker};

mod polling;

#[allow(warnings)]
mod bindings;

use bindings::component::plugin::host::{emit, now, subscribe_duration};
use bindings::component::plugin::types::Event;
use bindings::exports::component::plugin::run::Guest;
use bindings::wasi::io::poll::{poll, Pollable};

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
            render(ctx, `
                <Vertical>
                    <Label>Seconds since unix was invented: {{datetime}}</Label>
                    <Button on_click=ticker()>1s Refresh</Button>
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

    /// This function calls now() every second by
    fn ticker() {
        block_on(|reactor| async move {
            // we use sleep to wait for 1 second in between updates to datetime.
            let pollable = subscribe_duration(1000);
            reactor.wait_for(pollable).await;

            emit(&Event {
                name: "datetime".to_string(),
                value: Self::datetime(),
            });
        });
    }
}

bindings::export!(Component with_types_in bindings);
