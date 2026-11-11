#!/usr/bin/env bash
# a torch starts a clocker on frequency 0, the receiver from check_ether.sh should blink every second (1000 ms default)
# then stay off once the torch goes; run after check_ether.sh
set -euo pipefail
cmd() { tools/modport/modtest.sh cmd loader "$@" | grep -v "DEBUG\|level.dat" | sed 's/.*log:[0-9]*\] //'; }
receiver() { cmd column 0 10 0 | grep -oE 'receiver\[[^]]*\]' || true; }

y=$(($(cmd column 0 0 0 | grep -oE 'grass\[meta=0\] [0-9]+' | grep -oE '[0-9]+$') + 1))
cmd fill 0 0 "$y" 5 0 "$y" 5 wirelessredstone:clocker > /dev/null
cmd fill 0 1 "$y" 5 1 "$y" 5 minecraft:redstone_torch > /dev/null
echo "-- running: receiver sampled every 0.5 s should flip about every second"
for _ in 1 2 3 4 5 6 7 8; do receiver; sleep 0.5; done

cmd fill 0 1 "$y" 5 1 "$y" 5 minecraft:air > /dev/null
sleep 1
echo "-- stopped: receiver should stay off"
for _ in 1 2 3; do receiver; sleep 0.5; done
cmd column 0 0 5 | grep -oE 'clocker\[[^]]*\]' || true
