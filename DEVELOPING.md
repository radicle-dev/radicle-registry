Developer Manual
================

The code is bootstrapped with the [`substrate-node-template`][node-template].

[node-template]: https://github.com/substrate-developer-hub/substrate-node-template

Running development node
------------------------

~~~
BUILD_DUMMY_WASM_BINARY=0 cargo build --release -p radicle_registry_node
./scripts/run-dev-node
~~~

The run script purges the chain data before running to avoid consensus issues.
This means that state is not persisted between runs.

Packages
--------

* `runtime` contains the Substrate runtime code that defines the ledger and
  lives on chain.
* `node` contains the node code which includes the runtime code.
