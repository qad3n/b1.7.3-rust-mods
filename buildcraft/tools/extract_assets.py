"""Builds assets/buildcraft from the user's BuildCraft 2.1.1 zips. Run from the port folder:
    python3 -I tools/extract_assets.py <folder holding the BuildCraft *.zip files>
The block and item sheets are cut into single sprites, the guis and engine model textures are copied, and a first frame
of the oil effects BuildCraft draws in code stands in until the mod paints them live.
"""

import random
import struct
import sys
import zipfile
import zlib
from pathlib import Path

PORT = Path(__file__).resolve().parent.parent
OUT = PORT / "assets" / "buildcraft"
SHEET_ROOT = "net/minecraft/src/buildcraft/"
TILE = 16
SHEET_TILES = 16

# BuildCraftCore.customBuildCraftTexture slots, as the mod's blocks read them
BLOCKS = {
    0: "laser_transparent", 1: "laser_blue", 2: "laser_red", 3: "laser_stripes",
    16: "wooden_pipe", 17: "cobblestone_pipe", 18: "iron_pipe", 19: "iron_pipe_plain", 20: "golden_pipe",
    21: "diamond_pipe", 22: "diamond_pipe_down", 23: "diamond_pipe_up", 24: "diamond_pipe_north",
    25: "diamond_pipe_south", 26: "diamond_pipe_west", 27: "diamond_pipe_east", 28: "obsidian_pipe",
    29: "stone_pipe", 30: "golden_pipe_powered", 31: "wooden_pipe_plain",
    32: "mining_pipe", 33: "drill", 34: "frame", 35: "mining_well_front", 36: "mining_well_top",
    37: "mining_well_side", 38: "mining_well_back", 39: "quarry_front", 40: "quarry_top", 41: "quarry_side",
    42: "drill_head", 43: "auto_workbench_top", 44: "auto_workbench_side",
    48: "template_side", 49: "template_top_powered", 50: "template_top", 52: "template_front",
    53: "builder_side", 54: "builder_top", 55: "builder_front", 57: "marker",
    64: "filler_top_on", 65: "filler_top_off", 66: "filler_side", 67: "filler_fill_all", 68: "filler_remover",
    69: "filler_flattener", 70: "filler_walls", 71: "filler_pyramid", 72: "filler_stairs",
    96: "tank_side", 97: "tank_side_joined", 98: "tank_top", 99: "pump_side", 100: "pump_bottom", 101: "pump_top",
    102: "pump_tube",
}

# BuildCraftCore.customBuildCraftSprites slots
ITEMS = {
    2: "wrench", 16: "wooden_gear", 17: "stone_gear", 18: "iron_gear", 19: "gold_gear", 20: "diamond_gear",
    32: "template",
}

GUIS = {
    "builders/gui/builder.png": "gui/builder.png",
    "builders/gui/filler.png": "gui/filler.png",
    "builders/gui/template_gui.png": "gui/template.png",
    "transport/gui/filter.png": "gui/filter.png",
    "energy/gui/combustion_engine_gui.png": "gui/combustion_engine.png",
    "energy/gui/steam_engine_gui.png": "gui/steam_engine.png",
    "energy/gui/base_wood.png": "model/engine_base_wood.png",
    "energy/gui/base_stone.png": "model/engine_base_stone.png",
    "energy/gui/base_iron.png": "model/engine_base_iron.png",
    "energy/gui/chamber.png": "model/engine_chamber.png",
    "energy/gui/trunk_blue.png": "model/engine_trunk_blue.png",
    "energy/gui/trunk_green.png": "model/engine_trunk_green.png",
    "energy/gui/trunk_yellow.png": "model/engine_trunk_yellow.png",
    "energy/gui/trunk_red.png": "model/engine_trunk_red.png",
    "energy/gui/oil_bucket.png": "textures/item/oil_bucket.png",
}

LANG = """tile.buildcraft.wooden_pipe.name=Wooden Pipe
tile.buildcraft.stone_pipe.name=Stone Pipe
tile.buildcraft.cobblestone_pipe.name=Cobblestone Pipe
tile.buildcraft.iron_pipe.name=Iron Pipe
tile.buildcraft.golden_pipe.name=Golden Pipe
tile.buildcraft.diamond_pipe.name=Diamond Pipe
tile.buildcraft.obsidian_pipe.name=Obsidian Pipe
tile.buildcraft.redstone_engine.name=Redstone Engine
tile.buildcraft.steam_engine.name=Steam Engine
tile.buildcraft.combustion_engine.name=Combustion Engine
tile.buildcraft.oil.name=Oil
tile.buildcraft.oil_still.name=Oil
tile.buildcraft.mining_well.name=Mining Well
tile.buildcraft.mining_pipe.name=Mining Pipe
tile.buildcraft.auto_workbench.name=Automatic Crafting Table
tile.buildcraft.frame.name=Frame
tile.buildcraft.quarry.name=Quarry
tile.buildcraft.tank.name=Tank
tile.buildcraft.pump.name=Pump
tile.buildcraft.marker.name=Land Mark
tile.buildcraft.filler.name=Filler
tile.buildcraft.builder.name=Builder
tile.buildcraft.template_table.name=Template Drawing Table
item.buildcraft.wooden_gear.name=Wooden Gear
item.buildcraft.stone_gear.name=Stone Gear
item.buildcraft.iron_gear.name=Iron Gear
item.buildcraft.gold_gear.name=Gold Gear
item.buildcraft.diamond_gear.name=Diamond Gear
item.buildcraft.wrench.name=Wrench
item.buildcraft.oil_bucket.name=Oil Bucket
item.buildcraft.template.name=Blank Template
buildcraft.template=Template #%s
buildcraft.gui.filters=Filters
buildcraft.gui.inventory=Inventory
buildcraft.gui.steam_engine=Steam Engine
buildcraft.gui.combustion_engine=Combustion Engine
buildcraft.gui.crafting=Crafting
buildcraft.gui.filler=Filler
buildcraft.gui.filling_resources=Filling Resources
buildcraft.gui.builder_template=Template
buildcraft.gui.building_resources=Building Resources
buildcraft.gui.template=Template
"""


def readPng(data):
    width, height = struct.unpack(">II", data[16:24])
    depth, colorType = data[24], data[25]
    if depth != 8 or colorType not in (2, 6):
        raise SystemExit(f"unsupported png layout {depth} {colorType}")
    channels = 4 if colorType == 6 else 3
    position, raw = 8, b""
    while position < len(data):
        length, kind = struct.unpack(">I4s", data[position:position + 8])
        if kind == b"IDAT":
            raw += data[position + 8:position + 8 + length]
        position += 12 + length
    raw = zlib.decompress(raw)
    stride = width * channels
    rows, previous, at = [], bytearray(stride), 0
    for _ in range(height):
        kind, line = raw[at], bytearray(raw[at + 1:at + 1 + stride])
        at += 1 + stride
        for i in range(stride):
            left = line[i - channels] if i >= channels else 0
            up, corner = previous[i], previous[i - channels] if i >= channels else 0
            if kind == 1:
                line[i] = (line[i] + left) & 0xFF
            elif kind == 2:
                line[i] = (line[i] + up) & 0xFF
            elif kind == 3:
                line[i] = (line[i] + (left + up) // 2) & 0xFF
            elif kind == 4:
                guess = left + up - corner
                pick = min((abs(guess - left), 0, left), (abs(guess - up), 1, up), (abs(guess - corner), 2, corner))[2]
                line[i] = (line[i] + pick) & 0xFF
        rows.append([tuple(line[x * channels:x * channels + channels]) + ((255,) if channels == 3 else ()) for x in range(width)])
        previous = line
    return rows


def writePng(path, rows):
    height, width = len(rows), len(rows[0])
    raw = b"".join(b"\x00" + bytes(value for pixel in row for value in pixel) for row in rows)

    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)

    header = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))


def cut(rows, slot):
    top, left = slot // SHEET_TILES * TILE, slot % SHEET_TILES * TILE
    return [row[left:left + TILE] for row in rows[top:top + TILE]]


# TextureOilFX and TextureOilFlowFX step a heat field like the vanilla water effects and draw it in near black greys
def oilFrames(flowing, frames, seed):
    rng = random.Random(seed)
    current, next_, heat, spark = [0.0] * 256, [0.0] * 256, [0.0] * 256, [0.0] * 256
    out = []
    for tick in range(1, frames + 1):
        for i in range(16):
            for k in range(16):
                if flowing:
                    total = sum(current[(i & 15) + (j & 15) * 16] for j in range(k - 2, k + 1))
                    next_[i + k * 16] = total / 3.2 + heat[i + k * 16] * 0.8
                else:
                    total = sum(current[(j & 15) + (k & 15) * 16] for j in range(i - 1, i + 2))
                    next_[i + k * 16] = total / 3.3 + heat[i + k * 16] * 0.8
        for index in range(256):
            heat[index] = max(0.0, heat[index] + spark[index] * 0.05)
            spark[index] -= 0.3 if flowing else 0.1
            if rng.random() < (0.2 if flowing else 0.05):
                spark[index] = 0.5
        current, next_ = next_, current
        pixels = []
        for index in range(256):
            value = current[(index - tick * 16) & 0xFF] if flowing else current[index]
            value = min(1.0, max(0.0, value))
            grey = int(10.0 + value * value * (22.0 if flowing else 21.0))
            pixels.append((grey, grey, grey, 255))
        out.append([pixels[y * 16:y * 16 + 16] for y in range(16)])
    return out


# the first frame stands in until the mod's live effect paints the sprite each tick
def writeFirstFrame(name, frames):
    path = OUT / "textures" / "block" / f"{name}.png"
    writePng(path, frames[-1])
    path.with_name(path.name + ".mcmeta").unlink(missing_ok=True)


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    files = {}
    for archive in sorted(Path(sys.argv[1]).glob("BuildCraft*.zip")):
        with zipfile.ZipFile(archive) as opened:
            for name in opened.namelist():
                if name.startswith(SHEET_ROOT) and name.endswith(".png"):
                    files[name[len(SHEET_ROOT):]] = opened.read(name)
    if "core/gui/block_textures.png" not in files:
        raise SystemExit("no BuildCraft Core zip in that folder")

    blocks = readPng(files["core/gui/block_textures.png"])
    for slot, name in BLOCKS.items():
        writePng(OUT / "textures" / "block" / f"{name}.png", cut(blocks, slot))
    items = readPng(files["core/gui/item_textures.png"])
    for slot, name in ITEMS.items():
        writePng(OUT / "textures" / "item" / f"{name}.png", cut(items, slot))
    for source, target in GUIS.items():
        path = OUT / (target if target.startswith("textures/") else f"textures/{target}")
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(files[source])
    writeFirstFrame("oil_still", oilFrames(False, 64, 1))
    writeFirstFrame("oil_flow", oilFrames(True, 64, 2))
    lang = OUT / "lang" / "en_US.lang"
    lang.parent.mkdir(parents=True, exist_ok=True)
    lang.write_text(LANG)
    print(f"wrote {len(BLOCKS)} block sprites, {len(ITEMS)} item sprites and {len(GUIS)} textures into {OUT}")


main()
