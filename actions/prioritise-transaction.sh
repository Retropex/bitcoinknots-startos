#!/bin/sh

set -e

cat > input.json
TXID=$(jq -r '.["txid"]' input.json)
PRIORITY_DELTA=$(jq -r '.["priority-delta"]' input.json)
if [ $PRIORITY_DELTA = "null" ]; then
    PRIORITY_DELTA="0"
fi
FEE_DELTA=$(jq -r '.["fee-delta"]' input.json)
if [ $FEE_DELTA = "null" ]; then
    FEE_DELTA="0"
fi
rm input.json
result="    {
    \"version\": \"0\",
    \"message\": \"Transaction prioritised with priority delta $PRIORITY_DELTA and fee delta $FEE_DELTA\",
    \"value\": null,
    \"copyable\": false,
    \"qr\": false
}"
>&2 bitcoin-cli prioritisetransaction $TXID $PRIORITY_DELTA $FEE_DELTA && echo $result