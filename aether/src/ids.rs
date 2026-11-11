// registry names, shared by registration, generation and behaviors
pub(crate) mod block
{
    pub(crate) const AETHER_GRASS: &str = "aether_grass";
    pub(crate) const ENCHANTED_AETHER_GRASS: &str = "enchanted_aether_grass";
    pub(crate) const AETHER_DIRT: &str = "aether_dirt";
    pub(crate) const HOLYSTONE: &str = "holystone";
    pub(crate) const QUICKSOIL: &str = "quicksoil";
    pub(crate) const ICESTONE: &str = "icestone";
    pub(crate) const AMBROSIUM_ORE: &str = "ambrosium_ore";
    pub(crate) const ZANITE_ORE: &str = "zanite_ore";
    pub(crate) const GRAVITITE_ORE: &str = "gravitite_ore";
    pub(crate) const ENCHANTED_GRAVITITE: &str = "enchanted_gravitite";
    pub(crate) const COLD_AERCLOUD: &str = "cold_aercloud";
    pub(crate) const BLUE_AERCLOUD: &str = "blue_aercloud";
    pub(crate) const GOLDEN_AERCLOUD: &str = "golden_aercloud";
    pub(crate) const SKYROOT_LOG: &str = "skyroot_log";
    pub(crate) const GOLDEN_OAK_LOG: &str = "golden_oak_log";
    pub(crate) const SKYROOT_LEAVES: &str = "skyroot_leaves";
    pub(crate) const GOLDEN_OAK_LEAVES: &str = "golden_oak_leaves";
    pub(crate) const CRYSTAL_LEAVES: &str = "crystal_leaves";
    pub(crate) const CRYSTAL_FRUIT_LEAVES: &str = "crystal_fruit_leaves";
    pub(crate) const SKYROOT_SAPLING: &str = "skyroot_sapling";
    pub(crate) const GOLDEN_OAK_SAPLING: &str = "golden_oak_sapling";
    pub(crate) const WHITE_FLOWER: &str = "white_flower";
    pub(crate) const PURPLE_FLOWER: &str = "purple_flower";
    pub(crate) const BERRY_BUSH: &str = "berry_bush";
    pub(crate) const AETHER_PORTAL: &str = "aether_portal";
}

pub(crate) mod item
{
    pub(crate) const AMBROSIUM_SHARD: &str = "ambrosium_shard";
    pub(crate) const ZANITE_GEMSTONE: &str = "zanite_gemstone";
    pub(crate) const GOLDEN_AMBER: &str = "golden_amber";
    pub(crate) const SKYROOT_STICK: &str = "skyroot_stick";
    pub(crate) const BLUEBERRY: &str = "blueberry";
    pub(crate) const WHITE_APPLE: &str = "white_apple";
    pub(crate) const SWET_BALL: &str = "swet_ball";
    pub(crate) const AECHOR_PETAL: &str = "aechor_petal";
    pub(crate) const VICTORY_MEDAL: &str = "victory_medal";
}

pub(crate) fn blockId(name: &str) -> i32 { b173_api::Blocks.id(name).unwrap_or(0) }

// numeric ids change with every world and server, so generation resolves them once per call
pub(crate) struct Palette
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
    pub(crate) fn load() -> Self
    {
        Self
        {
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
    pub(crate) fn isSoil(&self, block: i32) -> bool
    {
        block == self.m_grass || block == self.m_dirt || block == self.m_enchantedGrass
    }
}
