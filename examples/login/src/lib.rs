#[allow(warnings)]
#[cfg_attr(rustfmt, rustfmt_skip)]
mod bindings;

use bindings::component::plugin::host::emit;
use bindings::component::plugin::types::Event;
use bindings::exports::component::plugin::run::Guest;

struct Component;

// static LOGIN_SCREEN: &str = r#"
//     <div>
//         <input on_change=username()>{{username}}</input>
//         <input on_change=password()>{{password}}</input>
//         <Button on_click=login()>Login</Button>
//     </div>
// "#;
//
// static LOGGED_IN_SCREEN: &str = r#"
//     <div>
//         <Text>Welcome, {{username}}!</Text>
//         <Button on_click=logout()>Logout</Button>
//     </div>
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
        render(`
            <div id="login1">
                <input value="{{username}}" data-on-change="username(username)">
                <input value="{{password}}" data-on-change="password(password)">
                <button class="" data-on-click=login()>Login</button>
            </div>
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
