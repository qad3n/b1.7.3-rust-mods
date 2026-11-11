use crate::ids::{self, block};
use crate::world::{dimension, isTwilight};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// BlockTFPortal and TFTeleporter, a pool ringed by flowers turns into a portal when a diamond lands in it
const PORTALS_KEY: &str = "portals";

pub(crate) mod travel
{
    pub(crate) const OVERWORLD: i32 = 0;
    // a fresh arrival waits above the world for one tick while the portal is found or built under it
    pub(crate) const HOLD_Y: f64 = 120.0;
    // TFTeleporter.placeInExistingPortal looks this far for a portal
    pub(crate) const SEARCH: i32 = 200;
    // TFTeleporter.findPortalCoords looks this far for a good spot to build one
    pub(crate) const BUILD_SEARCH: i32 = 16;
    pub(crate) const LOWEST: i32 = 30;
    pub(crate) const HIGHEST: i32 = 118;
    pub(crate) const SCAN_TOP: i32 = 127;
    // the overworld spot is weighed at twice the player's height, the twilight one at half
    pub(crate) const OVERWORLD_FACTOR: f64 = 2.0;
    pub(crate) const TWILIGHT_FACTOR: f64 = 0.5;
}

mod toss
{
    // a tossed diamond checks for a pool once a second, PortalEntityItem.onUpdate
    pub const CHECK: i64 = 20;
}

mod look
{
    pub const HARDNESS: f32 = -1.0;
    pub const RESISTANCE: f32 = 6000000.0;
    pub const LIGHT: f32 = 0.75;
    // the portal surface sits a quarter below the top of the block
    pub const HEIGHT: f32 = 12.0;
    pub const SOUND_ODDS: i32 = 100;
    pub const MOTES: i32 = 4;
    pub const EDGE: f64 = 0.25;
    pub const DRIFT: f64 = 0.5;
}

fn water(world: World, x: i32, y: i32, z: i32) -> bool
{
    let block: i32 = world.block(BlockPos::new(x, y, z));
    block == ids::blockId("minecraft:water") || block == ids::blockId("minecraft:flowing_water")
}

// BlockTFPortal.isGrassOrDirt, Material.grass and Material.ground
fn earth(world: World, x: i32, y: i32, z: i32) -> bool
{
    let block: i32 = world.block(BlockPos::new(x, y, z));
    block == ids::blockId("minecraft:grass")
        || block == ids::blockId("minecraft:dirt")
        || block == ids::blockId("minecraft:farmland")
}

// BlockTFPortal.isNatureBlock, Material.plants, vine and leaves
fn nature(world: World, x: i32, y: i32, z: i32) -> bool
{
    let block: i32 = world.block(BlockPos::new(x, y, z));
    let plants: [&str; 6] = [
        "minecraft:red_flower",
        "minecraft:yellow_flower",
        "minecraft:tallgrass",
        "minecraft:sapling",
        "minecraft:red_mushroom",
        "minecraft:brown_mushroom",
    ];
    plants.iter().any(|name| block == ids::blockId(name))
        || block == ids::blockId("minecraft:leaves")
        || ids::leaves(block)
}

// the twelve blocks ringing a two by two pool
const RING: [[i32; 2]; 12] =
    [[-1, -1], [-1, 0], [-1, 1], [-1, 2], [0, -1], [1, -1], [0, 2], [1, 2], [2, -1], [2, 0], [2, 1], [2, 2]];
const POOL: [[i32; 2]; 4] = [[0, 0], [1, 0], [1, 1], [0, 1]];

// BlockTFPortal.isGoodPortalPoolStrict, water over solid ground, earth around and plants on the earth
fn goodPool(world: World, x: i32, y: i32, z: i32) -> bool
{
    POOL.iter().all(|[dx, dz]|
    {
        water(world, x + dx, y, z + dz) && World::blockInfo(world.block(BlockPos::new(x + dx, y - 1, z + dz))).solid
    }) && RING.iter().all(|[dx, dz]| earth(world, x + dx, y, z + dz) && nature(world, x + dx, y + 1, z + dz))
}

// BlockTFPortal.tryToCreatePortal, any pool the spot belongs to lights up with a lightning strike
fn tryCreate(world: World, x: i32, y: i32, z: i32) -> bool
{
    let corners: [[i32; 2]; 9] = [[0, 0], [-1, -1], [0, -1], [1, -1], [-1, 0], [1, 0], [-1, 1], [0, 1], [1, 1]];
    if !corners.iter().any(|[dx, dz]| goodPool(world, x + dx, y, z + dz))
    {
        return false;
    }

    let half: f64 = 0.5;
    if let Err(error) = world.lightning([f64::from(x) + half, f64::from(y), f64::from(z) + half])
    {
        warn(&format!("portal lightning: {}", error.0));
    }

    let px: i32 = if water(world, x - 1, y, z) { x - 1 } else { x };
    let pz: i32 = if water(world, px, y, z - 1) { z - 1 } else { z };
    let portal: i32 = ids::blockId(block::PORTAL);

    for [dx, dz] in POOL
    {
        place(world, BlockPos::new(px + dx, y, pz + dz), portal);
    }

    remember(world.dimension, [px, y, pz]);
    true
}

fn place(world: World, pos: BlockPos, block: i32)
{
    if let Err(error) = world.setBlockQuiet(pos, block, 0)
    {
        warn(&format!("portal block not placed at {pos:?}: {}", error.0));
        return;
    }

    world.markBlockForRender(pos);
}

#[derive(Serialize, Deserialize, Default)]
struct PortalLog
{
    m_entries: Vec<(i32, [i32; 3])>,
}

fn remember(dimension: i32, corner: [i32; 3])
{
    let mut log: PortalLog = worldData(PORTALS_KEY);
    if !log.m_entries.contains(&(dimension, corner))
    {
        log.m_entries.push((dimension, corner));
        setWorldData(PORTALS_KEY, &log);
    }
}

fn isPortal(world: World, x: i32, y: i32, z: i32) -> bool
{
    world.block(BlockPos::new(x, y, z)) == ids::blockId(block::PORTAL)
}

// TFTeleporter.placeInExistingPortal over the portals this mod made, the nearest one still standing
fn existing(world: World, at: [f64; 3]) -> Option<[f64; 3]>
{
    let log: PortalLog = worldData(PORTALS_KEY);

    let distance = |corner: &[i32; 3]|
    {
        let [dx, dy, dz] = [0, 1, 2].map(|axis| f64::from(corner[axis]) + 0.5 - at[axis]);
        dx * dx + dy * dy + dz * dz
    };

    let [x, z] = [at[0].floor() as i32, at[2].floor() as i32];
    let corner: [i32; 3] = log
        .m_entries
        .iter()
        .filter(|(entry, [cx, cy, cz])|
        {
            *entry == world.dimension
                && (cx - x).abs() <= travel::SEARCH
                && (cz - z).abs() <= travel::SEARCH
                && isPortal(world, *cx, *cy, *cz)
        })
        .map(|(_, corner)| *corner)
        .min_by(|a, b| distance(a).total_cmp(&distance(b)))?;
    Some(arrival(world, corner))
}

// where TFTeleporter stands the traveller, beside the portal and a block above it
fn arrival(world: World, [x, y, z]: [i32; 3]) -> [f64; 3]
{
    let half: f64 = 0.5;
    let mut spot: [f64; 3] = [f64::from(x) + half, f64::from(y) + half, f64::from(z) + half];

    if isPortal(world, x - 1, y, z)
    {
        spot[0] -= half;
    }

    if isPortal(world, x + 1, y, z)
    {
        spot[0] += half;
    }

    if isPortal(world, x, y, z - 1)
    {
        spot[2] -= half;
    }

    if isPortal(world, x, y, z + 1)
    {
        spot[2] += half;
    }

    let (mut dx, mut dz): (i32, i32) = (0, 0);
    while dx == 0 && dz == 0
    {
        dx = world.random(3) - world.random(3);
        dz = world.random(3) - world.random(3);
    }

    [spot[0] + f64::from(dx), spot[1] + 1.0, spot[2] + f64::from(dz)]
}

// TFTeleporter.isIdealPortal and isOkayPortal, a four by four patch of grass or anything solid with room above
fn fits(world: World, x: i32, y: i32, z: i32, ideal: bool) -> bool
{
    let grass: i32 = ids::blockId("minecraft:grass");

    for tz in z - 1..z + 3
    {
        for tx in x - 1..x + 3
        {
            let floor: i32 = world.block(BlockPos::new(tx, y - 1, tz));

            let footing: bool = if ideal
            {
                floor == grass
            }
            else
            {
                World::blockInfo(floor).solid
            };

            if !footing
            {
                return false;
            }

            if (y..y + 3).any(|ty|
            {
                !World::blockInfo(world.block(BlockPos::new(tx, ty, tz))).replaceable
                    && !world.isAir(BlockPos::new(tx, ty, tz))
            })
            {
                return false;
            }
        }
    }

    true
}

// TFTeleporter.findPortalCoords, the nearest fitting floor around the traveller
fn findSpot(world: World, at: [f64; 3], ideal: bool, factor: f64) -> Option<[i32; 3]>
{
    let (x, z): (i32, i32) = (at[0].floor() as i32, at[2].floor() as i32);
    let mut best: Option<([i32; 3], f64)> = None;

    for rx in x - travel::BUILD_SEARCH..=x + travel::BUILD_SEARCH
    {
        for rz in z - travel::BUILD_SEARCH..=z + travel::BUILD_SEARCH
        {
            let mut ry: i32 = travel::SCAN_TOP;
            while ry >= 0
            {
                if world.isAir(BlockPos::new(rx, ry, rz))
                {
                    while ry > 0 && world.isAir(BlockPos::new(rx, ry - 1, rz))
                    {
                        ry -= 1;
                    }

                    if fits(world, rx, ry, rz, ideal)
                    {
                        let offset: [f64; 3] = [
                            f64::from(rx) + 0.5 - at[0],
                            f64::from(ry) + 0.5 - at[1] * factor,
                            f64::from(rz) + 0.5 - at[2],
                        ];

                        let weight: f64 = offset.iter().map(|axis| axis * axis).sum();
                        if best.is_none_or(|(_, least)| weight < least)
                        {
                            best = Some(([rx, ry, rz], weight));
                        }
                    }
                }

                ry -= 1;
            }
        }
    }

    best.map(|(spot, _)| spot)
}

// TFTeleporter.makePortalAt, a grassy rim with a ring of flowers and mushrooms around the pool
fn build(world: World, [x, y, z]: [i32; 3])
{
    let py: i32 = y.clamp(travel::LOWEST, travel::HIGHEST) - 1;
    let (grass, dirt, portal): (i32, i32, i32) =
        (ids::blockId("minecraft:grass"), ids::blockId("minecraft:dirt"), ids::blockId(block::PORTAL));

    for [dx, dz] in RING
    {
        place(world, BlockPos::new(x + dx, py, z + dz), grass);
    }

    for [dx, dz] in POOL
    {
        place(world, BlockPos::new(x + dx, py - 1, z + dz), dirt);
        place(world, BlockPos::new(x + dx, py, z + dz), portal);
    }

    for dx in -1..=2
    {
        for dz in -1..=2
        {
            for dy in 1..=5
            {
                place(world, BlockPos::new(x + dx, py + dy, z + dz), 0);
            }
        }
    }

    let plants: [&str; 5] = [
        "minecraft:brown_mushroom",
        "minecraft:red_mushroom",
        "minecraft:tallgrass",
        "minecraft:red_flower",
        "minecraft:yellow_flower",
    ];

    for [dx, dz] in RING
    {
        let plant: &str = plants[world.random(plants.len() as i32) as usize];
        // beta tall grass needs metadata one to show as grass
        let meta: i32 = i32::from(plant == "minecraft:tallgrass");
        if let Err(error) = world.setBlockQuiet(BlockPos::new(x + dx, py + 1, z + dz), ids::blockId(plant), meta)
        {
            warn(&format!("portal plant: {}", error.0));
        }
    }

    remember(world.dimension, [x, py, z]);
}

// TFTeleporter.makePortal, an ideal spot, an okay one, or right where the traveller stands
fn makePortal(world: World, at: [f64; 3]) -> [f64; 3]
{
    let factor: f64 = if world.dimension == travel::OVERWORLD
    {
        travel::OVERWORLD_FACTOR
    }
    else
    {
        travel::TWILIGHT_FACTOR
    };
    let spot: [i32; 3] = findSpot(world, at, true, factor).or_else(|| findSpot(world, at, false, factor)).unwrap_or([
        at[0].floor() as i32,
        (at[1] * factor).floor() as i32,
        at[2].floor() as i32,
    ]);
    build(world, spot);

    let py: i32 = spot[1].clamp(travel::LOWEST, travel::HIGHEST) - 1;
    arrival(world, [spot[0], py, spot[2]])
}

thread_local!
{
    // a player must step out of the portal before it sends them again, as timeUntilPortal holds them
    static SENT: RefCell<HashMap<i32, bool>> = RefCell::new(HashMap::new());
    static ARRIVALS: RefCell<HashMap<i32, [f64; 3]>> = RefCell::new(HashMap::new());
    // the player whose diamond is about to join the world, and the tossed diamonds being watched
    static TOSSER: std::cell::Cell<Option<i32>> = const { std::cell::Cell::new(None) };
    static TOSSED: RefCell<Vec<(Entity, Player)>> = const { RefCell::new(Vec::new()) };
    static TICKS: std::cell::Cell<i64> = const { std::cell::Cell::new(0) };
}

fn inPortal(info: &EntityInfo) -> bool
{
    let world: World = World { dimension: info.world };
    // a beta player's position is at its eyes, its feet are the bottom of its box
    let feet: f64 = Entity { world, id: info.id }.field(EntityField::Bottom);
    let [x, y, z] = [info.pos[0], feet, info.pos[2]].map(|axis| axis.floor() as i32);
    isPortal(world, x, y, z) || isPortal(world, x, y + 1, z)
}

// BlockTFPortal.onEntityCollidedWithBlock for a player, off to the twilight forest or back home
fn enter(world: World, player: Player)
{
    if SENT.with(|sent| sent.borrow().get(&player.id).copied().unwrap_or(false))
    {
        return;
    }
    let (Some(twilight), Some(info)) = (dimension(), player.info())
    else
    {
        return;
    };

    let target: i32 = if world.dimension == twilight
    {
        travel::OVERWORLD
    }
    else
    {
        twilight
    };

    if target == twilight
    {
        crate::progress::grant(player, crate::progress::Gate::Portal);
    }

    SENT.with(|sent| sent.borrow_mut().insert(player.id, true));
    ARRIVALS.with(|arrivals| arrivals.borrow_mut().insert(player.id, info.pos));

    let hold: [f64; 3] = [info.pos[0], travel::HOLD_Y, info.pos[2]];
    if let Err(error) = player.teleport(target, hold)
    {
        warn(&format!("portal travel to {target}: {}", error.0));
    }
}

// once the player stands in the target dimension, the nearest portal there or a new one takes them in
fn arrive(player: Player, to: i32)
{
    let Some(from) = ARRIVALS.with(|arrivals| arrivals.borrow_mut().remove(&player.id))
    else
    {
        return;
    };

    let world: World = World { dimension: to };
    let spot: [f64; 3] = existing(world, from).unwrap_or_else(|| makePortal(world, from));

    if isTwilight(world)
    {
        crate::progress::grant(player, crate::progress::Gate::Arrival);
    }

    if let Err(error) = player.teleport(to, spot)
    {
        warn(&format!("portal arrival: {}", error.0));
    }
}

fn rearm()
{
    let waiting: Vec<i32> =
        SENT.with(|sent| sent.borrow().iter().filter(|(_, held)| **held).map(|(id, _)| *id).collect());

    for id in waiting
    {
        let outside: bool = Player { id }.info().is_some_and(|info| !inPortal(&info));
        if outside
        {
            SENT.with(|sent| sent.borrow_mut().insert(id, false));
        }
    }
}

// TFTickHandler.PortalEntityItem.onUpdate, each watched diamond looks for a pool once a second
fn watchTosses()
{
    let now: i64 = TICKS.with(|ticks|
    {
        ticks.set(ticks.get() + 1);
        ticks.get()
    });
    if now % toss::CHECK != 0
    {
        return;
    }

    let overworld: i32 = travel::OVERWORLD;

    TOSSED.with(|tossed|
    {
        tossed.borrow_mut().retain(|(item, tosser)|
        {
            let Some(info) = item.info().filter(|info| info.alive)
            else
            {
                return false;
            };
            let world: World = World { dimension: info.world };
            if world.dimension != overworld && !isTwilight(world) && !crate::config::otherDimensions()
            {
                return true;
            }
            let [x, y, z] = info.pos.map(|axis| axis.floor() as i32);
            if !water(world, x, y, z) || !tryCreate(world, x, y, z)
            {
                return true;
            }

            item.kill();
            crate::progress::grant(*tosser, crate::progress::Gate::Portal);
            false
        });
    });
}

// BlockTFPortal: a waist high shimmering pool that carries players between the worlds
struct Portal;

#[block_behavior]
impl BlockBehavior for Portal
{
    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        // the source only sends a player who rides nothing and carries no rider
        if entity.info().is_some_and(|info| info.player.is_some())
            && entity.riding().is_none()
            && entity.rider().is_none()
        {
            enter(ctx.world(), Player { id: entity.id });
        }
    }

    // BlockTFPortal.onNeighborBlockChange, a portal block needs portal on one side and earth on the other along both axes
    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let BlockPos { x, y, z } = ctx.pos();

        let axis = |minus: [i32; 2], plus: [i32; 2]|
        {
            if isPortal(world, x + minus[0], y, z + minus[1])
            {
                earth(world, x + plus[0], y, z + plus[1])
            }
            else if isPortal(world, x + plus[0], y, z + plus[1])
            {
                earth(world, x + minus[0], y, z + minus[1])
            }
            else
            {
                false
            }
        };
        if !axis([-1, 0], [1, 0]) || !axis([0, -1], [0, 1])
        {
            let _ = world.setBlock(ctx.pos(), ids::blockId("minecraft:water"), 0);
        }
    }

    // BlockTFPortal.randomDisplayTick
    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let corner: [f64; 3] = [pos.x, pos.y, pos.z].map(f64::from);

        if world.random(look::SOUND_ODDS) == 0
        {
            let (volume, pitch): (f32, f32) = (1.0, ids::fraction(world) * 0.4 + 0.8);
            world.playSound(corner.map(|axis| axis + 0.5), "portal.portal", volume, pitch);
        }

        let alongX: bool = isPortal(world, pos.x - 1, pos.y, pos.z) || isPortal(world, pos.x + 1, pos.y, pos.z);

        for _ in 0..look::MOTES
        {
            let mut at: [f64; 3] = corner.map(|axis| axis + f64::from(ids::fraction(world)));
            let side: f64 = if world.random(2) == 0 { -1.0 } else { 1.0 };
            let mut motion: [f64; 3] = [0; 3].map(|_| (f64::from(ids::fraction(world)) - 0.5) * look::DRIFT);
            let axis: usize = if alongX { 2 } else { 0 };
            at[axis] = corner[axis] + 0.5 + look::EDGE * side;
            motion[axis] = f64::from(ids::fraction(world)) * 2.0 * side;
            world.particle("portal", at, motion);
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let surface: Cuboid = Cuboid { from: [0.0; 3], to: [16.0, look::HEIGHT, 16.0], ..Cuboid::default() };
    ctx.blocks().register(
        block::PORTAL,
        BlockSpecBuilder::template("minecraft:block")
            .name("Twilight Forest Portal")
            .material("glass")
            .texture(&crate::blocks::vanillaTexture(14))
            .hardness(look::HARDNESS)
            .resistance(look::RESISTANCE)
            .light(look::LIGHT)
            .sound("glass")
            .model(BlockModelSpec { cuboids: vec![surface], translucent: true, ..BlockModelSpec::default() })
            .collision(Vec::new())
            .drops(Vec::new())
            .behavior(Portal),
    )?;

    ctx.events().on::<ItemToss>(Priority::Normal, |event|
    {
        let (Some(stack), Some(player), Some(world)) = (event.stack(), event.player(), event.world())
        else
        {
            return;
        };

        let allowed: bool = crate::config::portals() && (!crate::config::adminPortals() || player.isOp());
        if world.isAuthority() && allowed && stack.item.is(&crate::config::portalItem())
        {
            TOSSER.with(|tosser| tosser.set(Some(player.id)));
        }
    })?;

    ctx.events().on::<EntityJoinWorld>(Priority::Normal, |event|
    {
        let EventData::Entity { entity } = event.data().clone()
        else
        {
            return;
        };
        let item: bool = entity.kind.as_ref().is_some_and(|kind| kind.path() == "item");
        if !item
        {
            return;
        }
        let Some(tosser) = TOSSER.with(std::cell::Cell::take)
        else
        {
            return;
        };

        let tossed: Entity = Entity { world: World { dimension: entity.world }, id: entity.id };
        TOSSED.with(|watched| watched.borrow_mut().push((tossed, Player { id: tosser })));
    })?;

    ctx.events().on::<TickPost>(Priority::Normal, |_|
    {
        rearm();
        watchTosses();
    })?;

    ctx.events().on::<PlayerChangeDimension>(Priority::Normal, |event|
    {
        if let (EventData::Dimension { to, .. }, Some(player)) = (event.data().clone(), event.player())
        {
            arrive(player, to);
        }
    })?;

    Ok(())
}
