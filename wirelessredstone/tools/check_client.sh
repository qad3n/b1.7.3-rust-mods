#!/usr/bin/env bash
# drives a test client in the single player world check_ether.sh style scene lives in: a transmitter at 3 103 4 on 1000
# powered by a torch, a receiver at 1 103 4 on 1, a clocker at 5 103 4, the player facing +z; prints what each step
# checked and leaves screenshots in $MODTEST_HOME/shots; the platform under them spans -3..9 on x and z at y 100
set -euo pipefail
source "$(dirname "$0")/client_lib.sh"

tile() { dev "/loader tile 0 $1 103 4" | grep -oE 'data \[[^]]*\]'; }

echo "== remote: frequency 1, pulse lights the receiver on 1 for about a second"
dev close > /dev/null
dev "hold wirelessredstone:remote" > /dev/null
look 3 101 -2 0 -60
dev use > /dev/null
sleep 0.6
dev screen
click 15 -15
dev held
shot remote_gui
click 0 15
sleep 0.3
echo "screen after pulse: $(dev screen)"
echo "receiver during pulse: $(state 1 4 receiver 103)"
sleep 1.5
echo "receiver after 1.5 s: $(state 1 4 receiver 103)"
dev held

echo "== power configurator on the receiver: U direct off, D indirect off"
dev close > /dev/null
dev "hold wirelessredstone:power_configurator" > /dev/null
look 1 102 1 0 -25
dev use > /dev/null
sleep 0.6
dev screen
echo "before: $(tile 1)"
click 30 -32
click 50 10
sleep 0.5
echo "after:  $(tile 1)"
shot configurator_gui
dev close > /dev/null

echo "== sniffer: frequency 1000 green at row 8 column 0"
dev close > /dev/null
dev "hold wirelessredstone:sniffer" > /dev/null
dev use > /dev/null
sleep 1
dev screen
shot sniffer_gui
dev close > /dev/null

echo "== clocker gui: +s takes the period from 1000 to 2000"
dev close > /dev/null
dev "hold minecraft:air" > /dev/null
look 5 102 1 0 -25
dev use > /dev/null
sleep 0.6
dev screen
echo "before: $(tile 5)"
click 40 55
sleep 0.5
echo "after:  $(tile 5)"
shot clocker_gui
dev close > /dev/null

echo "== triangulator: frequency 1000, needle toward the transmitter"
dev close > /dev/null
dev "hold wirelessredstone:triangulator" > /dev/null
look 3 101 -2 0 -60
dev use > /dev/null
sleep 0.6
dev screen
click 58 -32
dev held
shot triangulator_gui
dev close > /dev/null
sleep 1.5
shot triangulator_needle
