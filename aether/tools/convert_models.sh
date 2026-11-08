#!/usr/bin/env bash
# regenerates src/models/generated.rs from the decompiled Aether models and the vanilla models in reference/vanilla
set -euo pipefail
cd "$(dirname "$0")/.."
python3 -I ../../tools/modport/convert_models.py --out src/models/generated.rs \
    reference/src/com/gildedgames/the_aether/client/models/entities/*.java reference/vanilla/*.java \
    --arg FlyingCowSaddleModel.scale=0.5 --arg MoaSaddleModel.scale=0.5 2> >(grep -v "^warning" >&2)
