#!/usr/bin/env bash
# helpers for the client checks, sourced with MODTEST_HOME set to a running test client in a single player world
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

dev() { "$here/dev.sh" "$@"; }

# the test window loses focus and vanilla opens the pause menu, so close it before every in-world action
use()
{
    for _ in 1 2 3 4 5 6; do
        dev close > /dev/null
        sleep 0.3
        dev use > /dev/null
        sleep 0.6
        [[ "$(dev screen)" == GuiModScreen* ]] && return 0
    done
    echo "no mod screen opened"
    return 1
}

# stands the player at a block and turns the view, single player tp keeps the old angles
look() { dev "/loader tp Tester 0 $1 $2 $3" "look ${4:-0} ${5:-0}" > /dev/null; sleep 1; }

# a click at an offset from the screen center, as the source placed its buttons
click()
{
    local size width height
    size="$(dev screen | awk '{print $2}')"
    [[ "$size" == *x* ]] || { echo "no screen to click"; return 0; }
    width="${size%x*}"
    height="${size#*x}"
    dev "click $((width / 2 + $1)) $((height / 2 + $2))" > /dev/null
    sleep 0.4
}

state() { dev "/loader column 0 $1 $2" | grep -oE "$3\[[^]]*\] $4\.\.$4" | head -1; }

shot() { dev shot "$1" > /dev/null; echo "shot $1"; }
