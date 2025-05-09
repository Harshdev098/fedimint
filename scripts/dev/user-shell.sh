# shellcheck shell=bash

eval "$(devimint env)"
source ./scripts/dev/completion.sh
source ./scripts/dev/aliases.sh

echo Waiting for fedimint start

STATUS="$(devimint wait)"
if [ "$STATUS" = "ERROR" ]
then
    echo "fedimint didn't start correctly"
    echo "See other panes for errors"
    exit 1
fi

eval "$(devimint env)"

echo Done!
echo
echo "This shell provides the following aliases:"
echo ""
echo "  fm-cli   - cli client to interact with the federation"
echo "  lncli          - cli client for LND"
echo "  bitcoin-cli    - cli client for bitcoind"
echo "  gateway-lnd    - cli client for the LND gateway"
echo "  gateway-ldk    - cli client for the LDK gateway"
echo "  gateway-ldk2   - cli client for the second LDK gateway"
echo
echo "Use '--help' on each command for more information"
