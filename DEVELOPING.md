Developer Manual
================

The code is bootstrapped with the [`substrate-node-template`][node-template].

[node-template]: https://github.com/substrate-developer-hub/substrate-node-template

### Table of contents

<!-- toc -->

- [Build requirements](#build-requirements)
- [Running development node](#running-development-node)
- [Packages](#packages)
- [Running checks and tests](#running-checks-and-tests)
- [Continuous Deployment](#continuous-deployment)
- [Local devnet](#local-devnet)
- [Updating substrate](#updating-substrate)
- [Updating Continuous Integration's base Docker image](#updating-continuous-integrations-base-docker-image)

<!-- tocstop -->

Build requirements
------------------

1. Get [`rustup`][rustup-install]
2. Run `./scripts/rustup-setup` to install all required `rustup` components and
   targets.

[rustup-install]: https://github.com/rust-lang/rustup.rs#installation


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


Local devnet
------------

We provide a `docker-compose` file to run a local devnet. See
`./local-devnet/README.md` for more information.


Updating substrate
------------------

To update the revision of substrate run
~~~
./scripts/update-substrate REV
~~~
where `REV` is the new Git revision SHA.


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
