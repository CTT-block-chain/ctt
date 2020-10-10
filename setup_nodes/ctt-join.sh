#!/bin/bash

# remember to change bootnodes to actually n1

../target/debug/ctt \
  --base-path ~/cttdb/n"$1" \
  --chain ./customSpecRaw.json \
  --port "$2" \
  --ws-port "$3" \
  --rpc-port "$4" \
  --validator \
  --rpc-methods=Unsafe \
  --name "$5" \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWBo1QP6NEwQhU3zCZibQhStwwqbnXbbXLjj7hVSgWgq4K
