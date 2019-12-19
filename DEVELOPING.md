Developer Manual
================

The code is bootstrapped with the [`substrate-node-template`][node-template].

[node-template]: https://github.com/substrate-developer-hub/substrate-node-template

### Table of contents

<!-- toc -->

- [Running development node](#running-development-node)
- [Packages](#packages)
- [Testing](#testing)
- [Changelog](#changelog)
- [Local devnet](#local-devnet)
- [Upstream `subxt`](#upstream-subxt)
- [Updating substrate](#updating-substrate)
- [Updating Continuous Integration's base Docker image](#updating-continuous-integrations-base-docker-image)

<!-- tocstop -->

Running development node
------------------------

~~~
BUILD_DUMMY_WASM_BINARY=0 cargo build --release -p radicle-registry-node
./scripts/run-dev-node
~~~

The run script purges the chain data before running to avoid consensus issues.
This means that state is not persisted between runs.

Packages
--------

* `runtime` contains the Substrate runtime code that defines the ledger and
  lives on chain.
* `node` contains the node code which includes the runtime code.
* `client` contains the high-level client library for interacting with the
  registry through a node and an emulator implementation.
* `cli` contains a binary for interacting with the registry node to submit
  transactions and read state.
* `subxt` contains a copy of [`subxt`][subxt], the Rust client library for
  substrate. This package serves as the base for `client`.


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


Upstream `subxt`
----------------

This repository contains a modified copy of [`subxt`][subxt] in the `./subxt`
directory. The repository also contains a Git submodule as reference to the
`subxt` upstream.

To include upstream patches of `subxt` in our copy use the following recipe

~~~bash
# Extract latest patches
git --git-dir=subxt/vendor/.git fetch origin
git --git-dir=subxt/vendor/.git format-patch HEAD..origin/master

# Apply patches
git am --directory=subxt *.patch

# Update submodule revision
git --git-dir=subxt/vendor/.git checkout origin/master
~~~

Finlly squash all the upstream patches. Use the commit title `subxt: Apply
upstream patches` and include a list of all the upstream commits that were
applied.

[subxt]: https://github.com/paritytech/substrate-subxt


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
