pub(crate) mod biomes;
mod caves;
mod decorate;
pub(crate) mod features;
mod flora;
pub(crate) mod generate;
mod hollow;
mod layers;
pub(crate) mod noise;
pub(crate) mod palette;
mod rainbow;
mod ruins;
pub(crate) mod scatter;
pub(crate) mod structures;
pub(crate) mod terrain;
pub(crate) mod trees;
pub(crate) mod vanilla;
mod winter;

use b173_api::prelude::*;

pub(crate) const DIMENSION: &str = "twilight_forest";
pub(crate) const DIMENSION_ID: &str = "twilightforest:twilight_forest";
// TFWorld.CHUNKHEIGHT, structures such as the final castle build up to it while players keep beta's build limit
pub(crate) const WORLD_HEIGHT: i32 = 256;
pub(crate) const BUILD_LIMIT: i32 = 128;

mod sky
{
    // WorldProviderTwilightForest.getSkyColor and getFogColor, divided by the light the vanilla sky lends at angle 0.225
    pub const SKY: u32 = 0x35387A;
    pub const FOG: u32 = 0x7BA48E;
    // the world time whose celestial angle is the source's fixed 0.225, a sky stuck in the evening
    pub const TIME: i64 = 12216;
}

// the forest's biomes in the shared groups, so other mods' features and spawns that follow tags reach them
const BIOME_TAGS: [(&str, &[&str]); 8] = [
    (
        "twilightforest:is_twilight",
        &[
            "twilight_lake",
            "twilight_forest",
            "dense_twilight_forest",
            "twilight_highlands",
            "mushroom_forest",
            "twilight_swamp",
            "twilight_stream",
            "snowy_forest",
            "twilight_glacier",
            "twilight_clearing",
            "oak_savanna",
            "firefly_forest",
            "deep_mushroom_forest",
            "dark_forest",
            "enchanted_forest",
            "fire_swamp",
            "dark_forest_center",
            "highlands_center",
            "thornlands",
        ],
    ),
    (
        "minecraft:is_forest",
        &[
            "twilight_forest",
            "dense_twilight_forest",
            "firefly_forest",
            "dark_forest",
            "enchanted_forest",
            "snowy_forest",
            "dark_forest_center",
        ],
    ),
    ("minecraft:is_savanna", &["oak_savanna"]),
    ("c:is_swamp", &["twilight_swamp", "fire_swamp"]),
    ("c:is_snowy", &["snowy_forest", "twilight_glacier"]),
    ("c:is_plains", &["twilight_clearing"]),
    ("c:is_mushroom", &["mushroom_forest", "deep_mushroom_forest"]),
    ("c:is_magical", &["enchanted_forest"]),
];

fn own(name: &str) -> Option<Identifier> { Identifier::parse(&format!("twilightforest:{name}")).ok() }

fn biomeColumns(chunk: &ChunkInfo) -> ChunkBiomes
{
    ChunkBiomes { biomes: biomes::chunk(chunk.seed, chunk.x, chunk.z).to_vec() }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &biomes::BIOMES
    {
        ctx.dimensions().biome(
            row.m_name,
            BiomeSpec
            {
                name: row.m_label.to_owned(),
                color: row.m_map,
                temperature: row.m_temperature,
                rainfall: row.m_rainfall,
                top_block: None,
                filler_block: None,
                weight: 1,
            },
        )?;
    }

    for (tag, members) in BIOME_TAGS
    {
        ctx.tags().add(RegistryKind::Biome, tag, members)?;
    }

    let twilight: Dimension = ctx.dimensions().register(
        DIMENSION,
        DimensionSpec
        {
            name: "Twilight Forest".to_owned(),
            sky_color: sky::SKY,
            fog_color: sky::FOG,
            has_sky: true,
            fill_block: None,
            ground_height: 0,
            portal_block: None,
            portal_frame: None,
            biome: own(biomes::BIOMES[1].m_name),
            generator: None,
            beds: true,
            music: None,
        },
    )?;

    decorate::register(ctx)?;
    let names: Vec<&str> = biomes::BIOMES.iter().map(|row| row.m_name).collect();
    twilight
        .height(WORLD_HEIGHT, BUILD_LIMIT)?
        .biomeSource(&names, biomeColumns)?
        .terrain(terrain::generate)?
        .generator(decorate::populate)?
        .spawns(crate::entities::spawns::possible)?;

    if let Some(seed) = crate::config::seed()
    {
        twilight.seed(seed)?;
    }

    features::registerCommand(ctx)?;
    structures::register(ctx)?;

    // each side pins its own sky, again whenever the dimension's numeric id changes with a new server
    ctx.events().on::<TickPost>(Priority::Normal, |_|
    {
        let current: Option<i32> = dimension();
        if PINNED.get() != current
        {
            PINNED.set(current);

            if let Some(numeric) = current
            {
                World { dimension: numeric }.setSkyTime(Some(sky::TIME));
            }
        }
    })?;

    Ok(())
}

thread_local!
{
    static PINNED: std::cell::Cell<Option<i32>> = const { std::cell::Cell::new(None) };
}

pub(crate) fn dimension() -> Option<i32>
{
    Identifier::parse(DIMENSION_ID).ok().and_then(|id| b173_api::resolve(RegistryKind::Dimension, &id).ok())
}

pub(crate) fn isTwilight(world: World) -> bool { dimension() == Some(world.dimension) }
