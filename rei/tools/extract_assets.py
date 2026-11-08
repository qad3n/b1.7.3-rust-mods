"""Copy the lang lines the port uses out of the user's own Roughly Enough Items jar into assets/rei.

The strings belong to the original authors, so they are never committed. Run from the port folder:
    python3 -I tools/extract_assets.py <RoughlyEnoughItems-fabric.jar>
"""

import json
import sys
import zipfile
from pathlib import Path

SOURCE = "assets/roughlyenoughitems/lang/en_us.json"
TARGET = Path("assets/rei/lang/en_US.lang")
KEYS = [
    "text.rei.no_permission_cheat",
    "text.rei.failed_cheat_items",
    "text.rei.search.field.suggestion",
    "text.rei.shapeless",
    "text.rei.back",
    "text.rei.left_arrow",
    "text.rei.right_arrow",
    "category.rei.crafting",
    "category.rei.smelting",
]


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 1
    with zipfile.ZipFile(sys.argv[1]) as jar:
        lang = json.loads(jar.read(SOURCE).decode("utf-8"))
    missing = [key for key in KEYS if key not in lang]
    if missing:
        print(f"keys missing from the jar: {missing}", file=sys.stderr)
        return 1
    TARGET.parent.mkdir(parents=True, exist_ok=True)
    TARGET.write_text("".join(f"rei.{key}={lang[key]}\n" for key in KEYS), encoding="utf-8")
    print(f"{len(KEYS)} lang keys into {TARGET}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
