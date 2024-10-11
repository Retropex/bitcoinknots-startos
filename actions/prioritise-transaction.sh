#!/bin/sh

set -e

cat > input.json
TXID=$(jq -r '.["txid"]' input.json)
FEE_DELTA=$(jq -r '.["fee-delta"]' input.json)
rm input.json

bitcoin-cli prioritisetransaction $TXID $FEE_DELTA

echo "Transaction prioritised with fee $FEE_DELTA"