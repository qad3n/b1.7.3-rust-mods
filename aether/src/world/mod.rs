mod decorate;
pub(crate) mod dungeons;
mod gen_world;
mod lakes;
mod terrain;
pub(crate) mod trees;

use crate::names::block;
use b173_api::prelude::*;
pub(crate) use gen_world::GenWorld;

pub(crate) const DIMENSION: &str = "the_aether";
pub(crate) const DIMENSION_ID: &str = "aether:the_aether";
// AetherMusicTicker: aether1 to aether4, a minute to a minute and a quarter apart
const MUSIC: &str = "aether.music.aether";
const MUSIC_DELAY: [i32; 2] = [1200, 1500];
const BIOME: &str = "highlands";

mod sky
{
    // AetherBiome sky color and AetherWorldProvider fog color
    pub const SKY: u32 = 0xC0C0FF;
    pub const FOG: u32 = 0x9393BC;
    pub const MAP: u32 = 0x5DA243;
}

fn own(name: &str) -> Option<Identifier> { Identifier::parse(&format!("aether:{name}")).ok() }

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.dimensions().biome(
        BIOME,
        BiomeSpec
        {
            name: "Aether Highlands".to_owned(),
            color: sky::MAP,
            temperature: 0.5,
            rainfall: 0.0,
            top_block: own(block::AETHER_GRASS),
            filler_block: own(block::AETHER_DIRT),
            weight: 1,
        },
    )?;

    let aether: Dimension = ctx.dimensions().register(
        DIMENSION,
        DimensionSpec
        {
            name: "The Aether".to_owned(),
            sky_color: sky::SKY,
            fog_color: sky::FOG,
            has_sky: true,
            fill_block: None,
            ground_height: 0,
            portal_block: None,
            portal_frame: None,
            biome: own(BIOME),
            generator: None,
            // beds sleep here but beta keeps the spawn point in the overworld where Aether deaths respawn
            beds: true,
            music: Some(MusicSpec { sound: MUSIC.to_owned(), delay: MUSIC_DELAY }),
        },
    )?;
    dungeons::register(ctx)?;
    aether.terrain(terrain::generate)?.generator(decorate::populate)?;
    Ok(())
}
