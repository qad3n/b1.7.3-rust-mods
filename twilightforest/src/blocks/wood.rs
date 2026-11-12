use super::{Cube, Kind, Look, base, cube, drop, faces, texture, vanillaTexture};
use crate::colors::{self, Tint};
use crate::names::{self, block, item};
use crate::tags;
use crate::world::trees::{self, Tree};
use b173_api::prelude::*;

mod fire
{
    // TFBlocks fire info, encouragement then flammability
    pub const LOG: (i32, i32) = (5, 5);
    pub const LEAVES: (i32, i32) = (30, 60);
}

mod leaves
{
    pub const HARDNESS: f32 = 0.2;
    // fancy oak leaves
    pub const TEXTURE: &str = "oak_leaves";
    pub const SAPLING_CHANCE: f32 = 1.0 / 40.0;
    pub const MANGROVE_SAPLING_CHANCE: f32 = 1.0 / 20.0;
    pub const MAGIC_SAPLING_CHANCE: f32 = 1.0 / 150.0;
    pub const MAGIC_TREASURE_CHANCE: f32 = 1.0 / 200.0;
}

const LOG_HARDNESS: f32 = 2.0;
// BlockLog orientation bits as a state, metadata 0 stands upright, 1 and 2 lie along x and z, 3 is all bark
pub(crate) const LOG_AXES: [&str; 4] = ["y", "x", "z", "none"];
const PLANK_HARDNESS: f32 = 2.0;
// a root block splits into three or four sticks
const ROOT_STICKS: [i32; 2] = [3, 4];
const DARK_LEAVES_HARDNESS: f32 = 2.0;
const DARK_LEAVES_RESISTANCE: f32 = 10.0;
const CORE_LIGHT: f32 = 1.0;
// a magic core runs its effect every second
const CORE_INTERVAL: i32 = 20;

struct Wood
{
    m_log: &'static str,
    m_label: &'static str,
    m_top: &'static str,
    m_side: &'static str,
}

const LOGS: [Wood; 8] = [
    Wood { m_log: block::OAK_LOG, m_label: "Twilight Oak Wood", m_top: "oak_top", m_side: "oak_side" },
    Wood { m_log: block::CANOPY_LOG, m_label: "Canopy Tree Wood", m_top: "canopy_top", m_side: "canopy_side" },
    Wood { m_log: block::MANGROVE_LOG, m_label: "Mangrove Wood", m_top: "mangrove_top", m_side: "mangrove_side" },
    Wood { m_log: block::DARKWOOD_LOG, m_label: "Darkwood Wood", m_top: "darkwood_top", m_side: "darkwood_side" },
    Wood { m_log: block::TIMEWOOD, m_label: "Timewood", m_top: "time_section", m_side: "time_side" },
    Wood { m_log: block::TRANSWOOD, m_label: "Transwood", m_top: "trans_section", m_side: "trans_side" },
    Wood { m_log: block::MINEWOOD, m_label: "Minewood", m_top: "mine_section", m_side: "mine_side" },
    Wood { m_log: block::SORTINGWOOD, m_label: "Sortingwood", m_top: "sort_section", m_side: "sort_side" },
];

const PLANKS: [Cube; 8] = [
    cube("twilight_oak_planks", "Twilight Oak Planks", Kind::Wood, Look::All("twilight_oak_planks"), PLANK_HARDNESS),
    cube("canopy_planks", "Canopy Planks", Kind::Wood, Look::All("canopy_planks"), PLANK_HARDNESS),
    cube("mangrove_planks", "Mangrove Planks", Kind::Wood, Look::All("mangrove_planks"), PLANK_HARDNESS),
    cube("darkwood_planks", "Darkwood Planks", Kind::Wood, Look::All("darkwood_planks"), PLANK_HARDNESS),
    cube("timewood_planks", "Timewood Planks", Kind::Wood, Look::All("timewood_planks"), PLANK_HARDNESS),
    cube("transwood_planks", "Transwood Planks", Kind::Wood, Look::All("transwood_planks"), PLANK_HARDNESS),
    cube("minewood_planks", "Minewood Planks", Kind::Wood, Look::All("minewood_planks"), PLANK_HARDNESS),
    cube("sortingwood_planks", "Sortingwood Planks", Kind::Wood, Look::All("sortingwood_planks"), PLANK_HARDNESS),
];

struct Leaf
{
    m_name: &'static str,
    m_label: &'static str,
    m_tint: Tint,
    m_texture: Option<&'static str>,
    m_sapling: &'static str,
    m_chance: f32,
}

const LEAVES: [Leaf; 8] = [
    Leaf
    {
        m_name: block::OAK_LEAVES,
        m_label: "Twilight Oak Leaves",
        m_tint: Tint::Foliage,
        m_texture: None,
        m_sapling: block::OAK_SAPLING,
        m_chance: leaves::SAPLING_CHANCE,
    },
    Leaf
    {
        m_name: block::CANOPY_LEAVES,
        m_label: "Canopy Tree Leaves",
        m_tint: Tint::Canopy,
        m_texture: None,
        m_sapling: "canopy_sapling",
        m_chance: leaves::SAPLING_CHANCE,
    },
    Leaf
    {
        m_name: block::MANGROVE_LEAVES,
        m_label: "Mangrove Leaves",
        m_tint: Tint::Mangrove,
        m_texture: None,
        m_sapling: "mangrove_sapling",
        m_chance: leaves::MANGROVE_SAPLING_CHANCE,
    },
    Leaf
    {
        m_name: block::RAINBOW_LEAVES,
        m_label: "Rainbow Oak Leaves",
        m_tint: Tint::Rainbow,
        m_texture: None,
        m_sapling: "rainbow_oak_sapling",
        m_chance: leaves::SAPLING_CHANCE,
    },
    Leaf
    {
        m_name: block::TIME_LEAVES,
        m_label: "Timewood Leaves",
        m_tint: Tint::Time,
        m_texture: Some("time_leaves"),
        m_sapling: "time_sapling",
        m_chance: leaves::MAGIC_SAPLING_CHANCE,
    },
    Leaf
    {
        m_name: block::TRANS_LEAVES,
        m_label: "Leaves of Transformation",
        m_tint: Tint::Transformation,
        m_texture: Some("trans_leaves"),
        m_sapling: "transformation_sapling",
        m_chance: leaves::MAGIC_SAPLING_CHANCE,
    },
    Leaf
    {
        m_name: block::MINE_LEAVES,
        m_label: "Miner's Leaves",
        // the source draws miner's leaves with the time texture
        m_tint: Tint::Mining,
        m_texture: Some("time_leaves"),
        m_sapling: "mining_sapling",
        m_chance: leaves::MAGIC_SAPLING_CHANCE,
    },
    Leaf
    {
        m_name: block::SORT_LEAVES,
        m_label: "Sorting Leaves",
        m_tint: Tint::Sorting,
        m_texture: Some("sort_leaves"),
        m_sapling: "sorting_sapling",
        m_chance: leaves::MAGIC_SAPLING_CHANCE,
    },
];

struct Sapling
{
    m_name: &'static str,
    m_label: &'static str,
    m_texture: &'static str,
    m_tree: Tree,
}

const SAPLINGS: [Sapling; 10] = [
    Sapling
    {
        m_name: block::OAK_SAPLING,
        m_label: "Sickly Twilight Oak Sapling",
        m_texture: "sapling_oak",
        m_tree: Tree::SmallOak,
    },
    Sapling
    {
        m_name: "canopy_sapling",
        m_label: "Canopy Tree Sapling",
        m_texture: "sapling_canopy",
        m_tree: Tree::Canopy,
    },
    Sapling
    {
        m_name: "mangrove_sapling",
        m_label: "Twilight Mangrove Sapling",
        m_texture: "sapling_mangrove",
        m_tree: Tree::Mangrove,
    },
    Sapling
    {
        m_name: "darkwood_sapling",
        m_label: "Darkwood Sapling",
        m_texture: "sapling_darkwood",
        m_tree: Tree::DarkCanopy,
    },
    Sapling
    {
        m_name: "hollow_oak_sapling",
        m_label: "Robust Twilight Oak Sapling",
        m_texture: "sapling_hollow_oak",
        m_tree: Tree::Hollow,
    },
    Sapling { m_name: "time_sapling", m_label: "Tree of Time Sapling", m_texture: "sapling_time", m_tree: Tree::Time },
    Sapling
    {
        m_name: "transformation_sapling",
        m_label: "Tree of Transformation Sapling",
        m_texture: "sapling_transformation",
        m_tree: Tree::Transformation,
    },
    Sapling
    {
        m_name: "mining_sapling",
        m_label: "Miner's Tree Sapling",
        m_texture: "sapling_mining",
        m_tree: Tree::Miners,
    },
    Sapling
    {
        m_name: "sorting_sapling",
        m_label: "Sorting Tree Sapling",
        m_texture: "sapling_sorting",
        m_tree: Tree::Sorting,
    },
    Sapling
    {
        m_name: "rainbow_oak_sapling",
        m_label: "Rainbow Oak Sapling",
        m_texture: "sapling_rainboak",
        m_tree: Tree::Rainbow,
    },
];

// the four magic cores, BlockTFMagicLogSpecial, each with the face it shows lit and dark
struct Core
{
    m_name: &'static str,
    m_label: &'static str,
    m_top: &'static str,
    m_on: &'static str,
    m_off: &'static str,
    m_effect: Effect,
}

#[derive(Clone, Copy)]
pub(crate) enum Effect
{
    Time,
    Transformation,
    Mining,
    Sorting,
}

const CORES: [Core; 4] = [
    Core
    {
        m_name: block::TIME_CORE,
        m_label: "Timewood Clock",
        m_top: "time_section",
        m_on: "time_clock",
        m_off: "time_clock_off",
        m_effect: Effect::Time,
    },
    Core
    {
        m_name: block::TRANS_CORE,
        m_label: "Heart of Transformation",
        m_top: "trans_section",
        m_on: "trans_heart",
        m_off: "trans_heart_off",
        m_effect: Effect::Transformation,
    },
    Core
    {
        m_name: block::MINE_CORE,
        m_label: "Minewood Core",
        m_top: "mine_section",
        m_on: "mine_gem",
        m_off: "mine_gem_off",
        m_effect: Effect::Mining,
    },
    Core
    {
        m_name: block::SORT_CORE,
        m_label: "Sortingwood Engine",
        m_top: "sort_section",
        m_on: "sort_eye",
        m_off: "sort_eye_off",
        m_effect: Effect::Sorting,
    },
];

// the source rolls one of five faces and never the top
const RUNE_FACES: [[i32; 3]; 5] = [[0, -1, 0], [0, 0, 1], [0, 0, -1], [1, 0, 0], [-1, 0, 0]];

struct LeafTint
{
    m_tint: Tint,
    m_runes: bool,
}

#[block_behavior]
impl BlockBehavior for LeafTint
{
    // BlockTFMagicLeaves.sparkleRunes, transformation leaves shed a rune past one random open face
    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        if !self.m_runes
        {
            return;
        }

        let world: World = ctx.world();
        let step: [i32; 3] = RUNE_FACES[world.random(RUNE_FACES.len() as i32) as usize];
        let pos: BlockPos = ctx.pos();
        if !world.isAir(pos.offset(step[0], step[1], step[2]))
        {
            return;
        }

        crate::particles::spawn(world, "leafrune", crate::particles::outside(world, pos, step), [0.0; 3]);
    }

    fn colorMultiplier(&self, ctx: &mut BlockCtx) -> u32 { colors::tint(self.m_tint, ctx.world(), ctx.pos()) }
}

// BlockTFHedge: thorny to anything but spiders and items, hurts a player punching it
struct Hedge;

const HEDGE_DAMAGE: i32 = 3;

fn hurtsOnTouch(entity: Entity) -> bool
{
    let Some(info) = entity.info()
    else
    {
        return false;
    };

    let kind: String = info.kind.map(|kind| kind.to_string()).unwrap_or_default();
    !matches!(kind.as_str(), "minecraft:spider" | "minecraft:item") && !kind.ends_with("spider")
}

#[block_behavior]
impl BlockBehavior for Hedge
{
    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        if ctx.isAuthority() && hurtsOnTouch(entity)
        {
            crate::combat::hit(entity, HEDGE_DAMAGE, None, crate::combat::kind::CACTUS);
        }
    }

    fn onEntityWalking(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        if ctx.isAuthority() && hurtsOnTouch(entity)
        {
            crate::combat::hit(entity, HEDGE_DAMAGE, None, crate::combat::kind::CACTUS);
        }
    }

    fn onHarvest(&self, ctx: &mut BlockCtx, player: Player)
    {
        crate::combat::hit(
            Entity { world: ctx.world(), id: player.id },
            HEDGE_DAMAGE,
            None,
            crate::combat::kind::CACTUS,
        );
    }

    fn onClicked(&self, ctx: &mut BlockCtx, player: Player)
    {
        if ctx.isAuthority()
        {
            crate::combat::hit(
                Entity { world: ctx.world(), id: player.id },
                HEDGE_DAMAGE,
                None,
                crate::combat::kind::CACTUS,
            );
        }
    }
}

struct SaplingGrowth
{
    m_tree: Tree,
}

// BlockTFSapling: a sapling under light 9 or more grows one time in seven
const SAPLING_LIGHT: i32 = 9;
const SAPLING_ODDS: i32 = 7;

#[block_behavior]
impl BlockBehavior for SaplingGrowth
{
    fn canStay(&self, ctx: &mut BlockCtx) -> bool { trees::soil(ctx.world().block(ctx.pos().offset(0, -1, 0))) }

    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool
    {
        ctx.world().isAir(ctx.pos()) && trees::soil(ctx.world().block(ctx.pos().offset(0, -1, 0)))
    }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        if !ctx.isAuthority()
            || world.light(ctx.pos().offset(0, 1, 0)) < SAPLING_LIGHT
            || world.random(SAPLING_ODDS) != 0
        {
            return;
        }

        trees::growSapling(world, ctx.pos(), self.m_tree);
    }
}

pub(crate) fn isSapling(block: i32) -> bool { SAPLINGS.iter().any(|row| blockIs(block, row.m_name)) }

// TFEventListener.bonemealUsed, bone meal grows a twilight sapling at once
pub(crate) fn fertilize(world: World, pos: BlockPos) -> bool
{
    let here: i32 = world.block(pos);
    let Some(row) = SAPLINGS.iter().find(|row| blockIs(here, row.m_name))
    else
    {
        return false;
    };

    if world.isAuthority()
    {
        trees::growSapling(world, pos, row.m_tree);
    }

    true
}

const BONE_MEAL: i32 = 15;

fn boneMeal(event: &mut Event)
{
    let (Some(stack), Some(player), Some(world)) = (event.stack(), event.player(), event.world())
    else
    {
        return;
    };
    let EventData::ItemUse { pos: Some(pos), .. } = event.data().clone()
    else
    {
        return;
    };
    if !stack.item.is("minecraft:dye") || stack.damage != BONE_MEAL || !fertilize(world, pos)
    {
        return;
    }

    event.cancel();

    if world.isAuthority()
    {
        let left: Option<ItemStack> = (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack });
        let _ = player.setHeld(left);
    }
}

struct MagicCore
{
    m_effect: Effect,
}

#[block_behavior]
impl BlockBehavior for MagicCore
{
    fn onAdded(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority()
        {
            let _ = ctx.world().scheduleTick(ctx.pos(), ctx.block(), CORE_INTERVAL);
        }
    }

    fn onActivated(&self, ctx: &mut BlockCtx, _player: Player) -> bool
    {
        let Some(state) = ctx.stateAt()
        else
        {
            return true;
        };

        let stopped: bool = state.flag("stopped");
        if let Err(error) = ctx.setState(StateData::clone(&state).with("stopped", !stopped))
        {
            warn(&format!("magic core at {:?}: {}", ctx.pos(), error.0));
        }

        if stopped && ctx.isAuthority()
        {
            let _ = ctx.world().scheduleTick(ctx.pos(), ctx.block(), CORE_INTERVAL);
        }

        true
    }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        if names::flag(ctx.stateAt(), "stopped")
        {
            return;
        }

        if ctx.isAuthority()
        {
            crate::magic::coreEffect(ctx.world(), ctx.pos(), self.m_effect);
        }

        let _ = ctx.world().scheduleTick(ctx.pos(), ctx.block(), CORE_INTERVAL);
    }
}

// the forest's leaves name BlockLeaves' metadata bits, check decay then persistent then the kind in the low bits,
// so the engine's decay ticks read the same metadata the states store
const LEAF_KINDS: [&str; 4] = ["0", "1", "2", "3"];

pub(crate) fn leafStates(spec: BlockSpecBuilder) -> BlockSpecBuilder
{
    spec.property(Property::bool("check_decay"))
        .property(Property::bool("persistent"))
        .property(Property::values("kind", &LEAF_KINDS))
}

fn registerLogs(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &LOGS
    {
        let spec: BlockSpecBuilder = base("minecraft:block", row.m_label, Kind::Wood, LOG_HARDNESS)
            .faces(faces(&Look::Column { top: row.m_top, side: row.m_side }))
            .flammable(fire::LOG.0, fire::LOG.1)
            .sustainsLeaves();
        super::put(ctx, row.m_log, spec.property(Property::values("axis", &LOG_AXES)))?;
        ctx.tags().add(RegistryKind::Item, tags::LOGS, &[row.m_log])?;
    }

    for row in &CORES
    {
        let spec: BlockSpecBuilder = base("minecraft:block", row.m_label, Kind::Wood, LOG_HARDNESS)
            .faces(faces(&Look::Column { top: row.m_top, side: row.m_on }))
            .light(CORE_LIGHT)
            .flammable(fire::LOG.0, fire::LOG.1)
            .sustainsLeaves()
            .property(Property::bool("stopped"))
            .variant(
                Variant::when(&[("stopped", "true")]).faces(faces(&Look::Column { top: row.m_top, side: row.m_off })),
            )
            .behavior(MagicCore { m_effect: row.m_effect });
        super::put(ctx, row.m_name, spec)?;
    }

    super::put(
        ctx,
        block::ROOT,
        base("minecraft:block", "Roots", Kind::Wood, LOG_HARDNESS).texture(&texture("rootblock")).drops(vec![drop(
            "minecraft:stick",
            ROOT_STICKS.to_vec(),
            1.0,
        )?]),
    )?;
    super::put(
        ctx,
        block::LIVEROOT,
        base("minecraft:block", "Liveroots", Kind::Wood, LOG_HARDNESS).texture(&texture("oreroots")).drops(vec![drop(
            item::LIVEROOT,
            vec![1],
            1.0,
        )?]),
    )?;
    super::registerCubes(ctx, &PLANKS)?;

    for row in &PLANKS
    {
        tags::both(ctx, "minecraft:planks", row.m_name)?;
    }

    Ok(())
}

fn leafSpec(row: &Leaf) -> Result<BlockSpecBuilder>
{
    let look: String = row.m_texture.map_or_else(|| vanillaTexture(leaves::TEXTURE), texture);
    Ok(BlockSpecBuilder::template("minecraft:leaves")
        .name(row.m_label)
        .texture(&look)
        .hardness(leaves::HARDNESS)
        .sound("grass")
        .flammable(fire::LEAVES.0, fire::LEAVES.1)
        .drops(vec![drop(row.m_sapling, vec![1], row.m_chance)?])
        .behavior(LeafTint { m_tint: row.m_tint, m_runes: row.m_name == block::TRANS_LEAVES }))
}

// BlockTFMagicLeaves.func_150124_c, each magic tree hides a different treasure in its leaves
fn magicTreasure(name: &str) -> Result<Vec<DropSpec>>
{
    let chance: f32 = leaves::MAGIC_TREASURE_CHANCE;

    Ok(match name
    {
        block::TIME_LEAVES => vec![drop("minecraft:clock", vec![1], chance)?],
        block::TRANS_LEAVES => vec![drop(block::FIREFLY, vec![2], chance)?],

        block::MINE_LEAVES =>
        {
            let ores: [&str; 6] = [
                "minecraft:iron_ore",
                "minecraft:gold_ore",
                "minecraft:diamond_ore",
                "minecraft:lapis_ore",
                "minecraft:coal_ore",
                "minecraft:redstone_ore",
            ];

            let each: f32 = chance / ores.len() as f32;
            ores.iter().map(|ore| drop(ore, vec![1], each)).collect::<Result<Vec<DropSpec>>>()?
        }

        block::SORT_LEAVES => vec![drop("minecraft:chest", vec![1], chance)?],
        _ => Vec::new(),
    })
}

fn registerLeaves(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &LEAVES
    {
        let mut spec: BlockSpecBuilder = leafSpec(row)?;
        let treasure: Vec<DropSpec> = magicTreasure(row.m_name)?;
        if !treasure.is_empty()
        {
            let mut all: Vec<DropSpec> = vec![drop(row.m_sapling, vec![1], row.m_chance)?];
            all.extend(treasure);
            spec = spec.drops(all);
        }

        super::put(ctx, row.m_name, leafStates(spec))?;
        ctx.tags().add(RegistryKind::Item, "minecraft:leaves", &[row.m_name])?;
    }

    super::put(
        ctx,
        block::DARK_LEAVES,
        BlockSpecBuilder::template("minecraft:block")
            .name("Darkwood Leaves")
            .material("leaves")
            .texture(&texture("darkwood_leaves2"))
            .hardness(DARK_LEAVES_HARDNESS)
            .resistance(DARK_LEAVES_RESISTANCE)
            .sound("grass")
            .flammable(0, 1)
            .drops(vec![drop("darkwood_sapling", vec![1], leaves::SAPLING_CHANCE)?])
            .behavior(LeafTint { m_tint: Tint::Foliage, m_runes: false })
            .tag(tags::LEAVES),
    )?;
    ctx.tags().add(RegistryKind::Item, "minecraft:leaves", &[block::DARK_LEAVES])?;

    let sliver: f32 = 15.0;
    super::put(
        ctx,
        block::HEDGE,
        BlockSpecBuilder::template("minecraft:block")
            .name("Hedge")
            .material("cloth")
            .texture(&texture("hedge"))
            .hardness(DARK_LEAVES_HARDNESS)
            .resistance(DARK_LEAVES_RESISTANCE)
            .sound("grass")
            .drops(Vec::new())
            .collision(vec![Cuboid { from: [0.0; 3], to: [16.0, sliver, 16.0], ..Cuboid::default() }])
            .behavior(Hedge),
    )
}

fn registerSaplings(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &SAPLINGS
    {
        super::put(
            ctx,
            row.m_name,
            BlockSpecBuilder::template("minecraft:red_flower")
                .name(row.m_label)
                .texture(&texture(row.m_texture))
                .hardness(0.0)
                .sound("grass")
                .tickRandomly()
                .behavior(SaplingGrowth { m_tree: row.m_tree }),
        )?;
        tags::both(ctx, "minecraft:saplings", row.m_name)?;
    }

    Ok(())
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<ItemUseOnBlock>(Priority::Normal, boneMeal)?;
    registerLogs(ctx)?;
    registerLeaves(ctx)?;
    registerSaplings(ctx)
}
