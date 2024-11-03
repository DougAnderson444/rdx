build-couter:
  @echo "Building example counter"
  cargo component build --manifest-path examples/counter/Cargo.toml
  # cargo component build --manifest-path examples/counter/Cargo.toml --release

run: build-couter
  cargo run
