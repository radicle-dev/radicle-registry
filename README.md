Radicle Registry
================

Experimental Radicle Registry implementation with Substrate.

See [`DEVELOPING.md`][dev-manual] for developer information.


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

To reset the chain data run

~~~
./scripts/run-dev-node purge-chain
~~~

Note that `build-dev-node` will purge all chain data.

[dev-manual]: ./DEVELOPING.md
[rustup-install]: https://github.com/rust-lang/rustup.rs#installation
[wasm-gc]: https://github.com/alexcrichton/wasm-gc
