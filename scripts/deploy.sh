#!/usr/bin/env bash
# Testnet deploy helper for the tipjar contract.
#
# Not yet implemented — tracked as feature 9 in the README progress tracker.
# Until it lands, run the steps below by hand.
set -euo pipefail

cat <<'USAGE'
scripts/deploy.sh is not implemented yet.

Run these steps manually in the meantime:

  1. Build the contract
     cargo build -p tipjar --target wasm32v1-none --release

  2. Deploy it
     stellar contract deploy \
       --wasm target/wasm32v1-none/release/tipjar.wasm \
       --source default \
       --network testnet

  3. Initialise it with the token you want to accept
     stellar contract invoke \
       --id <CONTRACT_ID> \
       --source default \
       --network testnet \
       -- init \
       --token $(stellar contract id asset --asset native --network testnet)

The script will wrap these steps, capture the contract id, and skip init when
the jar is already initialised.
USAGE

exit 1
