use super::{Cube, Kind, Look, base, cube, faces, texture, tough, when};
use crate::colors;
use crate::names::{self, block, item};
use b173_api::math::{Face, add, floor, offset, scale};
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
            let phase: i32 = (pos.x + pos.z).abs() % aurora::PHASES;
            if let Err(error) = ctx.setState(names::state(block::AURORA_BRICK).with("phase", phase))
            {
                warn(&format!("aurora brick at {pos:?}: {}", error.0));
            }
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
            properties: vec![PropertySpec::new("phase", &values)],
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
// the six facings, then the latent shield the stronghold access chamber lays, outside on every face
const SIDES: [&str; 7] = ["down", "up", "north", "south", "west", "east", "none"];
pub(crate) const SHIELD_LATENT: &str = SIDES[6];
// the 1.7 shield metadata the structures name the latent shield by
pub(crate) const LATENT_META: i32 = 6;

// BlockTFShield, only the inside face can be dug, as fast as the player digs stone and a third slower
struct Shield;

mod shield
{
    pub const REACH: f64 = 6.0;
    pub const SLOWER: f32 = 1.5;
    pub const SCALE: f32 = 100.0;
    // BlockPistonBase.determineOrientation, a player this close looks up or down at it instead of across
    pub const NEAR: f64 = 2.0;
    pub const ABOVE: f64 = 2.0;
    pub const EYE_RISE: f64 = 0.2;
}

#[block_behavior]
impl BlockBehavior for Shield
{
    // BlockTFShield.onBlockPlacedBy, the inside faces the player that set it
    fn onPlacedBy(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        let Some(info) = entity.info()
        else
        {
            return;
        };

        let pos: BlockPos = ctx.pos();
        let close: bool = (info.pos[0] - f64::from(pos.x)).abs() < shield::NEAR
            && (info.pos[2] - f64::from(pos.z)).abs() < shield::NEAR;
        let eye: f64 = info.pos[1] + shield::EYE_RISE;
        let facing: usize = if close && eye - f64::from(pos.y) > shield::ABOVE
        {
            1
        }
        else if close && f64::from(pos.y) - eye > 0.0
        {
            0
        }
        else
        {
            let quarter: usize = (floor(f64::from(info.yaw) * 4.0 / 360.0 + 0.5) & 3) as usize;
            [2, 5, 3, 4][quarter]
        };
        if let Err(error) = ctx.setState(names::state(block::SHIELD).with("facing", SIDES[facing]))
        {
            warn(&format!("shield at {pos:?}: {}", error.0));
        }
    }

    fn strength(&self, ctx: &mut BlockCtx, player: Player) -> Option<f32>
    {
        let info: EntityInfo = player.info()?;
        let aim: [f64; 3] = crate::math::look(info.yaw, info.pitch);
        let to: [f64; 3] = add(info.pos, scale(aim, shield::REACH));
        let face: Option<i32> = ctx.world().rayTrace(info.pos, to, false).map(|hit| hit.face);
        let facing: Option<i32> = names::pick(ctx.stateAt(), "facing", &SIDES).map(|index| index as i32);
        if face != facing
        {
            return Some(0.0);
        }

        let stone: i32 = Blocks.id("minecraft:stone").ok()?;
        Some(player.digSpeed(stone) / shield::SLOWER / shield::SCALE)
    }
}

fn registerShield(ctx: &mut RegisterCtx) -> Result<()>
{
    let unbreakable: f32 = -1.0;
    let resistance: f32 = 6000000.0;
    super::put(
        ctx,
        block::SHIELD,
        base("minecraft:block", "Stronghold Shield", Kind::Metal, unbreakable)
            .resistance(resistance)
            .faces(FaceTextures
            {
                top: Some(texture("shield_outside")),
                bottom: Some(texture("shield_outside")),
                side: Some(texture("shield_outside")),
                front: Some(texture("shield_inside")),
            })
            .behavior(Shield),
    )?;
    ctx.blocks().states(
        block::SHIELD,
        BlockStatesSpec
        {
            properties: vec![PropertySpec::new("facing", &SIDES)],
            variants: vec![StateVariantSpec
            {
                when: when("facing", "none"),
                texture: Some(texture("shield_outside")),
                textures: None,
            }],
            ..BlockStatesSpec::default()
        },
    )
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

    // RenderBlockTFKnightMetal, a core with a spike on every corner, edge and face, each pulled in by a hair
    pub fn spikes() -> Vec<Cuboid>
    {
        let (step, size, hair): (f32, f32, f32) = (6.0, 4.0, 1.0 / 64.0);
        let mut cuboids: Vec<Cuboid> = vec![shell()];

        for x in 0..3
        {
            for y in 0..3
            {
                for z in 0..3
                {
                    let from: [f32; 3] = [x, y, z].map(|cell| cell as f32 * step + hair);
                    let to: [f32; 3] = from.map(|start| start + size - hair * 2.0);
                    cuboids.push(Cuboid { from, to, ..Cuboid::default() });
                }
            }
        }

        cuboids
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
            crate::combat::hit(entity, knightmetal::DAMAGE, None, crate::combat::kind::CACTUS);
        }
    }
}

// BlockTFTrollSteinn glints red on every face that looks into the dark
struct Trollsteinn;

const STEINN_DARK: i32 = 7;
const STEINN_GLOW: [f64; 3] = [0.25, -1.0, 0.5];
const STEINN_LIT: &str = "twilightforest:block/trollsteinn_light";

// BlockTFTrollSteinn.isBlockLit, an open neighbor brighter than the dark threshold
fn steinnLit(world: World, at: BlockPos) -> bool
{
    !World::blockInfo(world.block(at)).opaque && world.light(at) > STEINN_DARK
}

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

    // BlockTFTrollSteinn.getIcon, a face looking into light shows the lit stone
    fn faceTexture(&self, ctx: &mut BlockCtx, face: i32) -> Option<i32>
    {
        let face: Face = Face::fromIndex(face)?;
        steinnLit(ctx.world(), offset(ctx.pos(), face)).then(|| b173_api::world::terrainSlot(STEINN_LIT)).flatten()
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    // no block spec names the lit face, asking for its slot now gives it room in the atlas
    let _ = b173_api::world::terrainSlot(STEINN_LIT);

    for row in &MAZE
    {
        super::put(ctx, row.m_name, super::build(row).behavior(Wearing).tag(crate::tags::MAZESTONE))?;
    }

    for row in &CASTLE
    {
        super::put(ctx, row.m_name, super::build(row).behavior(Wearing))?;
    }

    for row in &PLAIN
    {
        let spec: BlockSpecBuilder = super::build(row);
        let spec: BlockSpecBuilder = match row.m_name
        {
            block::KNIGHTMETAL_BLOCK => spec
                .behavior(Knightmetal)
                .model(BlockModelSpec { cuboids: knightmetal::spikes(), ..BlockModelSpec::default() })
                .collision(vec![knightmetal::shell()]),
            block::TROLLSTEINN => spec.behavior(Trollsteinn).tag(crate::tags::STONY),
            block::DEADROCK | block::CRACKED_DEADROCK | block::SOLID_DEADROCK => spec.tag(crate::tags::STONY),
            _ => spec,
        };
        super::put(ctx, row.m_name, spec)?;
    }

    registerAurora(ctx)?;
    registerShield(ctx)?;
    registerSpecial(ctx)
}
