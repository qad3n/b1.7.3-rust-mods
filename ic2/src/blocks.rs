use crate::core::{blockTexture, dir, ids, own, random, sound, stacks};
use crate::world::tree;
use b173_api::prelude::*;

pub(crate) const RUBBER_WOOD: &str = "ic2:rubber_wood";
pub(crate) const RUBBER_LEAVES: &str = "ic2:rubber_leaves";
pub(crate) const RUBBER_SAPLING: &str = "ic2:rubber_sapling";
pub(crate) const RESIN: &str = "ic2:resin";
pub(crate) const IRON_FENCE: &str = "ic2:iron_fence";
pub(crate) const MINING_PIPE: &str = "ic2:mining_pipe";
pub(crate) const MINING_TIP: &str = "ic2:mining_pipe_tip";
pub(crate) const COPPER_ORE: &str = "ic2:copper_ore";
pub(crate) const TIN_ORE: &str = "ic2:tin_ore";
pub(crate) const URANIUM_ORE: &str = "ic2:uranium_ore";

// a plain cube of the IC2 sheet: BlockTex, BlockMetal and the reinforced blocks
struct Cube
{
    m_name: &'static str,
    m_material: &'static str,
    m_sound: &'static str,
    m_hardness: f32,
    m_resistance: f32,
    m_tool: Option<(ToolClass, i32)>,
    m_tag: Option<&'static str>,
}

const CUBES: [Cube; 8] = [
    cube("copper_ore", "rock", "stone", 3.0, 5.0, Some((ToolClass::Pickaxe, 1)), Some("c:ores/copper")),
    cube("tin_ore", "rock", "stone", 3.0, 5.0, Some((ToolClass::Pickaxe, 1)), Some("c:ores/tin")),
    cube("uranium_ore", "rock", "stone", 4.0, 6.0, Some((ToolClass::Pickaxe, 2)), Some("c:ores/uranium")),
    cube("reinforced_stone", "iron", "metal", 4.0, 100.0, Some((ToolClass::Pickaxe, 3)), None),
    cube("copper_block", "iron", "metal", 2.0, 10.0, Some((ToolClass::Pickaxe, 0)), Some("c:storage_blocks/copper")),
    cube("tin_block", "iron", "metal", 2.0, 10.0, Some((ToolClass::Pickaxe, 0)), Some("c:storage_blocks/tin")),
    cube("bronze_block", "iron", "metal", 2.0, 10.0, Some((ToolClass::Pickaxe, 0)), Some("c:storage_blocks/bronze")),
    cube("uranium_block", "iron", "metal", 2.0, 10.0, Some((ToolClass::Pickaxe, 0)), Some("c:storage_blocks/uranium")),
];

const fn cube(
    name: &'static str,
    material: &'static str,
    sound: &'static str,
    hardness: f32,
    resistance: f32,
    tool: Option<(ToolClass, i32)>,
    tag: Option<&'static str>,
) -> Cube
{
    Cube
    {
        m_name: name,
        m_material: material,
        m_sound: sound,
        m_hardness: hardness,
        m_resistance: resistance,
        m_tool: tool,
        m_tag: tag,
    }
}

fn drop(item: &str, chance: f32) -> Result<DropSpec>
{
    Ok(DropSpec { item: Identifier::parse(item)?, count: vec![1], meta: 0, chance })
}

fn registerCubes(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &CUBES
    {
        let mut spec: BlockSpecBuilder = BlockSpecBuilder::template("minecraft:block")
            .material(row.m_material)
            .sound(row.m_sound)
            .hardness(row.m_hardness)
            .resistance(row.m_resistance)
            .texture(&blockTexture(row.m_name));

        if let Some((class, level)) = row.m_tool
        {
            spec = spec.tool(class, level);
        }

        if let Some(tag) = row.m_tag
        {
            spec = spec.tag(tag);
        }

        // BlockTex.idDropped gives the uranium ore item instead of the block
        if row.m_name == "uranium_ore"
        {
            spec = spec.drops(vec![drop(&own("uranium_chunk"), 1.0)?]);
        }

        ctx.blocks().register(row.m_name, spec)?;
    }

    Ok(())
}

mod wood
{
    // BlockRubWood metadata: 1 plain, 2 to 5 a resin hole facing that face, plus 6 once tapped dry
    pub const PLAIN: i32 = 1;
    pub const FIRST_HOLE: i32 = 2;
    pub const DRY: i32 = 6;
    pub const REFILL_CHANCE: i32 = 200;
    pub const REFILL_TICKS: i32 = 100;
    // dropBlockAsItemWithChance adds a resin one time in six from a holed log
    pub const RESIN_CHANCE: i32 = 6;
}

pub(crate) fn rubberWoodTickRate() -> i32 { wood::REFILL_TICKS }

pub(crate) fn holed(meta: i32) -> bool { meta >= wood::FIRST_HOLE }

pub(crate) fn hole(meta: i32) -> i32 { meta % wood::DRY }

pub(crate) fn dry(meta: i32) -> bool { meta >= wood::DRY }

pub(crate) fn dried(meta: i32) -> i32 { meta + wood::DRY }

pub(crate) fn plain() -> i32 { wood::PLAIN }

struct RubberWood;

#[block_behavior]
impl BlockBehavior for RubberWood
{
    fn texture(&self, face: i32, meta: i32) -> i32
    {
        let sprite: &str = match face
        {
            dir::DOWN | dir::UP => "rubber_wood_top",
            face if holed(meta) && hole(meta) == face && dry(meta) => "rubber_wood_tapped",
            face if holed(meta) && hole(meta) == face => "rubber_wood_resin",
            _ => "rubber_wood",
        };
        crate::core::sprites::terrain(&blockTexture(sprite))
    }

    // BlockRubWood.updateTick: a tapped hole fills again one tick in two hundred
    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        let meta: i32 = i32::from(ctx.meta());
        if !ctx.isAuthority() || !dry(meta)
        {
            return;
        }

        let world: World = ctx.world();
        if world.random(wood::REFILL_CHANCE) == 0
        {
            ctx.setMeta(hole(meta) as u8);
            return;
        }

        if let Err(error) = world.scheduleTick(ctx.pos(), ctx.block(), wood::REFILL_TICKS)
        {
            warn(&format!("rubber wood could not schedule its refill: {error}"));
        }
    }

    fn onHarvest(&self, ctx: &mut BlockCtx, _player: Player)
    {
        let world: World = ctx.world();
        if !ctx.isAuthority() || ctx.meta() == 0 || world.random(wood::RESIN_CHANCE) != 0
        {
            return;
        }

        stacks::drop(world, ctx.pos(), &ItemStack::of(own("sticky_resin").as_str(), 1));
    }
}

mod sapling
{
    pub const LIGHT: i32 = 9;
    pub const CHANCE: i32 = 30;
    pub const BONE_MEAL: i32 = 15;
}

fn rooted(ctx: &BlockCtx) -> bool
{
    let below: i32 = ctx.world().block(ctx.pos().offset(0, -1, 0));
    ["minecraft:grass", "minecraft:dirt", "minecraft:farmland"].iter().any(|name| ids::block(name) == Some(below))
}

// BlockRubSapling: grows a rubber tree with light above, or at once from bone meal
struct RubberSapling;

#[block_behavior]
impl BlockBehavior for RubberSapling
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { ctx.world().isAir(ctx.pos()) && rooted(ctx) }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { rooted(ctx) }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        if !ctx.isAuthority() || world.light(pos.offset(0, 1, 0)) < sapling::LIGHT || world.random(sapling::CHANCE) != 0
        {
            return;
        }

        tree::grow(world, pos);
    }

    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if !ctx.isAuthority()
        {
            return false;
        }

        let Some(held) = player.held()
        else
        {
            return false;
        };

        if !held.item.is("minecraft:dye") || held.damage != sapling::BONE_MEAL
        {
            return false;
        }

        tree::grow(ctx.world(), ctx.pos());
        if let Err(error) = player.setHeld(stacks::less(held, 1))
        {
            warn(&format!("could not use up the bone meal: {error}"));
        }

        false
    }
}

mod sheet
{
    pub const THICKNESS: f32 = 2.0;
    pub const SUPPORTED: i32 = 1;
    // BlockRubberSheet.onEntityCollidedWithBlock bounces what falls faster than this
    pub const FALLING: f64 = -0.4;
    pub const SIDEWAYS: f64 = 1.1;
    pub const JUMP_BOUNCE: f64 = -1.3;
    pub const SNEAK_BOUNCE: f64 = -0.1;
    pub const BOUNCE: f64 = -0.8;
}

fn opaque(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    block != 0 && World::blockInfo(block).opaque
}

// BlockRubberSheet.isBlockSupporter: an opaque cube or another sheet
fn supporter(world: World, pos: BlockPos, sheet: i32) -> bool { opaque(world, pos) || world.block(pos) == sheet }

fn sheetHeld(world: World, pos: BlockPos, sheet: i32) -> bool
{
    [dir::WEST, dir::EAST, dir::NORTH, dir::SOUTH].iter().any(|face| supporter(world, dir::step(pos, *face), sheet))
}

// one direction of canSupportWeight: along the row of sheets until an opaque wall or a sheet resting on one
fn anchored(world: World, pos: BlockPos, sheet: i32, step: [i32; 3]) -> bool
{
    let mut at: BlockPos = pos;

    loop
    {
        if opaque(world, at)
        {
            return true;
        }

        if world.block(at) != sheet
        {
            return false;
        }

        if opaque(world, at.offset(0, -1, 0))
        {
            return true;
        }

        at = at.offset(step[0], step[1], step[2]);
    }
}

// BlockRubberSheet.canSupportWeight, a sheet spanned between two anchors on either axis holds a living thing
fn supportsWeight(ctx: &mut BlockCtx) -> bool
{
    if i32::from(ctx.meta()) == sheet::SUPPORTED
    {
        return true;
    }

    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    let block: i32 = ctx.block();
    let spans: bool = (anchored(world, pos, block, [-1, 0, 0]) && anchored(world, pos, block, [1, 0, 0]))
        || (anchored(world, pos, block, [0, 0, -1]) && anchored(world, pos, block, [0, 0, 1]));
    if spans
    {
        ctx.setMeta(sheet::SUPPORTED as u8);
    }

    spans
}

struct RubberSheet;

#[block_behavior]
impl BlockBehavior for RubberSheet
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { sheetHeld(ctx.world(), ctx.pos(), ctx.block()) }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if i32::from(ctx.meta()) == sheet::SUPPORTED
        {
            ctx.setMeta(0);
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        if sheetHeld(world, pos, ctx.block())
        {
            return;
        }

        if let Err(error) = world.dropBlockItems(pos, ctx.block(), i32::from(ctx.meta()))
        {
            warn(&format!("could not drop a rubber sheet: {error}"));
        }

        let _ = world.setBlock(pos, 0, 0);
    }

    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let below: i32 = world.block(pos.offset(0, -1, 0));
        if below != 0 && World::blockInfo(below).opaque
        {
            return;
        }

        let Some(info) = entity.info()
        else
        {
            return;
        };
        let living: bool = info.health.is_some();

        if living && !supportsWeight(ctx)
        {
            let _ = world.setBlock(pos, 0, 0);
            return;
        }

        let [x, y, z] = info.motion;
        if y > sheet::FALLING
        {
            return;
        }

        entity.setField(EntityField::FallDistance, 0.0);
        let bounce: f64 = match living
        {
            true if entity.flag(EntityField::Jumping) => sheet::JUMP_BOUNCE,
            true if info.player.is_some() && entity.flag(EntityField::Sneaking) => sheet::SNEAK_BOUNCE,
            _ => sheet::BOUNCE,
        };
        let _ = entity.setMotion([x * sheet::SIDEWAYS, y * bounce, z * sheet::SIDEWAYS]);
    }
}

mod resin
{
    pub const THICKNESS: f32 = 2.0;
    pub const DROP_CHANCE: f32 = 0.8;
    pub const FALL: f32 = 0.75;
    pub const SLOW_SIDEWAYS: f64 = 0.6;
    pub const SLOW_DOWN: f64 = 0.85;
}

// BlockResin.canPlaceBlockAt: an opaque solid block below
pub(crate) fn resinHolds(world: World, pos: BlockPos) -> bool
{
    let below: i32 = world.block(pos.offset(0, -1, 0));
    below != 0 && World::blockInfo(below).opaque && World::blockInfo(below).solid
}

// BlockResin: a sticky puddle on an opaque solid block that slows whatever walks through
struct Resin;

#[block_behavior]
impl BlockBehavior for Resin
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { resinHolds(ctx.world(), ctx.pos()) }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { self.canPlaceAt(ctx) }

    fn onEntityCollided(&self, _ctx: &mut BlockCtx, entity: Entity)
    {
        crate::gear::multiplyFall(entity, resin::FALL);
        let Some(info) = entity.info()
        else
        {
            return;
        };
        let [x, y, z] = info.motion;
        let _ = entity.setMotion([x * resin::SLOW_SIDEWAYS, y * resin::SLOW_DOWN, z * resin::SLOW_SIDEWAYS]);
    }
}

mod pole
{
    pub const LIFT: f64 = 0.075;
    pub const BOOST: f64 = 1.03;
    pub const SNEAK_TOP: f64 = 0.3;
    pub const TOP: f64 = 1.5;
    pub const SNEAK_FALL: f64 = -0.25;
    pub const SNEAK_BRAKE: f64 = 0.9;
    pub const NEAR: f32 = 6.0;
    pub const FAR: f32 = 10.0;
}

const METAL_SHOES: [&str; 6] = [
    "minecraft:iron_boots",
    "minecraft:golden_boots",
    "minecraft:chainmail_boots",
    "ic2:bronze_boots",
    "ic2:nano_boots",
    "ic2:quantum_boots",
];

// the boots armor slot of the player's inventory
const BOOTS_SLOT: i32 = 36;

fn isPole(world: World, pos: BlockPos, block: i32) -> bool
{
    [dir::WEST, dir::EAST, dir::NORTH, dir::SOUTH].iter().all(|face| world.block(dir::step(pos, *face)) != block)
}

// BlockPoleFence: a lone iron fence charged by a magnetizer lifts a player in metal boots, sneaking slides down
struct IronFence;

#[block_behavior]
impl BlockBehavior for IronFence
{
    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let Some(info) = entity.info()
        else
        {
            return;
        };

        if info.player.is_none() || !isPole(world, pos, ctx.block())
        {
            return;
        }

        let player: Player = Player { id: info.id };
        let metal: bool =
            player.slot(BOOTS_SLOT).is_some_and(|boots| METAL_SHOES.iter().any(|name| boots.item.is(name)));
        let charge: i32 = i32::from(ctx.meta());
        let sneaking: bool = entity.flag(EntityField::Sneaking);
        let [x, mut y, z] = info.motion;

        if charge > 0 && metal
        {
            ctx.setMeta((charge - 1) as u8);
            y += pole::LIFT;

            if y > 0.0
            {
                y *= pole::BOOST;
                entity.setField(EntityField::FallDistance, 0.0);
            }

            y = y.min(if sneaking { pole::SNEAK_TOP } else { pole::TOP });
        }
        else if sneaking
        {
            if y < pole::SNEAK_FALL
            {
                y *= pole::SNEAK_BRAKE;
            }
            else
            {
                entity.setField(EntityField::FallDistance, 0.0);
            }
        }

        let _ = entity.setMotion([x, y, z]);
    }
}

// BlockMiningPipe and BlockMiningTip only appear under a miner
struct Unplaceable;

#[block_behavior]
impl BlockBehavior for Unplaceable
{
    fn canPlaceAt(&self, _ctx: &mut BlockCtx) -> bool { false }
}

fn poleBox() -> Vec<Cuboid>
{
    vec![Cuboid { from: [pole::NEAR, 0.0, pole::NEAR], to: [pole::FAR, 16.0, pole::FAR], ..Cuboid::default() }]
}

fn slab(height: f32) -> Vec<Cuboid> { vec![Cuboid { from: [0.0; 3], to: [16.0, height, 16.0], ..Cuboid::default() }] }

fn registerRubberTree(ctx: &mut RegisterCtx) -> Result<()>
{
    const WOOD_HARDNESS: f32 = 1.0;
    const LEAVES_HARDNESS: f32 = 0.2;
    // ColorizerFoliage.getFoliageColorBirch
    const BIRCH: u32 = 0x80A755;
    const SAPLING_CHANCE: f32 = 1.0 / 35.0;

    ctx.blocks().register(
        "rubber_wood",
        BlockSpecBuilder::template("minecraft:block")
            .material("wood")
            .sound("wood")
            .hardness(WOOD_HARDNESS)
            .texture(&blockTexture("rubber_wood"))
            .tool(ToolClass::Axe, 0)
            .tickRandomly()
            .sustainsLeaves()
            .tag("minecraft:logs")
            .drops(vec![DropSpec { item: Identifier::parse(RUBBER_WOOD)?, count: vec![1], meta: 0, chance: 1.0 }])
            .behavior(RubberWood),
    )?;

    for sprite in ["rubber_wood_top", "rubber_wood_tapped", "rubber_wood_resin", "rubber_wood"]
    {
        crate::core::sprites::terrain(&blockTexture(sprite));
    }

    ctx.blocks().register(
        "rubber_leaves",
        BlockSpecBuilder::template("minecraft:leaves")
            .texture("minecraft:block/oak_leaves")
            .color(BIRCH)
            .hardness(LEAVES_HARDNESS)
            .opacity(1)
            .sound("grass")
            .drops(vec![drop(RUBBER_SAPLING, SAPLING_CHANCE)?]),
    )?;

    // BlockRubSapling keeps the vanilla sapling picture
    ctx.blocks().register(
        "rubber_sapling",
        BlockSpecBuilder::template("minecraft:red_flower")
            .texture("minecraft:block/oak_sapling")
            .hardness(0.0)
            .sound("grass")
            .tickRandomly()
            .behavior(RubberSapling),
    )?;
    Ok(())
}

fn registerSpecial(ctx: &mut RegisterCtx) -> Result<()>
{
    const RESIN_HARDNESS: f32 = 1.6;
    const RESIN_RESISTANCE: f32 = 0.5;
    const SHEET_HARDNESS: f32 = 0.8;
    const SHEET_RESISTANCE: f32 = 2.0;
    const FENCE_HARDNESS: f32 = 1.5;
    const FENCE_RESISTANCE: f32 = 5.0;
    const PIPE_HARDNESS: f32 = 6.0;
    const PIPE_RESISTANCE: f32 = 10.0;
    const GLASS_HARDNESS: f32 = 1.0;
    const GLASS_RESISTANCE: f32 = 100.0;

    ctx.blocks().register(
        "resin",
        BlockSpecBuilder::template("minecraft:block")
            .material("circuits")
            .sound("cloth")
            .hardness(RESIN_HARDNESS)
            .resistance(RESIN_RESISTANCE)
            .texture(&blockTexture("resin"))
            .opacity(0)
            .model(BlockModelSpec { cuboids: slab(resin::THICKNESS), ..BlockModelSpec::default() })
            .collision(Vec::new())
            .drops(vec![drop(&own("sticky_resin"), resin::DROP_CHANCE)?])
            .behavior(Resin),
    )?;

    ctx.blocks().register(
        "rubber_sheet",
        BlockSpecBuilder::template("minecraft:block")
            .material("cloth")
            .sound("cloth")
            .hardness(SHEET_HARDNESS)
            .resistance(SHEET_RESISTANCE)
            .texture(&blockTexture("rubber_sheet"))
            .opacity(0)
            .model(BlockModelSpec { cuboids: slab(sheet::THICKNESS), ..BlockModelSpec::default() })
            .collision(slab(sheet::THICKNESS))
            .behavior(RubberSheet),
    )?;

    // BlockPoleFence takes the iron furnace top sprite and lets entities into its pole
    ctx.blocks().register(
        "iron_fence",
        BlockSpecBuilder::template("minecraft:fence")
            .material("iron")
            .sound("metal")
            .hardness(FENCE_HARDNESS)
            .resistance(FENCE_RESISTANCE)
            .texture(&blockTexture("iron_furnace_top"))
            .tool(ToolClass::Pickaxe, 0)
            .collision(poleBox())
            .behavior(IronFence),
    )?;

    ctx.blocks().register(
        "mining_pipe",
        BlockSpecBuilder::template("minecraft:fence")
            .material("iron")
            .sound("metal")
            .hardness(PIPE_HARDNESS)
            .resistance(PIPE_RESISTANCE)
            .texture(&blockTexture("mining_pipe"))
            .tool(ToolClass::Pickaxe, 0)
            .collision(poleBox())
            .behavior(Unplaceable),
    )?;

    ctx.blocks().register(
        "mining_pipe_tip",
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("metal")
            .hardness(PIPE_HARDNESS)
            .resistance(PIPE_RESISTANCE)
            .texture(&blockTexture("mining_pipe_tip"))
            .tool(ToolClass::Pickaxe, 0)
            .behavior(Unplaceable),
    )?;

    ctx.blocks().register(
        "reinforced_glass",
        BlockSpecBuilder::template("minecraft:glass")
            .hardness(GLASS_HARDNESS)
            .resistance(GLASS_RESISTANCE)
            .sound("glass")
            .texture(&blockTexture("reinforced_glass"))
            .opacity(0),
    )?;
    Ok(())
}

// the treetap's sound and resin when it opens a hole, shared with nothing else but kept beside the wood rules
pub(crate) fn tapSound(world: World, pos: BlockPos) { world.playSound(dir::center(pos), &sound("treetap"), 1.0, 1.0); }

pub(crate) fn tapChance(world: World) -> bool
{
    const ONE_IN: i32 = 5;

    random::float(world) < 1.0 / f64::from(ONE_IN)
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    registerCubes(ctx)?;
    registerRubberTree(ctx)?;
    registerSpecial(ctx)?;
    registerDoor(ctx)
}

mod door
{
    pub const HARDNESS: f32 = 10.0;
    pub const RESISTANCE: f32 = 100.0;
    pub const TOP: i32 = 8;
    pub const TURNS: f64 = 4.0;
    pub const HALF_TURN: f64 = 180.0;
    pub const FULL_TURN: f64 = 360.0;
    pub const ROUND: f64 = 0.5;
    pub const QUARTERS: i32 = 3;
    pub const HINGE: i32 = 4;
    pub const BUILD_LIMIT: i32 = 127;
}

const DOOR: &str = "ic2:reinforced_door_block";

// BlockIC2Door: the reinforced door's top and bottom sprites, opened only by redstone as iron doors are
struct DoorBlock;

#[block_behavior]
impl BlockBehavior for DoorBlock
{
    fn texture(&self, _face: i32, meta: i32) -> i32
    {
        let half: &str = if meta & door::TOP != 0
        {
            "reinforced_door_top"
        }
        else
        {
            "reinforced_door_bottom"
        };
        crate::core::sprites::terrain(&blockTexture(half))
    }
}

fn doorFits(world: World, pos: BlockPos) -> bool
{
    let below: i32 = world.block(pos.offset(0, -1, 0));
    let solid: bool = below != 0 && World::blockInfo(below).opaque;
    pos.y < door::BUILD_LIMIT && solid && world.isAir(pos) && world.isAir(pos.offset(0, 1, 0))
}

fn normalCube(world: World, pos: BlockPos) -> i32
{
    let block: i32 = world.block(pos);
    i32::from(block != 0 && World::blockInfo(block).opaque)
}

// ItemIC2Door.onItemUse: ItemDoor's placement, hinged toward the wall or the neighbouring door
struct DoorItem;

#[item_behavior]
impl ItemBehavior for DoorItem
{
    fn onUseOnBlock(&self, world: World, player: Player, stack: ItemStack, pos: BlockPos, face: i32) -> Use
    {
        let at: BlockPos = pos.offset(0, 1, 0);
        let (Some(block), Some(info)) = (ids::block(DOOR), player.info())
        else
        {
            return Use::Pass;
        };

        if face != dir::UP || !doorFits(world, at)
        {
            return Use::Pass;
        }

        let turns: f64 = (f64::from(info.yaw) + door::HALF_TURN) * door::TURNS / door::FULL_TURN - door::ROUND;
        let mut quarter: i32 = (turns.floor() as i32) & door::QUARTERS;
        let (dx, dz): (i32, i32) = match quarter
        {
            0 => (0, 1),
            1 => (-1, 0),
            2 => (0, -1),
            _ => (1, 0),
        };
        let left: i32 = normalCube(world, at.offset(-dx, 0, -dz)) + normalCube(world, at.offset(-dx, 1, -dz));
        let right: i32 = normalCube(world, at.offset(dx, 0, dz)) + normalCube(world, at.offset(dx, 1, dz));
        let doorLeft: bool =
            world.block(at.offset(-dx, 0, -dz)) == block || world.block(at.offset(-dx, 1, -dz)) == block;
        let doorRight: bool = world.block(at.offset(dx, 0, dz)) == block || world.block(at.offset(dx, 1, dz)) == block;

        if doorLeft && !doorRight || right > left
        {
            quarter = ((quarter - 1) & door::QUARTERS) + door::HINGE;
        }

        let _ = world.setBlockQuiet(at, block, quarter);
        let _ = world.setBlockQuiet(at.offset(0, 1, 0), block, quarter + door::TOP);
        let _ = world.notifyNeighbors(at, block);
        let _ = world.notifyNeighbors(at.offset(0, 1, 0), block);
        Use::Done(stacks::less(stack, 1))
    }
}

pub(crate) fn registerDoor(ctx: &mut RegisterCtx) -> Result<()>
{
    for half in ["reinforced_door_top", "reinforced_door_bottom"]
    {
        crate::core::sprites::terrain(&blockTexture(half));
    }

    ctx.blocks().register(
        "reinforced_door_block",
        BlockSpecBuilder::template("minecraft:door")
            .material("iron")
            .sound("metal")
            .hardness(door::HARDNESS)
            .resistance(door::RESISTANCE)
            .texture(&blockTexture("reinforced_door_bottom"))
            .tool(ToolClass::Pickaxe, 3)
            .drops(vec![drop(&own("reinforced_door"), 1.0)?])
            .behavior(DoorBlock),
    )?;
    ctx.items().register(
        "reinforced_door",
        ItemSpecBuilder::new().texture(&crate::core::itemTexture("reinforced_door")).behavior(DoorItem),
    )?;
    Ok(())
}
