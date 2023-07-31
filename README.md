# API Rate Limiting

This is a demostration of a simple API rate limiting implementation in Rust.

## Building and Running

Prerequisite: A Rust toolchain installation is required to build, run and test the code.

For building the code, simply run `cargo build`.

For running the API server, use `cargo run`.

For running the test cases, use `cargo test`.

There also exists test case which test the timing behaviour of the current application configs, which needs
around 2 minutes to complete (for waiting the actual timeout). This is by default ignored and will
not be runned unless specifically requested. To run this as well, use `cargo test -- --ignored`.
