// registry names, shared by registration, generation and behaviors
pub mod block
{
    pub const AETHER_GRASS: &str = "aether_grass";
    pub const ENCHANTED_AETHER_GRASS: &str = "enchanted_aether_grass";
    pub const AETHER_DIRT: &str = "aether_dirt";
    pub const HOLYSTONE: &str = "holystone";
    pub const QUICKSOIL: &str = "quicksoil";
    pub const ICESTONE: &str = "icestone";
    pub const AMBROSIUM_ORE: &str = "ambrosium_ore";
    pub const ZANITE_ORE: &str = "zanite_ore";
    pub const GRAVITITE_ORE: &str = "gravitite_ore";
    pub const ENCHANTED_GRAVITITE: &str = "enchanted_gravitite";
    pub const COLD_AERCLOUD: &str = "cold_aercloud";
    pub const BLUE_AERCLOUD: &str = "blue_aercloud";
    pub const GOLDEN_AERCLOUD: &str = "golden_aercloud";
    pub const SKYROOT_LOG: &str = "skyroot_log";
    pub const GOLDEN_OAK_LOG: &str = "golden_oak_log";
    pub const SKYROOT_LEAVES: &str = "skyroot_leaves";
    pub const GOLDEN_OAK_LEAVES: &str = "golden_oak_leaves";
    pub const CRYSTAL_LEAVES: &str = "crystal_leaves";
    pub const CRYSTAL_FRUIT_LEAVES: &str = "crystal_fruit_leaves";
    pub const SKYROOT_SAPLING: &str = "skyroot_sapling";
    pub const GOLDEN_OAK_SAPLING: &str = "golden_oak_sapling";
    pub const WHITE_FLOWER: &str = "white_flower";
    pub const PURPLE_FLOWER: &str = "purple_flower";
    pub const BERRY_BUSH: &str = "berry_bush";
    pub const AETHER_PORTAL: &str = "aether_portal";
}

pub mod item
{
    pub const AMBROSIUM_SHARD: &str = "ambrosium_shard";
    pub const ZANITE_GEMSTONE: &str = "zanite_gemstone";
    pub const GOLDEN_AMBER: &str = "golden_amber";
    pub const SKYROOT_STICK: &str = "skyroot_stick";
    pub const BLUEBERRY: &str = "blueberry";
    pub const WHITE_APPLE: &str = "white_apple";
    pub const SWET_BALL: &str = "swet_ball";
    pub const AECHOR_PETAL: &str = "aechor_petal";
    pub const VICTORY_MEDAL: &str = "victory_medal";
}

pub fn blockId(name: &str) -> i32 { b173_api::Blocks.id(name).unwrap_or(0) }

// numeric ids change with every world and server, so generation resolves them once per call
pub struct Palette
{
    pub m_grass: i32,
    pub m_enchantedGrass: i32,
    pub m_dirt: i32,
    pub m_holystone: i32,
    pub m_quicksoil: i32,
    pub m_icestone: i32,
    pub m_ambrosiumOre: i32,
    pub m_zaniteOre: i32,
    pub m_gravititeOre: i32,
    pub m_coldCloud: i32,
    pub m_blueCloud: i32,
    pub m_goldenCloud: i32,
    pub m_skyrootLog: i32,
    pub m_goldenOakLog: i32,
    pub m_skyrootLeaves: i32,
    pub m_goldenOakLeaves: i32,
    pub m_crystalLeaves: i32,
    pub m_crystalFruitLeaves: i32,
    pub m_whiteFlower: i32,
    pub m_purpleFlower: i32,
    pub m_berryBush: i32,
}

impl Palette
{
    pub fn load() -> Palette
    {
        Palette {
            m_grass: blockId(block::AETHER_GRASS),
            m_enchantedGrass: blockId(block::ENCHANTED_AETHER_GRASS),
            m_dirt: blockId(block::AETHER_DIRT),
            m_holystone: blockId(block::HOLYSTONE),
            m_quicksoil: blockId(block::QUICKSOIL),
            m_icestone: blockId(block::ICESTONE),
            m_ambrosiumOre: blockId(block::AMBROSIUM_ORE),
            m_zaniteOre: blockId(block::ZANITE_ORE),
            m_gravititeOre: blockId(block::GRAVITITE_ORE),
            m_coldCloud: blockId(block::COLD_AERCLOUD),
            m_blueCloud: blockId(block::BLUE_AERCLOUD),
            m_goldenCloud: blockId(block::GOLDEN_AERCLOUD),
            m_skyrootLog: blockId(block::SKYROOT_LOG),
            m_goldenOakLog: blockId(block::GOLDEN_OAK_LOG),
            m_skyrootLeaves: blockId(block::SKYROOT_LEAVES),
            m_goldenOakLeaves: blockId(block::GOLDEN_OAK_LEAVES),
            m_crystalLeaves: blockId(block::CRYSTAL_LEAVES),
            m_crystalFruitLeaves: blockId(block::CRYSTAL_FRUIT_LEAVES),
            m_whiteFlower: blockId(block::WHITE_FLOWER),
            m_purpleFlower: blockId(block::PURPLE_FLOWER),
            m_berryBush: blockId(block::BERRY_BUSH),
        }
    }

    // blocks plants and saplings take root in, BlockAetherFlower.canThisPlantGrowOnThisBlock
    pub fn isSoil(&self, block: i32) -> bool
    {
        block == self.m_grass || block == self.m_dirt || block == self.m_enchantedGrass
    }
}
