use super::{put, texture};
use crate::ids::{self, item};
use crate::progress::{self, Gate};
use crate::world::biomes::{self, Biome};
use crate::world::features::{self, Feature};
use b173_api::math::floor;
use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

// ItemTFMagicMap, ItemTFMazeMap and their blank maps: the magic map paints the biome under every chunk and marks
// features, the maze map draws the blocks of one level, the ore map lights up the ores around it
const CHANNEL: &str = "twilightforest:map";
const SIZE: i32 = 128;
const HALF: i32 = SIZE / 2;
const STRIPES: i64 = 16;
const SEND_EVERY: i64 = 10;

#[derive(Clone, Copy, PartialEq)]
enum Kind
{
    Magic,
    Maze,
    Ore,
}

struct Row
{
    m_kind: Kind,
    m_item: &'static str,
    m_label: &'static str,
    m_texture: &'static str,
    m_blank: &'static str,
    m_blankLabel: &'static str,
    m_blankTexture: &'static str,
    // the world's unique id counter and save name prefix, the maze and ore maps share one
    m_data: &'static str,
    // log2 of the blocks per pixel
    m_scale: i32,
    m_gate: Gate,
}

const ROWS: [Row; 3] = [
    Row
    {
        m_kind: Kind::Magic,
        m_item: item::MAGIC_MAP,
        m_label: "Magic Map",
        m_texture: "magicmap",
        m_blank: item::EMPTY_MAGIC_MAP,
        m_blankLabel: "Blank Magic Map",
        m_blankTexture: "emptymagicmap",
        m_data: "magicmap",
        m_scale: 4,
        m_gate: Gate::MagicMap,
    },
    Row
    {
        m_kind: Kind::Maze,
        m_item: item::MAZE_MAP,
        m_label: "Maze Map",
        m_texture: "mazemap",
        m_blank: item::EMPTY_MAZE_MAP,
        m_blankLabel: "Blank Maze Map",
        m_blankTexture: "emptymazemap",
        m_data: "mazemap",
        m_scale: 0,
        m_gate: Gate::MazeMap,
    },
    Row
    {
        m_kind: Kind::Ore,
        m_item: item::ORE_MAP,
        m_label: "Maze/Ore Map",
        m_texture: "oremap",
        m_blank: item::EMPTY_ORE_MAP,
        m_blankLabel: "Blank Maze/Ore Map",
        m_blankTexture: "emptyoremap",
        m_data: "mazemap",
        m_scale: 0,
        m_gate: Gate::OreMap,
    },
];

fn row(kind: Kind) -> &'static Row { &ROWS[kind as usize] }

fn held(stack: &ItemStack) -> Option<&'static Row> { ROWS.iter().find(|row| stack.item.is(row.m_item)) }

mod magic
{
    // ItemTFMagicMap.updateMapData draws a circle 512 blocks across
    pub const REACH: i32 = 32;
    // rivers and streams count three times so thin water shows
    pub const WATER_WEIGHT: i32 = 3;
    pub const ICON_GRID: u8 = 8;
    pub const ICONS: &str = "twilightforest:gui/mapicons";
}

mod maze
{
    pub const REACH: i32 = 16;
    // the map only draws while its holder stands within three blocks of its level
    pub const LEVELS: i32 = 3;
    // the blank maze map centers off the 128 block grid by ten
    pub const GRID_SHIFT: i32 = 10;
}

mod shade
{
    // MapItemRenderer shades a color by its two low bits, deeper below, flat, raised above
    pub const BELOW: u8 = 0;
    pub const FLAT: u8 = 1;
    pub const ABOVE: u8 = 2;
    pub const LEVELS: u8 = 4;
    pub const DARK: u32 = 180;
    pub const PLAIN: u32 = 220;
    pub const BRIGHT: u32 = 255;
    // circles dither their rim over the last two pixels
    pub const RIM: i32 = 2;
}

mod argb
{
    pub const OPAQUE: u32 = 0xFF000000;
    pub const WHITE: u32 = 0xFFFFFF;
    pub const CHANNEL: u32 = 0xFF;
    pub const RED: u32 = 16;
    pub const GREEN: u32 = 8;
    pub const ALPHA: u32 = 24;
    // MapItemRenderer's blank checkerboard alpha steps
    pub const CHECKER_BASE: u32 = 16;
    pub const CHECKER_STEP: u32 = 8;
}

mod marker
{
    pub const RANGE: f32 = 63.0;
    // beta's map pointer, about four pixels to its tip, three back and three wide either side
    pub const ARROW_TIP: f32 = 4.0;
    pub const ARROW_TAIL: f32 = 3.0;
    pub const ARROW_HALF_WIDTH: f32 = 3.0;
    pub const TURN_STEPS: f32 = 16.0;
    pub const FULL_TURN: f32 = 360.0;
    // the java cast rounds the sixteenth of a turn half up
    pub const ROUNDING: f32 = 0.5;
}

mod panel
{
    pub const BACKGROUND: &str = "/misc/mapbg.png";
    pub const MARGIN: i32 = 4;
    // the map background's frame around the picture, as ItemRenderer draws it
    pub const BORDER: i32 = 7;
    pub const ICON: i32 = 8;
    pub const ICON_SHEET: f32 = 64.0;
}

const BIOME_SLOTS: usize = u8::MAX as usize + 1;

// the 1.7 MapColor rgb ItemTFMazeMap gives each ore on an ore map
struct OreColor
{
    m_block: &'static str,
    m_rgb: u32,
}

const ORE_COLORS: [OreColor; 7] = [
    OreColor { m_block: "minecraft:coal_ore", m_rgb: 0x15141F },
    OreColor { m_block: "minecraft:gold_ore", m_rgb: 0xFAEE4D },
    OreColor { m_block: "minecraft:iron_ore", m_rgb: 0xA7A7A7 },
    OreColor { m_block: "minecraft:lapis_ore", m_rgb: 0x4A80FF },
    OreColor { m_block: "minecraft:redstone_ore", m_rgb: 0x993333 },
    OreColor { m_block: "minecraft:lit_redstone_ore", m_rgb: 0x993333 },
    OreColor { m_block: "minecraft:diamond_ore", m_rgb: 0x5CDBD5 },
];

// TFMagicMapData and TFMazeMapData, a pixel is a palette slot times four plus its shade, slot 0 is blank
#[derive(Serialize, Deserialize, Default, Clone)]
struct Chart
{
    m_center: [i32; 2],
    m_level: i32,
    m_pixels: Vec<u8>,
    m_palette: Vec<u32>,
    // feature id with its half pixel offsets from the center
    m_features: Vec<[i8; 3]>,
}

impl Chart
{
    fn new(center: [i32; 2], level: i32) -> Self
    {
        Self
        {
            m_center: center,
            m_level: level,
            m_pixels: vec![0; (SIZE * SIZE) as usize],
            m_palette: vec![0],
            m_features: Vec::new(),
        }
    }

    fn slot(&mut self, rgb: u32) -> u8
    {
        if let Some(slot) = self.m_palette.iter().position(|color| *color == rgb)
        {
            return slot as u8;
        }
        let full: bool = self.m_palette.len() >= usize::from(u8::MAX / shade::LEVELS);
        if full
        {
            return 0;
        }

        self.m_palette.push(rgb);
        (self.m_palette.len() - 1) as u8
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct Sync
{
    m_data: String,
    m_id: i32,
    m_version: u32,
    m_chart: Chart,
}

struct Open
{
    m_chart: Chart,
    m_version: u32,
    m_saved: u32,
    m_sent: HashMap<i32, u32>,
}

thread_local!
{
    static SERVER: RefCell<HashMap<String, Open>> = RefCell::new(HashMap::new());
    static CLIENT: RefCell<HashMap<String, (u32, Chart)>> = RefCell::new(HashMap::new());
}

fn key(data: &str, id: i32) -> String { format!("{data}_{id}") }

fn withOpen<T>(data: &str, id: i32, run: impl FnOnce(&mut Open) -> T) -> T
{
    SERVER.with(|server|
    {
        let mut server: RefMut<HashMap<String, Open>> = server.borrow_mut();
        let open: &mut Open = server.entry(key(data, id)).or_insert_with(|| Open
        {
            m_chart: worldData(&key(data, id)),
            m_version: 1,
            m_saved: 1,
            m_sent: HashMap::new(),
        });
        run(open)
    })
}

// World.getUniqueDataId
fn nextId(data: &str) -> i32
{
    let counter: String = format!("{data}_count");
    let id: i32 = worldData::<i32>(&counter);
    setWorldData(&counter, &(id + 1));
    id
}

fn store(data: &str, id: i32, chart: Chart)
{
    setWorldData(&key(data, id), &chart);

    withOpen(data, id, |open|
    {
        open.m_chart = chart;
        open.m_version += 1;
        open.m_saved = open.m_version;
    });
}

fn round(value: f64, step: i32) -> i32 { (value / f64::from(step)).round() as i32 * step }

// ItemTFEmptyMagicMap and ItemTFEmptyMazeMap.onItemRightClick, only inside the twilight forest
fn chart(world: World, player: Player, kind: Kind) -> Option<i32>
{
    let info: EntityInfo = player.info()?;
    let row: &Row = row(kind);
    let step: i32 = SIZE << row.m_scale;
    let [x, y, z]: [i32; 3] = info.pos.map(floor);

    let center: [i32; 2] = match kind
    {
        Kind::Magic => [round(info.pos[0], step), round(info.pos[2], step)],

        Kind::Maze | Kind::Ore =>
        {
            let (feature, [centerX, centerZ]): (Feature, [i32; 2]) = features::nearest(world.seed(), x >> 4, z >> 4);
            if feature == Feature::Labyrinth
            {
                [centerX, centerZ]
            }
            else
            {
                [round(info.pos[0], step) + maze::GRID_SHIFT, round(info.pos[2], step) + maze::GRID_SHIFT]
            }
        }
    };

    let id: i32 = nextId(row.m_data);
    store(row.m_data, id, Chart::new(center, y));
    Some(id)
}

struct Blank
{
    m_kind: Kind,
}

#[item_behavior]
impl ItemBehavior for Blank
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        if !world.isAuthority() || !crate::world::isTwilight(world)
        {
            return Use::Done(Some(stack));
        }
        let Some(id) = chart(world, player, self.m_kind)
        else
        {
            return Use::Done(Some(stack));
        };

        let row: &Row = row(self.m_kind);
        progress::grant(player, row.m_gate);

        let map: ItemStack = ItemStack { damage: id, ..ItemStack::of(row.m_item, 1) };

        if stack.count <= 1
        {
            return Use::Done(Some(map));
        }

        if let Err(error) = player.give(map)
        {
            warn(&format!("cannot hand out map {id}: {}", error.0));
        }

        Use::Done(Some(ItemStack { count: stack.count - 1, ..stack }))
    }
}

struct Filled;

#[item_behavior]
impl ItemBehavior for Filled {}

// the biome a magic map pixel shows, most common across its chunk with water weighted up
fn commonBiome(seed: i64, chunkX: i32, chunkZ: i32) -> u8
{
    let mut counts: [i32; BIOME_SLOTS] = [0; BIOME_SLOTS];

    for column in biomes::chunk(seed, chunkX, chunkZ)
    {
        let water: bool = biomes::fromIndex(i32::from(column)) == Biome::Stream;
        counts[usize::from(column)] += if water { magic::WATER_WEIGHT } else { 1 };
    }

    let mut best: (u8, i32) = (0, 0);

    for (index, count) in counts.iter().enumerate()
    {
        if *count > best.1
        {
            best = (index as u8, *count);
        }
    }

    best.0
}

// TFMagicMapData.addFeatureToMap
fn addFeature(chart: &mut Chart, feature: Feature, x: i32, z: i32, scale: i32)
{
    let relative: [i32; 2] = [(x - chart.m_center[0]) >> scale, (z - chart.m_center[1]) >> scale];
    let inside: bool = relative.iter().all(|axis| (-HALF..=HALF).contains(axis));
    if feature == Feature::Nothing || !inside
    {
        return;
    }
    let (mapX, mapZ): (i8, i8) = ((relative[0] << 1) as i8, (relative[1] << 1) as i8);
    if chart.m_features.iter().any(|known| known[1] == mapX && known[2] == mapZ)
    {
        return;
    }

    chart.m_features.push([feature as i8, mapX, mapZ]);
}

// whether a pixel of the circle a map draws around its holder takes a new color, the rim dithers
fn inCircle(offset: [i32; 2], reach: i32, step: [i32; 2]) -> bool
{
    let distance: i32 = offset[0] * offset[0] + offset[1] * offset[1];
    let rim: bool = distance > (reach - shade::RIM) * (reach - shade::RIM);
    step[1] >= 0 && distance < reach * reach && (!rim || (step[0] + step[1]) & 1 != 0)
}

fn magicPixel(world: World, chart: &mut Chart, step: [i32; 2], offset: [i32; 2]) -> bool
{
    let row: &Row = row(Kind::Magic);
    let index: usize = (step[0] + step[1] * SIZE) as usize;
    if chart.m_pixels[index] != 0 || !inCircle(offset, magic::REACH, step)
    {
        return false;
    }

    let chunkX: i32 = (chart.m_center[0] >> row.m_scale) + step[0] - HALF;
    let chunkZ: i32 = (chart.m_center[1] >> row.m_scale) + step[1] - HALF;
    if features::featureChunk(chunkX, chunkZ)
    {
        let (feature, _): (Feature, [i32; 2]) = features::nearest(world.seed(), chunkX, chunkZ);
        addFeature(chart, feature, chunkX << row.m_scale, chunkZ << row.m_scale, row.m_scale);
    }

    let biome: u8 = commonBiome(world.seed(), chunkX, chunkZ);
    let rgb: u32 = biomes::rowOf(biomes::fromIndex(i32::from(biome))).m_map;
    chart.m_pixels[index] = chart.slot(rgb) * shade::LEVELS + shade::FLAT;
    true
}

// the block an ore map shows under stone, searching three up and down, with its shade
fn underStone(world: World, x: i32, level: i32, z: i32) -> (i32, u8)
{
    let stone: i32 = ids::blockId("minecraft:stone");

    for dy in -maze::LEVELS..=maze::LEVELS
    {
        let block: i32 = world.block(BlockPos::new(x, level + dy, z));
        if block != stone
        {
            let tint: u8 = match dy.signum()
            {
                1 => shade::ABOVE,
                -1 => shade::BELOW,
                _ => shade::FLAT,
            };
            return (block, tint);
        }
    }

    (stone, shade::FLAT)
}

fn mazePixel(world: World, chart: &mut Chart, step: [i32; 2], offset: [i32; 2], ores: bool) -> bool
{
    if !inCircle(offset, maze::REACH, step)
    {
        return false;
    }

    let x: i32 = chart.m_center[0] + step[0] - HALF;
    let z: i32 = chart.m_center[1] + step[1] - HALF;
    let mut block: i32 = world.block(BlockPos::new(x, chart.m_level, z));
    let mut tint: u8 = shade::FLAT;

    if ores && block == ids::blockId("minecraft:stone")
    {
        (block, tint) = underStone(world, x, chart.m_level, z);
    }

    let ore: Option<u32> =
        ORE_COLORS.iter().find(|ore| ores && ids::blockId(ore.m_block) == block).map(|ore| ore.m_rgb);
    let rgb: u32 = ore.unwrap_or_else(|| World::mapColor(block));
    let slot: u8 = if rgb == 0 { 0 } else { chart.slot(rgb) };
    let pixel: u8 = slot * shade::LEVELS + tint;
    let index: usize = (step[0] + step[1] * SIZE) as usize;
    if chart.m_pixels[index] == pixel
    {
        return false;
    }

    chart.m_pixels[index] = pixel;
    true
}

// ItemTFMagicMap and ItemTFMazeMap.updateMapData, one stripe of columns a tick around the holder
fn update(world: World, info: &EntityInfo, row: &Row, chart: &mut Chart, now: i64) -> bool
{
    let reach: i32 = if row.m_kind == Kind::Magic
    {
        magic::REACH
    }
    else
    {
        maze::REACH
    };
    let height: i32 = floor(info.pos[1]) - chart.m_level;
    if row.m_kind != Kind::Magic && !(-maze::LEVELS + 1..maze::LEVELS).contains(&height)
    {
        return false;
    }

    let draw: [i32; 2] = [
        ((floor(info.pos[0]) - chart.m_center[0]) >> row.m_scale) + HALF,
        ((floor(info.pos[2]) - chart.m_center[1]) >> row.m_scale) + HALF,
    ];

    let stripe: i64 = now % STRIPES;
    let mut changed: bool = false;

    for x in (draw[0] - reach + 1).max(0)..(draw[0] + reach).min(SIZE)
    {
        if i64::from(x) % STRIPES != stripe
        {
            continue;
        }

        for z in (draw[1] - reach - 1).max(0)..(draw[1] + reach).min(SIZE)
        {
            let offset: [i32; 2] = [x - draw[0], z - draw[1]];

            changed |= match row.m_kind
            {
                Kind::Magic => magicPixel(world, chart, [x, z], offset),
                Kind::Maze => mazePixel(world, chart, [x, z], offset, false),
                Kind::Ore => mazePixel(world, chart, [x, z], offset, true),
            };
        }
    }

    changed
}

fn sync(player: Player, row: &Row, id: i32, open: &mut Open)
{
    if open.m_sent.get(&player.id) == Some(&open.m_version)
    {
        return;
    }

    let message: Sync =
        Sync { m_data: row.m_data.to_owned(), m_id: id, m_version: open.m_version, m_chart: open.m_chart.clone() };

    if let Err(error) = send(CHANNEL, Target::Player(player.id), &message)
    {
        warn(&format!("cannot send map {id}: {}", error.0));
        return;
    }

    open.m_sent.insert(player.id, open.m_version);

    if open.m_saved != open.m_version
    {
        setWorldData(&key(row.m_data, id), &open.m_chart);
        open.m_saved = open.m_version;
    }
}

fn tickPlayers()
{
    let Some(twilight) = crate::world::dimension()
    else
    {
        return;
    };
    let world: World = World { dimension: twilight };
    if !world.isAuthority()
    {
        return;
    }

    let now: i64 = world.time();

    for player in world.players()
    {
        let (Some(stack), Some(info)) = (player.held(), player.info())
        else
        {
            continue;
        };
        let Some(row) = held(&stack)
        else
        {
            continue;
        };

        withOpen(row.m_data, stack.damage, |open|
        {
            if open.m_chart.m_pixels.is_empty()
            {
                return;
            }

            if update(world, &info, row, &mut open.m_chart, now)
            {
                open.m_version += 1;
            }

            if now % SEND_EVERY == 0
            {
                sync(player, row, stack.damage, open);
            }
        });
    }
}

fn shaded(rgb: u32, pixel: u8) -> u32
{
    let level: u32 = match pixel % shade::LEVELS
    {
        shade::BELOW => shade::DARK,
        shade::ABOVE => shade::BRIGHT,
        _ => shade::PLAIN,
    };
    let channel = |shift: u32| (((rgb >> shift) & argb::CHANNEL) * level / argb::CHANNEL) << shift;
    argb::OPAQUE | channel(argb::RED) | channel(argb::GREEN) | channel(0)
}

// MapItemRenderer leaves blank pixels as a faint checkerboard
fn blank(index: usize) -> u32
{
    let (n, row): (u32, u32) = (index as u32, index as u32 / SIZE as u32);
    let dark: u32 = (n + row) & 1;
    (dark * argb::CHECKER_STEP + argb::CHECKER_BASE) << argb::ALPHA
}

fn pixels(chart: &Chart) -> Vec<u32>
{
    chart
        .m_pixels
        .iter()
        .enumerate()
        .map(|(index, pixel)|
        {
            let slot: usize = usize::from(pixel / shade::LEVELS);
            match chart.m_palette.get(slot).filter(|_| slot > 0)
            {
                Some(rgb) => shaded(*rgb, *pixel),
                None => blank(index),
            }
        })
        .collect()
}

// MapData.updateVisiblePlayers for the local player: its map pixel and sixteenth of a turn, None off the map or, on
// a maze map, off the map's level where the source shows a dot beta's icon sheet lacks
fn marker(chart: &Chart, row: &Row, info: &EntityInfo) -> Option<[i32; 3]>
{
    let scale: f32 = (1 << row.m_scale) as f32;
    let x: f32 = (info.pos[0] as f32 - chart.m_center[0] as f32) / scale;
    let z: f32 = (info.pos[2] as f32 - chart.m_center[1] as f32) / scale;
    let height: i32 = floor(info.pos[1]) - chart.m_level;
    let away: bool = row.m_kind != Kind::Magic && !(-maze::LEVELS..=maze::LEVELS).contains(&height);
    if x.abs() > marker::RANGE || z.abs() > marker::RANGE || away
    {
        return None;
    }

    let turn: i32 = (info.yaw * marker::TURN_STEPS / marker::FULL_TURN + marker::ROUNDING) as i32;
    Some([x.floor() as i32 + HALF, z.floor() as i32 + HALF, turn.rem_euclid(marker::TURN_STEPS as i32)])
}

// the white pointer of beta's map icons drawn into the picture, it points where the player faces
fn stamp(pixels: &mut [u32], [x, z, turn]: [i32; 3])
{
    let angle: f32 = (turn as f32 * marker::FULL_TURN / marker::TURN_STEPS).to_radians();
    let ahead: [f32; 2] = [-angle.sin(), angle.cos()];
    let side: [f32; 2] = [angle.cos(), angle.sin()];
    let reach: i32 = marker::ARROW_TIP as i32;
    for dz in -reach..=reach
    {
        for dx in -reach..=reach
        {
            // the pixel in the arrow's own frame, along its length and across it
            let along: f32 = dx as f32 * ahead[0] + dz as f32 * ahead[1];
            let across: f32 = (dx as f32 * side[0] + dz as f32 * side[1]).abs();
            let width: f32 =
                (marker::ARROW_TIP - along) / (marker::ARROW_TIP + marker::ARROW_TAIL) * marker::ARROW_HALF_WIDTH;
            let inside: bool = (-marker::ARROW_TAIL..=marker::ARROW_TIP).contains(&along) && across <= width;
            let (px, pz): (i32, i32) = (x + dx, z + dz);
            if inside && (0..SIZE).contains(&px) && (0..SIZE).contains(&pz)
            {
                pixels[(px + pz * SIZE) as usize] = argb::OPAQUE | argb::WHITE;
            }
        }
    }
}

// what the client last sent of a held map, its chart version and the marker drawn in
struct Shown
{
    m_version: u32,
    m_marker: Option<[i32; 3]>,
}

thread_local!
{
    static SHOWN: RefCell<HashMap<String, Shown>> = RefCell::new(HashMap::new());
}

fn textureName(key: &str) -> String { format!("twilightforest:map_{key}") }

// a fresh picture only when the chart or the marker moved since the last one
fn refresh(name: &str, version: u32, chart: &Chart, at: Option<[i32; 3]>)
{
    let stale: bool = SHOWN
        .with(|shown| shown.borrow().get(name).is_none_or(|last| last.m_version != version || last.m_marker != at));

    if !stale
    {
        return;
    }

    let mut picture: Vec<u32> = pixels(chart);

    if let Some(at) = at
    {
        stamp(&mut picture, at);
    }

    if let Err(error) = setDynamicTexture(name, SIZE as u32, SIZE as u32, picture)
    {
        warn(&format!("map picture: {}", error.0));
        return;
    }

    SHOWN.with(|shown| shown.borrow_mut().insert(name.to_owned(), Shown { m_version: version, m_marker: at }));
}

// beta draws only its own map in the hand, so a held twilight map shows as a panel in the corner of the screen
fn hud(draw: &mut Draw, _partial: f32, width: i32, _height: i32)
{
    let Some(player) = localPlayer()
    else
    {
        return;
    };
    let (Some(stack), Some(info)) = (player.held(), player.info())
    else
    {
        return;
    };
    let Some(row) = held(&stack)
    else
    {
        return;
    };
    let mapKey: String = key(row.m_data, stack.damage);
    let Some((version, chart)) = CLIENT.with(|client| client.borrow().get(&mapKey).cloned())
    else
    {
        return;
    };

    let name: String = textureName(&mapKey);
    refresh(&name, version, &chart, marker(&chart, row, &info));

    let left: i32 = width - SIZE - panel::MARGIN - panel::BORDER;
    let top: i32 = panel::MARGIN + panel::BORDER;
    let framed: i32 = SIZE + panel::BORDER * 2;
    draw.image(panel::BACKGROUND, left - panel::BORDER, top - panel::BORDER, framed, framed);
    draw.image(&name, left, top, SIZE, SIZE);

    let grid: f32 = f32::from(magic::ICON_GRID);
    let cell: f32 = panel::ICON_SHEET / grid;

    for [feature, x, z] in &chart.m_features
    {
        let index: f32 = f32::from(*feature as u8);
        let uv: [f32; 4] = [(index % grid).floor() * cell, (index / grid).floor() * cell, cell, cell];
        let at: [i32; 2] = [left + i32::from(*x) / 2 + HALF, top + i32::from(*z) / 2 + HALF];
        let rect: [i32; 4] = [at[0] - panel::ICON / 2, at[1] - panel::ICON / 2, panel::ICON, panel::ICON];
        draw.region(magic::ICONS, rect, uv, [panel::ICON_SHEET, panel::ICON_SHEET]);
    }
}

fn received(sync: Sync, _player: Player)
{
    CLIENT.with(|client| client.borrow_mut().insert(key(&sync.m_data, sync.m_id), (sync.m_version, sync.m_chart)));
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &ROWS
    {
        put(
            ctx,
            row.m_blank,
            ItemSpecBuilder::new()
                .name(row.m_blankLabel)
                .texture(&texture(row.m_blankTexture))
                .behavior(Blank { m_kind: row.m_kind }),
        )?;
        put(
            ctx,
            row.m_item,
            ItemSpecBuilder::new().name(row.m_label).texture(&texture(row.m_texture)).maxStack(1).behavior(Filled),
        )?;
    }

    ctx.channels().register::<Sync>(CHANNEL, Direction::ToClient, received)?;
    ctx.screens().hud(hud)?;
    ctx.events().on::<TickPost>(Priority::Normal, |_| tickPlayers())?;
    Ok(())
}
