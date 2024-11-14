#[allow(warnings)]
mod bindings;

use bindings::exports::component::plugin::run::Guest;

struct Component;

impl Guest for Component {
    /// Say hello!
    fn load() -> String {
        r#"
            <Vertical>
                <Button on_click=increment()>Increment</Button>
                <Button on_click=decrement()>Decrement</Button>
                <Label>Login is: {{count}}</Label>
            </Vertical>
        "#
        .to_string()
    }
}

bindings::export!(Component with_types_in bindings);
