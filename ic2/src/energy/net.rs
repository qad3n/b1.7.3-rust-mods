use super::{Conductor, foreignSink, kind, tileAt};
use crate::core::{dir, sound};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, VecDeque};

mod loss
{
    // EnergyNet.minConductionLoss and the least a whole path loses
    pub const MIN_CONDUCTION: f64 = 1.0e-4;
    pub const MIN_PATH: f64 = 0.1;
    // emitEnergyFrom rounds each share to five decimals before flooring it
    pub const ROUNDING: f64 = 100000.0;
}

mod shock
{
    // onTick turns the shock energy an entity took into damage, a point per 64 EU rounded up
    pub const PER_DAMAGE: i32 = 64;
    pub const REACH: i32 = 1;
}

// EnergyNet.getValidReceivers walks the axes in this order: -x, +x, -y, +y, -z, +z
const ORDER: [i32; 6] = [dir::WEST, dir::EAST, dir::DOWN, dir::UP, dir::NORTH, dir::SOUTH];

// EnergyNet.EnergyPath from one source to one sink
#[derive(Clone)]
struct Path
{
    m_target: BlockPos,
    // the side of the target the energy comes in at
    m_face: i32,
    m_conductors: Vec<(BlockPos, &'static Conductor)>,
    m_min: [i32; 3],
    m_max: [i32; 3],
    m_loss: f64,
    m_absorption: i32,
    m_breakdown: i32,
    m_conducted: i64,
}

thread_local!
{
    // EnergyNet.energySourceToEnergyPathMap per dimension, dropped whenever a block of the net changes
    static PATHS: RefCell<HashMap<i32, HashMap<BlockPos, Vec<Path>>>> = RefCell::new(HashMap::new());
    // EnergyNet.entityLivingToShockEnergyMap, paid out once a tick
    static SHOCKS: RefCell<BTreeMap<(i32, i32), i32>> = const { RefCell::new(BTreeMap::new()) };
    // bumped by every change of the net, so an emit that burnt a cable or blew up a sink drops its stale paths
    static EPOCH: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

// addTileEntity and removeTileEntity rediscover every source's paths, dropping them all does the same lazily
pub(crate) fn changed(world: World)
{
    PATHS.with(|paths| paths.borrow_mut().remove(&world.dimension));
    EPOCH.with(|epoch| epoch.set(epoch.get() + 1));
}

struct Link
{
    m_face: i32,
    m_loss: f64,
}

// what the path search needs to know about the blocks around a source, the world itself outside of tests
trait Layout
{
    // the tile at pos takes part in the net and sends energy out of a face
    fn emits(&self, pos: BlockPos, face: i32) -> Option<bool>;
    fn accepts(&self, pos: BlockPos, face: i32) -> bool;
    fn conductor(&self, pos: BlockPos) -> Option<&'static Conductor>;
    fn sink(&self, pos: BlockPos, face: i32) -> bool;
}

struct Live
{
    m_world: World,
}

impl Layout for Live
{
    fn emits(&self, pos: BlockPos, face: i32) -> Option<bool>
    {
        tileAt(self.m_world, pos).map(|tile| tile.emits(self.m_world, pos, face))
    }

    fn accepts(&self, pos: BlockPos, face: i32) -> bool
    {
        match tileAt(self.m_world, pos)
        {
            Some(other) => other.accepts(self.m_world, pos, face),
            None => foreignSink(self.m_world, pos, face),
        }
    }

    fn conductor(&self, pos: BlockPos) -> Option<&'static Conductor>
    {
        tileAt(self.m_world, pos).and_then(|tile| tile.conductor(self.m_world, pos))
    }

    fn sink(&self, pos: BlockPos, face: i32) -> bool
    {
        match tileAt(self.m_world, pos)
        {
            Some(tile) => tile.sink(),
            None => foreignSink(self.m_world, pos, face),
        }
    }
}

// the blocks next to one that energy can move into, as getValidReceivers for a forward search
fn receivers(layout: &dyn Layout, pos: BlockPos) -> Vec<(BlockPos, i32)>
{
    ORDER
        .iter()
        .filter(|face| layout.emits(pos, **face) == Some(true))
        .map(|face| (dir::step(pos, *face), dir::reverse(*face)))
        .filter(|(next, back)| layout.accepts(*next, *back))
        .collect()
}

// EnergyNet.discover from a source: the cheapest way to every sink, cables relaxed again when a cheaper way appears
fn discover(layout: &dyn Layout, source: BlockPos, limit: f64) -> Vec<Path>
{
    let mut reached: HashMap<BlockPos, Link> = HashMap::new();
    let mut queue: VecDeque<BlockPos> = VecDeque::from([source]);

    while let Some(pos) = queue.pop_front()
    {
        let base: f64 = if pos == source
        {
            0.0
        }
        else
        {
            reached.get(&pos).map_or(0.0, |link| link.m_loss)
        };

        for (next, face) in receivers(layout, pos)
        {
            if next == source
            {
                continue;
            }

            let conductor: Option<&'static Conductor> = layout.conductor(next);
            let step: f64 = conductor.map_or(0.0, |cable| cable.m_loss.max(loss::MIN_CONDUCTION));
            if conductor.is_some() && base + step >= limit
            {
                continue;
            }

            let cheaper: bool = reached.get(&next).is_none_or(|link| link.m_loss > base + step);
            if !cheaper
            {
                continue;
            }

            reached.insert(next, Link { m_face: face, m_loss: base + step });
            if conductor.is_some()
            {
                queue.retain(|queued| *queued != next);
                queue.push_back(next);
            }
        }
    }

    reached.iter().filter_map(|(target, link)| path(layout, source, &reached, *target, link)).collect()
}

// walks back from a sink to the source along the links, collecting the cables and their weakest ratings
fn path(
    layout: &dyn Layout,
    source: BlockPos,
    reached: &HashMap<BlockPos, Link>,
    target: BlockPos,
    link: &Link,
) -> Option<Path>
{
    if !layout.sink(target, link.m_face)
    {
        return None;
    }

    let mut path: Path = Path
    {
        m_target: target,
        m_face: link.m_face,
        m_conductors: Vec::new(),
        m_min: [i32::MAX; 3],
        m_max: [i32::MIN; 3],
        m_loss: link.m_loss.max(loss::MIN_PATH),
        m_absorption: i32::MAX,
        m_breakdown: i32::MAX,
        m_conducted: 0,
    };
    let mut at: BlockPos = target;
    let mut face: i32 = link.m_face;

    loop
    {
        at = dir::step(at, face);
        if at == source
        {
            return Some(path);
        }

        let cable: &'static Conductor = layout.conductor(at)?;
        path.m_min = [path.m_min[0].min(at.x), path.m_min[1].min(at.y), path.m_min[2].min(at.z)];
        path.m_max = [path.m_max[0].max(at.x), path.m_max[1].max(at.y), path.m_max[2].max(at.z)];
        path.m_absorption = path.m_absorption.min(cable.m_absorption);
        path.m_breakdown = path.m_breakdown.min(cable.m_breakdown);
        path.m_conductors.push((at, cable));
        face = reached.get(&at)?.m_face;
    }
}

fn demands(world: World, path: &Path) -> bool
{
    match tileAt(world, path.m_target)
    {
        Some(tile) => tile.demands(world, path.m_target),
        None => foreignSink(world, path.m_target, path.m_face),
    }
}

fn inject(world: World, path: &Path, amount: i32) -> i32
{
    match tileAt(world, path.m_target)
    {
        Some(tile) => tile.inject(world, path.m_target, path.m_face, amount),
        None =>
        {
            let taken: i64 = world.sendEnergy(path.m_target, path.m_face, &kind(), i64::from(amount), false);
            amount - taken.clamp(0, i64::from(amount)) as i32
        }
    }
}

// TileEntityCable.removeConductor: the cable burns away with a fizz and smoke
fn burn(world: World, pos: BlockPos)
{
    const FIZZ_VOLUME: f32 = 0.5;
    const FIZZ_PITCH: f32 = 2.6;
    const FIZZ_SPREAD: f64 = 0.8;
    const PUFFS: i32 = 8;
    const PUFF_HEIGHT: f64 = 1.2;

    let _ = world.clear(pos);
    let spread: f64 = crate::core::random::float(world) - crate::core::random::float(world);
    world.playSound(dir::center(pos), "random.fizz", FIZZ_VOLUME, FIZZ_PITCH + (spread * FIZZ_SPREAD) as f32);

    for _ in 0..PUFFS
    {
        let at: [f64; 3] = [
            pos.x as f64 + crate::core::random::float(world),
            pos.y as f64 + PUFF_HEIGHT,
            pos.z as f64 + crate::core::random::float(world),
        ];
        world.particle("largesmoke", at, [0.0; 3]);
    }

    changed(world);
}

fn near(info: &EntityInfo, pos: BlockPos) -> bool
{
    let reach: f64 = f64::from(shock::REACH);
    let inside = |value: f64, at: i32| value >= f64::from(at) - reach && value <= f64::from(at) + 1.0 + reach;
    inside(info.pos[0], pos.x) && inside(info.pos[1], pos.y) && inside(info.pos[2], pos.z)
}

// the living things standing by the cables of a path take the energy the weakest insulation cannot hold back
fn shockAround(world: World, path: &Path, conducted: i32)
{
    let reach: f64 = f64::from(shock::REACH);
    let min: [f64; 3] = path.m_min.map(|value| f64::from(value) - reach);
    let max: [f64; 3] = path.m_max.map(|value| f64::from(value) + 1.0 + reach);

    for living in world.entitiesIn(min, max).into_iter().filter(|entity| entity.health.is_some())
    {
        let mut taken: i32 = 0;

        for (pos, cable) in &path.m_conductors
        {
            if !near(&living, *pos)
            {
                continue;
            }

            taken = taken.max(conducted - cable.m_absorption);
            if cable.m_absorption == path.m_absorption
            {
                break;
            }
        }

        SHOCKS.with(|shocks| *shocks.borrow_mut().entry((world.dimension, living.id)).or_insert(0) += taken);
    }
}

fn deliver(world: World, path: &mut Path, share: i32) -> i32
{
    let lost: i32 = path.m_loss.floor() as i32;
    if share <= lost
    {
        return 0;
    }

    let left: i32 = inject(world, path, share - lost);
    let conducted: i32 = share - lost - left;
    path.m_conducted += i64::from(conducted);

    if conducted > path.m_absorption
    {
        shockAround(world, path, conducted);
    }

    if conducted >= path.m_breakdown
    {
        for (pos, cable) in path.m_conductors.clone()
        {
            if conducted >= cable.m_breakdown
            {
                burn(world, pos);
            }
        }
    }

    share - left
}

// one sink's part of a packet, weighted by 1/loss and rounded to five decimals before it is floored
fn share(amount: i32, weight: f64, loss: f64) -> i32
{
    ((f64::from(amount) / weight / loss * loss::ROUNDING).round() / loss::ROUNDING).floor() as i32
}

// EnergyNet.emitEnergyFrom: the packet split between demanding sinks by how little each path loses, what is left
pub(crate) fn emit(world: World, source: BlockPos, amount: i32) -> i32
{
    let epoch: u64 = EPOCH.with(std::cell::Cell::get);
    let mut paths: Vec<Path> = PATHS
        .with(|paths| paths.borrow().get(&world.dimension).and_then(|map| map.get(&source).cloned()))
        .unwrap_or_else(|| discover(&Live { m_world: world }, source, f64::from(i32::MAX)));
    let chosen: Vec<usize> =
        (0..paths.len()).filter(|index| demands(world, &paths[*index])).take(amount.max(0) as usize).collect();
    let weight: f64 = chosen.iter().map(|index| 1.0 / paths[*index].m_loss).sum();
    let mut used: i32 = 0;

    for index in chosen
    {
        let path: &mut Path = &mut paths[index];
        used += deliver(world, path, share(amount, weight, path.m_loss));
    }

    if EPOCH.with(std::cell::Cell::get) == epoch
    {
        PATHS.with(|cache| cache.borrow_mut().entry(world.dimension).or_default().insert(source, paths));
    }

    amount - used
}

// EnergyNet.getTotalEnergyConducted for the EU-reader: what passed through a cable or into a sink so far
pub(crate) fn conducted(world: World, pos: BlockPos) -> i64
{
    PATHS.with(|paths|
    {
        paths.borrow().get(&world.dimension).map_or(0, |sources|
        {
            sources
                .iter()
                .map(|(source, list)|
                {
                    list.iter()
                        .filter(|path|
                        {
                            *source == pos || path.m_target == pos || path.m_conductors.iter().any(|(at, _)| *at == pos)
                        })
                        .map(|path| path.m_conducted)
                        .sum::<i64>()
                })
                .sum()
        })
    })
}

// EnergyNet.onTick
fn payShocks()
{
    let shocks: BTreeMap<(i32, i32), i32> = SHOCKS.with(|shocks| std::mem::take(&mut *shocks.borrow_mut()));

    for ((dimension, id), energy) in shocks
    {
        let world: World = World { dimension };
        let alive: bool = world.entity(id).is_some_and(|info| info.alive);
        let damage: i32 = (energy + shock::PER_DAMAGE - 1) / shock::PER_DAMAGE;
        if alive && damage > 0
        {
            Entity { world, id }.damage(damage, None);
        }
    }
}

// mod_IC2.explodeMachineAt: an overloaded machine goes up in a small IC2 explosion
pub(crate) fn overload(world: World, pos: BlockPos)
{
    const POWER: f32 = 2.5;

    let _ = world.clear(pos);
    crate::explosives::blast(world, dir::center(pos), POWER, crate::explosives::MACHINE);
    world.playSound(dir::center(pos), &sound("electricOverload"), 1.0, 1.0);
    changed(world);
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<TickPost>(Priority::Normal, |_| payShocks())?;
    ctx.events().on::<BlockPlace>(Priority::Low, |event|
    {
        if let Some(world) = event.world()
        {
            changed(world);
        }
    })?;
    ctx.events().on::<BlockBreak>(Priority::Low, |event|
    {
        if let Some(world) = event.world()
        {
            changed(world);
        }
    })
}

#[cfg(test)]
mod tests
{
    use super::*;
    use crate::energy::cable::conductorOf;

    // a cable's paint, a generator sending from every side, a sink taking from every side, a storage block sending
    // out of its facing only
    #[derive(Clone, Copy)]
    enum Part
    {
        Cable(&'static str, i32),
        Source,
        Sink,
        Storage(i32),
    }

    struct Fake
    {
        m_parts: HashMap<BlockPos, Part>,
    }

    impl Fake
    {
        fn new(parts: &[([i32; 3], Part)]) -> Fake
        {
            Fake { m_parts: parts.iter().map(|([x, y, z], part)| (BlockPos::new(*x, *y, *z), *part)).collect() }
        }

        // TileEntityCable.canInteractWithCable from the side of the cable at pos
        fn cableAllows(&self, pos: BlockPos, face: i32) -> bool
        {
            let own: i32 = match self.m_parts.get(&pos)
            {
                Some(Part::Cable(_, color)) => *color,
                _ => return true,
            };

            match self.m_parts.get(&dir::step(pos, face))
            {
                Some(Part::Cable(_, other)) => own == 0 || *other == 0 || own == *other,
                _ => true,
            }
        }
    }

    impl Layout for Fake
    {
        fn emits(&self, pos: BlockPos, face: i32) -> Option<bool>
        {
            Some(match self.m_parts.get(&pos)?
            {
                Part::Cable(..) => self.cableAllows(pos, face),
                Part::Source => true,
                Part::Sink => false,
                Part::Storage(facing) => *facing == face,
            })
        }

        fn accepts(&self, pos: BlockPos, face: i32) -> bool
        {
            match self.m_parts.get(&pos)
            {
                Some(Part::Cable(..)) => self.cableAllows(pos, face),
                Some(Part::Sink) => true,
                Some(Part::Storage(facing)) => *facing != face,
                _ => false,
            }
        }

        fn conductor(&self, pos: BlockPos) -> Option<&'static Conductor>
        {
            match self.m_parts.get(&pos)?
            {
                Part::Cable(name, _) => Some(conductorOf(name)),
                _ => None,
            }
        }

        fn sink(&self, pos: BlockPos, _face: i32) -> bool
        {
            matches!(self.m_parts.get(&pos), Some(Part::Sink | Part::Storage(_)))
        }
    }

    const ORIGIN: [i32; 3] = [0, 64, 0];

    fn at(x: i32) -> [i32; 3] { [x, ORIGIN[1], ORIGIN[2]] }

    fn search(fake: &Fake) -> Vec<Path>
    {
        let mut paths: Vec<Path> = discover(fake, BlockPos::new(ORIGIN[0], ORIGIN[1], ORIGIN[2]), f64::from(i32::MAX));
        paths.sort_by_key(|path| path.m_target.x);
        paths
    }

    #[test]
    fn copperLineLosesAFifthPerCable()
    {
        let copper: Part = Part::Cable("copper_cable", 0);
        let fake: Fake =
            Fake::new(&[(at(0), Part::Source), (at(1), copper), (at(2), copper), (at(3), copper), (at(4), Part::Sink)]);
        let paths: Vec<Path> = search(&fake);

        assert_eq!(paths.len(), 1);
        assert!((paths[0].m_loss - 0.6).abs() < 1e-9);
        assert_eq!(paths[0].m_face, dir::WEST);
        assert_eq!(paths[0].m_conductors.len(), 3);
        assert_eq!((paths[0].m_absorption, paths[0].m_breakdown), (32, 33));
    }

    #[test]
    fn aSinkBesideTheSourceStillLosesATenth()
    {
        let fake: Fake = Fake::new(&[(at(0), Part::Source), (at(1), Part::Sink)]);
        let paths: Vec<Path> = search(&fake);

        assert_eq!(paths.len(), 1);
        assert!((paths[0].m_loss - 0.1).abs() < 1e-9);
        assert!(paths[0].m_conductors.is_empty());
    }

    #[test]
    fn differentlyPaintedCablesDoNotJoinButPlainOnesJoinAnything()
    {
        let red: Part = Part::Cable("copper_cable", 1);
        let blue: Part = Part::Cable("copper_cable", 4);
        let split: Fake = Fake::new(&[(at(0), Part::Source), (at(1), red), (at(2), blue), (at(3), Part::Sink)]);
        assert!(search(&split).is_empty());

        let plain: Part = Part::Cable("copper_cable", 0);
        let joined: Fake =
            Fake::new(&[(at(0), Part::Source), (at(1), red), (at(2), plain), (at(3), blue), (at(4), Part::Sink)]);
        assert_eq!(search(&joined).len(), 1);
    }

    #[test]
    fn theCheapestRouteWins()
    {
        // a long glass fibre loop beats a short line of bare HV cable
        let hv: Part = Part::Cable("hv_cable", 0);
        let glass: Part = Part::Cable("glass_fibre_cable", 0);
        let mut parts: Vec<([i32; 3], Part)> =
            vec![(at(0), Part::Source), (at(1), hv), (at(2), hv), (at(3), Part::Sink)];
        parts.extend((0..=3).map(|x| ([x, ORIGIN[1] + 1, ORIGIN[2]], glass)));
        let fake: Fake = Fake::new(&parts);
        let paths: Vec<Path> = search(&fake);

        assert_eq!(paths.len(), 1);
        assert!((paths[0].m_loss - 4.0 * 0.025).abs() < 1e-9);
        assert_eq!(paths[0].m_face, dir::UP);
    }

    #[test]
    fn storageSendsOnlyOutOfItsFrontAndTakesFromTheRest()
    {
        let copper: Part = Part::Cable("copper_cable", 0);
        let backwards: Fake = Fake::new(&[(at(0), Part::Storage(dir::WEST)), (at(1), copper), (at(2), Part::Sink)]);
        assert!(search(&backwards).is_empty());

        let forwards: Fake =
            Fake::new(&[(at(0), Part::Storage(dir::EAST)), (at(1), copper), (at(2), Part::Storage(dir::EAST))]);
        let paths: Vec<Path> = search(&forwards);
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].m_target, BlockPos::new(2, ORIGIN[1], ORIGIN[2]));
    }

    #[test]
    fn packetsSplitByTheInverseOfEachPathLoss()
    {
        let weight: f64 = 1.0 / 0.2 + 1.0 / 0.6;
        assert_eq!(share(32, weight, 0.2), 24);
        assert_eq!(share(32, weight, 0.6), 8);
        assert_eq!(share(5, 1.0 / 0.1, 0.1), 5);
    }

    #[test]
    fn aSearchLimitPrunesLossyRoutes()
    {
        let hv: Part = Part::Cable("hv_cable", 0);
        let fake: Fake = Fake::new(&[(at(0), Part::Source), (at(1), hv), (at(2), hv), (at(3), Part::Sink)]);
        let paths: Vec<Path> = discover(&fake, BlockPos::new(ORIGIN[0], ORIGIN[1], ORIGIN[2]), 1.5);
        assert!(paths.is_empty());
    }
}
