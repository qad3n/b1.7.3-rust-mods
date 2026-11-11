#!/usr/bin/env bash
# after a server restart the ether is empty until tile ticks rebuild it: a torch beside the saved transmitter from
# check_ether.sh must still light the saved receiver; run after check_ether.sh and a fresh test.sh deploy
set -euo pipefail
cmd() { tools/modport/modtest.sh cmd loader "$@" | grep -v "DEBUG\|level.dat" | sed 's/.*log:[0-9]*\] //'; }

y=$(($(cmd column 0 0 0 | grep -oE 'grass\[meta=0\] [0-9]+' | grep -oE '[0-9]+$') + 1))
cmd gen 0 0 0 1 > /dev/null
sleep 1
cmd fill 0 1 "$y" 0 1 "$y" 0 minecraft:redstone_torch > /dev/null
sleep 1
echo "-- saved receiver should be on"
cmd column 0 10 0 | grep -oE 'receiver\[[^]]*\]' || true
cmd fill 0 1 "$y" 0 1 "$y" 0 minecraft:air > /dev/null
sleep 1
echo "-- and off again"
cmd column 0 10 0 | grep -oE 'receiver\[[^]]*\]' || true
