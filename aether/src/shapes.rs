use crate::names::{self, block};
use b173_api::math::floor;
use b173_api::prelude::*;

// a building stone or wood the Aether makes stairs and slabs from
struct Stone
{
    m_name: &'static str,
    m_texture: &'static str,
    m_material: &'static str,
    m_sound: &'static str,
    m_tool: ToolClass,
    m_hardness: f32,
    m_resistance: f32,
    // BlockAetherSlab sets its own strength, stairs copy their stone
    m_slabHardness: f32,
    m_slabResistance: f32,
}

const fn stone(name: &'static str, texture: &'static str, hardness: f32, resistance: f32, slab: (f32, f32)) -> Stone
{
    Stone
    {
        m_name: name,
        m_texture: texture,
        m_material: "rock",
        m_sound: "stone",
        m_tool: ToolClass::Pickaxe,
        m_hardness: hardness,
        m_resistance: resistance,
        m_slabHardness: slab.0,
        m_slabResistance: slab.1,
    }
}

// BlocksAether stairs and slabs, in registration order
const STONES: [Stone; 8] = [
    stone("carved", "carved_stone", 0.5, 2.5, (2.0, 10.0)),
    stone("angelic", "angelic_stone", 0.5, 2.5, (2.0, 10.0)),
    stone("hellfire", "hellfire_stone", 0.5, 2.5, (2.0, 10.0)),
    Stone
    {
        m_name: "skyroot",
        m_texture: "skyroot_planks",
        m_material: "wood",
        m_sound: "wood",
        m_tool: ToolClass::Axe,
        m_hardness: 2.0,
        m_resistance: 5.0,
        m_slabHardness: 2.0,
        m_slabResistance: 5.0,
    },
    stone("holystone", "holystone", 0.5, 2.5, (2.0, 10.0)),
    stone("holystone_brick", "holystone_brick", 0.5, 2.5, (2.0, 10.0)),
    stone("mossy_holystone", "mossy_holystone", 0.5, 2.5, (2.0, 10.0)),
    stone("aerogel", "aerogel", 1.0, 2000.0, (2.0, 2000.0)),
];

mod shape
{
    pub const FULL: f32 = 16.0;
    pub const HALF: f32 = 8.0;
    // BlockStairs bounds by facing: the raised half sits east, west, south or north
    pub const STEPS: [[f32; 6]; 4] = [
        [8.0, 8.0, 0.0, 16.0, 16.0, 16.0],
        [0.0, 8.0, 0.0, 8.0, 16.0, 16.0],
        [0.0, 8.0, 8.0, 16.0, 16.0, 16.0],
        [0.0, 8.0, 0.0, 16.0, 16.0, 8.0],
    ];
    pub const QUARTERS: f32 = 4.0;
    pub const FULL_TURN: f32 = 360.0;
}

// named states in metadata order, the last property changing fastest
mod state
{
    pub const FACING: &str = "facing";
    pub const HEAD: &str = "head";
    pub const OCCUPIED: &str = "occupied";
    // BlockStairs, the side the raised half sits on
    pub const STAIRS: [&str; 4] = ["east", "west", "south", "north"];
    // the quarter a player faces, which BlockStairs.onBlockPlacedBy and BlockBed store
    pub const QUARTERS: [&str; 4] = ["south", "west", "north", "east"];
}

fn texture(name: &str) -> String { format!("aether:block/{name}") }

fn cuboid(bounds: [f32; 6], metas: Vec<i32>) -> Cuboid
{
    Cuboid
    {
        from: [bounds[0], bounds[1], bounds[2]],
        to: [bounds[3], bounds[4], bounds[5]],
        texture: None,
        metas,
        connect: None,
    }
}

// the quarter a player faces, MathHelper.floor(yaw * 4 / 360 + 0.5) & 3
fn quarter(entity: Entity) -> usize
{
    let yaw: f32 = entity.info().map_or(0.0, |info| info.yaw);
    (floor(f64::from(yaw * shape::QUARTERS / shape::FULL_TURN + 0.5)) & 3) as usize
}

struct Stairs;

#[block_behavior]
impl BlockBehavior for Stairs
{
    fn onPlacedBy(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        names::setProperty(ctx, state::FACING, state::QUARTERS[quarter(entity)]);
    }
}

// BlockStep.onBlockAdded: a slab set on the same slab joins it into a double slab
struct Slab
{
    m_slab: String,
    m_double: String,
}

#[block_behavior]
impl BlockBehavior for Slab
{
    fn onAdded(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let below: BlockPos = ctx.pos().offset(0, -1, 0);
        if ctx.isAuthority() && world.is(below, &self.m_slab)
        {
            let _ = world.clear(ctx.pos());
            let _ = world.place(below, &self.m_double);
        }
    }
}

fn shaped(row: &Stone, hardness: f32, resistance: f32) -> BlockSpecBuilder
{
    BlockSpecBuilder::template("minecraft:block")
        .material(row.m_material)
        .texture(&texture(row.m_texture))
        .hardness(hardness)
        .resistance(resistance)
        .sound(row.m_sound)
        .tool(row.m_tool, 0)
}

fn stairs(row: &Stone) -> BlockSpecBuilder
{
    let mut cuboids: Vec<Cuboid> = vec![cuboid([0.0, 0.0, 0.0, shape::FULL, shape::HALF, shape::FULL], Vec::new())];
    cuboids.extend(shape::STEPS.iter().enumerate().map(|(meta, bounds)| cuboid(*bounds, vec![meta as i32])));
    shaped(row, row.m_hardness, row.m_resistance)
        .model(BlockModelSpec { cuboids, ..BlockModelSpec::default() })
        .behavior(Stairs)
}

// ItemSkyrootBed and BlockSkyrootBed: a two block bed of the bed template, dropping the item from its foot
struct Bed;

#[block_behavior]
impl BlockBehavior for Bed
{
    fn onHarvest(&self, ctx: &mut BlockCtx, _player: Player)
    {
        let pos: BlockPos = ctx.pos();

        let head: bool = ctx.stateAt().is_some_and(|current| current.flag(state::HEAD));
        if ctx.isAuthority() && !head
        {
            let _ = ctx.world().spawnItem(
                [pos.x as f64 + 0.5, pos.y as f64 + 0.5, pos.z as f64 + 0.5],
                ItemStack::of("skyroot_bed_item", 1),
            );
        }
    }
}

const BED: &str = "skyroot_bed";
// BlockBed.headBlockToFootBlockMap, where the head lies from the foot for each facing
const BED_AHEAD: [[i32; 2]; 4] = [[0, 1], [-1, 0], [0, -1], [1, 0]];

fn bedState(facing: &str) -> Result<StateData>
{
    Ok(StateData::of(Identifier::parseIn(BED, "aether")?).with(state::FACING, facing))
}

struct BedItem;

#[item_behavior]
impl ItemBehavior for BedItem
{
    // ItemBed.onItemUse: the foot goes on the clicked top face, the head one block ahead in the player's facing
    fn onUseOnBlock(&self, world: World, player: Player, stack: ItemStack, pos: BlockPos, face: i32) -> Use
    {
        const TOP: i32 = 1;

        if face != TOP
        {
            return Use::Pass;
        }
        let facing: usize = quarter(Entity { world, id: player.id });
        let foot: BlockPos = pos.offset(0, 1, 0);
        let head: BlockPos = foot.offset(BED_AHEAD[facing][0], 0, BED_AHEAD[facing][1]);
        let solid = |at: BlockPos| World::blockInfo(world.block(at.offset(0, -1, 0))).solid;
        if !world.isAir(foot) || !world.isAir(head) || !solid(foot) || !solid(head)
        {
            return Use::Pass;
        }

        if world.isAuthority()
        {
            let half = |head: bool| bedState(state::QUARTERS[facing]).map(|bed| bed.with(state::HEAD, head));
            if let Ok(bed) = half(false)
            {
                let _ = world.setBlockState(foot, bed);
            }
            if let Ok(bed) = half(true)
            {
                let _ = world.setBlockState(head, bed);
            }
        }

        Use::Done((stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack }))
    }
}

// BlockPresent.harvestBlock: one of four surprises, two of them a primed present
struct Present;

mod present
{
    pub const MIN: i32 = 6;
    pub const SPREAD: i32 = 4;
    pub const CRATES: i32 = 4;
    pub const SWORD_ODDS: i32 = 9;
}

#[block_behavior]
impl BlockBehavior for Present
{
    fn onHarvest(&self, ctx: &mut BlockCtx, _player: Player)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let at: [f64; 3] = [pos.x as f64 + 0.5, pos.y as f64 + 0.5, pos.z as f64 + 0.5];
        let amount: i32 = present::MIN + world.random(present::SPREAD);

        match world.random(present::CRATES)
        {
            // b1.7.3 has no experience orbs, the orb crate gives gingerbread men instead
            0 =>
            {
                let _ = world.spawnItem(at, ItemStack::of("gingerbread_man", amount));
            }

            1 if world.random(present::SWORD_ODDS) == 0 =>
            {
                let _ = world.spawnItem(at, ItemStack::of("candy_cane_sword", 1));
            }

            1 =>
            {
                let _ = world.spawnItem(at, ItemStack::of("gingerbread_man", amount));
            }

            _ =>
            {
                world.playSound(at, "random.fuse", 1.0, 1.0);
                crate::entities::present::prime(world, pos);
            }
        }
    }
}

// BlockBerryBushStem: regrows into a berry bush in light
struct Stem;

mod stem
{
    pub const LIGHT: i32 = 9;
    pub const ODDS: i32 = 60;
}

#[block_behavior]
impl BlockBehavior for Stem
{
    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        if ctx.isAuthority() && world.light(pos.offset(0, 1, 0)) >= stem::LIGHT && world.random(stem::ODDS) == 0
        {
            let _ = world.place(pos, block::BERRY_BUSH);
        }
    }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { ctx.world().tagged(ctx.pos().offset(0, -1, 0), crate::tags::SOIL) }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let blocks: Blocks = ctx.blocks();

    for row in &STONES
    {
        let stairsName: String = format!("{}_stairs", row.m_name);
        blocks.register(&stairsName, stairs(row).property(Property::values(state::FACING, &state::STAIRS)))?;

        let double: String = format!("{}_double_slab", row.m_name);
        let slabDrop: DropSpec = crate::entities::drop(&format!("{}_slab", row.m_name), [2, 2]);
        blocks.register(&double, shaped(row, row.m_slabHardness, row.m_slabResistance).drops(vec![slabDrop]))?;

        let half: BlockModelSpec = BlockModelSpec
        {
            cuboids: vec![cuboid([0.0, 0.0, 0.0, shape::FULL, shape::HALF, shape::FULL], Vec::new())],
            ..BlockModelSpec::default()
        };
        let slab: String = format!("{}_slab", row.m_name);
        blocks.register(
            &slab,
            shaped(row, row.m_slabHardness, row.m_slabResistance)
                .model(half)
                .behavior(Slab { m_slab: slab.clone(), m_double: double }),
        )?;
    }

    let sheet = |name: &str| texture(&format!("skyroot_bed_{name}"));

    let faces: BedFaces = BedFaces
    {
        top_head: sheet("head_top"),
        top_foot: sheet("feet_top"),
        side_head: sheet("head_side"),
        side_foot: sheet("feet_side"),
        end_head: sheet("head_end"),
        end_foot: sheet("feet_end"),
        bottom: texture("skyroot_planks"),
    };
    blocks.register(
        BED,
        BlockSpecBuilder::template("minecraft:bed")
            .bed(faces)
            .hardness(0.2)
            .sound("wood")
            .drops(Vec::new())
            .property(Property::bool(state::HEAD))
            .property(Property::bool(state::OCCUPIED))
            .property(Property::values(state::FACING, &state::QUARTERS))
            .behavior(Bed),
    )?;
    blocks.register(
        "present",
        BlockSpecBuilder::template("minecraft:block")
            .material("grass")
            .faces(FaceTextures
            {
                top: Some(texture("present_top")),
                bottom: Some(texture("present_top")),
                side: Some(texture("present_side")),
                front: None,
            })
            .hardness(0.6)
            .sound("grass")
            .drops(Vec::new())
            .behavior(Present),
    )?;

    for name in ["holiday_leaves", "decorated_holiday_leaves"]
    {
        blocks.register(
            name,
            BlockSpecBuilder::template("minecraft:leaves")
                .texture(&texture(name))
                .color(0xFFFFFF)
                .hardness(0.2)
                .sound("grass")
                .drops(Vec::new())
                .behavior(crate::sparkle::HOLIDAY),
        )?;
    }

    blocks.register(
        "berry_bush_stem",
        BlockSpecBuilder::template("minecraft:red_flower")
            .texture(&texture("berry_bush_stem"))
            .hardness(0.2)
            .sound("grass")
            .tickRandomly()
            .drops(Vec::new())
            .behavior(Stem),
    )?;
    ctx.items().register(
        "skyroot_bed_item",
        ItemSpecBuilder::new().texture("aether:item/skyroot_bed_item").maxStack(1).behavior(BedItem),
    )?;
    Ok(())
}
