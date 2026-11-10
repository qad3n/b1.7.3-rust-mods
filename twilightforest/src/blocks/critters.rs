use super::{PIXELS, texture};
use crate::ids::{self, block};
use b173_api::prelude::*;

// BlockTFCritter: tiny bugs that cling to a wall floor or ceiling like a torch
#[derive(Clone, Copy, PartialEq)]
pub enum Critter
{
    Firefly,
    Cicada,
    Moonworm,
}

struct CritterRow
{
    m_critter: Critter,
    m_name: &'static str,
    m_label: &'static str,
    m_texture: &'static str,
    m_light: f32,
    m_drops: bool,
}

const CRITTERS: [CritterRow; 3] = [
    CritterRow
    {
        m_critter: Critter::Firefly,
        m_name: block::FIREFLY,
        m_label: "Firefly",
        m_texture: "tffirefly",
        m_light: 0.9375,
        m_drops: true,
    },
    CritterRow
    {
        m_critter: Critter::Cicada,
        m_name: block::CICADA,
        m_label: "Cicada",
        m_texture: "tfcicada",
        m_light: 0.0,
        m_drops: true,
    },
    CritterRow
    {
        m_critter: Critter::Moonworm,
        m_name: block::MOONWORM,
        m_label: "Moonworm",
        m_texture: "tfmoonworm",
        m_light: 14.0 / 15.0,
        m_drops: false,
    },
];

// TileEntityTFCicada sings now and then while nobody is close to it
mod cicada
{
    pub const SING_ODDS: i32 = 200;
    pub const SOUND: &str = "twilightforest.mob.cicada";
}

struct CritterTile
{
    m_critter: Critter,
}

#[tile_behavior]
impl TileBehavior for CritterTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if self.m_critter == Critter::Cicada && !ctx.isAuthority() && ctx.world().random(cicada::SING_ODDS) == 0
        {
            ctx.world().playSound(ids::center(ctx.pos()), cicada::SOUND, 1.0, 1.0);
        }
    }
}

// BlockTFFireflyJar lets tiny fireflies drift around it
struct Jar;

#[block_behavior]
impl BlockBehavior for Jar
{
    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        let pos: [f64; 3] = ids::center(ctx.pos());
        for _ in 0..2
        {
            crate::particles::tinyFirefly(ctx.world(), [pos[0], pos[1] - 0.6, pos[2]]);
        }
    }
}

fn tileName(row: &CritterRow) -> String { format!("{}_tile", row.m_name) }

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &CRITTERS
    {
        let tile: String = tileName(row);
        let spec: TileEntitySpec =
            TileEntitySpec { inventory: 0, ticks: row.m_critter == Critter::Cicada, sync: false, behavior: None };
        ctx.tileEntities().register(&tile, spec, Some(std::rc::Rc::new(CritterTile { m_critter: row.m_critter })))?;
        let mut spec: BlockSpecBuilder = BlockSpecBuilder::template("minecraft:torch")
            .name(row.m_label)
            .texture(&texture(row.m_texture))
            .hardness(0.0)
            .light(row.m_light)
            .sound("cloth")
            .tileEntity(&tile);
        if !row.m_drops
        {
            spec = spec.drops(Vec::new());
        }
        super::put(ctx, row.m_name, spec)?;
    }
    let inset: f32 = 3.0;
    let hardness: f32 = 0.3;
    let jar: Cuboid =
        Cuboid { from: [inset, 0.0, inset], to: [PIXELS - inset, PIXELS, PIXELS - inset], ..Cuboid::default() };
    super::put(
        ctx,
        block::FIREFLY_JAR,
        BlockSpecBuilder::template("minecraft:block")
            .name("Firefly Jar")
            .material("glass")
            .faces(FaceTextures
            {
                top: Some(texture("fireflyjar_top")),
                bottom: Some(texture("fireflyjar_top")),
                side: Some(texture("fireflyjar_side")),
                front: None,
            })
            .hardness(hardness)
            .light(1.0)
            .sound("wood")
            .model(BlockModelSpec { cuboids: vec![jar.clone()], translucent: true, ..BlockModelSpec::default() })
            .collision(vec![jar])
            .behavior(Jar),
    )
}
