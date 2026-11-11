#!/usr/bin/env bash
# console check of the energy chain on the test server: a coal generator feeds a macerator through copper cable,
# then a batbox, and the tiles are printed after a few seconds; run from the workspace root
set -euo pipefail
mt="tools/modport/modtest.sh"
x="${1:-229}" y="${2:-100}" z="${3:--16}"

cmd() { timeout 20 "$mt" cmd "loader $*" | grep -v "saveWorld\|writeLevel\|console ran"; }

cmd fill 0 $((x - 1)) $((y - 1)) $((z - 1)) $((x + 6)) $((y + 1)) $((z + 1)) minecraft:air
cmd fill 0 "$x" "$y" "$z" "$x" "$y" "$z" ic2:generator
cmd fill 0 $((x + 1)) "$y" "$z" $((x + 3)) "$y" "$z" ic2:copper_cable
cmd fill 0 $((x + 4)) "$y" "$z" $((x + 4)) "$y" "$z" ic2:macerator
cmd slot 0 "$x" "$y" "$z" 1 minecraft:coal 8
cmd slot 0 $((x + 4)) "$y" "$z" 0 minecraft:iron_ore 4
sleep "${WAIT:-20}"
echo "== generator"; cmd tile 0 "$x" "$y" "$z"
echo "== macerator"; cmd tile 0 $((x + 4)) "$y" "$z"
