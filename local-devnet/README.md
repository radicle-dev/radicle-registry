Local DevNet
============

This directory provides tools to run and experiment with a local devnet.

To run the devnet you need to build the registry node with
```bash
GENESIS_CHAIN=local-devnet cargo build -p radicle-registry-node --release
```
Then you can start the network with `docker-compose up`

Overview
--------

* Devnet consists of three miner nodes.
* Chain data of nodes is persisted in Docker volumes.
* We expose the RPC API of the first node on the standard RPC API port.
* See chain spec details in file `../node/src/chain_spec.rs`, in function `chain_spec()` for `#[cfg(genesis_chain="local-devnet")]`.
* You need to build the node using a target that is compatible with Ubuntu 19.10
  (Eoan).

Monitoring
----------

The local DevNet provides Grafana dashboards for the nodes at
`http://localhost:9004`. The login credentials are `admin:admin`.

Monitoring can be configured with `./prometheus.yaml`, `./grafana-datasources.yaml`, `./grafana-dashboards.yaml`, and `./grafana-dashboards`.
