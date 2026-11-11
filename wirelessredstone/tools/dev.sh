#!/usr/bin/env bash
# sends dev lines to the running test client and prints its answers, or takes a screenshot
#   dev.sh <line>...        each argument is one devcmd line (world, hold, use, screen, click, close, /loader ...)
#   dev.sh shot <name>      screenshot into $MODTEST_HOME/shots/<name>.png
set -euo pipefail
home="${MODTEST_HOME:?set MODTEST_HOME to the test client folder}"

if [[ "${1:-}" == shot ]]; then
    exec "$(dirname "$0")/../../../tools/modport/modtest.sh" shot "$home/shots/$2.png"
fi

rm -f "$home/devcmd.done"
printf '%s\n' "$@" > "$home/devcmd.request"
for _ in $(seq 100); do
    [[ -f "$home/devcmd.done" ]] && break
    sleep 0.1
done
cat "$home/devcmd.done" 2>/dev/null || echo "no answer from the client"
echo
