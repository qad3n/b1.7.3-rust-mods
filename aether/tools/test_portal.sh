#!/usr/bin/env bash
# portal round trip on a fresh test world: light a glowstone frame with water, ride it to the Aether and back
set -euo pipefail
T="$(cd "$(dirname "$0")/../../.." && pwd)/tools/rewrite/modtest.sh"
players() { "$T" cmd loader entities "$1" | grep -o "players.*"; }
"$T" reset
"$T" server > /dev/null
"$T" client > /dev/null
sleep 2
"$T" cmd loader fill 0 -10 70 5 -7 74 5 minecraft:glowstone > /dev/null
"$T" cmd loader fill 0 -9 71 5 -8 73 5 minecraft:air > /dev/null
"$T" cmd loader fill 0 -10 70 6 -7 70 7 minecraft:stone > /dev/null
"$T" cmd loader tp Tester 0 -8 71 7 > /dev/null
"$T" cmd loader use Tester minecraft:water_bucket -9 70 5 1
"$T" cmd loader tp Tester 0 -9 71 5 > /dev/null
sleep 10
echo "in the aether: $(players 2)"
sleep 5
echo "still there: $(players 2)"
"$T" cmd loader column 2 -9 5
"$T" cmd loader tp Tester 2 -9 "${RETURN_Y:-65}" 7 > /dev/null
