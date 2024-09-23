#!/bin/bash
set -eo pipefail
seconds=$1
code=$2

[[ -z "$seconds" ]] && seconds=$(shuf -i 1-3 -n 1)
echo "Sleeping for $seconds seconds"
sleep $seconds

[[ -z "$code" ]] && code=$(shuf -i 0-1 -n 1)
echo "Exiting with status $code"
exit $code

