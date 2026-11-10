#!/usr/bin/env bash
# regenerates src/models/generated.rs from the decompiled twilight forest models and the vanilla models in reference/vanilla
# ModelTFTowerBoss builds its tentacles through overridden methods the converter cannot follow, models/ur_ghast.rs has it by hand
set -euo pipefail
cd "$(dirname "$0")/.."
shopt -s extglob
../../tools/modport/py.sh convert_models.py --out src/models/generated.rs \
    reference/src/twilightforest/client/model/!(ModelTFTowerBoss).java \
    reference/src/twilightforest/client/renderer/entity/ModelTFApocalypseCube.java \
    reference/vanilla/ModelSpider.java reference/vanilla/ModelWolf.java reference/vanilla/ModelSlime.java \
    reference/vanilla/ModelBook.java 2> >(grep -v "^warning" >&2)
