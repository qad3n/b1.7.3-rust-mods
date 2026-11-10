use super::{Cube, Kind, Look, base, cube, faces, property, texture, tough, when};
use crate::colors;
use crate::ids::{block, item};
use b173_api::prelude::*;

mod maze
{
    pub const HARDNESS: f32 = 100.0;
    pub const RESISTANCE: f32 = 5.0;
    // mining these with an ordinary tool wears it sixteen times over
    pub const WEAR: i32 = 16;
}

mod castle
{
    pub const HARDNESS: f32 = 100.0;
    pub const RESISTANCE: f32 = 15.0;
}

mod brick
{
    pub const HARDNESS: f32 = 1.5;
    pub const RESISTANCE: f32 = 10.0;
}

mod deadrock
{
    pub const HARDNESS: f32 = 100.0;
    pub const RESISTANCE: f32 = 6000000.0;
}

mod metal
{
    pub const HARDNESS: f32 = 5.0;
    pub const RESISTANCE: f32 = 41.0;
}

mod cloud
{
    pub const FLUFFY_HARDNESS: f32 = 0.8;
    pub const WISPY_HARDNESS: f32 = 0.3;
}

mod aurora
{
    pub const HARDNESS: f32 = 2.0;
    pub const RESISTANCE: f32 = 10.0;
    // the brick picks one of sixteen phases from its column, sixteen phases show eight textures there and back
    pub const PHASES: i32 = 16;
    pub const TEXTURES: i32 = 8;
}

const MAZE: [Cube; 8] = [
    tough(block::MAZESTONE, "Mazestone", Kind::Rock, Look::All("mazestone_plain"), maze::HARDNESS, maze::RESISTANCE),
    tough(
        block::MAZESTONE_BRICK,
        "Mazestone Brick",
        Kind::Rock,
        Look::All("mazestone_brick"),
        maze::HARDNESS,
        maze::RESISTANCE,
    ),
    tough(
        "chiseled_mazestone",
        "Chiseled Mazestone",
        Kind::Rock,
        Look::Column { top: "mazestone_plain", side: "mazestone_pillar" },
        maze::HARDNESS,
        maze::RESISTANCE,
    ),
    tough(
        "decorative_mazestone",
        "Decorative Mazestone Brick",
        Kind::Rock,
        Look::Column { top: "mazestone_brick", side: "mazestone_decorative" },
        maze::HARDNESS,
        maze::RESISTANCE,
    ),
    tough(
        block::CRACKED_MAZESTONE,
        "Cracked Mazestone Brick",
        Kind::Rock,
        Look::All("mazestone_cracked"),
        maze::HARDNESS,
        maze::RESISTANCE,
    ),
    tough(
        block::MOSSY_MAZESTONE,
        "Mossy Mazestone Brick",
        Kind::Rock,
        Look::All("mazestone_mossy"),
        maze::HARDNESS,
        maze::RESISTANCE,
    ),
    tough(
        block::MAZESTONE_MOSAIC,
        "Mazestone Mosaic",
        Kind::Rock,
        Look::Column { top: "mazestone_mosaic", side: "mazestone_brick" },
        maze::HARDNESS,
        maze::RESISTANCE,
    ),
    tough(
        block::MAZESTONE_BORDER,
        "Mazestone Border",
        Kind::Rock,
        Look::Column { top: "mazestone_border", side: "mazestone_brick" },
        maze::HARDNESS,
        maze::RESISTANCE,
    ),
];

const CASTLE: [Cube; 4] = [
    tough(
        block::CASTLE_BRICK,
        "Castle Brick",
        Kind::Rock,
        Look::All("castleblock_brick"),
        castle::HARDNESS,
        castle::RESISTANCE,
    ),
    tough(
        block::WORN_CASTLE_BRICK,
        "Worn Castle Brick",
        Kind::Rock,
        Look::All("castleblock_faded"),
        castle::HARDNESS,
        castle::RESISTANCE,
    ),
    tough(
        block::CRACKED_CASTLE_BRICK,
        "Cracked Castle Brick",
        Kind::Rock,
        Look::All("castleblock_cracked"),
        castle::HARDNESS,
        castle::RESISTANCE,
    ),
    tough(
        block::CASTLE_ROOF,
        "Castle Roof Tile",
        Kind::Rock,
        Look::All("castleblock_roof"),
        castle::HARDNESS,
        castle::RESISTANCE,
    ),
];

const PLAIN: [Cube; 12] = [
    tough(block::KNIGHTBRICK, "Underbrick", Kind::Rock, Look::All("knightbrick"), brick::HARDNESS, brick::RESISTANCE),
    tough(
        block::MOSSY_KNIGHTBRICK,
        "Mossy Underbrick",
        Kind::Rock,
        Look::All("knightbrick_mossy"),
        brick::HARDNESS,
        brick::RESISTANCE,
    ),
    tough(
        block::CRACKED_KNIGHTBRICK,
        "Cracked Underbrick",
        Kind::Rock,
        Look::All("knightbrick_cracked"),
        brick::HARDNESS,
        brick::RESISTANCE,
    ),
    tough(
        "underbrick_floor",
        "Underbrick Floor",
        Kind::Rock,
        Look::All("knightbrick_floor"),
        brick::HARDNESS,
        brick::RESISTANCE,
    ),
    tough(
        block::DEADROCK,
        "Weathered Deadrock",
        Kind::Rock,
        Look::All("deadrock_surface"),
        deadrock::HARDNESS,
        deadrock::RESISTANCE,
    ),
    tough(
        block::CRACKED_DEADROCK,
        "Cracked Deadrock",
        Kind::Rock,
        Look::All("deadrock_cracked"),
        deadrock::HARDNESS,
        deadrock::RESISTANCE,
    ),
    tough(
        block::SOLID_DEADROCK,
        "Deadrock",
        Kind::Rock,
        Look::All("deadrock_solid"),
        deadrock::HARDNESS,
        deadrock::RESISTANCE,
    ),
    tough(
        block::KNIGHTMETAL_BLOCK,
        "Block of Knightmetal",
        Kind::Metal,
        Look::All("knightmetal_block"),
        metal::HARDNESS,
        metal::RESISTANCE,
    ),
    cube(block::FLUFFY_CLOUD, "Fluffy Cloud", Kind::Cloud, Look::All("fluffy_cloud"), cloud::FLUFFY_HARDNESS),
    tough(
        block::TROLLSTEINN,
        "Trollsteinn",
        Kind::Rock,
        Look::All("trollsteinn"),
        aurora::HARDNESS,
        castle::RESISTANCE,
    ),
    tough(
        block::AURORA_PILLAR,
        "Aurora Pillar",
        Kind::Rock,
        Look::Column { top: "aurora_pillar_top", side: "aurora_pillar_side" },
        aurora::HARDNESS,
        aurora::RESISTANCE,
    ),
    tough(
        block::AURORA_DOUBLE_SLAB,
        "Aurora Double Slab",
        Kind::Rock,
        Look::Column { top: "aurora_pillar_top", side: "aurora_slab_side" },
        aurora::HARDNESS,
        aurora::RESISTANCE,
    ),
];

// BlockTFMazestone.harvestBlock and BlockTFCastleBlock.harvestBlock wear the tool out quickly
struct Wearing;

#[block_behavior]
impl BlockBehavior for Wearing
{
    fn onHarvest(&self, ctx: &mut BlockCtx, player: Player)
    {
        if ctx.isAuthority()
        {
            crate::items::wear(player, maze::WEAR, &[item::MAZEBREAKER_PICK]);
        }
        let _ = ctx.world().dropBlockItems(ctx.pos(), ctx.block(), i32::from(ctx.meta()));
    }
}

// the aurora blocks sparkle through colors by column, BlockTFAuroraBrick and BlockTFAuroraPillar
struct AuroraTint
{
    m_pale: bool,
}

#[block_behavior]
impl BlockBehavior for AuroraTint
{
    fn onAdded(&self, ctx: &mut BlockCtx)
    {
        if !self.m_pale
        {
            let pos: BlockPos = ctx.pos();
            ctx.setMeta(((pos.x + pos.z).abs() % aurora::PHASES) as u8);
        }
    }

    fn colorMultiplier(&self, ctx: &mut BlockCtx) -> u32
    {
        let color: u32 = colors::aurora(ctx.pos().x, ctx.pos().z);
        if self.m_pale
        {
            colors::pale(color)
        }
        else
        {
            color
        }
    }
}

fn auroraVariants() -> Vec<StateVariantSpec>
{
    (0..aurora::PHASES)
        .map(|phase|
        {
            let index: i32 = if phase < aurora::TEXTURES
            {
                phase
            }
            else
            {
                aurora::PHASES - 1 - phase
            };
            StateVariantSpec
            {
                when: when("phase", &phase.to_string()),
                texture: Some(texture(&format!("aurorabrick{index}"))),
                textures: None,
            }
        })
        .collect()
}

fn phases() -> Vec<String> { (0..aurora::PHASES).map(|phase| phase.to_string()).collect() }

fn registerAurora(ctx: &mut RegisterCtx) -> Result<()>
{
    super::put(
        ctx,
        block::AURORA_BRICK,
        base("minecraft:block", "Aurora Block", Kind::Rock, aurora::HARDNESS)
            .resistance(aurora::RESISTANCE)
            .texture(&texture("aurorabrick0"))
            .behavior(AuroraTint { m_pale: false }),
    )?;
    let names: Vec<String> = phases();
    let values: Vec<&str> = names.iter().map(String::as_str).collect();
    ctx.blocks().states(
        block::AURORA_BRICK,
        BlockStatesSpec
        {
            properties: vec![property("phase", &values)],
            variants: auroraVariants(),
            ..BlockStatesSpec::default()
        },
    )?;
    let slab: Cuboid =
        Cuboid { from: [0.0; 3], to: [super::PIXELS, super::PIXELS / 2.0, super::PIXELS], ..Cuboid::default() };
    super::put(
        ctx,
        block::AURORA_SLAB,
        base("minecraft:block", "Aurora Slab", Kind::Rock, aurora::HARDNESS)
            .resistance(aurora::RESISTANCE)
            .faces(faces(&Look::Column { top: "aurora_pillar_top", side: "aurora_slab_side" }))
            .model(BlockModelSpec { cuboids: vec![slab.clone()], ..BlockModelSpec::default() })
            .collision(vec![slab])
            .behavior(AuroraTint { m_pale: true }),
    )
}

// BlockTFShield: unbreakable, the face it was placed against shows the inside
const SIDES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];

fn registerShield(ctx: &mut RegisterCtx) -> Result<()>
{
    let unbreakable: f32 = -1.0;
    let resistance: f32 = 6000000.0;
    super::put(
        ctx,
        block::SHIELD,
        base("minecraft:block", "Stronghold Shield", Kind::Metal, unbreakable).resistance(resistance).faces(
            FaceTextures
            {
                top: Some(texture("shield_outside")),
                bottom: Some(texture("shield_outside")),
                side: Some(texture("shield_outside")),
                front: Some(texture("shield_inside")),
            },
        ),
    )?;
    ctx.blocks().states(
        block::SHIELD,
        BlockStatesSpec { properties: vec![property("facing", &SIDES)], ..BlockStatesSpec::default() },
    )
}

// BlockTFNagastone: heads face one of four sides, bodies lie along an axis
const HEAD_FACING: [&str; 4] = ["north", "south", "west", "east"];
const BODY_AXES: [&str; 3] = ["x", "z", "y"];

fn registerNagastone(ctx: &mut RegisterCtx) -> Result<()>
{
    let hardness: f32 = 1.5;
    super::put(
        ctx,
        block::NAGASTONE_HEAD,
        base("minecraft:block", "Nagastone Head", Kind::Rock, hardness).resistance(brick::RESISTANCE).faces(
            FaceTextures
            {
                top: Some(texture("nagastone_top_tip")),
                bottom: Some(texture("nagastone_bottom_tip")),
                side: Some(texture("nagastone_long_side")),
                front: Some(texture("nagastone_face_front")),
            },
        ),
    )?;
    ctx.blocks().states(
        block::NAGASTONE_HEAD,
        BlockStatesSpec { properties: vec![property("facing", &HEAD_FACING)], ..BlockStatesSpec::default() },
    )?;
    super::put(
        ctx,
        block::NAGASTONE,
        base("minecraft:block", "Nagastone", Kind::Rock, hardness)
            .resistance(brick::RESISTANCE)
            .faces(faces(&Look::Column { top: "nagastone_cross_section", side: "nagastone_long_side" })),
    )?;
    ctx.blocks().states(
        block::NAGASTONE,
        BlockStatesSpec { properties: vec![property("axis", &BODY_AXES)], ..BlockStatesSpec::default() },
    )
}

mod pedestal
{
    pub const HARDNESS: f32 = 2.0;
    pub const RESISTANCE: f32 = 2000.0;
}

fn registerSpecial(ctx: &mut RegisterCtx) -> Result<()>
{
    super::put(
        ctx,
        block::WISPY_CLOUD,
        BlockSpecBuilder::template("minecraft:glass")
            .name("Wispy Cloud")
            .material("snow")
            .texture(&texture("wispy_cloud"))
            .hardness(cloud::WISPY_HARDNESS)
            .sound("cloth")
            .model(BlockModelSpec { translucent: true, ..BlockModelSpec::default() })
            .drops(Vec::new()),
    )?;
    super::put(
        ctx,
        block::TROPHY_PEDESTAL,
        base("minecraft:block", "Trophy Pedestal", Kind::Rock, pedestal::HARDNESS)
            .resistance(pedestal::RESISTANCE)
            .faces(FaceTextures
            {
                top: Some(texture("pedestal_top")),
                bottom: Some(texture("pedestal_top")),
                side: Some(texture("pedestal_naga")),
                front: None,
            })
            .behavior(crate::progress::Pedestal),
    )?;
    let active: FaceTextures = FaceTextures
    {
        top: Some(texture("pedestal_top_active")),
        bottom: Some(texture("pedestal_top_active")),
        side: Some(texture("pedestal_naga_active")),
        front: None,
    };
    ctx.blocks().states(
        block::TROPHY_PEDESTAL,
        BlockStatesSpec
        {
            properties: vec![property("active", &["false", "true"])],
            variants: vec![StateVariantSpec { when: when("active", "true"), texture: None, textures: Some(active) }],
            ..BlockStatesSpec::default()
        },
    )
}

mod knightmetal
{
    use b173_api::prelude::Cuboid;

    pub const DAMAGE: i32 = 4;

    // the collision box sits a pixel inside the block so touching it counts as standing in it
    pub fn shell() -> Cuboid
    {
        let inset: f32 = 1.0;
        let far: f32 = 16.0 - inset;
        Cuboid { from: [inset; 3], to: [far; 3], ..Cuboid::default() }
    }
}

// BlockTFKnightmetalBlock pricks anything touching it like a cactus
struct Knightmetal;

#[block_behavior]
impl BlockBehavior for Knightmetal
{
    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        if ctx.isAuthority()
        {
            entity.damage(knightmetal::DAMAGE, None);
        }
    }
}

// BlockTFTrollSteinn glints red on every face that looks into the dark
struct Trollsteinn;

const STEINN_DARK: i32 = 7;
const STEINN_GLOW: [f64; 3] = [0.25, -1.0, 0.5];

#[block_behavior]
impl BlockBehavior for Trollsteinn
{
    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        if world.random(2) != 0
        {
            return;
        }

        crate::particles::glint(world, ctx.pos(), "reddust", STEINN_GLOW, |at|
        {
            !World::blockInfo(world.block(at)).opaque && world.light(at) <= STEINN_DARK
        });
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in MAZE.iter().chain(CASTLE.iter())
    {
        super::put(ctx, row.m_name, super::build(row).behavior(Wearing))?;
    }
    for row in &PLAIN
    {
        let spec: BlockSpecBuilder = super::build(row);
        let spec: BlockSpecBuilder = match row.m_name
        {
            block::KNIGHTMETAL_BLOCK => spec.behavior(Knightmetal).collision(vec![knightmetal::shell()]),
            block::TROLLSTEINN => spec.behavior(Trollsteinn),
            _ => spec,
        };
        super::put(ctx, row.m_name, spec)?;
    }
    registerAurora(ctx)?;
    registerShield(ctx)?;
    registerNagastone(ctx)?;
    registerSpecial(ctx)
}
