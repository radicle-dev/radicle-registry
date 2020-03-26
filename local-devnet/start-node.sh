#!/usr/bin/bash

set -euo pipefail

declare -a extra_args
if [[ "$NODE_NAME" = "alice" ]]; then
  extra_args=(
    # Boot node id: QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR
    --node-key 0000000000000000000000000000000000000000000000000000000000000001
  )
fi

exec /usr/local/bin/radicle-registry-node \
  --data-path /data \
  --name "$NODE_NAME" \
  --chain local-devnet \
  --unsafe-rpc-external \
  --prometheus-external \
  --bootnodes /dns4/alice/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR \
  "${extra_args[@]}"
