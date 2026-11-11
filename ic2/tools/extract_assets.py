"""Builds assets/ic2 from the user's IndustrialCraft 2 v1.00 zip. Run from the port folder:
    python3 -I tools/extract_assets.py <path to "IndustrialCraft2 v1.00.zip">
The block, cable and item sheets are cut into single sprites, the guis and armor layers are copied, and the lang file
is written from the names the mod gave its content. The zip has no sounds: IC2 played its sound names from a separate
resource download, so the port plays them as ic2.<name> and a pack may add them under assets/ic2/sounds.
"""

import struct
import sys
import zipfile
import zlib
from pathlib import Path

PORT = Path(__file__).resolve().parent.parent
OUT = PORT / "assets" / "ic2"
TILE = 16
SHEET_TILES = 16

# /IC2sprites/block_0.png slots as the mod's blocks read them
BLOCKS = {
    0: "iron_furnace_bottom", 1: "iron_furnace_top", 2: "iron_furnace_side", 4: "iron_furnace_front_active",
    5: "iron_furnace_front", 6: "electric_furnace_front", 7: "electric_furnace_front_active", 8: "canner_front",
    9: "canner_front_active", 12: "reinforced_stone", 13: "reinforced_glass", 14: "reinforced_door_top",
    15: "reinforced_door_bottom",
    16: "machine_bottom", 17: "machine_top", 18: "machine_side", 19: "macerator_front", 20: "macerator_top",
    21: "macerator_top_active", 22: "extractor_front", 23: "extractor_front_active", 24: "extractor_side",
    25: "extractor_side_active", 26: "extractor_top", 27: "compressor_front", 28: "compressor_front_active",
    29: "miner_bottom", 30: "miner_side", 31: "luminator",
    32: "copper_ore", 33: "tin_ore", 34: "uranium_ore", 35: "mining_pipe", 36: "mining_pipe_tip", 40: "rubber_sheet",
    43: "resin", 44: "rubber_wood", 45: "rubber_wood_resin", 46: "rubber_wood_tapped", 47: "rubber_wood_top",
    48: "generator_front", 49: "generator_front_active", 50: "geothermal_generator_front",
    51: "geothermal_generator_front_active", 52: "solar_panel_top", 53: "wind_mill_side", 54: "water_mill_side",
    56: "remote_dynamite", 57: "dynamite", 58: "industrial_tnt_bottom", 59: "industrial_tnt_top",
    60: "industrial_tnt_side", 61: "nuke_bottom", 62: "nuke_top", 63: "nuke_side",
    64: "nuclear_reactor_top", 65: "nuclear_reactor_side_active", 66: "nuclear_reactor_side",
    67: "reactor_chamber_side", 68: "batbox_side", 69: "mfe_front", 70: "mfsu_front", 71: "mv_transformer_front",
    72: "batbox_front", 73: "lv_transformer_front", 74: "hv_transformer_front", 75: "mfsu_side", 76: "tesla_coil",
    80: "personal_safe_bottom", 81: "personal_safe_top", 82: "personal_safe_side", 83: "personal_safe_front",
    84: "recycler_top", 85: "trade_o_mat_front", 86: "advanced_machine", 87: "induction_furnace_front",
    88: "induction_furnace_front_active", 89: "terraformer_side",
    92: "copper_block", 93: "tin_block", 94: "bronze_block", 95: "uranium_block",
}

# /IC2sprites/block_cable.png: a row per cable type, a column per paint color
CABLES = [
    "copper_cable", "uninsulated_copper_cable", "gold_cable", "insulated_gold_cable", "double_insulated_gold_cable",
    "hv_cable", "insulated_hv_cable", "double_insulated_hv_cable", "quadruple_insulated_hv_cable",
    "glass_fibre_cable", "ultra_low_current_cable",
]

COLORS = [
    "black", "red", "green", "brown", "blue", "purple", "cyan", "light_gray", "gray", "pink", "lime", "yellow",
    "light_blue", "magenta", "orange", "white",
]

# /IC2sprites/item_0.png slots
ITEMS = {
    0: "coal_dust", 1: "iron_dust", 2: "gold_dust", 3: "copper_dust", 4: "tin_dust", 5: "bronze_dust",
    6: "small_iron_dust", 7: "refined_iron", 8: "copper_ingot", 9: "tin_ingot", 10: "bronze_ingot",
    11: "mixed_metal_ingot", 12: "refined_uranium", 13: "uranium_chunk",
    16: "re_battery_0", 17: "re_battery_1", 18: "re_battery_2", 19: "re_battery_3", 20: "re_battery_4",
    21: "energy_crystal_0", 22: "energy_crystal_1", 23: "energy_crystal_2", 24: "energy_crystal_3",
    25: "energy_crystal_4", 26: "lapotron_crystal_0", 27: "lapotron_crystal_1", 28: "lapotron_crystal_2",
    29: "lapotron_crystal_3", 30: "lapotron_crystal_4", 31: "single_use_battery",
    32: "empty_cell", 33: "lava_cell", 34: "coal_cell", 35: "bio_cell", 37: "water_cell", 38: "uranium_cell",
    39: "coolant_cell", 40: "depleted_isotope_cell", 41: "re_enriched_uranium_cell", 42: "near_depleted_uranium_cell",
    43: "electrolyzed_water_cell",
    48: "mining_drill", 49: "diamond_drill", 50: "chainsaw", 51: "fuel_can_empty", 52: "fuel_can",
    53: "hydrated_coal_dust", 54: "hydrated_coal", 55: "plantball", 56: "compressed_plants", 57: "tin_can",
    58: "filled_tin_can", 59: "od_scanner", 60: "ov_scanner", 61: "dynamite_o_mote", 62: "dynamite",
    63: "sticky_dynamite", 64: "sticky_resin", 65: "rubber", 66: "treetap", 67: "rubber_boots", 68: "jetpack",
    69: "electric_jetpack", 70: "mining_laser", 71: "reactor_plating", 72: "heat_disperser", 73: "batpack",
    74: "raw_carbon_fibre", 75: "raw_carbon_mesh", 76: "carbon_plate", 77: "nano_saber", 78: "nano_saber_active",
    79: "nano_saber_shine",
    80: "bronze_pickaxe", 81: "bronze_axe", 82: "bronze_sword", 83: "bronze_shovel", 84: "bronze_hoe",
    85: "bronze_helmet", 86: "bronze_chestplate", 87: "bronze_leggings", 88: "bronze_boots", 89: "wrench",
    90: "eu_reader", 91: "painter", 92: "insulation_cutter", 93: "iridium_plate", 94: "electric_wrench",
    95: "frequency_transmitter",
    96: "electronic_circuit", 97: "advanced_circuit", 98: "scrap", 99: "uu_matter", 100: "advanced_alloy",
    101: "industrial_credit", 102: "reinforced_door", 103: "composite_vest", 104: "nano_helmet",
    105: "nano_chestplate", 106: "nano_leggings", 107: "nano_boots", 108: "quantum_helmet", 109: "quantum_chestplate",
    110: "quantum_leggings", 111: "quantum_boots",
    144: "tfbp", 145: "tfbp_cultivation", 146: "tfbp_irrigation", 147: "tfbp_chilling", 148: "tfbp_desertification",
    149: "tfbp_flatification", 155: "industrial_diamond", 156: "coal_chunk", 157: "compressed_coal_ball",
    158: "coal_ball", 159: "scrap_box",
}

ITEMS.update({112 + index: name for index, name in enumerate(CABLES)})
ITEMS.update({128 + index: f"painter_{color}" for index, color in enumerate(COLORS)})

GUIS = [
    "GUICanner", "GUICompressor", "GUIElecFurnace", "GUIElectricBlock", "GUIElectrolyzer", "GUIExtractor",
    "GUIGenerator", "GUIGeoGenerator", "GUIInduction", "GUIMacerator", "GUIMatter", "GUIMiner",
    "GUIPump", "GUIRecycler", "GUISolarGenerator", "GUITradeOMatClosed", "GUITradeOMatOpen", "GUIWaterGenerator",
    "GUIWindGenerator", "GUI6by3", "GUI6by4", "GUI6by5", "GUI6by6", "GUI6by7", "GUI6by8", "GUI6by9",
]

ARMOR = ["alloy_1", "batpack_1", "bronze_1", "bronze_2", "jetpack_1", "nano_1", "nano_2", "quantum_1", "quantum_2",
         "rubber_1"]

BLOCK_NAMES = {
    "machine": "Machine Block", "iron_furnace": "Iron Furnace", "electric_furnace": "Electric Furnace",
    "macerator": "Macerator", "extractor": "Extractor", "compressor": "Compressor", "canning_machine": "Canning Machine",
    "miner": "Miner", "pump": "Pump", "magnetizer": "Magnetizer", "electrolyzer": "Electrolyzer",
    "recycler": "Recycler", "advanced_machine": "Advanced Machine Block", "induction_furnace": "Induction Furnace",
    "mass_fabricator": "Mass Fabricator", "terraformer": "Terraformer", "copper_ore": "Copper Ore",
    "tin_ore": "Tin Ore", "uranium_ore": "Uranium Ore", "generator": "Generator",
    "geothermal_generator": "Geothermal Generator", "water_mill": "Water Mill", "solar_panel": "Solar Panel",
    "wind_mill": "Wind Mill", "nuclear_reactor": "Nuclear Reactor", "mining_pipe": "Mining Pipe",
    "mining_pipe_tip": "Mining Pipe", "rubber_wood": "Rubber Wood", "rubber_leaves": "Leaves",
    "rubber_sapling": "Rubber Tree Sapling", "resin": "Sticky Resin", "industrial_tnt": "Industrial TNT",
    "nuke": "Nuke", "dynamite_block": "Dynamite", "remote_dynamite_block": "Dynamite", "rubber_sheet": "Rubber Sheet",
    "reactor_chamber": "Reactor Chamber", "iron_fence": "Iron Fence", "reinforced_stone": "Reinforced Stone",
    "reinforced_glass": "Reinforced Glass", "reinforced_door_block": "Reinforced Door", "batbox": "BatBox", "mfe": "MFE",
    "mfsu": "MFSU", "lv_transformer": "LV-Transformer", "mv_transformer": "MV-Transformer",
    "hv_transformer": "HV-Transformer", "luminator": "Luminator", "personal_safe": "Personal Safe",
    "trade_o_mat": "Trade-O-Mat", "copper_block": "Copper Block", "tin_block": "Tin Block",
    "bronze_block": "Bronze Block", "uranium_block": "Uranium Block", "teleporter": "Teleporter",
    "tesla_coil": "Tesla Coil", "copper_cable": "Copper Cable", "uninsulated_copper_cable": "Uninsulated Copper Cable",
    "gold_cable": "Gold Cable", "insulated_gold_cable": "Insulated Gold Cable",
    "double_insulated_gold_cable": "2xIns. Gold Cable", "hv_cable": "HV Cable", "insulated_hv_cable": "Insulated HV Cable",
    "double_insulated_hv_cable": "2xIns. HV Cable", "quadruple_insulated_hv_cable": "4xIns. HV Cable",
    "glass_fibre_cable": "Glass Fibre Cable", "ultra_low_current_cable": "Ultra-Low-Current Cable",
}

ITEM_NAMES = {
    "coal_dust": "Coal Dust", "iron_dust": "Iron Dust", "gold_dust": "Gold Dust", "copper_dust": "Copper Dust",
    "tin_dust": "Tin Dust", "bronze_dust": "Bronze Dust", "small_iron_dust": "Small Pile of Iron Dust",
    "refined_iron": "Refined Iron", "copper_ingot": "Copper", "tin_ingot": "Tin", "bronze_ingot": "Bronze",
    "mixed_metal_ingot": "Mixed Metal Ingot", "refined_uranium": "Refined Uranium", "uranium_chunk": "Uranium Ore",
    "re_battery": "RE-Battery", "re_battery_empty": "RE-Battery", "single_use_battery": "Single-Use Battery",
    "energy_crystal": "Energy Crystal", "lapotron_crystal": "Lapotron Crystal", "empty_cell": "Empty Cell",
    "lava_cell": "Lava Cell", "mining_drill": "Mining Drill", "diamond_drill": "Diamond Drill", "chainsaw": "Chainsaw",
    "fuel_can": "Filled Fuel Can", "fuel_can_empty": "(Empty) Fuel Can", "coal_cell": "H. Coal Cell",
    "coalfuel_cell": "Coalfuel Cell", "bio_cell": "Bio Cell", "biofuel_cell": "Biofuel Cell",
    "hydrated_coal_dust": "Hydrated Coal Dust", "hydrated_coal": "H. Coal", "plantball": "Plantball",
    "compressed_plants": "Compressed Plants", "tin_can": "Tin Can", "filled_tin_can": "(Filled) Tin Can",
    "od_scanner": "OD Scanner", "ov_scanner": "OV Scanner", "water_cell": "Water Cell", "sticky_resin": "Sticky Resin",
    "rubber": "Rubber", "dynamite": "Dynamite", "sticky_dynamite": "Sticky Dynamite",
    "dynamite_o_mote": "Dynamite-O-Mote", "treetap": "Treetap", "rubber_boots": "Rubber Boots", "jetpack": "Jetpack",
    "electric_jetpack": "Electric Jetpack", "mining_laser": "Mining Laser", "uranium_cell": "Uranium Cell",
    "coolant_cell": "Coolant Cell", "reactor_plating": "Integrated Reactor Plating",
    "heat_disperser": "Integrated Heat Disperser", "depleted_isotope_cell": "Depleted Isotope Cell",
    "re_enriched_uranium_cell": "Re-Enriched Uranium Cell", "near_depleted_uranium_cell": "Near-depleted Uranium Cell",
    "bronze_pickaxe": "Bronze Pickaxe", "bronze_axe": "Bronze Axe", "bronze_sword": "Bronze Sword",
    "bronze_shovel": "Bronze Shovel", "bronze_hoe": "Bronze Hoe", "bronze_helmet": "Bronze Helmet",
    "bronze_chestplate": "Bronze Chestplate", "bronze_leggings": "Bronze Legs", "bronze_boots": "Bronze Boots",
    "electronic_circuit": "Electronic Circuit", "advanced_circuit": "Advanced Circuit",
    "advanced_alloy": "Advanced Alloy", "scrap": "Scrap", "uu_matter": "UU-Matter",
    "industrial_credit": "Industrial Credit", "reinforced_door": "Reinforced Door", "wrench": "Wrench",
    "eu_reader": "EU-Reader", "electrolyzed_water_cell": "Electrolyzed Water Cell", "batpack": "BatPack",
    "composite_vest": "Composite Vest", "nano_helmet": "NanoSuit Helmet", "nano_chestplate": "NanoSuit Bodyarmor",
    "nano_leggings": "NanoSuit Leggings", "nano_boots": "NanoSuit Boots", "quantum_helmet": "QuantumSuit Helmet",
    "quantum_chestplate": "QuantumSuit Bodyarmor", "quantum_leggings": "QuantumSuit Leggings",
    "quantum_boots": "QuantumSuit Boots", "painter": "Painter", "insulation_cutter": "Insulation Cutter",
    "raw_carbon_fibre": "Raw Carbon Fibre", "raw_carbon_mesh": "Raw Carbon Mesh", "carbon_plate": "Carbon Plate",
    "nano_saber": "Nano Saber", "nano_saber_active": "Nano Saber", "iridium_plate": "Iridium Plate",
    "tfbp": "TFBP - Empty", "tfbp_cultivation": "TFBP - Cultivation", "tfbp_irrigation": "TFBP - Irrigation",
    "tfbp_desertification": "TFBP - Desertification", "tfbp_chilling": "TFBP - Chilling",
    "tfbp_flatification": "TFBP - Flatification", "electric_wrench": "Electric Wrench", "scrap_box": "Scrap Box",
    "coal_ball": "Coal Ball", "compressed_coal_ball": "Compressed Coal Ball", "coal_chunk": "Coal Chunk",
    "industrial_diamond": "Industrial Diamond", "frequency_transmitter": "Frequency Transmitter",
}

ITEM_NAMES.update({f"painter_{color}": "Painter" for color in COLORS})

# text the screens and tools print, IC2 wrote it inline
MESSAGES = {
    "gui.inventory": "Inventory",
    "gui.energy": "Energy",
    "gui.power_level": "Power Level:",
    "gui.output": "Output: %s EU/t",
    "gui.storage": "%s / %s",
    "gui.reactor": "Nuclear Reactor",
    "gui.reactor_output": "Output: %s EU/t",
    "gui.pump": "Pump",
    "gui.miner": "Miner",
    "gui.matter": "Mass Fabricator",
    "gui.matter_progress": "Progress: %s",
    "gui.amplifier": "Amplifier: %s",
    "gui.trade_o_mat": "Trade-O-Mat",
    "gui.want": "Want",
    "gui.offer": "Offer",
    "gui.stock": "Stock: %s",
    "gui.heat": "Heat: %s",
    "gui.sun": "Sun",
    "gui.wind": "Wind",
    "gui.water": "Water",
    "gui.redstone_charging": "Redstone Charging",
    "meter.reading": "Average EU conducted: %s EU/t",
    "meter.idle": "EU-Reader: no energy flowing",
    "scanner.found": "Scan result: %s ore value",
    "scanner.nothing": "Scan result: nothing",
    "jetpack.hover_on": "Hover Mode enabled.",
    "jetpack.hover_off": "Hover Mode disabled.",
    "laser.mode": "Laser Mode: %s",
    "laser.mining": "Mining",
    "laser.low_focus": "Low-Focus",
    "laser.long_range": "Long-Range",
    "laser.scatter": "Scatter",
    "laser.explosive": "Explosive",
    "nano.vision_on": "Nightvision enabled.",
    "nano.vision_off": "Nightvision disabled.",
    "teleporter.linked": "Frequency Transmitter linked to Teleporter.",
    "teleporter.set": "Teleportation Target set.",
    "teleporter.unlinked": "Frequency Transmitter unlinked.",
    "personal.locked": "This safe is owned by %s.",
    "key.suit": "Suit Activate",
    "key.hover": "Hover Mode",
    "key.laser": "Laser Mode",
    "charge": "%s / %s EU",
}


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
    raw = b"".join(b"\x00" + bytes(value for pixel in row for value in pixel) for row in rows)

    def chunk(kind, body):
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)

    header = struct.pack(">IIBBBBB", len(rows[0]), len(rows), 8, 6, 0, 0, 0)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))


def cut(rows, slot):
    top, left = slot // SHEET_TILES * TILE, slot % SHEET_TILES * TILE
    return [row[left:left + TILE] for row in rows[top:top + TILE]]


def lang():
    lines = [f"tile.ic2.{name}.name={label}" for name, label in BLOCK_NAMES.items()]
    lines += [f"item.ic2.{name}.name={label}" for name, label in ITEM_NAMES.items()]
    lines += [f"ic2.{key}={text}" for key, text in MESSAGES.items()]
    return "\n".join(lines) + "\n"


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    with zipfile.ZipFile(sys.argv[1]) as archive:
        files = {name: archive.read(name) for name in archive.namelist() if not name.endswith("/")}

    def sheet(name):
        key = f"IC2sprites/{name}.png"
        if key not in files:
            raise SystemExit(f"{key} is missing, is this the IndustrialCraft 2 v1.00 zip?")
        return readPng(files[key])

    blocks = sheet("block_0")
    for slot, name in BLOCKS.items():
        writePng(OUT / "textures" / "block" / f"{name}.png", cut(blocks, slot))

    cables = sheet("block_cable")
    for row, cable in enumerate(CABLES):
        for column in range(len(COLORS)):
            # column 0 is unpainted, which the black painter also leaves
            suffix = "" if column == 0 else f"_{COLORS[column]}"
            writePng(OUT / "textures" / "block" / f"{cable}{suffix}.png", cut(cables, row * SHEET_TILES + column))

    items = sheet("item_0")
    for slot, name in ITEMS.items():
        writePng(OUT / "textures" / "item" / f"{name}.png", cut(items, slot))

    for gui in GUIS:
        (OUT / "textures" / "gui").mkdir(parents=True, exist_ok=True)
        (OUT / "textures" / "gui" / f"{gui[3:].lower()}.png").write_bytes(files[f"IC2sprites/{gui}.png"])

    (OUT / "textures" / "entity").mkdir(parents=True, exist_ok=True)
    (OUT / "textures" / "entity" / "laser.png").write_bytes(files["IC2sprites/laser.png"])

    for layer in ARMOR:
        name, index = layer.rsplit("_", 1)
        (OUT / "textures" / "armor").mkdir(parents=True, exist_ok=True)
        (OUT / "textures" / "armor" / f"{name}_layer_{index}.png").write_bytes(files[f"armor/{layer}.png"])

    (OUT / "lang").mkdir(parents=True, exist_ok=True)
    (OUT / "lang" / "en_US.lang").write_text(lang())
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
