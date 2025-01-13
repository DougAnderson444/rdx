#[allow(warnings)]
#[cfg_attr(rustfmt, rustfmt_skip)]
mod bindings;

use bindings::component::plugin::host::{emit, random_byte};
use bindings::component::plugin::types::Event;
use bindings::exports::component::plugin::run::Guest;

use rand::RngCore;
use std::fmt::Write;

/// Custom function to use the import for random byte generation.
///
/// We do this is because "js" feature is incompatible with the component model
/// if you ever got the __wbindgen_placeholder__ error when trying to use the `js` feature
/// of getrandom,
fn imported_random(dest: &mut [u8]) -> Result<(), getrandom::Error> {
    // iterate over the length of the destination buffer and fill it with random bytes
    (0..dest.len()).for_each(|i| {
        dest[i] = random_byte();
    });

    Ok(())
}

getrandom::register_custom_getrandom!(imported_random);

struct Component;

impl Guest for Component {
    fn load() -> String {
        r#"
        // call the system function `render` on the template with the ctx from scope
        if !is_def_var("number") {
            render(`
                <div>
                    <span>Click to generate a Random number</span>
                    <button data-on-click="random()">Generate</button>
                </div>
            `)
        } else {
            render(`
                <div>
                    <span>Random number is: {{number}}</span>
                    <button data-on-click="random()">Re-Generate</button>
                </div>
            `)
        }
        "#
        .to_string()
    }

    // Return the random number
    fn random() -> Vec<u8> {
        let mut buf = vec![0u8; 32];

        // could also use getrandom crate
        // getrandom::getrandom(&mut buf).unwrap();

        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut buf);

        // converts the vec to a string, use fold and write!() to format the string
        let value = buf.iter().fold(String::new(), |mut acc, &x| {
            write!(acc, "{:02x}", x).unwrap();
            acc
        });

        emit(&Event {
            name: "number".to_string(),
            value,
        });

        buf
    }
}

bindings::export!(Component with_types_in bindings);
