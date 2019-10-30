Radicle Registry
================

Experimental Radicle Registry implementation with Substrate.

See [`DEVELOPING.md`][dev-manual] for developer information.

Using the Client
----------------

The client to the registry is provided by the `radicle_registry_client` package
in the `./client` directory.

You’ll need to build the client with Rust Nightly.

To build and view the client documentation run `./scripts/build-client-docs
--open`.

Building and running the node
-----------------------------

Building the development node:

1. Get [`rustup`][rustup-install]
2. Run `./scripts/rustup-setup` to install all required `rustup` components and
   targets.
3. Install [`wasm-gc`][wasm-gc] via `cargo install --git https://github.com/alexcrichton/wasm-gc`
4. Build the node with `./scripts/build-dev-node`

You can run the node with

~~~
./scripts/run-dev-node
~~~

To reset the chain state run

~~~
./scripts/run-dev-node purge-chain
~~~

Note that `build-dev-node` will also reset the chain state.

[dev-manual]: ./DEVELOPING.md
[rustup-install]: https://github.com/rust-lang/rustup.rs#installation
[wasm-gc]: https://github.com/alexcrichton/wasm-gc

Registry Specification
--------------------

In the `registry-spec` folder, there is a Rust crate that details the Oscoin
registry specification with traits, types and a sizable amount of documentation.

It is intended to bridge the formal description of the registry from the
whitepaper with the registry's future implementation, providing a "sandbox"
with which to test and discuss design ideas before implementing them in
earnest.

The `registry-spec` crate is meant to evolve with the project, and at each point
in time its contents will reflect the team's requirements from and
understanding of the Oscoin registry.

Note that although there is no actual implementation of any function or
datatype in the crate, it compiles and is part of the build process.

### Structure

`registry-spec` is a library with three modules:
* `lib.rs`, defining the main traits with which to interact with the Oscoin
  registry
* `error.rs` defining errors that may arise when interacting with the registry.
* `types.rs`, defining the primitive types that will populate the registry state.
