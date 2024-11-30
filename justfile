# for each dir in crates which has a `wit` directory in it, AND has src/bindings.rs, build it
build-wits:
 for dir in crates/*; do \
    if ([ -d $dir/wit ] && [ -f $dir/src/bindings.rs ]); then \
     cargo component build --manifest-path=$dir/Cargo.toml; \
     cargo component build --manifest-path=$dir/Cargo.toml --release; \
   fi \
 done

# build all wit examples in examples/ directory 
build-examples:
  for dir in examples/*; do \
    if ([ -d $dir/wit ] && [ -f $dir/src/bindings.rs ]); then \
     cargo component build --manifest-path=$dir/Cargo.toml --target wasm32-unknown-unknown ; \
     cargo component build --manifest-path=$dir/Cargo.toml --target wasm32-unknown-unknown --release; \
   fi \
  done

build: build-wits build-examples

test: build
  cargo test

run: build
  cargo run

web-dev: build
  trunk serve --open

check: build
  ./check.sh

check32:
  cargo check --target wasm32-unknown-unknown

force:
  cargo run --bin force-build-wasm-bins
