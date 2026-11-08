#!/usr/bin/env bash
# runs every mod's build.sh so each release lands in that mod's out folder
set -euo pipefail
cd "$(dirname "$0")"

failed=()
for script in */build.sh; do
    mod="$(dirname "$script")"
    echo "== $mod"
    if ! "./$script"; then
        failed+=("$mod")
    fi
done

if (( ${#failed[@]} )); then
    echo "failed: ${failed[*]}" >&2
    exit 1
fi
