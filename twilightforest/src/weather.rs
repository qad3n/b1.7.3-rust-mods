use crate::progress::{self, Gate};
use crate::world::biomes::{self, Biome};
use b173_api::math::floor;
use b173_api::prelude::*;
use b173_api::terrain::Random;
use b173_api::terrain::structure::StructureBox;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

// TFWeatherRenderer: in a biome the player has not earned, blizzards, mosquito clouds, ash, dark streams or heavy
// rain fill the air around them, and a guarded piece they were turned away from glitters inside its box while
// EntityTFProtectionBox outlines it
const PROGRESS_CHANNEL: &str = "twilightforest:progress";
const BOX_CHANNEL: &str = "twilightforest:protected_box";

mod tuning
{
    pub const RANGE: i32 = 10;
    pub const NEAR: i32 = 15;
    pub const SYNC_EVERY: i64 = 20;
    // EntityTFProtectionBox lives sixty ticks and fades over its last twenty
    pub const BOX_LIFE: i64 = 60;
    pub const BOX_FADE: f32 = 20.0;
    pub const BOX_SCROLL: f32 = 0.05;
    pub const BOX_INFLATE: f32 = 1.0 / 16.0;
    pub const STRUCTURE_MARGIN: i32 = 4;
    pub const FADE_BASE: f32 = 0.5;
    pub const FADE_SPAN: f32 = 0.3;
}

mod texture
{
    pub const BLIZZARD: &str = "twilightforest:environment/blizzard";
    pub const MOSQUITOES: &str = "twilightforest:environment/mosquitoes";
    pub const ASHES: &str = "twilightforest:environment/ashes";
    pub const DARK_STREAM: &str = "twilightforest:environment/darkstream";
    pub const BIG_RAIN: &str = "twilightforest:environment/bigrain";
    pub const SPARKLES: &str = "twilightforest:environment/sparkles";
    pub const BOX: &str = "twilightforest:model/protectionbox";
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Earned
{
    m_gates: u64,
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Guarded
{
    m_min: [i32; 3],
    m_max: [i32; 3],
}

thread_local!
{
    static SENT: RefCell<HashMap<i32, u64>> = RefCell::new(HashMap::new());
    static GATES: Cell<u64> = const { Cell::new(0) };
    static BOX: Cell<Option<(Guarded, i64)>> = const { Cell::new(None) };
}

// the server tells each player in the forest what they have earned, the client draws the locks from it
fn syncProgress()
{
    let Some(twilight) = crate::world::dimension()
    else
    {
        return;
    };
    let world: World = World { dimension: twilight };
    if !world.isAuthority() || world.time() % tuning::SYNC_EVERY != 0
    {
        return;
    }

    for player in world.players()
    {
        let gates: u64 = progress::earned(player);
        if SENT.with(|sent| sent.borrow().get(&player.id) == Some(&gates))
        {
            continue;
        }

        if let Err(error) = send(PROGRESS_CHANNEL, Target::Player(player.id), &Earned { m_gates: gates })
        {
            warn(&format!("progress sync: {}", error.0));
            continue;
        }

        SENT.with(|sent| sent.borrow_mut().insert(player.id, gates));
    }
}

// TFGenericPacketHandler.makeAreaProtectionPacket, the refused player learns the guarded box
pub(crate) fn showBox(player: Player, bounds: StructureBox)
{
    let message: Guarded = Guarded { m_min: bounds.m_min, m_max: bounds.m_max };
    if let Err(error) = send(BOX_CHANNEL, Target::Player(player.id), &message)
    {
        warn(&format!("protection box: {}", error.0));
    }
}

fn locked(biome: Biome) -> bool
{
    progress::required(biome).is_some_and(|gate: Gate| GATES.get() & 1 << gate as u32 == 0)
}

// a column's draw, its texture, colour and the quad's texture scroll
struct Column
{
    m_texture: &'static str,
    m_color: [f32; 4],
    m_scroll: [f32; 3],
    m_span: [i32; 2],
}

fn gaussian(random: &Random) -> f32 { random.nextGaussian() as f32 }

// the per column effect of renderLockedBiome, None where nothing falls this frame
fn effect(world: World, biome: Biome, random: &Random, at: [i32; 2], span: [i32; 2], frame: [f32; 3])
-> Option<Column>
{
    let [ticks, partial, fade] = frame;
    let precise: f32 = ticks + partial;
    let whole: i32 = ticks as i32;
    let alpha: f32 = fade * tuning::FADE_SPAN + tuning::FADE_BASE;

    let column = |texture: &'static str, color: [f32; 4], scroll: [f32; 3], span: [i32; 2]|
    {
        Some(Column { m_texture: texture, m_color: color, m_scroll: scroll, m_span: span })
    };

    match biome
    {
        Biome::Swamp =>
        {
            let u: f32 = random.nextFloat() + precise * 0.03 * gaussian(random);
            let v: f32 = random.nextFloat() + precise * 0.003 * gaussian(random);
            let tint: [f32; 3] = [0, 1, 2].map(|_| random.nextFloat() * 0.3);
            column(texture::MOSQUITOES, [tint[0], tint[1], tint[2], 1.0], [0.0, u, v], span)
        }

        Biome::FireSwamp =>
        {
            let count: f32 = -((whole & 1023) as f32 + partial) / 1024.0;
            let u: f32 = random.nextFloat() + precise * 0.001 * gaussian(random);
            let v: f32 = random.nextFloat() + precise * 0.001 * gaussian(random);
            let bright: f32 = random.nextFloat() * 0.2 + 0.8;
            column(texture::ASHES, [bright, bright, bright, alpha], [count, u, v], span)
        }

        Biome::DarkForest | Biome::DarkForestCenter =>
        {
            if random.nextIntBounded(2) != 0
            {
                return None;
            }

            let top: i32 = span[1].min(world.height(at[0], at[1]));
            let count: f32 = -((whole & 511) as f32 + partial) / 512.0;
            let v: f32 = random.nextFloat() + precise * 0.01 * gaussian(random);
            let shade: f32 = random.nextFloat();
            column(texture::DARK_STREAM, [1.0, 1.0, 1.0, alpha * shade], [count, 0.0, v], [span[0].min(top), top])
        }

        Biome::Highlands | Biome::Thornlands | Biome::FinalPlateau =>
        {
            let [x, z] = at;
            let mix: i32 =
                whole.wrapping_add(x.wrapping_mul(x).wrapping_mul(3121)).wrapping_add(x.wrapping_mul(45238971));
            let mix: i32 = mix.wrapping_add(z.wrapping_mul(z).wrapping_mul(418711)).wrapping_add(z.wrapping_mul(13761));
            let count: f32 = ((mix & 31) as f32 + partial) / 32.0 * (3.0 + random.nextFloat());
            let (u, v): (f32, f32) = (random.nextFloat(), random.nextFloat());
            column(texture::BIG_RAIN, [1.0, 1.0, 1.0, alpha], [count, u, v], span)
        }

        _ =>
        {
            let count: f32 = ((whole & 511) as f32 + partial) / 512.0;
            let u: f32 = random.nextFloat() + precise * 0.03 * gaussian(random);
            let v: f32 = random.nextFloat() + precise * 0.001 * gaussian(random);
            column(texture::BLIZZARD, [1.0, 1.0, 1.0, alpha], [count, u, v], span)
        }
    }
}

// the per column seed every TF and vanilla weather pass uses
fn columnRandom(x: i32, z: i32) -> Random
{
    let first: i32 = x.wrapping_mul(x).wrapping_mul(3121).wrapping_add(x.wrapping_mul(45238971));
    let second: i32 = z.wrapping_mul(z).wrapping_mul(418711).wrapping_add(z.wrapping_mul(13761));
    Random::withSeed(i64::from(first ^ second))
}

// initializeRainCoords, each column's sheet faces the player, half a block either side
fn sheet(at: [i32; 2], player: [i32; 2]) -> Option<[f32; 2]>
{
    let (dx, dz): (f32, f32) = ((at[0] - player[0]) as f32, (at[1] - player[1]) as f32);
    let length: f32 = (dx * dx + dz * dz).sqrt();
    (length > 0.0).then(|| [-dz / length * 0.5, dx / length * 0.5])
}

fn quad(ops: &mut Vec<ModelOp>, at: [i32; 2], sides: [f32; 2], camera: [f64; 3], column: &Column)
{
    let [count, u, v] = column.m_scroll;
    let [low, high] = column.m_span;

    let corner = |side: f32, y: i32, edge: f32|
    {
        let x: f32 = (f64::from(at[0]) + 0.5 - camera[0]) as f32 + side * sides[0];
        let z: f32 = (f64::from(at[1]) + 0.5 - camera[2]) as f32 + side * sides[1];
        [x, (f64::from(y) - camera[1]) as f32, z, edge + u, y as f32 / 4.0 + count + v]
    };

    ops.push(ModelOp::Color(column.m_color));
    ops.push(ModelOp::Quads(vec![
        corner(-1.0, low, 0.0),
        corner(1.0, low, 1.0),
        corner(1.0, high, 1.0),
        corner(-1.0, high, 0.0),
    ]));
}

fn nearLocked(seed: i64, player: [i32; 2]) -> bool
{
    let [px, pz] = player;
    (pz - tuning::NEAR..=pz + tuning::NEAR)
        .any(|z| (px - tuning::NEAR..=px + tuning::NEAR).any(|x| locked(biomes::at(seed, x, z))))
}

fn spanAround(y: i32) -> [i32; 2] { [(y - tuning::RANGE).max(0), (y + tuning::RANGE * 2).max(0)] }

fn fade(at: [i32; 2], pos: [f64; 3]) -> f32
{
    let (dx, dz): (f64, f64) = (f64::from(at[0]) + 0.5 - pos[0], f64::from(at[1]) + 0.5 - pos[2]);
    let distance: f32 = (dx * dx + dz * dz).sqrt() as f32 / tuning::RANGE as f32;
    1.0 - distance * distance
}

// renderLockedBiome
fn lockedBiomes(world: World, info: &EntityInfo, camera: [f64; 3], frame: [f32; 2], ops: &mut Vec<ModelOp>)
{
    let player: [i32; 2] = [floor(info.pos[0]), floor(info.pos[2])];
    let seed: i64 = world.seed();
    if !nearLocked(seed, player)
    {
        return;
    }

    let span: [i32; 2] = spanAround(floor(info.pos[1]));
    let mut columns: Vec<([i32; 2], [f32; 2], Column)> = Vec::new();

    for z in player[1] - tuning::RANGE..=player[1] + tuning::RANGE
    {
        for x in player[0] - tuning::RANGE..=player[0] + tuning::RANGE
        {
            let biome: Biome = biomes::at(seed, x, z);
            let Some(sides) = sheet([x, z], player).filter(|_| locked(biome))
            else
            {
                continue;
            };

            let random: Random = columnRandom(x, z);
            let shown: Option<Column> =
                effect(world, biome, &random, [x, z], span, [frame[0], frame[1], fade([x, z], info.pos)]);

            if let Some(column) = shown.filter(|column| column.m_span[0] != column.m_span[1])
            {
                columns.push(([x, z], sides, column));
            }
        }
    }

    columns.sort_by_key(|(_, _, column)| column.m_texture);

    let mut bound: Option<&str> = None;

    for (at, sides, column) in &columns
    {
        if bound != Some(column.m_texture)
        {
            ops.push(ModelOp::Texture(column.m_texture.to_owned()));
            bound = Some(column.m_texture);
        }

        quad(ops, *at, *sides, camera, column);
    }
}

// renderLockedStructure, sparkles in the guarded box's columns near the player
fn lockedStructure(guarded: Guarded, info: &EntityInfo, camera: [f64; 3], frame: [f32; 2], ops: &mut Vec<ModelOp>)
{
    let player: [i32; 2] = [floor(info.pos[0]), floor(info.pos[2])];
    let [around, high] = spanAround(floor(info.pos[1]));
    let floor: i32 = guarded.m_min[1] - tuning::STRUCTURE_MARGIN;
    let ceiling: i32 = guarded.m_max[1] + tuning::STRUCTURE_MARGIN;
    let span: [i32; 2] = [around.clamp(floor, ceiling), high.clamp(floor, ceiling)];
    if span[0] == span[1]
    {
        return;
    }

    ops.push(ModelOp::Texture(texture::SPARKLES.to_owned()));

    let precise: f32 = frame[0] + frame[1];
    let count: f32 = ((frame[0] as i32 & 511) as f32 + frame[1]) / 512.0;

    for z in (player[1] - tuning::RANGE).max(guarded.m_min[2])..=(player[1] + tuning::RANGE).min(guarded.m_max[2])
    {
        for x in (player[0] - tuning::RANGE).max(guarded.m_min[0])..=(player[0] + tuning::RANGE).min(guarded.m_max[0])
        {
            let Some(sides) = sheet([x, z], player)
            else
            {
                continue;
            };

            let random: Random = columnRandom(x, z);
            let u: f32 = random.nextFloat() + precise * 0.01 * gaussian(&random);
            let v: f32 = random.nextFloat() + precise * 0.01 * gaussian(&random);
            let alpha: f32 = (fade([x, z], info.pos) * tuning::FADE_SPAN + tuning::FADE_BASE) * random.nextFloat();

            let column: Column = Column
            {
                m_texture: texture::SPARKLES,
                m_color: [1.0, 1.0, 1.0, alpha],
                m_scroll: [count, u, v],
                m_span: span,
            };
            quad(ops, [x, z], sides, camera, &column);
        }
    }
}

// RenderTFProtectionBox, a scrolling see through shell a sixteenth outside the guarded box
fn protectionBox(guarded: Guarded, age: f32, camera: [f64; 3], ops: &mut Vec<ModelOp>)
{
    let left: f32 = tuning::BOX_LIFE as f32 - age;
    let alpha: f32 = (left / tuning::BOX_FADE).min(1.0);
    let scroll: f32 = age * tuning::BOX_SCROLL;
    let low: [f32; 3] =
        [0, 1, 2].map(|axis| (f64::from(guarded.m_min[axis]) - camera[axis]) as f32 - tuning::BOX_INFLATE);
    let high: [f32; 3] =
        [0, 1, 2].map(|axis| (f64::from(guarded.m_max[axis] + 1) - camera[axis]) as f32 + tuning::BOX_INFLATE);
    let [x0, y0, z0] = low;
    let [x1, y1, z1] = high;

    let face = |corners: [[f32; 3]; 4], width: f32, height: f32|
    {
        let uv: [[f32; 2]; 4] = [[0.0, 0.0], [width, 0.0], [width, height], [0.0, height]];
        let mut vertices: [[f32; 5]; 4] = [[0.0; 5]; 4];
        for (vertex, ([x, y, z], [u, v])) in vertices.iter_mut().zip(corners.into_iter().zip(uv))
        {
            *vertex = [x, y, z, u + scroll, v + scroll];
        }

        vertices
    };

    let (w, h, d): (f32, f32, f32) = (x1 - x0, y1 - y0, z1 - z0);
    let faces: [[[f32; 5]; 4]; 6] = [
        face([[x0, y0, z0], [x1, y0, z0], [x1, y1, z0], [x0, y1, z0]], w, h),
        face([[x0, y0, z1], [x1, y0, z1], [x1, y1, z1], [x0, y1, z1]], w, h),
        face([[x0, y0, z0], [x0, y0, z1], [x0, y1, z1], [x0, y1, z0]], d, h),
        face([[x1, y0, z0], [x1, y0, z1], [x1, y1, z1], [x1, y1, z0]], d, h),
        face([[x0, y0, z0], [x1, y0, z0], [x1, y0, z1], [x0, y0, z1]], w, d),
        face([[x0, y1, z0], [x1, y1, z0], [x1, y1, z1], [x0, y1, z1]], w, d),
    ];
    ops.push(ModelOp::Texture(texture::BOX.to_owned()));
    ops.push(ModelOp::Color([1.0, 1.0, 1.0, alpha]));
    ops.push(ModelOp::Quads(faces.concat()));
}

fn render(partial: f32, camera: [f64; 3]) -> Vec<ModelOp>
{
    let mut ops: Vec<ModelOp> = Vec::new();

    let Some(info) = localPlayer().and_then(Player::info)
    else
    {
        return ops;
    };
    let world: World = World { dimension: info.world };
    if !crate::world::isTwilight(world) || !crate::config::enforced()
    {
        return ops;
    }

    let ticks: f32 = world.time() as f32;

    ops.extend([ModelOp::Normal([0.0, 1.0, 0.0]), ModelOp::Lighting(false), ModelOp::Blend(true)]);
    lockedBiomes(world, &info, camera, [ticks, partial], &mut ops);

    if let Some((guarded, since)) = BOX.get()
    {
        let age: f32 = (world.time() - since) as f32 + partial;
        if age < tuning::BOX_LIFE as f32
        {
            lockedStructure(guarded, &info, camera, [ticks, partial], &mut ops);
            protectionBox(guarded, age, camera, &mut ops);
        }
        else
        {
            BOX.set(None);
        }
    }

    ops.extend([ModelOp::Color([1.0; 4]), ModelOp::Blend(false), ModelOp::Lighting(true)]);
    ops
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.channels().register::<Earned>(PROGRESS_CHANNEL, Direction::ToClient, |earned, _| GATES.set(earned.m_gates))?;

    ctx.channels().register::<Guarded>(BOX_CHANNEL, Direction::ToClient, |guarded, _|
    {
        let now: i64 = localPlayer().and_then(Player::info).map_or(0, |info| World { dimension: info.world }.time());
        BOX.set(Some((guarded, now)));
    })?;

    ctx.screens().worldRender(render)?;
    ctx.events().on::<TickPost>(Priority::Normal, |_| syncProgress())?;
    Ok(())
}
