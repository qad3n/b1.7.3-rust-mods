pub mod biomes;
mod caves;
mod decorate;
pub mod features;
mod flora;
pub mod generate;
mod hollow;
mod layers;
pub mod noise;
mod rainbow;
mod ruins;
pub mod scatter;
pub mod structures;
pub mod terrain;
pub mod trees;
pub mod vanilla;
mod winter;

use b173_api::prelude::*;

pub const DIMENSION: &str = "twilight_forest";
pub const DIMENSION_ID: &str = "twilightforest:twilight_forest";
// TFWorld.CHUNKHEIGHT, structures such as the final castle build up to it while players keep beta's build limit
pub const WORLD_HEIGHT: i32 = 256;
pub const BUILD_LIMIT: i32 = 128;

mod sky
{
    // WorldProviderTwilightForest.getSkyColor and getFogColor, divided by the light the vanilla sky lends at angle 0.225
    pub const SKY: u32 = 0x35387A;
    pub const FOG: u32 = 0x7BA48E;
    // the world time whose celestial angle is the source's fixed 0.225, a sky stuck in the evening
    pub const TIME: i64 = 12216;
}

fn own(name: &str) -> Option<Identifier> { Identifier::parse(&format!("twilightforest:{name}")).ok() }

fn biomeColumns(chunk: &ChunkInfo) -> ChunkBiomes
{
    ChunkBiomes { biomes: biomes::chunk(chunk.seed, chunk.x, chunk.z).to_vec() }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
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

pub fn dimension() -> Option<i32>
{
    Identifier::parse(DIMENSION_ID).ok().and_then(|id| b173_api::resolve(RegistryKind::Dimension, &id).ok())
}

pub fn isTwilight(world: World) -> bool { dimension() == Some(world.dimension) }
