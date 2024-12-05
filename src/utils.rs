// /// Utility function to get the workspace dir
// pub fn workspace_dir() -> PathBuf {
//     let output = std::process::Command::new(env!("CARGO"))
//         .arg("locate-project")
//         .arg("--workspace")
//         .arg("--message-format=plain")
//         .output()
//         .unwrap()
//         .stdout;
//     let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
//     cargo_path.parent().unwrap().to_path_buf()
// }
//
// /// Gets the wasm bytes path from the given package name
// /// Will convert the package name to snake case if it contains a hyphen
// pub fn get_wasm_path(pkg_name: &str) -> Result<PathBuf, super::Error> {
//     let pkg_name = pkg_name.replace('-', "_");
//     let workspace = workspace_dir();
//     let wasm_path = format!("target/wasm32-unknown-unknown/debug/{pkg_name}.wasm");
//     Ok(workspace.join(wasm_path))
// }

/**/

//         // Example RDX source with components
//         let rdx_source = r#"
//
// let message = "Default message";
//
// // Define a component render function
// fn render(message) {
//     let msg = message;
//
//     `
//     <Horizontal>
//         <Label size="large" color="blue">${message}</Label>
//
//         <Label>The current count is: ${count}</Label>
//
//         <Button onClick="increment" color="green">Increment</Button>
//         <Button onClick="decrement" color="red">Decrement</Button>
//
//         <Button color="gray">
//             <Label size="small">This is a demo of RDX - Rhai + UI components!</Label>
//         </Button>
//     </Horizontal>
//     `
// }
//
// // Define actions
// fn increment() {
//     count += 1;
//     render(message)
// }
//
// fn decrement() {
//     count -= 1;
//     render(message)
// }
//
// // Initial render
// render(message)
// "#
//         .to_string();
