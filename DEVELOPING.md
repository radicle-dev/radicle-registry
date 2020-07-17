# Developer Manual

The code is bootstrapped with the [`substrate-node-template`][node-template].

[node-template]: https://github.com/substrate-developer-hub/substrate-node-template

### Table of contents

<!-- toc -->

- [Prerequisites](#prerequisites)
- [Running development node](#running-development-node)
- [Packages](#packages)
- [Running checks and tests](#running-checks-and-tests)
- [Continuous Deployment](#continuous-deployment)
- [Make a release](#make-a-release)
- [Local devnet](#local-devnet)
- [Updating substrate](#updating-substrate)
- [Runtime updates](#runtime-updates)
- [Chain Specs](#chain-specs)
- [Updating Continuous Integration's base Docker image](#updating-continuous-integrations-base-docker-image)

<!-- tocstop -->

## Prerequisites

Follow the [README's Prerequisites guide](README.md#prerequisites).


## Running development node

For development and testing it is recommended to run the node in development
mode.

```
cargo run -p radicle-registry-node -- --dev
```

The `--dev` flag has the following effects.

* The node uses an ephemeral in-memory chain database. All chain state is lost
  when the node is killed.
* The node will use the `dev` chain spec. In particular, it will use
  `runtime-cache/latest.wasm` as the genesis runtime.
* The node will mine blocks for the author derived from the key `//Mine`.

## Packages

* `runtime` contains the Substrate runtime code that defines the ledger and
  lives on chain.
* `runtime-tests` contains a comprehensive test suite for the runtime code that
  uses the client.
* `node` contains the node code which includes the runtime code.
* `client` contains the high-level client library for interacting with the
  registry through a node and an emulator implementation.
* `cli` contains a binary for interacting with the registry node to submit
  transactions and read state.
* `core` contains basic types used throughout the Radicle Registry.
  If e.g. a trait or a datatype is used by more than one of the above packages,
  it should probably go into `core`. See `./core/README.md` for details.


## Running checks and tests

We check our code with clippy and `cargo fmt`
```
cargo clippy --workspace --all-targets -- -D clippy::all
cargo fmt --workspace -- --check
```

To check the Wasm build of the runtime run
```
cargo clippy \
  --manifest-path runtime/Cargo.toml \
  --no-default-features \
  --target wasm32-unknown-unknown \
  -- -D clippy::all
```

You can run all tests with `cargo test --workspace --all-targets`. For the
end-to-end tests you need to [build and run a dev
node](#running-development-node)

Black-box tests for the runtime logic are implemented with the client emulator
in `runtime/tests/main.rs`.

End-to-end tests that run against a real node are implemented in
`client/tests/end_to_end.rs`.

To run specific tests sequentially as opposed to the parallel default,
we use the [serial-test](https://crates.io/crates/serial_test) crate, simply
having to mark the targeted tests with `#[serial]`.

## Continuous Deployment

We’re continuously deploying master builds of the node to the devnet. We’re
using Kubernetes for this. You can find the code in `ci/deploy`.


## Make a release

To create a release from the current master branch run `./scripts/create-release`.

### Tags

Our tags are composed of the release date followed by an optional number representing the
number of releases done in that same date. For example, `2020.04.09` would be
the first release done that day. Should we need to make a new release in that
same day, it'd be tagged `2020.04.09-1`.


## Local devnet

We provide a `docker-compose` file to run a local devnet. See
`./local-devnet/README.md` for more information.


## Updating substrate

To update the revision of substrate run
~~~
./scripts/update-substrate <revision>
~~~
where `<revision>` is the new Git revision SHA.

## Runtime updates

There are special policies and processes around updates to the `runtime` package.

Updates to the runtime are tracked by the `VERSION` exported from the `runtime`
crate. `VERSION.spec_version` is always the same as
`VERSION.transaction_version`. It must match the `minor` version of the
`runtime` package. `VERSION.impl_version` must match the patch version of the
package.

Runtime updates fall into two categories: Implementation updates and semantic updates.

### Implementation updates

Implementation updates only change the implementation of the runtime but do not
affect the semantics.

Commits with implementation updates must increment the `impl_version` field of
`VERSION` and the patch version of the crate.

### Semantic updates

Semantic changes must increment the `VERSION.spec_version` and
`VERSION.transaction_version` fields and the `runtime` crate minor version and
reset the `impl_version` field and the crate patch version to `0`.

### Testing after runtime update

After all the changes are applied you should also update your local runtime cache:

```
./scripts/rebuild-runtime-cache
```

This way your local tests can run against multiple runtimes, including the fresh one:

```
./scripts/run-tests-all-runtimes
```

### Publishing runtime updates

The CI on the `master` branch builds the canonical runtime WASM.
Download it from:

```
https://storage.googleapis.com/radicle-registry-runtime/v<spec>_<impl>.wasm
e.g.
https://storage.googleapis.com/radicle-registry-runtime/v16_0.wasm
```

Next it must be deployed to an existing chain in order for runtime updates to take effect:

```
radicle-registry-cli runtime update <wasm_file> --author <sudo_key>
```

The author key must be the sudo key configured in the chain specification for
the chain that is updated.

For the runtime update transaction to be accepted the `spec_version` of the
submitted runtime must be greater than the `spec_version` of the on-chain
runtime.

Changes to the chain state must be backwards-compatibility. See the “Versioning”
section in `core/README.md` for details.

## Chain Specs

All available chain specs that the node can use are defined in
`node/src/chain_spec.rs`.

For public chains (e.g. the ffnet) the chain spec must be static and not
generated by code to prevent runtime code changes from changing the genesis
state.

To build a static chain spec in JSON format you first need to add a dynamic
chain spec with `ChainSpec::from_genesis` to `node/src/chain_spec.rs`. You can
then run

~~~
radicle-registry-node build-spec --chain foo > ./node/src/chain_spec/foo.json
~~~

Now, you can load the spec for the `foo` chain from the JSON file.

## Updating Continuous Integration's base Docker image

1. In `.buildkite/pipeline.yaml`, in value of `.test` -> `env` -> `DOCKER_IMAGE` replace image tag (last part after `:`) with a nonexistent tag (e.g. `does_not_exist`).

Example:
```
DOCKER_IMAGE: gcr.io/opensourcecoin/radicle-registry/ci-base:0d7ce69abca7dfe7dcbf26e319645405f31f4901
```
to
```
DOCKER_IMAGE: gcr.io/opensourcecoin/radicle-registry/ci-base:does_not_exist
```

2. Push the commit to the repository and let the build agent finish all the work for this commit. **Make sure that this commit is preserved!** Do not amend, squash, rebase or delete it, it should be merged unmodified into master. This way it will be easy to look up the state of the project used by the build agent.

**What happens on the build agent:** no docker image can be found for the given tag. The agent will run the full pipeline and save the docker image under a tag same as the current commit ID.

3. Copy the current commit ID and set it as the previously edited image tag.

Example:
```
DOCKER_IMAGE: gcr.io/opensourcecoin/radicle-registry/ci-base:does_not_exist
```
to
```
DOCKER_IMAGE: gcr.io/opensourcecoin/radicle-registry/ci-base:e8c699d4827ed893d8dcdab6e72de40732ad5f3c
```

**What happens on the build agent:** when any commit with this change is pushed, the build agent will find the image under the configured tag. It will reuse it instead of rebuilding and save time.
