use super::{Kind, base, drop, property, texture, vanillaTexture, when};
use crate::ids::{self, block, item};
use crate::world::biomes;
use b173_api::prelude::*;

// how a small plant decides it may stay where it is, BlockTFPlant.canBlockStay
#[derive(Clone, Copy, PartialEq)]
enum Footing
{
    // dirt or grass below, the forest grass and dried bush
    Soil,
    // soil below and light 3 or a view of the sky
    Lit,
    // any solid top below, moss and mushgloom
    Solid,
    // hangs under roots dirt or grass
    Hanging,
    // hangs under stone or other troll roots
    Stone,
}

#[derive(Clone, Copy)]
enum Shape
{
    Cross,
    // a flat patch this many sixteenths tall
    Patch(f32),
    Mayapple,
}

struct Plant
{
    m_name: &'static str,
    m_label: &'static str,
    m_texture: &'static str,
    m_shape: Shape,
    m_footing: Footing,
    m_light: f32,
    m_tinted: bool,
    m_drop: Option<(&'static str, [i32; 2])>,
}

mod light
{
    pub const MUSHGLOOM: f32 = 3.0 / 15.0;
    pub const TORCHBERRY: f32 = 8.0 / 15.0;
    pub const TROLLBER: f32 = 1.0;
    pub const GLOOM: f32 = 0.3125;
}

const UNTINTED: u32 = 0xFFFFFF;
// a lit plant needs light 3 when it cannot see the sky
const LIT_PLANT_LIGHT: i32 = 3;
// the ripe troll berry spills four to eight torchberries
const TROLLBER_DROP: [i32; 2] = [4, 8];
const ONE: [i32; 2] = [1, 1];
const TALLGRASS_TEXTURE: &str = "minecraft:39";
const DEADBUSH_TEXTURE: &str = "minecraft:55";

const PLANTS: [Plant; 12] = [
    Plant
    {
        m_name: "moss_patch",
        m_label: "Moss Patch",
        m_texture: "mosspatch",
        m_shape: Shape::Patch(1.0),
        m_footing: Footing::Solid,
        m_light: 0.0,
        m_tinted: false,
        m_drop: None,
    },
    Plant
    {
        m_name: "mayapple",
        m_label: "Mayapple",
        m_texture: "mayapple",
        m_shape: Shape::Mayapple,
        m_footing: Footing::Lit,
        m_light: 0.0,
        m_tinted: true,
        m_drop: None,
    },
    Plant
    {
        m_name: "clover_patch",
        m_label: "Clover Patch [WIP]",
        m_texture: "cloverpatch",
        m_shape: Shape::Patch(2.0),
        m_footing: Footing::Lit,
        m_light: 0.0,
        m_tinted: true,
        m_drop: None,
    },
    Plant
    {
        m_name: block::FIDDLEHEAD,
        m_label: "Fiddlehead",
        m_texture: "fiddlehead",
        m_shape: Shape::Cross,
        m_footing: Footing::Lit,
        m_light: 0.0,
        m_tinted: true,
        m_drop: Some((block::FIDDLEHEAD, ONE)),
    },
    Plant
    {
        m_name: block::MUSHGLOOM,
        m_label: "Mushgloom",
        m_texture: "mushgloom",
        m_shape: Shape::Cross,
        m_footing: Footing::Solid,
        m_light: light::MUSHGLOOM,
        m_tinted: false,
        m_drop: Some((block::MUSHGLOOM, ONE)),
    },
    Plant
    {
        m_name: block::FOREST_GRASS,
        m_label: "Forest Grass",
        m_texture: TALLGRASS_TEXTURE,
        m_shape: Shape::Cross,
        m_footing: Footing::Soil,
        m_light: 0.0,
        m_tinted: true,
        m_drop: None,
    },
    Plant
    {
        m_name: block::DRIED_BUSH,
        m_label: "Dried Bush",
        m_texture: DEADBUSH_TEXTURE,
        m_shape: Shape::Cross,
        m_footing: Footing::Soil,
        m_light: 0.0,
        m_tinted: false,
        m_drop: None,
    },
    Plant
    {
        m_name: block::TORCHBERRY_PLANT,
        m_label: "Torchberry Plant",
        m_texture: "torchberry",
        m_shape: Shape::Cross,
        m_footing: Footing::Hanging,
        m_light: light::TORCHBERRY,
        m_tinted: false,
        m_drop: Some((item::TORCHBERRIES, ONE)),
    },
    Plant
    {
        m_name: block::ROOT_STRAND,
        m_label: "Root Strands",
        m_texture: "rootstrand",
        m_shape: Shape::Cross,
        m_footing: Footing::Hanging,
        m_light: 0.0,
        m_tinted: false,
        m_drop: None,
    },
    Plant
    {
        m_name: block::TROLL_ROOT,
        m_label: "Trollvidr",
        m_texture: "troll_root",
        m_shape: Shape::Cross,
        m_footing: Footing::Stone,
        m_light: 0.0,
        m_tinted: false,
        m_drop: None,
    },
    Plant
    {
        m_name: block::UNRIPE_TROLLBER,
        m_label: "Unripe Trollber",
        m_texture: "unripe_torch_cluster",
        m_shape: Shape::Cross,
        m_footing: Footing::Stone,
        m_light: 0.0,
        m_tinted: false,
        m_drop: None,
    },
    Plant
    {
        m_name: block::TROLLBER,
        m_label: "Trollber",
        m_texture: "ripe_torch_cluster",
        m_shape: Shape::Cross,
        m_footing: Footing::Stone,
        m_light: light::TROLLBER,
        m_tinted: false,
        m_drop: Some((item::TORCHBERRIES, TROLLBER_DROP)),
    },
];

fn plantTexture(name: &str) -> String
{
    match name.strip_prefix("minecraft:")
    {
        Some(index) => vanillaTexture(index.parse().unwrap_or(0)),
        None => texture(name),
    }
}

fn shapeModel(shape: Shape) -> Option<BlockModelSpec>
{
    let pixels: f32 = 16.0;
    let cuboid = |from: [f32; 3], to: [f32; 3]| Cuboid { from, to, ..Cuboid::default() };
    match shape
    {
        Shape::Cross => None,
        Shape::Patch(height) => Some(BlockModelSpec
        {
            cuboids: vec![cuboid([0.0; 3], [pixels, height, pixels])],
            ..BlockModelSpec::default()
        }),
        // BlockTFPlant.setBlockBoundsBasedOnState mayapple bounds
        Shape::Mayapple => Some(BlockModelSpec
        {
            cuboids: vec![cuboid([4.0, 0.0, 4.0], [13.0, 6.0, 13.0])],
            ..BlockModelSpec::default()
        }),
    }
}

fn solidTop(world: World, pos: BlockPos) -> bool { World::blockInfo(world.block(pos)).opaque }

fn footed(footing: Footing, world: World, pos: BlockPos) -> bool
{
    let below: i32 = world.block(pos.offset(0, -1, 0));
    let above: BlockPos = pos.offset(0, 1, 0);
    match footing
    {
        Footing::Soil => ids::soil(below),
        Footing::Lit =>
        {
            ids::soil(below) && (world.light(pos) >= LIT_PLANT_LIGHT || world.height(pos.x, pos.z) <= pos.y)
        }
        Footing::Solid => solidTop(world, pos.offset(0, -1, 0)),
        Footing::Hanging => rootHold(world, above),
        Footing::Stone => trollHold(world, above),
    }
}

// BlockTFPlant.canBlockStay for a plant placed by world generation
pub fn plantStays(world: World, pos: BlockPos, name: &str) -> bool
{
    PLANTS.iter().find(|row| row.m_name == name).is_some_and(|row| footed(row.m_footing, world, pos))
}

// BlockTFPlant.canPlaceRootBelow, roots grow down from dirt grass root strands and root blocks
pub fn rootHold(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    ids::soil(block) || block == ids::blockId(block::ROOT_STRAND) || block == ids::blockId(block::ROOT)
}

// BlockTFTrollRoot.canPlaceRootBelow
pub fn trollHold(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    ids::stony(block)
        || [block::TROLL_ROOT, block::TROLLBER, block::UNRIPE_TROLLBER].iter().any(|name| block == ids::blockId(name))
}

struct SmallPlant
{
    m_footing: Footing,
    m_tinted: bool,
    m_ripens: bool,
}

// BlockTFUnripeTorchCluster ripens once its spot is lit to six
const RIPEN_LIGHT: i32 = 6;

#[block_behavior]
impl BlockBehavior for SmallPlant
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool
    {
        ctx.world().isAir(ctx.pos()) && footed(self.m_footing, ctx.world(), ctx.pos())
    }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { footed(self.m_footing, ctx.world(), ctx.pos()) }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if ctx.isAuthority() && !footed(self.m_footing, ctx.world(), ctx.pos())
        {
            let world: World = ctx.world();
            let _ = world.dropBlockItems(ctx.pos(), ctx.block(), i32::from(ctx.meta()));
            let _ = world.setBlock(ctx.pos(), 0, 0);
        }
    }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        if self.m_ripens && ctx.isAuthority() && ctx.world().light(ctx.pos()) >= RIPEN_LIGHT
        {
            let _ = ctx.world().setBlock(ctx.pos(), ids::blockId(block::TROLLBER), 0);
        }
    }

    // shears cut any of these plants loose whole
    fn onHarvest(&self, ctx: &mut BlockCtx, player: Player)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        if player.held().is_some_and(|stack| stack.item.is("minecraft:shears"))
        {
            let name: Option<Identifier> = b173_api::nameOf(RegistryKind::Block, ctx.block());
            if let Some(name) = name
            {
                let _ = world.spawnItem(ids::center(pos), ItemStack::of(name, 1));
            }
            return;
        }
        let _ = world.dropBlockItems(pos, ctx.block(), i32::from(ctx.meta()));
    }

    fn colorMultiplier(&self, ctx: &mut BlockCtx) -> u32
    {
        if self.m_tinted
        {
            biomes::grass(ctx.world(), ctx.pos())
        }
        else
        {
            UNTINTED
        }
    }
}

fn plantSpec(row: &Plant) -> Result<BlockSpecBuilder>
{
    let mut spec: BlockSpecBuilder = BlockSpecBuilder::template("minecraft:red_flower")
        .name(row.m_label)
        .texture(&plantTexture(row.m_texture))
        .hardness(0.0)
        .sound("grass")
        .light(row.m_light)
        .tickRandomly()
        .behavior(SmallPlant
        {
            m_footing: row.m_footing,
            m_tinted: row.m_tinted,
            m_ripens: row.m_name == block::UNRIPE_TROLLBER,
        });
    if let Some(model) = shapeModel(row.m_shape)
    {
        spec = spec.model(model).collision(Vec::new());
    }
    spec = match row.m_drop
    {
        Some((name, count)) => spec.drops(vec![drop(
            name,
            if count[0] == count[1]
            {
                vec![count[0]]
            }
            else
            {
                count.to_vec()
            },
            1.0,
        )?]),
        None => spec.drops(Vec::new()),
    };
    Ok(spec)
}

// BlockTFThorns: rotated like a log, axis kept in metadata 0 1 2
const AXES: [&str; 3] = ["y", "x", "z"];

mod thorns
{
    pub const HARDNESS: f32 = 50.0;
    pub const RESISTANCE: f32 = 2000.0;
    pub const DAMAGE: i32 = 4;
    pub const BURNT_HARDNESS: f32 = 0.01;
    // a thorn column fills the middle ten sixteenths
    pub const INSET: f32 = 3.0;
    // each burst arm grows up to two blocks
    pub const ARM: i32 = 3;
    // a burst grows its own axis both ways and three random arms
    pub const RANDOM_ARMS: i32 = 3;
}

const DIRECTIONS: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

pub fn axisOf(direction: [i32; 3]) -> u8
{
    match direction
    {
        [0, _, 0] => 0,
        [_, 0, 0] => 1,
        _ => 2,
    }
}

fn thornModel() -> BlockModelSpec
{
    let inner: f32 = thorns::INSET;
    let outer: f32 = 16.0 - thorns::INSET;
    let full: f32 = 16.0;
    let column = |from: [f32; 3], to: [f32; 3], meta: i32| Cuboid { from, to, metas: vec![meta], ..Cuboid::default() };
    BlockModelSpec
    {
        cuboids: vec![
            column([inner, 0.0, inner], [outer, full, outer], 0),
            column([0.0, inner, inner], [full, outer, outer], 1),
            column([inner, inner, 0.0], [outer, outer, full], 2),
        ],
        ..BlockModelSpec::default()
    }
}

struct Thorns
{
    m_burnt: bool,
}

fn growArm(world: World, pos: BlockPos, direction: [i32; 3], green: i32)
{
    let length: i32 = 1 + world.random(thorns::ARM);
    for step in 1..length
    {
        let at: BlockPos = pos.offset(direction[0] * step, direction[1] * step, direction[2] * step);
        if !world.isAir(at)
        {
            break;
        }
        let _ = world.setBlock(at, green, i32::from(axisOf(direction)));
    }
}

// BlockTFThorns.removedByPlayer, breaking a thorn only turns it green and bursts new thorns
fn burst(world: World, pos: BlockPos, axis: u8)
{
    let green: i32 = ids::blockId(block::GREEN_THORNS);
    let _ = world.setBlock(pos, green, i32::from(axis));
    let along: usize = match axis
    {
        0 => 0,
        1 => 4,
        _ => 2,
    };
    growArm(world, pos, DIRECTIONS[along], green);
    growArm(world, pos, DIRECTIONS[along + 1], green);
    for _ in 0..thorns::RANDOM_ARMS
    {
        growArm(world, pos, DIRECTIONS[world.random(DIRECTIONS.len() as i32) as usize], green);
    }
}

#[block_behavior]
impl BlockBehavior for Thorns
{
    // BlockTFThorns.breakBlock, live thorns hold leaves like a log
    fn onRemoved(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority() && !self.m_burnt
        {
            super::wood::shedLeaves(ctx.world(), ctx.pos());
        }
    }

    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        if !ctx.isAuthority()
        {
            return;
        }
        if !self.m_burnt
        {
            entity.damage(thorns::DAMAGE, None);
            return;
        }
        if entity.info().is_some_and(|info| info.health.is_some())
        {
            let world: World = ctx.world();
            let _ = world.breakEffect(ctx.pos(), ctx.block(), i32::from(ctx.meta()));
            let _ = world.setBlock(ctx.pos(), 0, 0);
        }
    }

    fn onDestroyedByPlayer(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority() && !self.m_burnt
        {
            burst(ctx.world(), ctx.pos(), ctx.meta());
        }
    }
}

fn thornSpec(label: &str, look: &str, burnt: bool) -> BlockSpecBuilder
{
    let hardness: f32 = if burnt
    {
        thorns::BURNT_HARDNESS
    }
    else
    {
        thorns::HARDNESS
    };
    let mut spec: BlockSpecBuilder = base("minecraft:block", label, Kind::Wood, hardness)
        .faces(FaceTextures
        {
            top: Some(texture(&format!("{look}_thorns_top"))),
            bottom: Some(texture(&format!("{look}_thorns_top"))),
            side: Some(texture(&format!("{look}_thorns_side"))),
            front: None,
        })
        .model(thornModel())
        .drops(Vec::new())
        .behavior(Thorns { m_burnt: burnt });
    if burnt
    {
        spec = spec.sound("sand").resistance(0.0);
    }
    else
    {
        spec = spec.resistance(thorns::RESISTANCE).sustainsLeaves();
    }
    spec
}

// BlockTFThornRose stays only beside a block that holds leaves
struct ThornRose;

fn holdsLeaves(world: World, pos: BlockPos) -> bool
{
    DIRECTIONS.iter().any(|direction|
    {
        let block: i32 = world.block(pos.offset(direction[0], direction[1], direction[2]));
        ids::holdsLeaves(block)
    })
}

#[block_behavior]
impl BlockBehavior for ThornRose
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool
    {
        ctx.world().isAir(ctx.pos()) && holdsLeaves(ctx.world(), ctx.pos())
    }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { holdsLeaves(ctx.world(), ctx.pos()) }
}

fn registerThorns(ctx: &mut RegisterCtx) -> Result<()>
{
    super::put(ctx, block::BROWN_THORNS, thornSpec("Thorns", "brown", false))?;
    super::put(ctx, block::GREEN_THORNS, thornSpec("Green Thorns", "green", false))?;
    super::put(ctx, block::BURNT_THORNS, thornSpec("Burnt Thorns", "burnt", true))?;
    for name in [block::BROWN_THORNS, block::GREEN_THORNS, block::BURNT_THORNS]
    {
        ctx.blocks().states(
            name,
            BlockStatesSpec { properties: vec![property("axis", &AXES)], ..BlockStatesSpec::default() },
        )?;
    }
    let roseHardness: f32 = 10.0;
    super::put(
        ctx,
        block::THORN_ROSE,
        BlockSpecBuilder::template("minecraft:red_flower")
            .name("Thorn Rose")
            .texture(&texture("thornrose"))
            .hardness(roseHardness)
            .sound("grass")
            .behavior(ThornRose),
    )
}

// BlockTFHugeLilyPad: a 2x2 pad, each quarter keeps its piece and the pad's turn in metadata
mod lily
{
    pub const COLOR: u32 = 0x208030;
    pub const PIECES: i32 = 4;
    // a pad lies one sixty fourth of a block above the water
    pub const THICKNESS: f32 = 0.25;
}

struct HugeLilyPad;

fn padHeld(world: World, pos: BlockPos, pad: i32) -> bool
{
    let base: BlockPos = BlockPos::new(pos.x >> 1 << 1, pos.y, pos.z >> 1 << 1);
    let water: i32 = ids::blockId("minecraft:water");
    (0..2).all(|dx|
    {
        (0..2).all(|dz|
        {
            let at: BlockPos = base.offset(dx, 0, dz);
            world.block(at) == pad && world.block(at.offset(0, -1, 0)) == water && world.meta(at.offset(0, -1, 0)) == 0
        })
    })
}

fn clearPad(world: World, pos: BlockPos, pad: i32)
{
    let base: BlockPos = BlockPos::new(pos.x >> 1 << 1, pos.y, pos.z >> 1 << 1);
    for dx in 0..2
    {
        for dz in 0..2
        {
            let at: BlockPos = base.offset(dx, 0, dz);
            if at != pos && world.block(at) == pad
            {
                let _ = world.setBlock(at, 0, 0);
            }
        }
    }
}

#[block_behavior]
impl BlockBehavior for HugeLilyPad
{
    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if ctx.isAuthority() && !padHeld(ctx.world(), ctx.pos(), ctx.block())
        {
            clearPad(ctx.world(), ctx.pos(), ctx.block());
            let _ = ctx.world().setBlock(ctx.pos(), 0, 0);
        }
    }

    fn onDestroyedByPlayer(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority()
        {
            clearPad(ctx.world(), ctx.pos(), ctx.block());
        }
    }

    fn colorMultiplier(&self, _ctx: &mut BlockCtx) -> u32 { lily::COLOR }
}

// BlockTFHugeLilyPad.getIcon, the quarter shown is the piece turned by the pad orientation
fn padVariants() -> Vec<StateVariantSpec>
{
    let mut variants: Vec<StateVariantSpec> = Vec::new();
    for orient in 0..lily::PIECES
    {
        for piece in 0..lily::PIECES
        {
            let turned: i32 = match orient
            {
                1 => 3,
                3 => 1,
                other => other,
            };
            let display: i32 = (piece + turned) % lily::PIECES;
            let mut values: std::collections::BTreeMap<String, String> = when("orient", &orient.to_string());
            values.insert("piece".to_owned(), piece.to_string());
            let top: String = texture(&format!("huge_lilypad_{display}"));
            variants.push(StateVariantSpec
            {
                when: values,
                texture: None,
                textures: Some(FaceTextures
                {
                    top: Some(top.clone()),
                    bottom: Some(top),
                    side: Some(texture("blank")),
                    front: None,
                }),
            });
        }
    }
    variants
}

struct WaterLily;

fn onStillWater(world: World, pos: BlockPos) -> bool
{
    let below: BlockPos = pos.offset(0, -1, 0);
    world.block(below) == ids::blockId("minecraft:water") && world.meta(below) == 0
}

#[block_behavior]
impl BlockBehavior for WaterLily
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool
    {
        ctx.world().isAir(ctx.pos()) && onStillWater(ctx.world(), ctx.pos())
    }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { onStillWater(ctx.world(), ctx.pos()) }
}

fn registerWater(ctx: &mut RegisterCtx) -> Result<()>
{
    let flat: Cuboid = Cuboid { from: [0.0; 3], to: [16.0, lily::THICKNESS, 16.0], ..Cuboid::default() };
    super::put(
        ctx,
        block::HUGE_LILY_PAD,
        BlockSpecBuilder::template("minecraft:block")
            .name("Huge Lily Pad")
            .material("plants")
            .texture(&texture("huge_lilypad_0"))
            .hardness(0.0)
            .sound("grass")
            .model(BlockModelSpec { cuboids: vec![flat.clone()], ..BlockModelSpec::default() })
            .collision(vec![flat])
            .drops(Vec::new())
            .behavior(HugeLilyPad),
    )?;
    let quarters: Vec<String> = (0..lily::PIECES).map(|piece| piece.to_string()).collect();
    let values: Vec<&str> = quarters.iter().map(String::as_str).collect();
    ctx.blocks().states(
        block::HUGE_LILY_PAD,
        BlockStatesSpec
        {
            properties: vec![property("piece", &values), property("orient", &values)],
            variants: padVariants(),
            ..BlockStatesSpec::default()
        },
    )?;
    super::put(
        ctx,
        block::HUGE_WATER_LILY,
        BlockSpecBuilder::template("minecraft:red_flower")
            .name("Huge Water Lily")
            .texture(&texture("huge_waterlily"))
            .hardness(0.0)
            .sound("grass")
            .behavior(WaterLily),
    )
}

// BlockTFUberousSoil: smothered soil turns to dirt, a plant set on it grows on the spot
struct UberousSoil;

// Material.isSolid of the block above, uberous soil under it turns to dirt
fn covered(world: World, pos: BlockPos) -> bool { World::blockInfo(world.block(pos.offset(0, 1, 0))).solid }

#[block_behavior]
impl BlockBehavior for UberousSoil
{
    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority() && covered(ctx.world(), ctx.pos())
        {
            let _ = ctx.world().setBlock(ctx.pos(), ids::blockId("minecraft:dirt"), 0);
        }
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        // a covered patch turning to dirt soil by soil within one neighbor chain overflows the wasm stack
        if covered(ctx.world(), ctx.pos())
        {
            let _ = ctx.world().scheduleTick(ctx.pos(), ids::blockId(block::UBEROUS_SOIL), 1);
            return;
        }

        crate::magic::uberous(ctx.world(), ctx.pos());
    }
}

mod foliage
{
    // ColorizerFoliage pine and birch colors
    pub const PINE: u32 = 0x619961;
    pub const BIRCH: u32 = 0x80A755;
    pub const LEAVES_TEXTURE: i32 = 52;
    pub const LEAVES_HARDNESS: f32 = 0.2;
}

mod soil
{
    pub const HARDNESS: f32 = 0.6;
    pub const HEIGHT: f32 = 15.0;
}

fn registerGiantPlants(ctx: &mut RegisterCtx) -> Result<()>
{
    let top: Cuboid = Cuboid { from: [0.0; 3], to: [16.0, soil::HEIGHT, 16.0], ..Cuboid::default() };
    super::put(
        ctx,
        block::UBEROUS_SOIL,
        base("minecraft:block", "Uberous Soil", Kind::Ground, soil::HARDNESS)
            .texture(&texture("uberous_soil"))
            .tickRandomly()
            .model(BlockModelSpec { cuboids: vec![top], ..BlockModelSpec::default() })
            .drops(vec![drop("minecraft:dirt", vec![1], 1.0)?])
            .behavior(UberousSoil),
    )?;
    let stalkHardness: f32 = 1.25;
    let stalkResistance: f32 = 7.0;
    super::put(
        ctx,
        block::HUGE_STALK,
        base("minecraft:block", "Huge Stalk", Kind::Wood, stalkHardness)
            .sound("grass")
            .resistance(stalkResistance)
            .faces(super::faces(&super::Look::Column { top: "huge_stalk_top", side: "huge_stalk" }))
            .sustainsLeaves()
            .behavior(super::wood::Log),
    )?;
    let gloomHardness: f32 = 0.2;
    // BlockTFHugeGloomBlock drops nothing seven times in ten and up to two mushglooms otherwise
    let gloomDrop: DropSpec = drop(block::MUSHGLOOM, vec![1, 2], 0.3)?;
    super::put(
        ctx,
        block::HUGE_GLOOM,
        base("minecraft:block", "Huge Mushgloom", Kind::Wood, gloomHardness)
            .light(light::GLOOM)
            .faces(FaceTextures
            {
                top: Some(texture("huge_gloom_cap")),
                bottom: Some(texture("huge_gloom_inside")),
                side: Some(texture("huge_gloom_cap")),
                front: None,
            })
            .drops(vec![gloomDrop]),
    )?;
    let stem: FaceTextures = FaceTextures
    {
        top: Some(texture("huge_gloom_inside")),
        bottom: Some(texture("huge_gloom_inside")),
        side: Some(texture("huge_gloom_stem")),
        front: None,
    };
    ctx.blocks().states(
        block::HUGE_GLOOM,
        BlockStatesSpec
        {
            properties: vec![property("part", &["cap", "stem"])],
            variants: vec![StateVariantSpec
            {
                when: when("part", "stem"),
                texture: None,
                textures: Some(stem.clone()),
            }],
            ..BlockStatesSpec::default()
        },
    )?;
    registerMushrooms(ctx, &stem)?;
    for (name, label, pine) in
        [(block::THORN_LEAVES, "Thorn Leaves", true), (block::BEANSTALK_LEAVES, "Beanstalk Leaves", false)]
    {
        let color: u32 = if pine { foliage::PINE } else { foliage::BIRCH };
        super::put(
            ctx,
            name,
            BlockSpecBuilder::template("minecraft:leaves")
                .name(label)
                .texture(&vanillaTexture(foliage::LEAVES_TEXTURE))
                .hardness(foliage::LEAVES_HARDNESS)
                .sound("grass")
                .color(color)
                .drops(Vec::new()),
        )?;
    }
    Ok(())
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &PLANTS
    {
        super::put(ctx, row.m_name, plantSpec(row)?)?;
    }
    registerThorns(ctx)?;
    registerWater(ctx)?;
    registerGiantPlants(ctx)
}

// beta has no huge mushrooms, the canopy mushrooms reuse the gloom textures tinted brown and red
struct HugeMushroom
{
    m_cap: u32,
}

const WHITE: u32 = 0xFFFFFF;

#[block_behavior]
impl BlockBehavior for HugeMushroom
{
    fn colorMultiplier(&self, ctx: &mut BlockCtx) -> u32 { if ctx.meta() == 0 { self.m_cap } else { WHITE } }
}

fn registerMushrooms(ctx: &mut RegisterCtx, stem: &FaceTextures) -> Result<()>
{
    let hardness: f32 = 0.2;
    let chance: f32 = 0.3;
    let rows: [(&str, &str, &str, u32); 2] = [
        (block::HUGE_BROWN_MUSHROOM, "Huge Brown Mushroom", "minecraft:brown_mushroom", 0x8D6A50),
        (block::HUGE_RED_MUSHROOM, "Huge Red Mushroom", "minecraft:red_mushroom", 0xC8302A),
    ];
    for (name, label, small, cap) in rows
    {
        super::put(
            ctx,
            name,
            base("minecraft:block", label, Kind::Wood, hardness)
                .texture(&texture("huge_gloom_cap"))
                .drops(vec![drop(small, vec![1, 2], chance)?])
                .behavior(HugeMushroom { m_cap: cap }),
        )?;
        ctx.blocks().states(
            name,
            BlockStatesSpec
            {
                properties: vec![property("part", &["cap", "stem"])],
                variants: vec![StateVariantSpec
                {
                    when: when("part", "stem"),
                    texture: None,
                    textures: Some(stem.clone()),
                }],
                ..BlockStatesSpec::default()
            },
        )?;
    }
    Ok(())
}
