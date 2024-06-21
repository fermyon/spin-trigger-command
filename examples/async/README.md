# Async Sample

This example illustrates how you can run asyncoperations within your WebAssembly module leveraging the `run` function provided by the `spin-executor` crate.

The sample app wraps async code (an outbound HTTP request) as shown below:


```rust
spin_executor::run(async move {
  // async code goes here
})
```

Upon running the Spin App (using `spin up`) it will send an HTTP request to `https://myip.fermyon.app` and print your public IP address to `stdout`.


## Prerequisites

You need to have the following installed on your machine:

- Latest `spin` CLI
  - The `command` trigger plugin
- Rust must be installed on your machine and the `wasm32-wasi` target
  - `cargo-component`


## Compiling and running the sample

The Spin Manifest contains necessary commands for building and running this app. Allowing you to simply use the `spin` CLI:

```bash
# Compile the App
spin build

# Run the App
spin up
```

