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
- [Changelog](#changelog)
- [Continuous Deployment](#continuous-deployment)
- [Local devnet](#local-devnet)
- [Updating substrate](#updating-substrate)
- [Updating Continuous Integration's base Docker image](#updating-continuous-integrations-base-docker-image)
- [Git Flow](#git-flow)

<!-- tocstop -->

Build requirements
------------------

1. Get [`rustup`][rustup-install]
2. Run `./scripts/rustup-setup` to install all required `rustup` components and
   targets.
3. Install [`wasm-gc`][wasm-gc] via `cargo install --git https://github.com/alexcrichton/wasm-gc`

[rustup-install]: https://github.com/rust-lang/rustup.rs#installation
[wasm-gc]: https://github.com/alexcrichton/wasm-gc


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
  it should probably go into `core`.


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
`client/tests/end_to_end.rs`


Changelog
---------

We use `./CHANGELOG.md` to record all changes visible to users of the client.
Changes are added to the “Upcoming” section of the change log as part of commit
that makes the change. That is they are included in every pull request. For
breaking changes a migration path must be provided.


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

After performing the necessary changes to the Dockerfile located in
`ci/base-image/Dockerfile`, move to the root of the `radicle-registry`
repository and run the following:

```bash
docker build ci/base-image --tag gcr.io/opensourcecoin/radicle-registry/ci-base
docker push gcr.io/opensourcecoin/radicle-registry/ci-base
```

The `docker push` command outputs the pushed image’s digest. To use the pushed
image in Buildkite runs, update the `DOCKER_IMAGE` value in
`.buildkite/pipeline.yaml` with the new digest.

Note that an account with permission to push to the Google Cloud Registry
address at `gcr.io/opensourcecoin/radicle-registry` is required in order for
these commands to work.
Specifically, you'll need to run
`gsutil iam ch user:<your_monadic_email_address>@monadic.xyz:objectViewer gs://artifacts.opensourcecoin.appspot.com`

For more information on GCR permissions, consult
https://cloud.google.com/container-registry/docs/access-control.

If all this fails, request assistance to someone that can grant these
permissions.


Git Flow
--------

TL;DR: This repository follows the Git rebase flow for the most part. Each feature branch must be reviewed and approved in a pull request.
Once ready to merge, consider squashing the branch's commits when the separate commits don't add value, rebase it, force push with lease, and merge it via the Github UI.

### Branches

1. Create a separate branch for each issue your are working on
2. Do your magic
3. Keep your branch up to date by rebasing it from its base branch
4. Delete the branch after its been both approved and merged. Github does this automatically for you.

### Commits

1. Make sure you author your commits with the right username and email
2. Follow the git commit convention:
  - Use the imperative mood in the subject line
  - Limit the subject line to 50 chars
  - Capitalise the subject line
  - Wrap the description at 72 characters
  - Have the description preferably explaining what and why instead of how
  - Separate the subject from the body with an empty line
