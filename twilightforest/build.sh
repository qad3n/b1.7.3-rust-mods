#!/usr/bin/env bash
# builds the release .bmod with cargo-b173 from PATH or from the modding SDK vendored in the rewrite, then puts it in out
set -euo pipefail
cd "$(dirname "$0")"

if [[ ! -d assets ]]; then
    echo "assets/ is missing, extract it first: ../../tools/modport/py.sh extract_assets.py <TwilightForest-2.4.3.jar> twilightforest" >&2
    exit 1
fi

tool=(cargo-b173)
if ! command -v cargo-b173 > /dev/null; then
    sdk="${B173_SDK:-$PWD/../../b1.7.3_rewrite/vendor/b1.7.3-modding-sdk}"
    if [[ ! -f "$sdk/Cargo.toml" ]]; then
        echo "modding SDK not found at $sdk, set B173_SDK or install cargo-b173" >&2
        exit 1
    fi
    tool=(cargo run --quiet --release --manifest-path "$sdk/Cargo.toml" --bin cargo-b173 --)
fi
# cargo-b173 prints the built file as its last stdout line
built="$("${tool[@]}" build --path "$PWD" | tail -n 1)"

out="$PWD/out"
mkdir -p "$out"
id="$(basename "$built" | sed -E 's/-[^-]+\.bmod$//')"
rm -f "$out/$id"-*.bmod
cp "$built" "$out/"
echo "$out/$(basename "$built")"
