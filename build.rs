use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Uses wasmparser to determine if parsed bytes Payload version is Encoding::Component
fn is_component(bytes: &[u8]) -> bool {
    wasmparser::Parser::is_component(bytes)
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap_or_default();
    let dest_path = Path::new(&out_dir).join("codegen.rs");

    let dir_path = if cfg!(debug_assertions) {
        "target/wasm32-unknown-unknown/debug"
    } else {
        "target/wasm32-unknown-unknown/release"
    };

    let project_root = std::env::current_dir().expect("Failed to get current directory");

    let mut code = "pub static BUILTIN_PLUGINS: [(&str, &[u8]); 0] = [];".to_string();

    if let Ok(dir) = std::fs::read_dir(project_root.join(dir_path)) {
        let this_root_crate = env::var("CARGO_PKG_NAME").unwrap_or_default();

        let file_paths: Vec<PathBuf> = dir
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                path.extension()
                    // filter on 1) wasm files only, 2) not named the same as root crate
                    .filter(|&ext| {
                        let Some(bytes) = fs::read(&path).ok() else {
                            return false;
                        };
                        ext == "wasm"
                            && *path.file_stem().unwrap() != *this_root_crate
                            && is_component(&bytes)
                    })
                    .map(|_| path.to_path_buf())
            })
            .collect();

        let count = file_paths.len();

        if count == 0 {
            return;
        }

        code = format!("pub static BUILTIN_PLUGINS: [(&str, &[u8]); {count}] = [");

        for path in file_paths.iter() {
            let name = path
                .file_name()
                .unwrap_or_else(|| path.as_os_str())
                .to_str()
                .unwrap_or_else(|| path.as_os_str().to_str().unwrap_or_default());
            code.push_str(&format!(
                "(\"{name}\", include_bytes!(\"{}\")),",
                path.to_string_lossy()
            ));
        }

        code.push_str("];");

        println!("cargo:rerun-if-changed=build.rs");
        for path in file_paths.iter() {
            println!(
                "cargo:rerun-if-changed={}",
                path.to_str().unwrap_or_default()
            );
        }
    };

    if let Err(e) = fs::write(&dest_path, code) {
        eprintln!("Failed to write to {}: {}", dest_path.display(), e);
    }
}