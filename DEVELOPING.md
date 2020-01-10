Developer Manual
================

The code is bootstrapped with the [`substrate-node-template`][node-template].

[node-template]: https://github.com/substrate-developer-hub/substrate-node-template

### Table of contents

<!-- toc -->

- [Build requirements](#build-requirements)
- [Running development node](#running-development-node)
- [Packages](#packages)
- [Testing](#testing)
- [Changelog](#changelog)
- [Local devnet](#local-devnet)
- [Updating substrate](#updating-substrate)
- [Updating Continuous Integration's base Docker image](#updating-continuous-integrations-base-docker-image)

<!-- tocstop -->

Build requirements
------------------

1. Get [`rustup`][rustup-install]
2. Run `./scripts/rustup-setup` to install all required `rustup` components and
   targets.
3. Install [`wasm-gc`][wasm-gc] via `cargo install --git https://github.com/alexcrichton/wasm-gc`


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


Testing
-------

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
