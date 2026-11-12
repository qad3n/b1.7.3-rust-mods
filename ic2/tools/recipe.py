"""Writes assets.toml, the recipe the loader follows to build IC2's art from the player's own v1.00 zip.

Run from the port folder after changing a table: python3 -I tools/recipe.py
The zip has no sounds: IC2 played its sound names from a separate resource download, so the port plays them as
ic2.<name> and a pack may add them under assets/ic2/sounds.
"""

from pathlib import Path

RECIPE = Path(__file__).resolve().parent.parent / "assets.toml"
DOWNLOAD = "IndustrialCraft2 v1.00.zip"
SHA256 = "0c72d3480b1574eba1ef76fa42fe3bea270a49859fba730be7bf2f7e73e54409"
SPRITES = "IC2sprites/"
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



def quote(text: str) -> str:
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def copy(source: str, target: str) -> list[str]:
    return ["[[copy]]", f"from = {quote(source)}", f"to = {quote(target)}", ""]


def cut(sheet: str, target: str, tiles: dict[int, str]) -> list[str]:
    lines = ["[[slice]]", f"from = {quote(SPRITES + sheet)}", f"to = {quote(target)}", f"tile = {TILE}", "",
             "[slice.tiles]"]
    lines += [f"{slot} = {quote(name)}" for slot, name in sorted(tiles.items())]
    return lines + [""]


# a row per cable type and a column per paint color, column 0 is unpainted which the black painter also leaves
def cables() -> dict[int, str]:
    tiles = {}
    for row, cable in enumerate(CABLES):
        for column, color in enumerate(COLORS):
            suffix = "" if column == 0 else f"_{color}"
            tiles[row * SHEET_TILES + column] = f"{cable}{suffix}"
    return tiles


def main() -> None:
    lines = ["# generated by tools/recipe.py, edit the tables there", ""]
    lines += ["[[download]]", f"name = {quote(DOWNLOAD)}", f"sha256 = [{quote(SHA256)}]", ""]
    lines += [line for gui in GUIS for line in copy(f"{SPRITES}{gui}.png", f"textures/gui/{gui[3:].lower()}.png")]
    lines += copy(f"{SPRITES}laser.png", "textures/entity/laser.png")
    for layer in ARMOR:
        name, index = layer.rsplit("_", 1)
        lines += copy(f"armor/{layer}.png", f"textures/armor/{name}_layer_{index}.png")
    lines += cut("block_0.png", "textures/block/", BLOCKS)
    lines += cut("block_cable.png", "textures/block/", cables())
    lines += cut("item_0.png", "textures/item/", ITEMS)
    RECIPE.write_text("\n".join(lines).rstrip("\n") + "\n")
    print(f"wrote {RECIPE}")


if __name__ == "__main__":
    main()
