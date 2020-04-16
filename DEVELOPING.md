Developer Manual
================

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
- [Updating Continuous Integration's base Docker image](#updating-continuous-integrations-base-docker-image)

<!-- tocstop -->

Prerequisites
-------------

Follow the [README's Prerequisites guide](README.md#prerequisites).


Running development node
------------------------

~~~
./scripts/build-dev-node
./scripts/run-dev-node
~~~

The build script purges the chain data before running to avoid consensus issues.

To purge the chain state manually run

~~~
./scripts/run-dev-node purge-chain
~~~


Packages
--------

* `runtime` contains the Substrate runtime code that defines the ledger and
  lives on chain.
* `node` contains the node code which includes the runtime code.
* `client` contains the high-level client library for interacting with the
  registry through a node and an emulator implementation.
* `cli` contains a binary for interacting with the registry node to submit
  transactions and read state.
* `core` contains basic types used throughout the Radicle Registry.
  If e.g. a trait or a datatype is used by more than one of the above packages,
  it should probably go into `core`. See `./core/README.md` for details.


Running checks and tests
------------------------

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

Continuous Deployment
---------------------

We’re continuously deploying master builds of the node to the devnet. We’re
using Kubernetes for this. You can find the code in `ci/deploy`.


Make a release
--------------

1. Download the latest binaries for the targeted master build [in Builkite](https://buildkite.com/monadic/radicle-registry/builds?branch=master)
    - Select the targeted build
    - Expand the `Test ci/run` section
    - Open the `Artifacts` tab
    - Download the following binaries:
        - `radicle-registry-cli`
        - `radicle-registry-node`
2. Create a new release
    - Open the [Radicle Registry Github Releases page](https://github.com/radicle-dev/radicle-registry/releases)
    - Define the tag version (see [Tags](#tags))
    - Select the right target
    - Add a release title
    - Describe the changes included in this release to users
    - Attach the binaries downloaded in step 1.

### Tags

Our tags are composed of the release date followed by a number representing the
number of releases done in that same date. For example, `2020-04-09-0` would
be the first release done that day. Should we need to make a new release in
that same day, it'd be tagged `2020-04-09-1`.


Local devnet
------------

We provide a `docker-compose` file to run a local devnet. See
`./local-devnet/README.md` for more information.


Updating substrate
------------------

To update the revision of substrate run
~~~
./scripts/update-substrate <revision>
~~~
where `<revision>` is the new Git revision SHA.

Building a WASM runtime binary
------------------------------

### Building a runtime for the genesis

To build a genesis WASM runtime binary run
~~~
./scripts/build-genesis-runtime-wasm
~~~
This will create or update the `runtime/genesis_runtime.wasm` file.
It will be then used as the genesis WASM runtime in all consecutive compilations of the node.

Remember that this operation should **never** be done after starting the public network!
It will make the node incompatible with any older nodes on the network.

### Building a runtime for an update

To build a WASM runtime binary meant for packing into a transaction and broadcasting on the network run
~~~
./scripts/build-runtime-wasm <output_file>
~~~

Updating Continuous Integration's base Docker image
---------------------------------------------------

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
