Radicle Registry
================

[![Build status](https://badge.buildkite.com/dbdd1481a6275cb41c5de15e33b34c159b17a025be13116103.svg)](https://buildkite.com/monadic/radicle-registry)
[![Download master](https://api.bintray.com/packages/oscoin/radicle-registry-files/radicle-registry/images/download.svg)][package-latest-master]

Experimental Radicle Registry implementation with Substrate.

See [`DEVELOPING.md`][dev-manual] for developer information.

<!-- toc -->

- [Getting the Node](#getting-the-node)
- [Running the node](#running-the-node)
- [Using the Client](#using-the-client)
- [Account Keys](#account-keys)
- [Developing with the Client](#developing-with-the-client)
- [Using the CLI](#using-the-cli)
- [License](#license)

<!-- tocstop -->

Getting the Node
----------------

We build binaries of the node and docker images for every pushed commit. Node
binaries are available for the `x86_64-unknown-linux-gnu` target triple.

You can get the latest master build [here][package-latest-master]

You can directly download node binaries for every build from
```
https://dl.bintray.com/oscoin/radicle-registry-files/by-commit/$COMMIT_SHA/x86_64-linux-gnu/radicle-registry-node
```

You can pull a docker image of the node with
```bash
docker pull gcr.io/opensourcecoin/radicle-registry/node:<commit-sha>
```
In the image the node binary is located at `/usr/local/bin/radicle-registry-node`

To build the node from source see [`DEVELOPING.md`][dev-manual].

[package-latest-master]: https://bintray.com/oscoin/radicle-registry-files/radicle-registry/_latestVersion


Running the node
----------------

The node can be run in development mode or with a specified chain. Currently,
only the `devnet` chain is available.

For more information use the `--help` flag.

### Logging

The node prints logs to stdout in the following format

~~~
<local time> <level> <target> <msg>
~~~

You can adjust the global log level and the log level for specific targets with
the [`RUST_LOG` environment variable][rust-log-docs].

[rust-log-docs]: https://docs.rs/env_logger/0.7.1/env_logger/#enabling-logging

### Dev Mode

In development mode the node runs an isolated network with only the node as a block miner

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
package in the `./client` directory. To get started take a look at
`./client/examples/getting_started.rs`.

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


License
-------

This code is licensed under [GPL v3.0](./LICENSE.md).
