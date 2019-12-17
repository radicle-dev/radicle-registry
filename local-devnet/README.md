Local DevNet
============

This directory provides tools to run and experiment with a local devnet.

To run the devnet you need to build the registry node with
```bash
cargo build --release -p radicle-registry-node
```
Then you can start the network with `docker-compose up`

### Overview

* Devnet consists of three authority nodes running Aura.
* Grandpa (for finalization) is currently enabled.
* Chain data of nodes is persisted in Docker volumes.
* We expose the RPC API of the first node on the standard RPC API port.
* See `local_devnet()` for chain spec in `../node/src/chain_spec.rs`.
* You need to build the node using a target that is compatible with Ubuntu 19.10
  (Eoan).
* Aura keys used by the nodes are located in `./keystore` and created with
  `subkey`.
