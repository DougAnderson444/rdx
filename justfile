# for each dir in crates which has a `wit` directory in it, AND has src/bindings.rs, build it
build-wits:
  for dir in crates/*; do \
    if [ -d $dir/wit ] && [ -f $dir/src/bindings.rs ]; then \
      echo "Processing $dir"; \
      (cd $dir && cargo component build); \
      (cd $dir && cargo component build --release); \
    fi; \
  done

# build all wit examples in examples/ directory 
build-examples:
  for dir in examples/*; do \
    if [ -d $dir/wit ] && [ -f $dir/src/bindings.rs ]; then \
      echo "Processing $dir"; \
      (cd $dir && cargo component build --target wasm32-unknown-unknown); \
      (cd $dir && cargo component build --target wasm32-unknown-unknown --release); \
    fi; \
  done

build: build-examples
  cargo build

test: build
  cargo test

run: build
  cargo run

web-dev: build
  trunk serve --open

check: build
  ./check.sh

check32:
  RUSTFLAGS="--deny warnings" cargo check --target wasm32-unknown-unknown

build32:
  cargo +nightly build -Z build-std --target wasm32-unknown-unknown

force:
  cargo run --bin force-build-wasm-bins
