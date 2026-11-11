use b173_api::prelude::*;

// one ingredient or result: the item, how many and the damage it must have
#[derive(Clone, Copy)]
struct Part
{
    m_item: &'static str,
    m_count: i32,
    m_damage: i32,
}

const fn one(item: &'static str) -> Part { Part { m_item: item, m_count: 1, m_damage: 0 } }

const fn stack(item: &'static str, count: i32, damage: i32) -> Part
{
    Part { m_item: item, m_count: count, m_damage: damage }
}

enum Recipe
{
    Shaped(Part, &'static [&'static str], &'static [(char, Part)]),
    Shapeless(Part, &'static [Part]),
}

const fn shaped(result: Part, pattern: &'static [&'static str], keys: &'static [(char, Part)]) -> Recipe
{
    Recipe::Shaped(result, pattern, keys)
}

const fn shapeless(result: Part, inputs: &'static [Part]) -> Recipe { Recipe::Shapeless(result, inputs) }

// beta's materials by their shared tag, IC2's own come from their item rows
const VANILLA_MATERIALS: [(&str, &str); 7] = [
    ("minecraft:iron_ingot", "c:ingots/iron"),
    ("minecraft:gold_ingot", "c:ingots/gold"),
    ("minecraft:diamond", "c:gems/diamond"),
    ("minecraft:redstone", "c:dusts/redstone"),
    ("minecraft:glowstone_dust", "c:dusts/glowstone"),
    ("minecraft:stick", "c:rods/wooden"),
    ("minecraft:glass", "c:glass_blocks"),
];

impl Part
{
    fn stack(self) -> ItemStack { ItemStack::of(self.m_item, self.m_count).withDamage(self.m_damage) }

    // a plain material ingredient as its tag, so other mods' ingots, circuits and dusts craft IC2 items
    fn tag(self) -> Option<&'static str>
    {
        if self.m_damage != 0 || self.m_count != 1
        {
            return None;
        }

        crate::items::materialTag(self.m_item)
            .or_else(|| VANILLA_MATERIALS.iter().find(|(item, _)| *item == self.m_item).map(|(_, tag)| *tag))
    }

    fn ingredient(self) -> Result<IngredientSpec>
    {
        let ingredient: IngredientSpec = match self.tag()
        {
            Some(tag) => IngredientSpec { item: None, tag: Some(Identifier::parse(tag)?), meta: -1 },
            None => IngredientSpec { item: Some(Identifier::parse(self.m_item)?), tag: None, meta: self.m_damage },
        };
        Ok(ingredient)
    }
}

// mod_IC2's static block in its order, charged and empty variants of the electric parts kept apart by damage
const RECIPES: [Recipe; 246] = [
    shaped(one("ic2:iron_furnace"), &["III", "I I", "III"], &[('I', one("minecraft:iron_ingot"))]),
    shaped(
        one("ic2:iron_furnace"),
        &[" I ", "I I", "IFI"],
        &[('I', one("minecraft:iron_ingot")), ('F', one("minecraft:furnace"))],
    ),
    shaped(stack("minecraft:planks", 3, 0), &["W"], &[('W', one("ic2:rubber_wood"))]),
    shaped(one("ic2:treetap"), &[" P ", "PPP", "P  "], &[('P', one("minecraft:planks"))]),
    shaped(
        stack("ic2:copper_cable", 6, 0),
        &["RRR", "CCC", "RRR"],
        &[('C', one("ic2:copper_ingot")), ('R', one("ic2:rubber"))],
    ),
    shaped(
        stack("ic2:copper_cable", 6, 0),
        &["RCR", "RCR", "RCR"],
        &[('C', one("ic2:copper_ingot")), ('R', one("ic2:rubber"))],
    ),
    shaped(stack("ic2:uninsulated_copper_cable", 6, 0), &["CCC"], &[('C', one("ic2:copper_ingot"))]),
    shapeless(one("ic2:copper_cable"), &[one("ic2:rubber"), one("ic2:uninsulated_copper_cable")]),
    shaped(stack("ic2:gold_cable", 12, 0), &["GGG"], &[('G', one("minecraft:gold_ingot"))]),
    shaped(
        stack("ic2:insulated_gold_cable", 4, 0),
        &[" R ", "RGR", " R "],
        &[('G', one("minecraft:gold_ingot")), ('R', one("ic2:rubber"))],
    ),
    shapeless(one("ic2:insulated_gold_cable"), &[one("ic2:rubber"), one("ic2:gold_cable")]),
    shapeless(one("ic2:double_insulated_gold_cable"), &[one("ic2:rubber"), one("ic2:insulated_gold_cable")]),
    shapeless(one("ic2:double_insulated_gold_cable"), &[one("ic2:rubber"), one("ic2:rubber"), one("ic2:gold_cable")]),
    shaped(stack("ic2:hv_cable", 12, 0), &["III"], &[('I', one("ic2:refined_iron"))]),
    shaped(
        stack("ic2:insulated_hv_cable", 4, 0),
        &[" R ", "RIR", " R "],
        &[('I', one("ic2:refined_iron")), ('R', one("ic2:rubber"))],
    ),
    shapeless(one("ic2:insulated_hv_cable"), &[one("ic2:rubber"), one("ic2:hv_cable")]),
    shapeless(one("ic2:double_insulated_hv_cable"), &[one("ic2:rubber"), one("ic2:insulated_hv_cable")]),
    shapeless(one("ic2:quadruple_insulated_hv_cable"), &[one("ic2:rubber"), one("ic2:double_insulated_hv_cable")]),
    shapeless(
        one("ic2:quadruple_insulated_hv_cable"),
        &[one("ic2:rubber"), one("ic2:rubber"), one("ic2:insulated_hv_cable")],
    ),
    shapeless(one("ic2:double_insulated_hv_cable"), &[one("ic2:rubber"), one("ic2:rubber"), one("ic2:hv_cable")]),
    shapeless(
        one("ic2:quadruple_insulated_hv_cable"),
        &[one("ic2:rubber"), one("ic2:rubber"), one("ic2:rubber"), one("ic2:hv_cable")],
    ),
    shaped(
        stack("ic2:glass_fibre_cable", 16, 0),
        &["GGG", "RDR", "GGG"],
        &[('G', one("minecraft:glass")), ('R', one("minecraft:redstone")), ('D', one("minecraft:diamond"))],
    ),
    shaped(
        stack("ic2:glass_fibre_cable", 16, 0),
        &["GGG", "RDR", "GGG"],
        &[('G', one("minecraft:glass")), ('R', one("minecraft:redstone")), ('D', one("ic2:industrial_diamond"))],
    ),
    shaped(stack("ic2:ultra_low_current_cable", 9, 0), &["TTT"], &[('T', one("ic2:tin_ingot"))]),
    shaped(
        one("ic2:painter"),
        &[" CC", " IC", "I  "],
        &[('C', one("minecraft:wool")), ('I', one("minecraft:iron_ingot"))],
    ),
    shapeless(one("ic2:painter_black"), &[one("ic2:painter"), one("minecraft:dye")]),
    shapeless(one("ic2:painter_red"), &[one("ic2:painter"), stack("minecraft:dye", 1, 1)]),
    shapeless(one("ic2:painter_green"), &[one("ic2:painter"), stack("minecraft:dye", 1, 2)]),
    shapeless(one("ic2:painter_brown"), &[one("ic2:painter"), stack("minecraft:dye", 1, 3)]),
    shapeless(one("ic2:painter_blue"), &[one("ic2:painter"), stack("minecraft:dye", 1, 4)]),
    shapeless(one("ic2:painter_purple"), &[one("ic2:painter"), stack("minecraft:dye", 1, 5)]),
    shapeless(one("ic2:painter_cyan"), &[one("ic2:painter"), stack("minecraft:dye", 1, 6)]),
    shapeless(one("ic2:painter_light_gray"), &[one("ic2:painter"), stack("minecraft:dye", 1, 7)]),
    shapeless(one("ic2:painter_gray"), &[one("ic2:painter"), stack("minecraft:dye", 1, 8)]),
    shapeless(one("ic2:painter_pink"), &[one("ic2:painter"), stack("minecraft:dye", 1, 9)]),
    shapeless(one("ic2:painter_lime"), &[one("ic2:painter"), stack("minecraft:dye", 1, 10)]),
    shapeless(one("ic2:painter_yellow"), &[one("ic2:painter"), stack("minecraft:dye", 1, 11)]),
    shapeless(one("ic2:painter_light_blue"), &[one("ic2:painter"), stack("minecraft:dye", 1, 12)]),
    shapeless(one("ic2:painter_magenta"), &[one("ic2:painter"), stack("minecraft:dye", 1, 13)]),
    shapeless(one("ic2:painter_orange"), &[one("ic2:painter"), stack("minecraft:dye", 1, 14)]),
    shapeless(one("ic2:painter_white"), &[one("ic2:painter"), stack("minecraft:dye", 1, 15)]),
    shaped(
        one("ic2:insulation_cutter"),
        &["A A", " A ", "I I"],
        &[('A', one("ic2:refined_iron")), ('I', one("minecraft:iron_ingot"))],
    ),
    shaped(
        one("ic2:electronic_circuit"),
        &["CCC", "RIR", "CCC"],
        &[('I', one("ic2:refined_iron")), ('R', one("minecraft:redstone")), ('C', one("ic2:copper_cable"))],
    ),
    shaped(
        one("ic2:electronic_circuit"),
        &["CRC", "CIC", "CRC"],
        &[('I', one("ic2:refined_iron")), ('R', one("minecraft:redstone")), ('C', one("ic2:copper_cable"))],
    ),
    shaped(
        one("ic2:eu_reader"),
        &[" C ", "c c"],
        &[('C', one("ic2:electronic_circuit")), ('c', one("ic2:copper_cable"))],
    ),
    shaped(
        one("ic2:advanced_circuit"),
        &["RGR", "LCL", "RGR"],
        &[
            ('L', stack("minecraft:dye", 1, 4)),
            ('G', one("minecraft:glowstone_dust")),
            ('R', one("minecraft:redstone")),
            ('C', one("ic2:electronic_circuit")),
        ],
    ),
    shaped(
        one("ic2:advanced_circuit"),
        &["RLR", "GCG", "RLR"],
        &[
            ('L', stack("minecraft:dye", 1, 4)),
            ('G', one("minecraft:glowstone_dust")),
            ('R', one("minecraft:redstone")),
            ('C', one("ic2:electronic_circuit")),
        ],
    ),
    shaped(one("ic2:machine"), &["III", "I I", "III"], &[('I', one("ic2:refined_iron"))]),
    shaped(
        one("ic2:personal_safe"),
        &["c", "M", "C"],
        &[('c', one("ic2:electronic_circuit")), ('C', one("minecraft:chest")), ('M', one("ic2:machine"))],
    ),
    shaped(
        one("ic2:trade_o_mat"),
        &["RRR", "CMC"],
        &[('R', one("minecraft:redstone")), ('C', one("minecraft:chest")), ('M', one("ic2:machine"))],
    ),
    shaped(stack("ic2:refined_iron", 8, 0), &["M"], &[('M', one("ic2:machine"))]),
    shaped(
        one("ic2:electric_furnace"),
        &[" C ", "RFR"],
        &[('C', one("ic2:electronic_circuit")), ('R', one("minecraft:redstone")), ('F', one("ic2:iron_furnace"))],
    ),
    shaped(
        one("ic2:macerator"),
        &["FFF", "SMS", " C "],
        &[
            ('F', one("minecraft:flint")),
            ('S', one("minecraft:cobblestone")),
            ('M', one("ic2:machine")),
            ('C', one("ic2:electronic_circuit")),
        ],
    ),
    shaped(
        one("ic2:extractor"),
        &["TMT", "TCT"],
        &[('T', one("ic2:treetap")), ('M', one("ic2:machine")), ('C', one("ic2:electronic_circuit"))],
    ),
    shaped(
        one("ic2:compressor"),
        &["S S", "SMS", "SCS"],
        &[('S', one("minecraft:stone")), ('M', one("ic2:machine")), ('C', one("ic2:electronic_circuit"))],
    ),
    shaped(
        one("ic2:recycler"),
        &[" G ", "DMD", "IDI"],
        &[
            ('D', one("minecraft:dirt")),
            ('G', one("minecraft:glowstone_dust")),
            ('M', one("ic2:compressor")),
            ('I', one("ic2:refined_iron")),
        ],
    ),
    shaped(one("minecraft:bucket"), &["T T", " T "], &[('T', one("ic2:tin_ingot"))]),
    shaped(stack("ic2:tin_can", 4, 0), &["T T", "TTT"], &[('T', one("ic2:tin_ingot"))]),
    shaped(one("ic2:fuel_can_empty"), &[" TT", "T T", "TTT"], &[('T', one("ic2:tin_ingot"))]),
    shaped(
        one("ic2:canning_machine"),
        &["TCT", "TMT", "TTT"],
        &[('T', one("ic2:tin_ingot")), ('M', one("ic2:machine")), ('C', one("ic2:electronic_circuit"))],
    ),
    shaped(
        one("ic2:re_battery_empty"),
        &[" C ", "TRT", "TRT"],
        &[('T', one("ic2:tin_ingot")), ('R', one("minecraft:redstone")), ('C', one("ic2:copper_cable"))],
    ),
    shaped(
        stack("ic2:single_use_battery", 5, 0),
        &["C", "R", "D"],
        &[('D', one("ic2:coal_dust")), ('R', one("minecraft:redstone")), ('C', one("ic2:copper_cable"))],
    ),
    shaped(
        stack("ic2:single_use_battery", 5, 0),
        &["C", "D", "R"],
        &[('D', one("ic2:coal_dust")), ('R', one("minecraft:redstone")), ('C', one("ic2:copper_cable"))],
    ),
    shaped(
        stack("ic2:energy_crystal", 1, 10001),
        &["RRR", "RDR", "RRR"],
        &[('D', one("minecraft:diamond")), ('R', one("minecraft:redstone"))],
    ),
    shaped(
        stack("ic2:energy_crystal", 1, 10001),
        &["RRR", "RDR", "RRR"],
        &[('D', one("ic2:industrial_diamond")), ('R', one("minecraft:redstone"))],
    ),
    shaped(
        stack("ic2:lapotron_crystal", 1, 10001),
        &["LCL", "LDL", "LCL"],
        &[
            ('D', stack("ic2:energy_crystal", 1, 10001)),
            ('C', one("ic2:electronic_circuit")),
            ('L', stack("minecraft:dye", 1, 4)),
        ],
    ),
    shaped(
        stack("ic2:lapotron_crystal", 1, 9901),
        &["LCL", "LDL", "LCL"],
        &[
            ('D', stack("ic2:energy_crystal", 1, 1)),
            ('C', one("ic2:electronic_circuit")),
            ('L', stack("minecraft:dye", 1, 4)),
        ],
    ),
    shaped(
        stack("ic2:mining_drill", 1, 201),
        &[" I ", "ICI", "IBI"],
        &[('I', one("ic2:refined_iron")), ('B', one("ic2:re_battery_empty")), ('C', one("ic2:electronic_circuit"))],
    ),
    shaped(
        stack("ic2:mining_drill", 1, 1),
        &[" I ", "ICI", "IBI"],
        &[('I', one("ic2:refined_iron")), ('B', stack("ic2:re_battery", 1, 1)), ('C', one("ic2:electronic_circuit"))],
    ),
    shaped(
        stack("ic2:diamond_drill", 1, 121),
        &[" D ", "DdD"],
        &[('D', one("minecraft:diamond")), ('d', stack("ic2:mining_drill", 1, 201))],
    ),
    shaped(
        stack("ic2:diamond_drill", 1, 121),
        &[" D ", "DdD"],
        &[('D', one("ic2:industrial_diamond")), ('d', stack("ic2:mining_drill", 1, 201))],
    ),
    shaped(
        stack("ic2:diamond_drill", 1, 1),
        &[" D ", "DdD"],
        &[('D', one("minecraft:diamond")), ('d', stack("ic2:mining_drill", 1, 1))],
    ),
    shaped(
        stack("ic2:diamond_drill", 1, 1),
        &[" D ", "DdD"],
        &[('D', one("ic2:industrial_diamond")), ('d', stack("ic2:mining_drill", 1, 1))],
    ),
    shaped(
        stack("ic2:chainsaw", 1, 201),
        &[" II", "ICI", "BI "],
        &[('I', one("ic2:refined_iron")), ('B', one("ic2:re_battery_empty")), ('C', one("ic2:electronic_circuit"))],
    ),
    shaped(
        stack("ic2:chainsaw", 1, 1),
        &[" II", "ICI", "BI "],
        &[('I', one("ic2:refined_iron")), ('B', stack("ic2:re_battery", 1, 1)), ('C', one("ic2:electronic_circuit"))],
    ),
    shaped(
        stack("ic2:od_scanner", 1, 201),
        &[" G ", "CBC", "ccc"],
        &[
            ('B', one("ic2:re_battery_empty")),
            ('c', one("ic2:copper_cable")),
            ('G', one("minecraft:glowstone_dust")),
            ('C', one("ic2:electronic_circuit")),
        ],
    ),
    shaped(
        stack("ic2:od_scanner", 1, 1),
        &[" G ", "CBC", "ccc"],
        &[
            ('B', stack("ic2:re_battery", 1, 1)),
            ('c', one("ic2:copper_cable")),
            ('G', one("minecraft:glowstone_dust")),
            ('C', one("ic2:electronic_circuit")),
        ],
    ),
    shaped(
        stack("ic2:ov_scanner", 1, 201),
        &[" G ", "GCG", "cSc"],
        &[
            ('S', stack("ic2:od_scanner", 1, 201)),
            ('c', one("ic2:copper_cable")),
            ('G', one("minecraft:glowstone_dust")),
            ('C', one("ic2:advanced_circuit")),
        ],
    ),
    shaped(
        stack("ic2:ov_scanner", 1, 1),
        &[" G ", "GCG", "cSc"],
        &[
            ('S', stack("ic2:od_scanner", 1, 1)),
            ('c', one("ic2:copper_cable")),
            ('G', one("minecraft:glowstone_dust")),
            ('C', one("ic2:advanced_circuit")),
        ],
    ),
    shaped(
        stack("ic2:mining_pipe", 8, 0),
        &["I I", "I I", "ITI"],
        &[('I', one("ic2:refined_iron")), ('T', one("ic2:treetap"))],
    ),
    shaped(
        one("ic2:miner"),
        &["CMC", " P ", " P "],
        &[('P', one("ic2:mining_pipe")), ('M', one("ic2:machine")), ('C', one("ic2:electronic_circuit"))],
    ),
    shaped(stack("ic2:empty_cell", 16, 0), &[" T ", "T T", " T "], &[('T', one("ic2:tin_ingot"))]),
    shapeless(
        one("minecraft:obsidian"),
        &[one("ic2:water_cell"), one("ic2:water_cell"), one("ic2:lava_cell"), one("ic2:lava_cell")],
    ),
    shapeless(one("ic2:hydrated_coal_dust"), &[one("ic2:coal_dust"), one("minecraft:water_bucket")]),
    shapeless(one("ic2:hydrated_coal_dust"), &[one("ic2:coal_dust"), one("ic2:water_cell")]),
    shaped(
        stack("ic2:hydrated_coal_dust", 8, 0),
        &["CCC", "CWC", "CCC"],
        &[('C', one("ic2:coal_dust")), ('W', one("minecraft:water_bucket"))],
    ),
    shaped(
        stack("ic2:hydrated_coal_dust", 8, 0),
        &["CCC", "CWC", "CCC"],
        &[('C', one("ic2:coal_dust")), ('W', one("ic2:water_cell"))],
    ),
    shapeless(one("ic2:coal_cell"), &[one("ic2:empty_cell"), one("ic2:hydrated_coal")]),
    shaped(one("ic2:plantball"), &["PPP", "P P", "PPP"], &[('P', one("minecraft:sapling"))]),
    shaped(one("ic2:plantball"), &["PPP", "P P", "PPP"], &[('P', one("ic2:rubber_sapling"))]),
    shaped(one("ic2:plantball"), &["PPP", "P P", "PPP"], &[('P', one("minecraft:wheat"))]),
    shaped(one("ic2:plantball"), &["PPP", "P P", "PPP"], &[('P', one("minecraft:reeds"))]),
    shaped(one("ic2:plantball"), &["PPP", "P P", "PPP"], &[('P', one("minecraft:cactus"))]),
    shapeless(
        one("ic2:bio_cell"),
        &[one("ic2:empty_cell"), one("ic2:compressed_plants"), one("ic2:compressed_plants")],
    ),
    shaped(
        one("ic2:pump"),
        &["cCc", "cMc", "PTP"],
        &[
            ('c', one("ic2:empty_cell")),
            ('T', one("ic2:treetap")),
            ('P', one("ic2:mining_pipe")),
            ('M', one("ic2:machine")),
            ('C', one("ic2:electronic_circuit")),
        ],
    ),
    shaped(stack("ic2:iron_fence", 12, 0), &["III", "III"], &[('I', one("minecraft:iron_ingot"))]),
    shaped(
        one("ic2:magnetizer"),
        &["RFR", "RMR", "RFR"],
        &[('R', one("minecraft:redstone")), ('F', one("ic2:iron_fence")), ('M', one("ic2:machine"))],
    ),
    shaped(
        one("ic2:electrolyzer"),
        &["c c", "cCc", "EME"],
        &[
            ('E', one("ic2:empty_cell")),
            ('c', one("ic2:copper_cable")),
            ('M', one("ic2:machine")),
            ('C', one("ic2:electronic_circuit")),
        ],
    ),
    shaped(
        one("ic2:generator"),
        &["B", "M", "F"],
        &[('B', one("ic2:re_battery_empty")), ('F', one("minecraft:furnace")), ('M', one("ic2:machine"))],
    ),
    shaped(
        one("ic2:generator"),
        &["B", "M", "F"],
        &[('B', stack("ic2:re_battery", 1, 1)), ('F', one("minecraft:furnace")), ('M', one("ic2:machine"))],
    ),
    shaped(
        one("ic2:generator"),
        &["B", "I", "F"],
        &[('B', one("ic2:re_battery_empty")), ('F', one("ic2:iron_furnace")), ('I', one("ic2:refined_iron"))],
    ),
    shaped(
        one("ic2:generator"),
        &["B", "I", "F"],
        &[('B', stack("ic2:re_battery", 1, 1)), ('F', one("ic2:iron_furnace")), ('I', one("ic2:refined_iron"))],
    ),
    shaped(
        one("ic2:geothermal_generator"),
        &["gCg", "gCg", "IGI"],
        &[
            ('G', one("ic2:generator")),
            ('C', one("ic2:empty_cell")),
            ('g', one("minecraft:glass")),
            ('I', one("ic2:refined_iron")),
        ],
    ),
    shaped(
        one("ic2:water_mill"),
        &["SPS", "PGP", "SPS"],
        &[('S', one("minecraft:stick")), ('P', one("minecraft:planks")), ('G', one("ic2:generator"))],
    ),
    shaped(
        one("ic2:solar_panel"),
        &["CgC", "gCg", "cGc"],
        &[
            ('G', one("ic2:generator")),
            ('C', one("ic2:coal_dust")),
            ('g', one("minecraft:glass")),
            ('c', one("ic2:copper_cable")),
        ],
    ),
    shaped(
        one("ic2:wind_mill"),
        &["I I", " G ", "I I"],
        &[('I', one("minecraft:iron_ingot")), ('G', one("ic2:generator"))],
    ),
    shapeless(
        stack("ic2:bronze_dust", 2, 0),
        &[one("ic2:tin_dust"), one("ic2:copper_dust"), one("ic2:copper_dust"), one("ic2:copper_dust")],
    ),
    shaped(
        one("ic2:bronze_pickaxe"),
        &["BBB", " S ", " S "],
        &[('B', one("ic2:bronze_ingot")), ('S', one("minecraft:stick"))],
    ),
    shaped(
        one("ic2:bronze_axe"),
        &["BB", "SB", "S "],
        &[('B', one("ic2:bronze_ingot")), ('S', one("minecraft:stick"))],
    ),
    shaped(
        one("ic2:bronze_hoe"),
        &["BB", "S ", "S "],
        &[('B', one("ic2:bronze_ingot")), ('S', one("minecraft:stick"))],
    ),
    shaped(one("ic2:bronze_sword"), &["B", "B", "S"], &[('B', one("ic2:bronze_ingot")), ('S', one("minecraft:stick"))]),
    shaped(
        one("ic2:bronze_shovel"),
        &[" B ", " S ", " S "],
        &[('B', one("ic2:bronze_ingot")), ('S', one("minecraft:stick"))],
    ),
    shaped(one("ic2:bronze_helmet"), &["BBB", "B B"], &[('B', one("ic2:bronze_ingot"))]),
    shaped(one("ic2:bronze_chestplate"), &["B B", "BBB", "BBB"], &[('B', one("ic2:bronze_ingot"))]),
    shaped(one("ic2:bronze_leggings"), &["BBB", "B B", "B B"], &[('B', one("ic2:bronze_ingot"))]),
    shaped(one("ic2:bronze_boots"), &["B B", "B B"], &[('B', one("ic2:bronze_ingot"))]),
    shaped(one("ic2:wrench"), &["B B", "BBB", " B "], &[('B', one("ic2:bronze_ingot"))]),
    shaped(
        stack("ic2:electric_wrench", 1, 201),
        &["  W", " C ", "B  "],
        &[('W', one("ic2:wrench")), ('B', one("ic2:re_battery_empty")), ('C', one("ic2:electronic_circuit"))],
    ),
    shaped(
        stack("ic2:electric_wrench", 1, 1),
        &["  W", " C ", "B  "],
        &[('W', one("ic2:wrench")), ('B', stack("ic2:re_battery", 1, 1)), ('C', one("ic2:electronic_circuit"))],
    ),
    shaped(
        stack("minecraft:rail", 8, 0),
        &["B B", "BsB", " B "],
        &[('B', one("ic2:bronze_ingot")), ('s', one("minecraft:stick"))],
    ),
    shaped(
        stack("ic2:mixed_metal_ingot", 2, 0),
        &["III", "BBB", "TTT"],
        &[('I', one("ic2:refined_iron")), ('B', one("ic2:bronze_ingot")), ('T', one("ic2:tin_ingot"))],
    ),
    shaped(
        stack("ic2:reinforced_stone", 8, 0),
        &["SSS", "SAS", "SSS"],
        &[('S', one("minecraft:stone")), ('A', one("ic2:advanced_alloy"))],
    ),
    shaped(
        stack("ic2:reinforced_glass", 7, 0),
        &["GAG", "GGG", "GAG"],
        &[('G', one("minecraft:glass")), ('A', one("ic2:advanced_alloy"))],
    ),
    shaped(
        stack("ic2:reinforced_glass", 7, 0),
        &["GGG", "AGA", "GGG"],
        &[('G', one("minecraft:glass")), ('A', one("ic2:advanced_alloy"))],
    ),
    shaped(one("ic2:reinforced_door"), &["SS", "SS", "SS"], &[('S', one("ic2:reinforced_stone"))]),
    shaped(
        one("ic2:composite_vest"),
        &["A A", "ALA", "AIA"],
        &[
            ('L', one("minecraft:leather_chestplate")),
            ('I', one("minecraft:iron_chestplate")),
            ('A', one("ic2:advanced_alloy")),
        ],
    ),
    shaped(
        one("ic2:composite_vest"),
        &["A A", "AIA", "ALA"],
        &[
            ('L', one("minecraft:leather_chestplate")),
            ('I', one("minecraft:iron_chestplate")),
            ('A', one("ic2:advanced_alloy")),
        ],
    ),
    shaped(
        one("ic2:reactor_plating"),
        &[" C ", "CAC", " C "],
        &[('C', one("ic2:copper_ingot")), ('A', one("ic2:advanced_alloy"))],
    ),
    shaped(
        one("ic2:heat_disperser"),
        &["ici", "CPC"],
        &[
            ('C', one("ic2:coolant_cell")),
            ('c', one("ic2:advanced_circuit")),
            ('i', one("ic2:copper_ingot")),
            ('P', one("ic2:reactor_plating")),
        ],
    ),
    shaped(
        one("ic2:reactor_chamber"),
        &["ACA", "PMP", "APA"],
        &[
            ('A', one("ic2:advanced_alloy")),
            ('C', one("ic2:heat_disperser")),
            ('P', one("ic2:reactor_plating")),
            ('M', one("ic2:machine")),
        ],
    ),
    shaped(
        stack("ic2:near_depleted_uranium_cell", 8, 0),
        &["CCC", "CUC", "CCC"],
        &[('C', one("ic2:empty_cell")), ('U', one("ic2:refined_uranium"))],
    ),
    shapeless(one("ic2:uranium_cell"), &[one("ic2:empty_cell"), one("ic2:refined_uranium")]),
    shapeless(one("ic2:uranium_cell"), &[one("ic2:re_enriched_uranium_cell"), one("ic2:coal_dust")]),
    shapeless(
        stack("ic2:depleted_isotope_cell", 1, 9999),
        &[one("ic2:near_depleted_uranium_cell"), one("ic2:coal_dust")],
    ),
    shaped(
        one("ic2:nuclear_reactor"),
        &["AcA", "CGC", "AcA"],
        &[
            ('A', one("ic2:advanced_alloy")),
            ('C', one("ic2:reactor_chamber")),
            ('c', one("ic2:advanced_circuit")),
            ('G', one("ic2:generator")),
        ],
    ),
    shaped(one("ic2:raw_carbon_fibre"), &["CC", "CC"], &[('C', one("ic2:coal_dust"))]),
    shapeless(one("ic2:raw_carbon_mesh"), &[one("ic2:raw_carbon_fibre"), one("ic2:raw_carbon_fibre")]),
    shaped(
        stack("ic2:nano_helmet", 1, 251),
        &["CcC", "CGC"],
        &[('C', one("ic2:carbon_plate")), ('c', stack("ic2:energy_crystal", 1, 10001)), ('G', one("minecraft:glass"))],
    ),
    shaped(
        stack("ic2:nano_chestplate", 1, 251),
        &["C C", "CcC", "CCC"],
        &[('C', one("ic2:carbon_plate")), ('c', stack("ic2:energy_crystal", 1, 10001))],
    ),
    shaped(
        stack("ic2:nano_leggings", 1, 251),
        &["CcC", "C C", "C C"],
        &[('C', one("ic2:carbon_plate")), ('c', stack("ic2:energy_crystal", 1, 10001))],
    ),
    shaped(
        stack("ic2:nano_boots", 1, 251),
        &["C C", "CcC"],
        &[('C', one("ic2:carbon_plate")), ('c', stack("ic2:energy_crystal", 1, 10001))],
    ),
    shaped(
        stack("ic2:nano_saber", 1, 5001),
        &["GA ", "GA ", "CcC"],
        &[
            ('C', one("ic2:carbon_plate")),
            ('c', stack("ic2:energy_crystal", 1, 10001)),
            ('G', one("minecraft:glowstone_dust")),
            ('A', one("ic2:advanced_alloy")),
        ],
    ),
    shaped(
        stack("ic2:nano_helmet", 1, 1),
        &["CcC", "CGC"],
        &[('C', one("ic2:carbon_plate")), ('c', stack("ic2:energy_crystal", 1, 1)), ('G', one("minecraft:glass"))],
    ),
    shaped(
        stack("ic2:nano_chestplate", 1, 1),
        &["C C", "CcC", "CCC"],
        &[('C', one("ic2:carbon_plate")), ('c', stack("ic2:energy_crystal", 1, 1))],
    ),
    shaped(
        stack("ic2:nano_leggings", 1, 1),
        &["CcC", "C C", "C C"],
        &[('C', one("ic2:carbon_plate")), ('c', stack("ic2:energy_crystal", 1, 1))],
    ),
    shaped(
        stack("ic2:nano_boots", 1, 1),
        &["C C", "CcC"],
        &[('C', one("ic2:carbon_plate")), ('c', stack("ic2:energy_crystal", 1, 1))],
    ),
    shaped(
        stack("ic2:nano_saber", 1, 1),
        &["GA ", "GA ", "CcC"],
        &[
            ('C', one("ic2:carbon_plate")),
            ('c', stack("ic2:energy_crystal", 1, 1)),
            ('G', one("minecraft:glowstone_dust")),
            ('A', one("ic2:advanced_alloy")),
        ],
    ),
    shaped(
        one("ic2:advanced_machine"),
        &[" A ", "CMC", " A "],
        &[('A', one("ic2:advanced_alloy")), ('C', one("ic2:carbon_plate")), ('M', one("ic2:machine"))],
    ),
    shaped(
        one("ic2:advanced_machine"),
        &[" C ", "AMA", " C "],
        &[('A', one("ic2:advanced_alloy")), ('C', one("ic2:carbon_plate")), ('M', one("ic2:machine"))],
    ),
    shaped(
        one("ic2:induction_furnace"),
        &["CCC", "CFC", "CMC"],
        &[('C', one("ic2:copper_ingot")), ('F', one("ic2:electric_furnace")), ('M', one("ic2:advanced_machine"))],
    ),
    shaped(
        stack("ic2:industrial_tnt", 4, 0),
        &["FFF", "TTT", "FFF"],
        &[('F', one("minecraft:flint")), ('T', one("minecraft:tnt"))],
    ),
    shaped(
        stack("ic2:industrial_tnt", 4, 0),
        &["FTF", "FTF", "FTF"],
        &[('F', one("minecraft:flint")), ('T', one("minecraft:tnt"))],
    ),
    shaped(
        stack("ic2:dynamite", 8, 0),
        &["S", "T"],
        &[('S', one("minecraft:string")), ('T', one("ic2:industrial_tnt"))],
    ),
    shaped(
        stack("ic2:sticky_dynamite", 8, 0),
        &["DDD", "DRD", "DDD"],
        &[('D', one("ic2:dynamite")), ('R', one("ic2:sticky_resin"))],
    ),
    shaped(
        stack("ic2:industrial_tnt", 4, 0),
        &[" c ", "GCG", "TTT"],
        &[
            ('c', one("ic2:copper_cable")),
            ('G', one("minecraft:glowstone_dust")),
            ('C', one("ic2:electronic_circuit")),
            ('T', one("minecraft:tnt")),
        ],
    ),
    shaped(stack("ic2:rubber_sheet", 3, 0), &["RRR", "RRR"], &[('R', one("ic2:rubber"))]),
    shaped(one("ic2:rubber_boots"), &["R R", "R R", "RCR"], &[('R', one("ic2:rubber")), ('C', one("minecraft:wool"))]),
    shaped(
        one("minecraft:glowstone_dust"),
        &["RGR", "GRG", "RGR"],
        &[('R', one("minecraft:redstone")), ('G', one("ic2:gold_dust"))],
    ),
    shaped(
        stack("minecraft:gunpowder", 3, 0),
        &["RCR", "CRC", "RCR"],
        &[('R', one("minecraft:redstone")), ('C', one("ic2:coal_dust"))],
    ),
    shaped(
        stack("ic2:single_use_battery", 8, 0),
        &["c", "C", "R"],
        &[('R', one("minecraft:redstone")), ('C', one("ic2:hydrated_coal_dust")), ('c', one("ic2:copper_cable"))],
    ),
    shaped(
        stack("ic2:single_use_battery", 8, 0),
        &["c", "R", "C"],
        &[('R', one("minecraft:redstone")), ('C', one("ic2:hydrated_coal_dust")), ('c', one("ic2:copper_cable"))],
    ),
    shaped(
        one("ic2:batbox"),
        &["PCP", "BBB", "PPP"],
        &[('P', one("minecraft:planks")), ('C', one("ic2:copper_cable")), ('B', one("ic2:re_battery_empty"))],
    ),
    shaped(
        one("ic2:mfe"),
        &["cCc", "CMC", "cCc"],
        &[('M', one("ic2:machine")), ('c', one("ic2:copper_cable")), ('C', stack("ic2:energy_crystal", 1, 10001))],
    ),
    shaped(
        one("ic2:mfsu"),
        &["LCL", "LML", "LAL"],
        &[
            ('M', one("ic2:mfe")),
            ('A', one("ic2:advanced_machine")),
            ('C', one("ic2:advanced_circuit")),
            ('L', stack("ic2:lapotron_crystal", 1, 10001)),
        ],
    ),
    shaped(
        one("ic2:lv_transformer"),
        &["PCP", "ccc", "PCP"],
        &[('P', one("minecraft:planks")), ('C', one("ic2:copper_cable")), ('c', one("ic2:copper_ingot"))],
    ),
    shaped(
        one("ic2:mv_transformer"),
        &[" C ", " M ", " C "],
        &[('M', one("ic2:machine")), ('C', one("ic2:copper_cable"))],
    ),
    shaped(
        one("ic2:hv_transformer"),
        &[" c ", "CED", " c "],
        &[
            ('E', one("ic2:mv_transformer")),
            ('c', one("ic2:copper_cable")),
            ('D', stack("ic2:energy_crystal", 1, 10001)),
            ('C', one("ic2:electronic_circuit")),
        ],
    ),
    shaped(
        one("ic2:eu_reader"),
        &[" G ", "cCc", "c c"],
        &[('G', one("minecraft:glowstone_dust")), ('c', one("ic2:copper_cable")), ('C', one("ic2:electronic_circuit"))],
    ),
    shaped(
        stack("ic2:jetpack", 1, 18001),
        &["ICI", "IFI", "R R"],
        &[
            ('I', one("ic2:refined_iron")),
            ('C', one("ic2:electronic_circuit")),
            ('F', one("ic2:fuel_can_empty")),
            ('R', one("minecraft:redstone")),
        ],
    ),
    shaped(
        stack("ic2:electric_jetpack", 1, 1),
        &["ICI", "IBI", "G G"],
        &[
            ('I', one("ic2:refined_iron")),
            ('C', one("ic2:advanced_circuit")),
            ('B', one("ic2:batbox")),
            ('G', one("minecraft:glowstone_dust")),
        ],
    ),
    shaped(
        stack("ic2:electric_jetpack", 1, 10001),
        &["ICI", "IBI", "G G"],
        &[
            ('I', one("ic2:refined_iron")),
            ('C', one("ic2:advanced_circuit")),
            ('B', one("ic2:batbox")),
            ('G', one("minecraft:glowstone_dust")),
        ],
    ),
    shaped(
        stack("ic2:batpack", 1, 30001),
        &["BCB", "BTB", "B B"],
        &[('T', one("ic2:tin_ingot")), ('C', one("ic2:electronic_circuit")), ('B', one("ic2:re_battery_empty"))],
    ),
    shaped(
        stack("ic2:batpack", 1, 1),
        &["BCB", "BTB", "B B"],
        &[('T', one("ic2:tin_ingot")), ('C', one("ic2:electronic_circuit")), ('B', stack("ic2:re_battery", 1, 1))],
    ),
    shaped(
        stack("ic2:mining_laser", 1, 8001),
        &["Rcc", "AAC", " AA"],
        &[
            ('A', one("ic2:advanced_alloy")),
            ('C', one("ic2:advanced_circuit")),
            ('c', stack("ic2:energy_crystal", 1, 10001)),
            ('R', one("minecraft:redstone")),
        ],
    ),
    shaped(
        stack("ic2:mining_laser", 1, 1),
        &["Rcc", "AAC", " AA"],
        &[
            ('A', one("ic2:advanced_alloy")),
            ('C', one("ic2:advanced_circuit")),
            ('c', stack("ic2:energy_crystal", 1, 1)),
            ('R', one("minecraft:redstone")),
        ],
    ),
    shaped(stack("ic2:industrial_credit", 8, 0), &["II", "II"], &[('I', one("ic2:refined_iron"))]),
    shaped(
        one("ic2:mass_fabricator"),
        &["GCG", "ALA", "GCG"],
        &[
            ('A', one("ic2:advanced_machine")),
            ('L', stack("ic2:lapotron_crystal", 1, 10001)),
            ('G', one("minecraft:glowstone_dust")),
            ('C', one("ic2:advanced_circuit")),
        ],
    ),
    shaped(
        one("ic2:mass_fabricator"),
        &["GCG", "ALA", "GCG"],
        &[
            ('A', one("ic2:advanced_machine")),
            ('L', stack("ic2:lapotron_crystal", 1, 1)),
            ('G', one("minecraft:glowstone_dust")),
            ('C', one("ic2:advanced_circuit")),
        ],
    ),
    shaped(stack("minecraft:stone", 16, 0), &["   ", " M ", "   "], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:glass", 32, 0), &[" M ", "M M", " M "], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:grass", 16, 0), &["   ", "M  ", "M  "], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:mossy_cobblestone", 16, 0), &["   ", " M ", "M M"], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:sandstone", 16, 0), &["   ", "  M", " M "], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:snow", 4, 0), &["M M", "   ", "   "], &[('M', one("ic2:uu_matter"))]),
    shaped(one("minecraft:water"), &["   ", " M ", " M "], &[('M', one("ic2:uu_matter"))]),
    shaped(one("minecraft:lava"), &[" M ", " M ", " M "], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:coal", 2, 0), &["  M", "M  ", "  M"], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:iron_ore", 2, 0), &["M M", " M ", "M M"], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:gold_ore", 2, 0), &[" M ", "MMM", " M "], &[('M', one("ic2:uu_matter"))]),
    shaped(one("minecraft:diamond"), &["MMM", "MMM", "MMM"], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:redstone", 24, 0), &["   ", " M ", "MMM"], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("ic2:copper_ore", 5, 0), &["  M", "M M", "   "], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("ic2:tin_ore", 5, 0), &["   ", "M M", "  M"], &[('M', one("ic2:uu_matter"))]),
    shaped(one("ic2:uranium_chunk"), &["M M", "M M", "M M"], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:obsidian", 12, 0), &["M M", "M M", "   "], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:netherrack", 16, 0), &["  M", " M ", "M  "], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:glowstone", 8, 0), &[" M ", "M M", "MMM"], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:log", 8, 0), &[" M ", "   ", "   "], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:dye", 9, 4), &[" M ", " M ", " MM"], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:feather", 32, 0), &[" M ", " M ", "M M"], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:snowball", 16, 0), &["   ", "   ", "MMM"], &[('M', one("ic2:uu_matter"))]),
    shaped(stack("minecraft:gunpowder", 15, 0), &["MMM", "M  ", "MMM"], &[('M', one("ic2:uu_matter"))]),
    shaped(one("ic2:iridium_plate"), &["MMM", " M ", "MMM"], &[('M', one("ic2:uu_matter"))]),
    shaped(
        stack("ic2:quantum_helmet", 1, 1001),
        &["ILI", " G "],
        &[
            ('I', one("ic2:advanced_circuit")),
            ('L', stack("ic2:lapotron_crystal", 1, 10001)),
            ('G', one("ic2:reinforced_glass")),
        ],
    ),
    shaped(
        stack("ic2:quantum_chestplate", 1, 1001),
        &["A A", "ILI", "IAI"],
        &[
            ('I', one("ic2:iridium_plate")),
            ('L', stack("ic2:lapotron_crystal", 1, 10001)),
            ('A', one("ic2:advanced_alloy")),
        ],
    ),
    shaped(
        stack("ic2:quantum_leggings", 1, 1001),
        &["MLM", "I I", "G G"],
        &[
            ('I', one("ic2:iridium_plate")),
            ('L', stack("ic2:lapotron_crystal", 1, 10001)),
            ('G', one("minecraft:glowstone_dust")),
            ('M', one("ic2:machine")),
        ],
    ),
    shaped(
        stack("ic2:quantum_boots", 1, 1001),
        &["I I", "RLR"],
        &[
            ('I', one("ic2:iridium_plate")),
            ('L', stack("ic2:lapotron_crystal", 1, 10001)),
            ('R', one("ic2:rubber_boots")),
        ],
    ),
    shaped(
        stack("ic2:quantum_helmet", 1, 1),
        &["ILI", " G "],
        &[
            ('I', one("ic2:advanced_circuit")),
            ('L', stack("ic2:lapotron_crystal", 1, 1)),
            ('G', one("ic2:reinforced_glass")),
        ],
    ),
    shaped(
        stack("ic2:quantum_chestplate", 1, 1),
        &["A A", "ILI", "IAI"],
        &[
            ('I', one("ic2:iridium_plate")),
            ('L', stack("ic2:lapotron_crystal", 1, 1)),
            ('A', one("ic2:advanced_alloy")),
        ],
    ),
    shaped(
        stack("ic2:quantum_leggings", 1, 1),
        &["MLM", "I I", "G G"],
        &[
            ('I', one("ic2:iridium_plate")),
            ('L', stack("ic2:lapotron_crystal", 1, 1)),
            ('G', one("minecraft:glowstone_dust")),
            ('M', one("ic2:machine")),
        ],
    ),
    shaped(
        stack("ic2:quantum_boots", 1, 1),
        &["I I", "RLR"],
        &[('I', one("ic2:iridium_plate")), ('L', stack("ic2:lapotron_crystal", 1, 1)), ('R', one("ic2:rubber_boots"))],
    ),
    shaped(one("ic2:copper_block"), &["MMM", "MMM", "MMM"], &[('M', one("ic2:copper_ingot"))]),
    shaped(one("ic2:tin_block"), &["MMM", "MMM", "MMM"], &[('M', one("ic2:tin_ingot"))]),
    shaped(one("ic2:bronze_block"), &["MMM", "MMM", "MMM"], &[('M', one("ic2:bronze_ingot"))]),
    shaped(one("ic2:uranium_block"), &["MMM", "MMM", "MMM"], &[('M', one("ic2:refined_uranium"))]),
    shaped(stack("ic2:copper_ingot", 9, 0), &["B"], &[('B', one("ic2:copper_block"))]),
    shaped(stack("ic2:tin_ingot", 9, 0), &["B"], &[('B', one("ic2:tin_block"))]),
    shaped(stack("ic2:bronze_ingot", 9, 0), &["B"], &[('B', one("ic2:bronze_block"))]),
    shaped(stack("ic2:refined_uranium", 9, 0), &["B"], &[('B', one("ic2:uranium_block"))]),
    shaped(one("ic2:scrap_box"), &["SSS", "SSS", "SSS"], &[('S', one("ic2:scrap"))]),
    shaped(one("ic2:coal_ball"), &["CCC", "CFC", "CCC"], &[('C', one("ic2:coal_dust")), ('F', one("minecraft:flint"))]),
    shaped(
        one("ic2:coal_chunk"),
        &["###", "#O#", "###"],
        &[('#', one("ic2:compressed_coal_ball")), ('O', one("minecraft:obsidian"))],
    ),
    shaped(
        one("ic2:coal_chunk"),
        &["###", "#O#", "###"],
        &[('#', one("ic2:compressed_coal_ball")), ('O', one("minecraft:iron_block"))],
    ),
    shaped(
        one("ic2:coal_chunk"),
        &["###", "#O#", "###"],
        &[('#', one("ic2:compressed_coal_ball")), ('O', one("minecraft:brick_block"))],
    ),
    shaped(
        one("minecraft:diamond_pickaxe"),
        &["DDD", " S ", " S "],
        &[('S', one("minecraft:stick")), ('D', one("ic2:industrial_diamond"))],
    ),
    shaped(
        one("minecraft:diamond_hoe"),
        &["DD ", " S ", " S "],
        &[('S', one("minecraft:stick")), ('D', one("ic2:industrial_diamond"))],
    ),
    shaped(
        one("minecraft:diamond_shovel"),
        &["D", "S", "S"],
        &[('S', one("minecraft:stick")), ('D', one("ic2:industrial_diamond"))],
    ),
    shaped(
        one("minecraft:diamond_axe"),
        &["DD ", "DS ", " S "],
        &[('S', one("minecraft:stick")), ('D', one("ic2:industrial_diamond"))],
    ),
    shaped(
        one("minecraft:diamond_sword"),
        &["D", "D", "S"],
        &[('S', one("minecraft:stick")), ('D', one("ic2:industrial_diamond"))],
    ),
    shaped(
        one("ic2:tfbp"),
        &[" C ", " A ", "R R"],
        &[('C', one("ic2:electronic_circuit")), ('A', one("ic2:advanced_circuit")), ('R', one("minecraft:redstone"))],
    ),
    shaped(
        one("ic2:terraformer"),
        &["TLT", " D ", " M "],
        &[
            ('M', one("ic2:advanced_machine")),
            ('C', one("ic2:copper_cable")),
            ('T', one("ic2:tfbp")),
            ('L', one("minecraft:lapis_block")),
            ('D', one("ic2:industrial_diamond")),
        ],
    ),
    shaped(
        one("ic2:terraformer"),
        &["TLT", " D ", " M "],
        &[
            ('M', one("ic2:advanced_machine")),
            ('C', one("ic2:copper_cable")),
            ('T', one("ic2:tfbp")),
            ('L', one("minecraft:lapis_block")),
            ('D', one("minecraft:diamond")),
        ],
    ),
    shaped(
        one("ic2:tfbp_cultivation"),
        &[" # ", "#T#", " # "],
        &[('T', one("ic2:tfbp")), ('#', one("minecraft:wheat_seeds"))],
    ),
    shaped(
        one("ic2:tfbp_irrigation"),
        &[" # ", "#T#", " # "],
        &[('T', one("ic2:tfbp")), ('#', one("minecraft:water_bucket"))],
    ),
    shaped(
        one("ic2:tfbp_desertification"),
        &[" # ", "#T#", " # "],
        &[('T', one("ic2:tfbp")), ('#', one("minecraft:sand"))],
    ),
    shaped(
        one("ic2:tfbp_chilling"),
        &[" # ", "#T#", " # "],
        &[('T', one("ic2:tfbp")), ('#', one("minecraft:snowball"))],
    ),
    shaped(
        one("ic2:tfbp_flatification"),
        &[" # ", "#T#", " # "],
        &[('T', one("ic2:tfbp")), ('#', one("minecraft:dirt"))],
    ),
    shaped(one("ic2:tfbp"), &["T"], &[('T', one("ic2:tfbp_cultivation"))]),
    shaped(one("ic2:tfbp"), &["T"], &[('T', one("ic2:tfbp_irrigation"))]),
    shaped(one("ic2:tfbp"), &["T"], &[('T', one("ic2:tfbp_desertification"))]),
    shaped(one("ic2:tfbp"), &["T"], &[('T', one("ic2:tfbp_chilling"))]),
    shaped(one("ic2:tfbp"), &["T"], &[('T', one("ic2:tfbp_flatification"))]),
    shaped(
        one("ic2:frequency_transmitter"),
        &["c", "C", "C"],
        &[('C', one("ic2:electronic_circuit")), ('c', one("ic2:copper_cable"))],
    ),
    shaped(
        one("ic2:teleporter"),
        &["GFG", "CMC", "GDG"],
        &[
            ('M', one("ic2:advanced_machine")),
            ('C', one("ic2:copper_cable")),
            ('F', one("ic2:frequency_transmitter")),
            ('G', one("minecraft:glowstone_dust")),
            ('D', one("ic2:industrial_diamond")),
        ],
    ),
    shaped(
        one("ic2:teleporter"),
        &["GFG", "CMC", "GDG"],
        &[
            ('M', one("ic2:advanced_machine")),
            ('C', one("ic2:copper_cable")),
            ('F', one("ic2:frequency_transmitter")),
            ('G', one("minecraft:glowstone_dust")),
            ('D', one("minecraft:diamond")),
        ],
    ),
    shaped(
        one("ic2:tesla_coil"),
        &["RRR", "RMR", "ICI"],
        &[
            ('M', one("ic2:mv_transformer")),
            ('R', one("minecraft:redstone")),
            ('C', one("ic2:electronic_circuit")),
            ('I', one("ic2:refined_iron")),
        ],
    ),
];

// ModLoader.AddSmelting in mod_IC2's constructor
const SMELTING: [(&str, &str); 11] = [
    ("minecraft:iron_ingot", "ic2:refined_iron"),
    ("ic2:tin_ore", "ic2:tin_ingot"),
    ("ic2:copper_ore", "ic2:copper_ingot"),
    ("ic2:iron_dust", "minecraft:iron_ingot"),
    ("ic2:gold_dust", "minecraft:gold_ingot"),
    ("ic2:tin_dust", "ic2:tin_ingot"),
    ("ic2:copper_dust", "ic2:copper_ingot"),
    ("ic2:hydrated_coal_dust", "ic2:coal_dust"),
    ("ic2:bronze_dust", "ic2:bronze_ingot"),
    ("ic2:rubber_wood", "minecraft:log"),
    ("ic2:sticky_resin", "ic2:rubber"),
];

// mod_IC2.AddFuel
const FUELS: [(&str, i32); 4] =
    [("ic2:rubber_sapling", 80), ("minecraft:reeds", 50), ("minecraft:cactus", 50), ("ic2:scrap", 350)];

fn name(prefix: &str, index: usize, result: &Part) -> String
{
    format!("{prefix}_{index}_{}", result.m_item.trim_start_matches("minecraft:").trim_start_matches("ic2:"))
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for (index, recipe) in RECIPES.iter().enumerate()
    {
        match recipe
        {
            Recipe::Shaped(result, pattern, keys) =>
            {
                let mut builder: ShapedBuilder =
                    ctx.recipes().shaped(&name("shaped", index, result), result.stack()).rows(pattern);
                for (symbol, part) in keys.iter()
                {
                    builder = match part.tag()
                    {
                        Some(tag) => builder.keyTag(*symbol, tag),
                        None => builder.keyStack(*symbol, part.stack()),
                    };
                }

                builder.build()?;
            }

            Recipe::Shapeless(result, inputs) =>
            {
                let ingredients: Vec<IngredientSpec> =
                    inputs.iter().map(|part| part.ingredient()).collect::<Result<_>>()?;
                ctx.recipes().shapelessTagged(&name("shapeless", index, result), result.stack(), ingredients)?;
            }
        }
    }

    for (input, output) in SMELTING
    {
        let label: String = format!("smelt_{}", input.trim_start_matches("minecraft:").trim_start_matches("ic2:"));
        ctx.recipes().smelting(&label, input, ItemStack::of(output, 1))?;
    }

    for (item, ticks) in FUELS
    {
        let label: String = format!("fuel_{}", item.trim_start_matches("minecraft:").trim_start_matches("ic2:"));
        ctx.recipes().fuel(&label, item, ticks)?;
    }

    Ok(())
}
