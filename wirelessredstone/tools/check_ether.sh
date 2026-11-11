#!/usr/bin/env bash
# drives a running test server: a torch powers a transmitter, the receiver 10 blocks away on the same frequency lights
# the wire beside it, then the torch goes and both go dark; run from the workspace root after test.sh deploy
set -euo pipefail
cmd() { tools/modport/modtest.sh cmd loader "$@" | grep -v "DEBUG\|level.dat" | sed 's/.*log:[0-9]*\] //'; }

ground="$(cmd column 0 0 0 | grep -oE 'grass\[meta=0\] [0-9]+' | grep -oE '[0-9]+$')"
y=$((ground + 1))
echo "ground $ground, building at y $y"

cmd fill 0 0 "$y" 0 0 "$y" 0 wirelessredstone:transmitter > /dev/null
cmd fill 0 10 "$y" 0 10 "$y" 0 wirelessredstone:receiver > /dev/null
cmd fill 0 11 "$y" 0 11 "$y" 0 minecraft:redstone_wire > /dev/null
sleep 1
echo "-- unpowered: transmitter, receiver and wire should be off"
cmd column 0 0 0 | grep -oE 'wirelessredstone:[a-z]+\[[^]]*\]' || true
cmd column 0 10 0 | grep -oE 'wirelessredstone:[a-z]+\[[^]]*\]' || true
cmd column 0 11 0 | grep -oE 'redstone_wire\[[^]]*\]' || true

cmd fill 0 1 "$y" 0 1 "$y" 0 minecraft:redstone_torch > /dev/null
sleep 1
echo "-- torch beside the transmitter: all three should be on, wire meta 15"
cmd column 0 0 0 | grep -oE 'wirelessredstone:[a-z]+\[[^]]*\]' || true
cmd column 0 10 0 | grep -oE 'wirelessredstone:[a-z]+\[[^]]*\]' || true
cmd column 0 11 0 | grep -oE 'redstone_wire\[[^]]*\]' || true

cmd fill 0 1 "$y" 0 1 "$y" 0 minecraft:air > /dev/null
sleep 1
echo "-- torch gone: all three off again"
cmd column 0 0 0 | grep -oE 'wirelessredstone:[a-z]+\[[^]]*\]' || true
cmd column 0 10 0 | grep -oE 'wirelessredstone:[a-z]+\[[^]]*\]' || true
cmd column 0 11 0 | grep -oE 'redstone_wire\[[^]]*\]' || true
