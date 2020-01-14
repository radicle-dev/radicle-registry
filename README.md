Radicle Registry
================

Experimental Radicle Registry implementation with Substrate.

See [`DEVELOPING.md`][dev-manual] for developer information.

<!-- toc -->

- [Getting the Node](#getting-the-node)
- [Running the node](#running-the-node)
- [Using the Client](#using-the-client)
- [Account Keys](#account-keys)
- [Developing with the Client](#developing-with-the-client)
- [Using the CLI](#using-the-cli)
- [Registry Specification](#registry-specification)
- [License](#license)

<!-- tocstop -->

Getting the Node
----------------

We build binaries of the node and docker images for every pushed commit.

Node binaries are available for the `x86_64-unknown-linux-gnu` target triple.
You can download them from
```
https://dl.bintray.com/oscoin/radicle-registry-files/git-<COMMIT_SHA>/x86_64-linux-gnu/radicle-registry-node
```

You can pull a docker image of the node with
```bash
docker pull gcr.io/opensourcecoin/radicle-registry/node:<commit-sha>
```
In the image the node binary is located at `/usr/local/bin/radicle-registry-node`

To build the node from source see [`DEVELOPING.md`][dev-manual].


Running the node
----------------

The node can be run in development mode or with a specified chain. Currently,
only the `devnet` chain is available.

For more information use the `--help` flag.

### Dev Mode

In development mode the node runs an isolated network with only the node as an
Aura validator and block producer.

~~~
radicle-registry-node --dev
~~~

To reset the chain state and start fresh run

~~~
radicle-registry-node purge-chain --dev
~~~

### Devnet

We host a devnet that you can connect to. To join you need to use the most
recent pre-built binary (see “Getting the node”).

~~~
radicle-registry-node --chain devnet
~~~

We are frequently resetting the devnet blockchain. If you local node is not
syncing blocks download the most recent version and run `radicle-registry-node
purge-chain --chain devnet`.


Using the Client
----------------

The client for the registry node is provided by the `radicle-registry-client`
package in the `./client` directory.

You’ll need to build the client with Rust Nightly.

To build and view the client documentation run `./scripts/build-client-docs
--open`.

You can find examples in the `./client/examples` directory.


Account Keys
------------

We use Ed25519 keys for accounts. Key creation and handling functionality is
provided by the `radicle-registry-client::ed25519` module. To use this module
you will likely need to import the `radicle-registry-client::CryptoPair` and
`radicle-registry-client::CryptoPublic` traits.

You can create key pairs using [`CryptoPair::generate()`][api-pair-generate]
```rust
use radicle-registry-client::{ed25519, CryptoPair};
let (key, seed) = ed25519::Pair::generate();
```

To create keys from human readable strings, use [`CryptoPair::from_string`][api-pair-from-string].
```rust
use radicle-registry-client::{ed25519, CryptoPair};
let alice = ed25519::Pair::from_string("//Alice", None);
```

The `radicle-registry-client::ed25519` module and the crypto traits are
re-exports from [`substrate_primitives::ed25519`][api-ed25519] and
[`substrate_primitives::crypto`][api-crypto], respectively

[api-ed25519]: https://crates.parity.io/substrate_primitives/ed25519/index.html
[api-crypto]: https://crates.parity.io/substrate_primitives/crypto/index.html
[api-pair-generate]: https://crates.parity.io/substrate_primitives/crypto/trait.Pair.html#method.generate
[api-pair-from-string]: https://crates.parity.io/substrate_primitives/crypto/trait.Pair.html#method.from_string


Developing with the Client
--------------------------

For development you can create a ledger emulator with
`radicle-registry-client::Client::new_emulator()`. Instead of connecting to a
node this client runs the ledger in memory. See the API docs for more
information.


Using the CLI
-------------

We provide a CLI to talk read and update the ledger in the `cli` directory. To
learn more run `cargo run -p radicle-registry-cli -- --help`.


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

License
-------

This code is licensed under [GPL v3.0](./LICENSE.md).
