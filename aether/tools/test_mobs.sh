#!/usr/bin/env bash
# summons each Aether mob on the island at 0,0 of a running test server and reports them after a few seconds
set -euo pipefail
T="$(cd "$(dirname "$0")/../../.." && pwd)/tools/rewrite/modtest.sh"
mobs=("$@")
[[ ${#mobs[@]} -eq 0 ]] && mobs=(aerbunny sheepuff phyg flying_cow aerwhale)
"$T" cmd loader gen 2 0 0 3 > /dev/null
x=0
for mob in "${mobs[@]}"; do
    MODTEST_WAIT=0.3 "$T" cmd loader summon 2 "aether:$mob" "$x" 56 0
    x=$((x + 4))
done
sleep "${SETTLE:-5}"
"$T" cmd loader entities 2 | grep -o "[0-9]* aether:[a-z_]*[^;]*" | sed 's/] players.*//'
"$T" log server 200 | grep -i "\[aether\]\|panic\|trap\|error" | tail -5 || true
