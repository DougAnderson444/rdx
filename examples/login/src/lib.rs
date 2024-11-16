#[allow(warnings)]
mod bindings;

use bindings::component::plugin::host::emit;
use bindings::component::plugin::types::Event;
use bindings::exports::component::plugin::run::Guest;

struct Component;

// static LOGIN_SCREEN: &str = r#"
//     <Vertical>
//         <TextEdit on_change=username()>{{username}}</TextEdit>
//         <TextEdit on_change=password()>{{password}}</TextEdit>
//         <Button on_click=login()>Login</Button>
//     </Vertical>
// "#;
//
// static LOGGED_IN_SCREEN: &str = r#"
//     <Vertical>
//         <Text>Welcome, {{username}}!</Text>
//         <Button on_click=logout()>Logout</Button>
//     </Vertical>
// "#;

impl Guest for Component {
    // fn rdx() -> String {
    //     // This is the Rhai script with embedded RX in it.
    //     r#"
    //             let username = "";
    //             let password = "";
    //
    //             let logged_in = false;
    //
    //             if !this.logged_in {
    //                 // the template gets parsed into an AST
    //                 login_screen
    //             } else {
    //                 logged_in_screen
    //             }
    //
    //         "#
    //     .to_string()
    // }

    /// Return ALL templates so that the AST can be built once and used over
    /// and over again without having to re-parse the scripts.
    fn load() -> String {
        r#"
        // call the system function `render` on the template with the ctx from scope
        render(ctx, `
            <Vertical>
                <TextEdit on_change=username(username)>{{username}}</TextEdit>
                <TextEdit on_change=password(password)>{{password}}</TextEdit>
                <Button on_click=login()>Login</Button>
            </Vertical>
        `)
        "#
        .to_string()
    }

    fn login(username: String, password: String) {
        let evt = Event {
            name: "login".to_string(),
            value: format!("{}: {}", username, password),
        };
        emit(&evt);
    }
}

bindings::export!(Component with_types_in bindings);
