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
docker build ci/base-image --tag gcr.io/opensourcecoin/radicle-registry;
docker push gcr.io/opensourcecoin/radicle-registry
```

Note that an account with permission to push to the Google Cloud Registry
address at `gcr.io/opensourcecoin/radicle-registry` is required in order for
these commands to work.
Specifically, you'll need to run
`gsutil iam ch user:<your_monadic_email_address>@monadic.xyz:objectViewer gs://artifacts.opensourcecoin.appspot.com`

For more information on GCR permissions, consult
https://cloud.google.com/container-registry/docs/access-control.

If all this fails, request assistance to someone that can grant these
permissions.

Updating the Buildkite pipeline
-------------------------------

After this is done, the Buildkite pipeline located at `.buildkite/pipeline.yml`
will also need to be updated with the correct Docker image digest -
specifically, the `DOCKER_IMAGE` environment variable.


To find the digest for the image build from the above steps, run

```bash
docker images --digests | grep radicle-registry
```

which should produce output like

```
gcr.io/opensourcecoin/radicle-registry       latest                                     sha256:264a6e976faeb63adda312cd3ecf2fb9ff71fd6f030128b3f012daf9c4cff05b   c824ae208aff        50 minutes ago      3.15GB
gcr.io/opensourcecoin/radicle-registry       <none>                                     sha256:41347023e88ff45254a95bcf09cb3085e0d6bc677eda45e8de2a4611191eb2fe   7f985cd4d719        3 days ago          3.23GB
```

The new image digest in this case will be
`sha256:264a6e976faeb63adda312cd3ecf2fb9ff71fd6f030128b3f012daf9c4cff05b`, as it
is the most recently built.
