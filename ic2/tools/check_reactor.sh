#!/usr/bin/env bash
# console check of a two cell reactor feeding an MFSU through glass fibre; prints both tiles after WAIT seconds
set -euo pipefail
mt="tools/modport/modtest.sh"
x="$1" y="${2:-104}" z="$3"

cmd() { timeout 20 "$mt" cmd "loader $*" | grep -v "saveWorld\|writeLevel\|console ran" | sed 's/.*log:655\] //'; }

cmd fill 0 $((x - 1)) $((y - 1)) $((z - 1)) $((x + 8)) $((y + 1)) $((z + 1)) minecraft:air
cmd fill 0 "$x" "$y" "$z" "$x" "$y" "$z" ic2:nuclear_reactor
cmd slot 0 "$x" "$y" "$z" 0 ic2:uranium_cell 1
cmd slot 0 "$x" "$y" "$z" 1 ic2:uranium_cell 1
cmd fill 0 $((x + 1)) "$y" "$z" $((x + 3)) "$y" "$z" ic2:glass_fibre_cable
cmd fill 0 $((x + 4)) "$y" "$z" $((x + 4)) "$y" "$z" ic2:mfsu
sleep "${WAIT:-30}"
echo "== reactor"; cmd tile 0 "$x" "$y" "$z"
echo "== mfsu"; cmd tile 0 $((x + 4)) "$y" "$z"
