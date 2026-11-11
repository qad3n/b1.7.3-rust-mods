mod bronze;
mod golden;
mod loot;
mod silver;
mod writer;

use crate::world::GenWorld;
use b173_api::prelude::*;
use b173_api::screen::Draw;
use b173_api::terrain::Random;
use b173_api::terrain::structure::{self, StructureBox};

// every Aether structure component starts as this box two blocks into its chunk: lowest y, highest y, width
const START_BOX: [i32; 3] = [80, 220, 100];
const START_OFFSET: i32 = 2;
// components are lifted by up to this many blocks
const START_LIFT: i32 = 64;
// MapGenBase.range
const RANGE: i32 = 8;

mod bronze_spot
{
    // ChunkProviderAether.populate places the bronze dungeon this high
    pub const LOWEST: i32 = 24;
    pub const SPREAD: i32 = 48;
}

// MapGenSilverDungeon.Start and MapGenGoldenDungeon.Start reseed from the world seed and their chunk
fn startRandom(seed: i64, chunkX: i32, chunkZ: i32) -> Random
{
    let random: Random = Random::withSeed(seed);
    let xScale: i64 = random.nextLong();
    let zScale: i64 = random.nextLong();
    random.setSeed((chunkX as i64).wrapping_mul(xScale) ^ (chunkZ as i64).wrapping_mul(zScale) ^ seed);
    random
}

// the Java lifts each structure up to 63 blocks in a 256 high world, here the same draw spreads it over the
// heights that keep its whole height under the b1.7.3 ceiling
fn fitLift(component: &mut StructureBox, lift: i32, lowest: i32, highest: i32)
{
    let height: i32 = lowest + lift * (highest - lowest) / (START_LIFT - 1);
    component.offset([0, height - component.m_min[1], 0]);
}

// the dungeons as structure kinds of the loader's registry, the silver and golden ones locatable from the seed
struct KindRow
{
    m_name: &'static str,
    m_label: &'static str,
    m_finder: Option<fn(&Random, i32, i32) -> bool>,
}

const BRONZE: usize = 0;
const SILVER: usize = 1;
const GOLDEN: usize = 2;

const KINDS: [KindRow; 3] = [
    KindRow { m_name: "bronze_dungeon", m_label: "Bronze Dungeon", m_finder: None },
    KindRow { m_name: "silver_dungeon", m_label: "Silver Dungeon", m_finder: Some(silver::canSpawn) },
    KindRow { m_name: "golden_dungeon", m_label: "Golden Dungeon", m_finder: Some(golden::canSpawn) },
];

fn registerKinds(ctx: &mut RegisterCtx) -> Result<()>
{
    let structures: Structures = ctx.structures();

    for row in &KINDS
    {
        match row.m_finder
        {
            Some(finder) =>
            {
                structures.withFinder(row.m_name, row.m_label, super::DIMENSION, move |seed, chunkX, chunkZ|
                {
                    finder(&structure::candidate(seed, chunkX, chunkZ), chunkX, chunkZ)
                })?
            }

            None => structures.register(row.m_name, row.m_label, super::DIMENSION)?,
        };
    }

    Ok(())
}

// hands a built dungeon to the loader's structure registry, which keeps one record per origin
fn record(world: &GenWorld, kind: usize, origin: BlockPos, rooms: &[StructureBox])
{
    let name: &str = KINDS[kind].m_name;
    let Ok(id) = Identifier::parse(&format!("aether:{name}"))
    else
    {
        warn(&format!("structure name {name} is not an identifier"));
        return;
    };

    let pieces: Vec<StructurePiece> = rooms.iter().map(|room| room.piece("")).collect();
    if let Err(error) = world.m_world.addStructure(structure::start(&id, origin, pieces))
    {
        warn(&format!("could not record {id}: {}", error.0));
    }
}

// a start's origin is the middle of its chunk at the bottom of its box
fn startOrigin(chunkX: i32, chunkZ: i32, component: &StructureBox) -> BlockPos
{
    const MIDDLE: i32 = 8;
    BlockPos::new((chunkX << 4) + MIDDLE, component.m_min[1], (chunkZ << 4) + MIDDLE)
}

// MapGenStructure.generateStructuresInChunk for the silver and golden dungeons, before the biome decorates
pub(crate) fn structures(world: &GenWorld, random: &Random, chunk: &ChunkInfo)
{
    let clip: StructureBox = StructureBox::populated(chunk.x, chunk.z);
    let mut silvers: Vec<(i32, i32, silver::Start)> = Vec::new();

    structure::candidates(chunk.seed, chunk.x, chunk.z, RANGE, |x, z, candidate|
    {
        if silver::canSpawn(candidate, x, z)
        {
            silvers.push((x, z, silver::start(chunk.seed, x, z)));
        }
    });

    for (x, z, start) in silvers.iter().filter(|(_, _, start)| start.m_box.intersects(&clip))
    {
        record(world, SILVER, startOrigin(*x, *z, &start.m_box), &[start.m_box]);
        silver::generate(world, random, start, clip);
    }

    let mut goldens: Vec<(i32, i32, golden::Start)> = Vec::new();

    structure::candidates(chunk.seed, chunk.x, chunk.z, RANGE, |x, z, candidate|
    {
        if golden::canSpawn(candidate, x, z)
        {
            goldens.push((x, z, golden::start(chunk.seed, x, z)));
        }
    });

    for (x, z, start) in goldens.iter().filter(|(_, _, start)| start.m_box.intersects(&clip))
    {
        record(world, GOLDEN, startOrigin(*x, *z, &start.m_box), &[start.m_box]);
        golden::generate(world, random, start, clip);
    }
}

// the bronze dungeon tries once per chunk after the decorations
pub(crate) fn bronze(world: &GenWorld, random: &Random, x: i32, z: i32)
{
    let y: i32 = random.nextIntBounded(bronze_spot::SPREAD) + bronze_spot::LOWEST;
    let rooms: Vec<StructureBox> = bronze::generate(world, random, x, y, z);
    if !rooms.is_empty()
    {
        record(world, BRONZE, BlockPos::new(x, y, z), &rooms);
    }
}

// BlockDungeonBase, the unbreakable copies a dungeon is built from until its boss falls, and BlockDungeonTrap
struct Stone
{
    m_name: &'static str,
    m_texture: &'static str,
    m_light: bool,
    m_trapSpawns: Option<(&'static str, [f64; 3])>,
}

const fn stone(name: &'static str, texture: &'static str, light: bool) -> Stone
{
    Stone { m_name: name, m_texture: texture, m_light: light, m_trapSpawns: None }
}

// a sentry appears two blocks off its trap, the others on top of it
const BESIDE: [f64; 3] = [2.0, 1.0, 2.0];
const ON_TOP: [f64; 3] = [0.5, 1.0, 0.5];

const STONES: [Stone; 9] = [
    stone("locked_carved_stone", "carved_stone", false),
    stone("locked_sentry_stone", "sentry_stone", true),
    stone("locked_angelic_stone", "angelic_stone", false),
    stone("locked_light_angelic_stone", "light_angelic_stone", true),
    stone("locked_hellfire_stone", "hellfire_stone", false),
    stone("locked_light_hellfire_stone", "light_hellfire_stone", true),
    Stone { m_name: "carved_trap", m_texture: "carved_stone", m_light: false, m_trapSpawns: Some(("sentry", BESIDE)) },
    Stone
    {
        m_name: "angelic_trap",
        m_texture: "angelic_stone",
        m_light: false,
        m_trapSpawns: Some(("valkyrie", ON_TOP)),
    },
    Stone
    {
        m_name: "hellfire_trap",
        m_texture: "hellfire_stone",
        m_light: false,
        m_trapSpawns: Some(("fire_minion", ON_TOP)),
    },
];

mod block
{
    pub const LOCKED_RESISTANCE: f32 = 6000000.0;
    pub const UNBREAKABLE: f32 = -1.0;
    // BlockDungeonBase lit variants give light 11
    pub const LIGHT: f32 = 11.0 / 15.0;
    pub const MIMIC_HARDNESS: f32 = 2.0;
    pub const TREASURE_RESISTANCE: f32 = 2000.0;
    pub const OPEN_TREASURE_HARDNESS: f32 = 5.0;
    pub const OPEN_TREASURE_RESISTANCE: f32 = 10.0;
    pub const CHEST_SLOTS: i32 = 27;
    pub const TREASURE_ROLLS: i32 = 5;
    pub const SPRING: f32 = 0.2;
    pub const DOOR_PITCH: f32 = 1.2;
    pub const DOOR_VOLUME: f32 = 2.0;
    pub const MIMIC_LIFT: f64 = 1.5;
    pub const CHEST_VOLUME: f32 = 0.5;
    pub const CHEST_PITCH: f32 = 0.9;
    pub const CHEST_WOBBLE: f32 = 0.1;
    // BlockChest sprites on terrain.png
    pub const CHEST_TOP: &str = "minecraft:block/chest_top";
    pub const CHEST_SIDE: &str = "minecraft:block/chest_side";
    pub const CHEST_FRONT: &str = "minecraft:block/chest_front";
}

// the plain stone a locked dungeon block turns into once its boss is beaten
pub(crate) fn unlocked(name: &str) -> &str { name.trim_start_matches("locked_") }

// BlockDungeonTrap.onEntityWalking: a player springs it, it turns to plain stone and calls its guard
struct Trap
{
    m_stone: &'static str,
    m_guard: &'static str,
    m_offset: [f64; 3],
}

#[block_behavior]
impl BlockBehavior for Trap
{
    fn onEntityWalking(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        if !ctx.isAuthority() || entity.info().is_none_or(|info| info.player.is_none())
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let _ = world.place(pos, self.m_stone);
        let at: [f64; 3] =
            [pos.x as f64 + self.m_offset[0], pos.y as f64 + self.m_offset[1], pos.z as f64 + self.m_offset[2]];
        let _ = world.spawnEntity(&format!("aether:{}", self.m_guard), at);
        let pitch: f32 = crate::entities::fraction(world) as f32
            - crate::entities::fraction(world) as f32 * block::SPRING
            + block::DOOR_PITCH;
        world.playSound([pos.x as f64, pos.y as f64, pos.z as f64], "random.door_close", block::DOOR_VOLUME, pitch);
    }
}

// BlockMimicChest: opening or breaking it lets a mimic out at whoever touched it
struct MimicChest;

fn releaseMimic(ctx: &mut BlockCtx, player: Player)
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    if let Ok(mimic) =
        world.spawnEntity("aether:mimic", [pos.x as f64 + 0.5, pos.y as f64 + block::MIMIC_LIFT, pos.z as f64 + 0.5])
    {
        mimic.setField(EntityField::TargetId, player.id as f64);
    }

    let _ = world.clear(pos);
}

#[block_behavior]
impl BlockBehavior for MimicChest
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            let pos: BlockPos = ctx.pos();
            let pitch: f32 = crate::entities::fraction(ctx.world()) as f32 * block::CHEST_WOBBLE + block::CHEST_PITCH;
            ctx.world().playSound(
                [pos.x as f64 + 0.5, pos.y as f64 + 0.5, pos.z as f64 + 0.5],
                "random.chestopen",
                block::CHEST_VOLUME,
                pitch,
            );
            releaseMimic(ctx, player);
        }

        true
    }

    fn onHarvest(&self, ctx: &mut BlockCtx, player: Player)
    {
        if ctx.isAuthority()
        {
            releaseMimic(ctx, player);
        }
    }
}

// ItemDungeonKey types and the TileEntityTreasureChest loot each one unlocks
struct KeyRow
{
    m_key: &'static str,
    m_title: &'static str,
    m_loot: &'static str,
}

const KEYS: [KeyRow; 3] = [
    KeyRow { m_key: "bronze_key", m_title: "aether.gui.treasure_chest.bronze", m_loot: loot::BRONZE_TREASURE_TABLE },
    KeyRow { m_key: "silver_key", m_title: "aether.gui.treasure_chest.silver", m_loot: loot::SILVER_TREASURE_TABLE },
    KeyRow { m_key: "golden_key", m_title: "aether.gui.treasure_chest.gold", m_loot: loot::GOLD_TREASURE_TABLE },
];

const TREASURE: &str = "treasure_chest";
const OPEN_TREASURE: &str = "unlocked_treasure_chest";

// BlockTreasureChest: a dungeon key unlocks it, swapping in the breakable open chest filled with its key's loot
struct TreasureChest;

#[block_behavior]
impl BlockBehavior for TreasureChest
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        let Some((kind, held)) =
            player.held().and_then(|held| KEYS.iter().position(|row| held.item.is(row.m_key)).map(|kind| (kind, held)))
        else
        {
            return false;
        };
        if !ctx.isAuthority()
        {
            return true;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let _ = world.place(pos, OPEN_TREASURE);
        let _ = world.setTileData(pos, &(kind as i32));
        let random: Random = Random::withSeed(world.random(i32::MAX) as i64);
        for _ in 0..block::TREASURE_ROLLS + random.nextIntBounded(1)
        {
            let slot: i32 = random.nextIntBounded(block::CHEST_SLOTS);
            if let Some(stack) = loot::pick(&random, KEYS[kind].m_loot)
            {
                let _ = world.setTileSlot(pos, slot, Some(stack));
            }
        }

        let _ = player.setHeld((held.count > 1).then(|| ItemStack { count: held.count - 1, ..held }));

        true
    }
}

struct OpenTreasureChest;

#[block_behavior]
impl BlockBehavior for OpenTreasureChest
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            let _ = player.openScreen(TREASURE, Some(ctx.pos()), Vec::new());
        }

        true
    }
}

mod gui
{
    // GuiTreasureChest on the vanilla chest sheet, three rows
    pub const SHEET: &str = "minecraft:gui/container";
    pub const WIDTH: i32 = 176;
    pub const ROWS: i32 = 3;
    pub const SLOT: i32 = 18;
    pub const TOP: i32 = 17;
    pub const BOTTOM: i32 = 96;
    pub const BOTTOM_V: i32 = 126;
    pub const LEFT: i32 = 8;
    pub const TITLE_Y: i32 = 6;
    pub const TEXT: u32 = 4210752;
    pub const INVENTORY_Y: i32 = 85;
}

struct TreasureScreen
{
    m_pos: Option<BlockPos>,
}

impl Screen for TreasureScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        let top: i32 = gui::ROWS * gui::SLOT + gui::TOP;
        draw.texture(gui::SHEET, 0, 0, 0, 0, gui::WIDTH, top);
        draw.texture(gui::SHEET, 0, top, 0, gui::BOTTOM_V, gui::WIDTH, gui::BOTTOM);

        let kind: i32 = match (self.m_pos, localPlayer().and_then(Player::info))
        {
            (Some(pos), Some(local)) => World { dimension: local.world }.tileData(pos),
            _ => 0,
        };
        let title: &str = KEYS.get(kind as usize).map_or(KEYS[0].m_title, |row| row.m_title);
        draw.text(gui::LEFT, gui::TITLE_Y, &translate(title), gui::TEXT);
    }
}

fn faces(top: String, side: String, front: String) -> FaceTextures
{
    FaceTextures { top: Some(top.clone()), bottom: Some(top), side: Some(side), front: Some(front) }
}

fn texture(name: &str) -> String { format!("aether:block/{name}") }

fn registerChests(ctx: &mut RegisterCtx) -> Result<()>
{
    let vanilla = |name: &str| name.to_owned();
    ctx.blocks().register(
        "chest_mimic",
        BlockSpecBuilder::template("minecraft:block")
            .material("wood")
            .faces(faces(vanilla(block::CHEST_TOP), vanilla(block::CHEST_SIDE), vanilla(block::CHEST_FRONT)))
            .hardness(block::MIMIC_HARDNESS)
            .sound("wood")
            .tool(ToolClass::Axe, 0)
            .drops(Vec::new())
            .behavior(MimicChest),
    )?;

    let treasureFaces =
        || faces(texture("treasure_chest_top"), texture("treasure_chest_side"), texture("treasure_chest_front"));
    ctx.blocks().register(
        TREASURE,
        BlockSpecBuilder::template("minecraft:block")
            .material("rock")
            .faces(treasureFaces())
            .hardness(block::UNBREAKABLE)
            .resistance(block::TREASURE_RESISTANCE)
            .sound("stone")
            .drops(Vec::new())
            .behavior(TreasureChest),
    )?;
    ctx.tileEntities().register(
        OPEN_TREASURE,
        TileEntitySpec { inventory: block::CHEST_SLOTS, ticks: false, sync: true, behavior: None },
        None,
    )?;
    ctx.blocks().register(
        OPEN_TREASURE,
        BlockSpecBuilder::template("minecraft:block")
            .material("rock")
            .faces(treasureFaces())
            .hardness(block::OPEN_TREASURE_HARDNESS)
            .resistance(block::OPEN_TREASURE_RESISTANCE)
            .sound("stone")
            .drops(Vec::new())
            .tileEntity(OPEN_TREASURE)
            .behavior(OpenTreasureChest),
    )?;

    let mut screen: ScreenBuilder =
        ScreenBuilder::new("", gui::WIDTH, gui::ROWS * gui::SLOT + gui::TOP + gui::BOTTOM + 1);

    for slot in 0..block::CHEST_SLOTS
    {
        screen = screen.slot(slot, gui::LEFT + slot % 9 * gui::SLOT, gui::SLOT + slot / 9 * gui::SLOT);
    }

    ctx.screens().register(
        TREASURE,
        screen.playerInventory(gui::LEFT, gui::INVENTORY_Y),
        TreasureScreen { m_pos: None },
    )
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    loot::register(ctx)?;
    registerKinds(ctx)?;

    for row in &STONES
    {
        let mut spec: BlockSpecBuilder = BlockSpecBuilder::template("minecraft:block")
            .material("rock")
            .texture(&texture(row.m_texture))
            .hardness(block::UNBREAKABLE)
            .resistance(block::LOCKED_RESISTANCE)
            .sound("stone");

        if row.m_light
        {
            spec = spec.light(block::LIGHT);
        }

        if let Some((guard, offset)) = row.m_trapSpawns
        {
            spec = spec.behavior(Trap { m_stone: row.m_texture, m_guard: guard, m_offset: offset });
        }

        ctx.blocks().register(row.m_name, spec)?;
    }

    registerChests(ctx)
}
