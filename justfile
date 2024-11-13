build-couter:
  @echo "Building example counter"
  cargo component build --manifest-path examples/counter/Cargo.toml --target wasm32-unknown-unknown
  # cargo component build --manifest-path examples/counter/Cargo.toml --release

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
     cargo component build --manifest-path=$dir/Cargo.toml; \
     cargo component build --manifest-path=$dir/Cargo.toml --release; \
   fi \
  done

test: build-couter
  cargo test

run: build-couter
  cargo run

web-dev: build-couter
  trunk serve --open
