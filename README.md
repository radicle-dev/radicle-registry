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

Using the CLI
-------------

We provide a CLI to talk read and update the ledger in the `cli` directory. To
learn more run `cargo run -p radicle_registry_cli -- --help`.


[dev-manual]: ./DEVELOPING.md
[rustup-install]: https://github.com/rust-lang/rustup.rs#installation
[wasm-gc]: https://github.com/alexcrichton/wasm-gc
