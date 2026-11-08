"""Copy Aether Legacy textures, sounds and lang out of the user's own mod jar into assets/aether.

The assets are All Rights Reserved, so they are never committed. Run:
    python3 -I tools/extract_assets.py /path/to/aether-1.7.10-v1.1.2.5.jar
"""

import re
import sys
import zipfile
from pathlib import Path

SOURCE = "assets/aether_legacy/"
TARGET = Path(__file__).resolve().parent.parent / "assets" / "aether"

# jar folder -> mod folder, textures keep their subfolders
FOLDERS = [
    ("textures/blocks/", "textures/block/"),
    ("textures/items/", "textures/item/"),
    ("textures/entities/", "textures/entity/"),
    ("textures/bosses/", "textures/entity/boss/"),
    ("textures/armor/", "textures/armor/"),
    ("textures/gui/", "textures/gui/"),
    ("textures/other/", "textures/other/"),
    ("textures/tile_entities/", "textures/tile/"),
    ("sounds/", "sounds/"),
]

LANG_KEY = re.compile(r"^(tile|item)\.([a-z0-9_]+)\.name=(.*)$")


def extract(jar: zipfile.ZipFile) -> int:
    count = 0
    for name in jar.namelist():
        if not name.startswith(SOURCE) or name.endswith("/"):
            continue
        rest = name[len(SOURCE):]
        for source, target in FOLDERS:
            if rest.startswith(source):
                out = TARGET / target / rest[len(source):]
                out.parent.mkdir(parents=True, exist_ok=True)
                out.write_bytes(jar.read(name))
                count += 1
                break
    return count


# chat lines and screen text the port shows, kept under its own namespace
MESSAGE_PREFIX = "gui."
# boss titles shown on the boss bar
TITLE_PREFIX = "title.aether_legacy."
# book of lore entries
LORE_PREFIX = "lore.aether_legacy."


def lang(jar: zipfile.ZipFile) -> int:
    lines = jar.read(SOURCE + "lang/en_US.lang").decode("utf-8").splitlines()
    out = []
    for line in lines:
        match = LANG_KEY.match(line.strip())
        if match:
            kind, key, value = match.groups()
            out.append(f"{kind}.aether.{key}.name={value}")
        elif line.startswith(MESSAGE_PREFIX):
            out.append(f"aether.{line.strip()}")
        elif line.startswith(TITLE_PREFIX):
            key, value = line.strip()[len(TITLE_PREFIX):].split("=", 1)
            out.append(f"aether.title.{key.removesuffix('.name')}={value}")
        elif line.startswith(LORE_PREFIX):
            out.append(f"aether.lore.{line.strip()[len(LORE_PREFIX):]}")
    path = TARGET / "lang" / "en_US.lang"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(out) + "\n", encoding="utf-8")
    return len(out)


# ModelChest regions on a 64 by 64 chest sheet, stacked lid over base and scaled to one 16 pixel block face,
# beta draws chests as plain cubes
CHEST_FACES = {
    "front": [(14, 14, 28, 19), (14, 33, 28, 43)],
    "side": [(0, 14, 14, 19), (0, 33, 14, 43)],
    "top": [(14, 0, 28, 14)],
}
CHEST_LATCH = (1, 1, 3, 5)
CHEST_LATCH_AT = (7, 3)
FACE = 16


def chestSprites() -> int:
    from PIL import Image

    sheet = Image.open(TARGET / "textures" / "tile" / "treasure_chest.png").convert("RGBA")
    for name, regions in CHEST_FACES.items():
        parts = [sheet.crop(box) for box in regions]
        width = max(part.width for part in parts)
        stacked = Image.new("RGBA", (width, sum(part.height for part in parts)))
        top = 0
        for part in parts:
            stacked.paste(part, (0, top))
            top += part.height
        if name == "front":
            stacked.paste(sheet.crop(CHEST_LATCH), CHEST_LATCH_AT)
        stacked.resize((FACE, FACE), Image.NEAREST).save(TARGET / "textures" / "block" / f"treasure_chest_{name}.png")
    return len(CHEST_FACES)


def main() -> None:
    if len(sys.argv) != 2:
        sys.exit("usage: extract_assets.py <aether-1.7.10 jar>")
    with zipfile.ZipFile(sys.argv[1]) as jar:
        files = extract(jar)
        keys = lang(jar)
    files += chestSprites()
    print(f"{files} files and {keys} lang keys in {TARGET}")


if __name__ == "__main__":
    main()
