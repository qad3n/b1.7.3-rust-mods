use super::processing::provide;
use super::screen::{self, Bar};
use super::{Faces, Look, Machine, registerBlock, setActive};
use crate::blocks::{MINING_PIPE, MINING_TIP};
use crate::core::{dir, sound, stacks};
use crate::electric;
use crate::energy::{self, EnergyTile, net};
use b173_api::prelude::*;
use std::rc::Rc;

const HARDNESS: f32 = 2.0;

const fn machine(name: &'static str, look: Look, slots: i32) -> Machine
{
    Machine
    {
        m_name: name,
        m_look: look,
        m_hardness: HARDNESS,
        m_screen: true,
        m_wrench: 1.0,
        m_drop: "machine",
        m_passes: None,
        m_slots: slots,
    }
}

pub(crate) static MACHINES: [Machine; 2] = [
    machine(
        "miner",
        Look
        {
            m_idle: ["miner_bottom", "machine_top", "miner_side", "miner_side"],
            m_active: ["miner_bottom", "machine_top", "miner_side", "miner_side"],
            m_faces: Faces::Machine,
        },
        miner::SLOTS,
    ),
    machine(
        "pump",
        Look
        {
            m_idle: ["machine_bottom", "machine_top", "extractor_front", "extractor_front"],
            m_active: ["machine_bottom", "machine_top", "extractor_front_active", "extractor_front_active"],
            m_faces: Faces::Machine,
        },
        pump::SLOTS,
    ),
];

// ItemScanner.blockValues, what the scanners and the miner look for, by ore tag
const ORE_VALUES: [(&str, i32); 9] = [
    ("c:ores/coal", 1),
    ("c:ores/tin", 2),
    ("c:ores/copper", 2),
    ("c:ores/gold", 3),
    ("c:ores/redstone", 3),
    ("c:ores/lapis", 3),
    ("c:ores/iron", 4),
    ("c:ores/uranium", 4),
    ("c:ores/diamond", 5),
];

pub(crate) fn oreValue(block: i32) -> i32
{
    ORE_VALUES.iter().find(|(tag, _)| blockTagged(block, tag)).map_or(0, |(_, value)| *value)
}

// ItemElectricToolDrill.canHarvestBlock: its mineable list and rock and iron material, the diamond drill adds obsidian
pub(crate) fn drillHarvests(block: i32, diamond: bool) -> bool
{
    blockTagged(block, "ic2:mineable/drill") || diamond && blockTagged(block, "c:obsidians")
}

// a still source block, as the miner and pump only take metadata 0 liquid
fn source(world: World, pos: BlockPos) -> bool { world.fluidAt(pos).is_some_and(|fluid| fluid.source) }

const WATER: &str = "minecraft:water";
const LAVA: &str = "minecraft:lava";

fn fluid(block: i32) -> Option<&'static str>
{
    if blockTagged(block, WATER)
    {
        return Some(WATER);
    }

    blockTagged(block, LAVA).then_some(LAVA)
}

// TileEntityMiner.distributeDrop: into a chest or personal safe next to the machine, else onto the ground
pub(crate) fn distribute(world: World, pos: BlockPos, stack: ItemStack)
{
    const PICKUP_DELAY: i32 = 10;

    for face in [dir::UP, dir::DOWN, dir::EAST, dir::WEST, dir::SOUTH, dir::NORTH]
    {
        let at: BlockPos = dir::step(pos, face);
        if !world.tagged(at, "ic2:miner_outputs")
        {
            continue;
        }

        // putIntoChest only stores a find the chest takes whole, through the inventory's item capability
        let side: i32 = dir::reverse(face);
        let fits: bool = world.insertItem(at, side, stack.clone(), true).is_ok_and(|left| left == 0);
        if fits && world.insertItem(at, side, stack.clone(), false).is_ok_and(|left| left == 0)
        {
            return;
        }
    }

    let at: [f64; 3] = [pos.x as f64, pos.y as f64, pos.z as f64];
    if let Err(error) = world.dropItem(at, &stack, [0.0; 3], PICKUP_DELAY)
    {
        warn(&format!("the miner at {pos:?} could not drop its find: {error}"));
    }
}

mod miner
{
    pub const SLOTS: i32 = 4;
    pub const FUEL: i32 = 0;
    pub const SCANNER: i32 = 1;
    pub const PIPES: i32 = 2;
    pub const DRILL: i32 = 3;
    pub const CAPACITY: i32 = 1000;
    pub const INPUT: i32 = 32;
    pub const READY: i32 = 100;
    pub const DIG_TICKS: i32 = 200;
    pub const WITHDRAW_TICKS: i32 = 20;
    pub const WITHDRAW_COST: i32 = 2;
    pub const DIAMOND_SPEED: i32 = 3;
    pub const DIAMOND_COST: i32 = 14;
    pub const DEPTH_COST: i32 = 2;
    pub const PIPE_COST: i32 = 10;
    pub const HUM_EVERY: i32 = 32;
    pub const SCAN: i32 = 2;
    pub const SCAN_ADVANCED: i32 = 4;
    pub const SCAN_COST: i32 = 1;
    pub const SCAN_ADVANCED_COST: i32 = 5;
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Dig
{
    pub(crate) m_energy: i32,
    m_target: [i32; 3],
    m_ticker: i32,
    m_sound: i32,
}

impl Default for Dig
{
    fn default() -> Self { Dig { m_energy: 0, m_target: [0, -1, 0], m_ticker: 0, m_sound: 0 } }
}

struct Miner
{
    m_world: World,
    m_pos: BlockPos,
}

impl Miner
{
    fn slot(&self, slot: i32) -> Option<ItemStack> { self.m_world.tileSlot(self.m_pos, slot) }

    fn block(&self, x: i32, y: i32, z: i32) -> i32 { self.m_world.block(BlockPos::new(x, y, z)) }

    fn air(&self, x: i32, y: i32, z: i32) -> bool { self.m_world.isAir(BlockPos::new(x, y, z)) }

    fn shaft(&self, block: i32) -> bool { blockIs(block, MINING_PIPE) || blockIs(block, MINING_TIP) }

    // TileEntityMiner.getPipeTip
    fn tip(&self) -> i32
    {
        let mut y: i32 = self.m_pos.y;
        while self.shaft(self.block(self.m_pos.x, y - 1, self.m_pos.z))
        {
            y -= 1;
        }

        y
    }

    fn drill(&self) -> Option<ItemStack>
    {
        self.slot(miner::DRILL).filter(|drill| drill.item.is("ic2:mining_drill") || drill.item.is("ic2:diamond_drill"))
    }

    fn diamond(&self) -> bool { self.drill().is_some_and(|drill| drill.item.is("ic2:diamond_drill")) }

    fn canOperate(&self, dig: &Dig) -> bool
    {
        let pipes: bool = self.slot(miner::PIPES).is_some_and(|pipes| pipes.item.is(MINING_PIPE));
        pipes && self.drill().is_some() && dig.m_ticker >= 0
    }

    fn operating(&self, dig: &Dig) -> bool { dig.m_energy > miner::READY && self.canOperate(dig) }

    fn pump(&self) -> Option<BlockPos>
    {
        [dir::UP, dir::DOWN, dir::EAST, dir::WEST, dir::SOUTH, dir::NORTH]
            .iter()
            .map(|face| dir::step(self.m_pos, *face))
            .find(|at| self.m_world.is(*at, "pump") && pumpHarvests(self.m_world, *at))
    }

    // TileEntityMiner.canMine
    fn canMine(&self, at: [i32; 3]) -> bool
    {
        let [x, y, z] = at;
        let block: i32 = self.block(x, y, z);
        if self.air(x, y, z) || fluid(block).is_some() && self.pump().is_some()
        {
            return true;
        }

        let hard: bool = World::blockInfo(block).hardness < 0.0;
        !hard && self.drill().is_some() && drillHarvests(block, self.diamond())
    }

    // TileEntityMiner.canPass, what the miner's reach goes through sideways
    fn canPass(&self, at: [i32; 3]) -> bool
    {
        let [x, y, z] = at;
        let block: i32 = self.block(x, y, z);
        self.air(x, y, z) || fluid(block).is_some() || self.shaft(block) || blockIs(block, "machine")
    }

    // TileEntityMiner.canReachTarget, stepping back toward the shaft along the longer axis
    fn reachable(&self, target: [i32; 3], first: bool) -> bool
    {
        let [mut x, y, mut z] = target;
        let mut first: bool = first;

        loop
        {
            if x == self.m_pos.x && z == self.m_pos.z
            {
                return true;
            }

            if !first && !self.canPass([x, y, z])
            {
                return false;
            }

            first = false;
            (x, z) = stepToward(x, z, self.m_pos.x, self.m_pos.z);
        }
    }

    // TileEntityMiner.aquireTarget, a scanner looks for ore or pumpable lava around the tip first
    fn acquire(&self, dig: &mut Dig)
    {
        let tip: i32 = self.tip();
        dig.m_target = [self.m_pos.x, tip - 1, self.m_pos.z];

        let Some(mut scanner) = self.slot(miner::SCANNER)
        else
        {
            return;
        };
        let (range, cost): (i32, i32) = match scanner.item.name().map(|id| id.to_string()).as_deref()
        {
            Some("ic2:od_scanner") => (miner::SCAN, miner::SCAN_COST),
            Some("ic2:ov_scanner") => (miner::SCAN_ADVANCED, miner::SCAN_ADVANCED_COST),
            _ => return,
        };

        let scanned: bool = electric::use_(&mut scanner, cost);
        stacks::set(self.m_world, self.m_pos, miner::SCANNER, Some(scanner));
        if !scanned
        {
            return;
        }

        for x in self.m_pos.x - range..=self.m_pos.x + range
        {
            for z in self.m_pos.z - range..=self.m_pos.z + range
            {
                let at: BlockPos = BlockPos::new(x, tip, z);
                let block: i32 = self.m_world.block(at);
                let ore: bool = oreValue(block) > 0 && self.canMine([x, tip, z]);
                let lava: bool =
                    self.pump().is_some() && source(self.m_world, at) && fluid(block) == Some("minecraft:lava");
                if ore || lava
                {
                    dig.m_target = [x, tip, z];
                    return;
                }
            }
        }
    }

    // TileEntityMiner.updateMineTip
    fn setTip(&self, tip: i32)
    {
        if tip == self.m_pos.y
        {
            return;
        }

        for y in (tip + 1..self.m_pos.y).rev()
        {
            let at: BlockPos = BlockPos::new(self.m_pos.x, y, self.m_pos.z);
            if !self.m_world.is(at, MINING_PIPE)
            {
                let _ = self.m_world.place(at, MINING_PIPE);
            }
        }

        let _ = self.m_world.place(BlockPos::new(self.m_pos.x, tip, self.m_pos.z), MINING_TIP);
    }

    // TileEntityMiner.mineBlock
    fn mineBlock(&self, dig: &mut Dig)
    {
        let world: World = self.m_world;
        let [x, y, z] = dig.m_target;
        let at: BlockPos = BlockPos::new(x, y, z);

        if let Some(mut drill) = self.slot(miner::DRILL)
        {
            electric::use_(&mut drill, 1);
            stacks::set(world, self.m_pos, miner::DRILL, Some(drill));
        }

        let block: i32 = world.block(at);
        let liquid: Option<&str> = fluid(block);
        let air: bool = world.isAir(at);
        let kept: bool = !air && (liquid.is_none() || source(world, at));

        if kept
        {
            let drops: Vec<ItemStack> = if liquid.is_none()
            {
                world.blockDrops(at)
            }
            else
            {
                Vec::new()
            };
            let _ = world.clear(at);

            for stack in drops
            {
                distribute(world, self.m_pos, stack);
            }

            if let (Some(fluid), Some(pump)) = (liquid, self.pump())
            {
                pumpThis(world, pump, fluid);
            }

            dig.m_energy -= miner::DEPTH_COST * (self.m_pos.y - y);
        }
        else if !air
        {
            let _ = world.clear(at);
        }

        if x == self.m_pos.x && z == self.m_pos.z
        {
            if let Some(pipes) = self.slot(miner::PIPES)
            {
                let _ = StateHandle::named(MINING_PIPE).and_then(|pipe| pipe.place(world, at, false));
                stacks::set(world, self.m_pos, miner::PIPES, stacks::less(pipes, 1));
            }

            dig.m_energy -= miner::PIPE_COST;
        }

        self.setTip(y);
        dig.m_target[1] = -1;
    }

    // TileEntityMiner.mine
    fn mine(&self, dig: &mut Dig)
    {
        if dig.m_target[1] < 0
        {
            self.acquire(dig);
            return;
        }

        if !self.reachable(dig.m_target, true)
        {
            let (x, z) = stepToward(dig.m_target[0], dig.m_target[2], self.m_pos.x, self.m_pos.z);
            dig.m_target[0] = x;
            dig.m_target[2] = z;
            return;
        }

        let [x, y, z] = dig.m_target;
        if !self.canMine([x, y, z])
        {
            dig.m_ticker = -1;
            return;
        }

        dig.m_sound += 1;
        if dig.m_sound % miner::HUM_EVERY == 0
        {
            let at: [f64; 3] = [self.m_pos.x as f64, f64::from(self.tip()), self.m_pos.z as f64];
            self.m_world.playSound(at, &sound("machineMinerLoop"), 1.0, 1.0);
        }

        dig.m_ticker += 1;
        dig.m_energy -= 1;
        if self.diamond()
        {
            dig.m_ticker += miner::DIAMOND_SPEED;
            dig.m_energy -= miner::DIAMOND_COST;
        }

        if dig.m_ticker >= miner::DIG_TICKS
        {
            dig.m_ticker = 0;
            self.mineBlock(dig);
        }
    }

    // TileEntityMiner.withdrawPipe: pulls the shaft back a block, leaving the block in the pipe slot in its place
    fn withdraw(&self)
    {
        let world: World = self.m_world;
        let tip: i32 = self.tip();
        let at: BlockPos = BlockPos::new(self.m_pos.x, tip, self.m_pos.z);
        let _ = world.clear(at);
        distribute(world, self.m_pos, ItemStack::of(MINING_PIPE, 1));

        if let Some(filler) = self
            .slot(miner::PIPES)
            .filter(|stack| !stack.item.is(MINING_PIPE) && stack.info().is_some_and(|info| info.block))
            && let Some(block) = filler.item.name()
        {
            let _ = world.setBlockState(at, StateData::of(block));
            stacks::set(world, self.m_pos, miner::PIPES, stacks::less(filler, 1));
        }

        self.setTip(tip + 1);
    }

    fn tick(&self, dig: &mut Dig)
    {
        let was: bool = self.operating(dig);

        if was
        {
            dig.m_energy -= 1;
            for (slot, tier) in [(miner::SCANNER, 2), (miner::DRILL, 1)]
            {
                if let Some(mut item) = self.slot(slot).filter(electric::chargeable)
                {
                    dig.m_energy -= electric::give(&mut item, dig.m_energy, tier);
                    stacks::set(self.m_world, self.m_pos, slot, Some(item));
                }
            }
        }

        if dig.m_energy <= miner::CAPACITY
        {
            provide(&(self.m_world, self.m_pos), miner::FUEL, &mut dig.m_energy, miner::CAPACITY);
        }

        if was
        {
            self.mine(dig);
        }
        else if self.slot(miner::DRILL).is_none()
        {
            let below: i32 = self.block(self.m_pos.x, self.m_pos.y - 1, self.m_pos.z);
            if dig.m_energy >= miner::WITHDRAW_COST && self.shaft(below)
            {
                dig.m_target[1] = -1;
                dig.m_ticker += 1;
                dig.m_energy -= miner::WITHDRAW_COST;
                if dig.m_ticker >= miner::WITHDRAW_TICKS
                {
                    dig.m_ticker = 0;
                    self.withdraw();
                }
            }
            else if dig.m_ticker < 0
            {
                dig.m_ticker = 0;
            }
        }

        setActive(self.m_world, self.m_pos, self.operating(dig));
    }
}

fn stepToward(x: i32, z: i32, toX: i32, toZ: i32) -> (i32, i32)
{
    let (dx, dz): (i32, i32) = (x - toX, z - toZ);
    if dx.abs() > dz.abs()
    {
        return (if dx > 0 { x - 1 } else { x + 1 }, z);
    }

    (x, if dz > 0 { z - 1 } else { z + 1 })
}

mod pump
{
    pub const SLOTS: i32 = 2;
    pub const CONTAINER: i32 = 0;
    pub const FUEL: i32 = 1;
    pub const CAPACITY: i32 = 200;
    pub const INPUT: i32 = 32;
    pub const READY: i32 = 200;
    pub const HUM_EVERY: i32 = 64;
    pub const LAVA: i32 = 3000;
    pub const TOOK: i32 = 2980;
    pub const FOUNTAIN_EVERY: i64 = 10;
    pub const FOUNTAIN_HEIGHT: i32 = 4;
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Pumping
{
    pub(crate) m_energy: i32,
    pub(crate) m_charge: i32,
    m_sound: i32,
}

fn chestWithBucket(world: World, pos: BlockPos) -> Option<(BlockPos, i32)>
{
    [dir::UP, dir::DOWN, dir::EAST, dir::WEST, dir::SOUTH, dir::NORTH]
        .iter()
        .map(|face| dir::step(pos, *face))
        .filter(|at| world.tagged(*at, "c:chests"))
        .find_map(|at|
        {
            (0..world.inventorySize(at))
                .find(|slot| world.tileSlot(at, *slot).is_some_and(|stack| stack.item.is("minecraft:bucket")))
                .map(|slot| (at, slot))
        })
}

// TileEntityPump.canHarvest: charged, with an empty cell or bucket at hand
pub(crate) fn pumpHarvests(world: World, pos: BlockPos) -> bool
{
    let pumping: Pumping = world.tileData(pos);
    let held: bool = world
        .tileSlot(pos, pump::CONTAINER)
        .is_some_and(|stack| stack.item.is("ic2:empty_cell") || stack.item.is("minecraft:bucket"));
    pumping.m_charge >= pump::READY && (held || chestWithBucket(world, pos).is_some())
}

// TileEntityPump.deliverLavaToGeo: up to 3000 lava into the geothermal generators around
fn deliverLava(world: World, pos: BlockPos) -> bool
{
    let mut lava: i32 = pump::LAVA;

    for face in [dir::UP, dir::DOWN, dir::EAST, dir::WEST, dir::SOUTH, dir::NORTH]
    {
        let at: BlockPos = dir::step(pos, face);
        if lava > 0 && world.is(at, "geothermal_generator")
        {
            lava = super::generators::addLava(world, at, lava);
        }
    }

    lava < pump::TOOK
}

// TileEntityPump.pumpThis: one source block of a fluid into the geothermal generators, a container or a chest bucket
pub(crate) fn pumpThis(world: World, pos: BlockPos, fluid: &str) -> bool
{
    let mut pumping: Pumping = world.tileData(pos);
    let lava: bool = fluid == "minecraft:lava";
    let done: bool = pumpInto(world, pos, lava);
    pumping.m_charge = 0;
    let _ = world.setTileData(pos, &pumping);
    done
}

fn pumpInto(world: World, pos: BlockPos, lava: bool) -> bool
{
    if lava && deliverLava(world, pos)
    {
        return true;
    }

    let filledBucket: &str = if lava
    {
        "minecraft:lava_bucket"
    }
    else
    {
        "minecraft:water_bucket"
    };
    let filledCell: &str = if lava
    {
        "ic2:lava_cell"
    }
    else
    {
        "ic2:water_cell"
    };

    match world.tileSlot(pos, pump::CONTAINER)
    {
        Some(bucket) if bucket.item.is("minecraft:bucket") =>
        {
            distribute(world, pos, ItemStack::of(filledBucket, bucket.count));
            stacks::set(world, pos, pump::CONTAINER, None);
            true
        }

        Some(cell) if cell.item.is("ic2:empty_cell") =>
        {
            stacks::set(world, pos, pump::CONTAINER, stacks::less(cell, 1));
            distribute(world, pos, ItemStack::of(filledCell, 1));
            true
        }

        _ =>
        {
            let Some((chest, slot)) = chestWithBucket(world, pos)
            else
            {
                return false;
            };
            let count: i32 = world.tileSlot(chest, slot).map_or(1, |stack| stack.count);
            stacks::set(world, chest, slot, Some(ItemStack::of(filledBucket, count)));
            true
        }
    }
}

// TileEntityPump.fountain: with nothing to fill, water below is pushed up to three blocks above
fn fountain(world: World, pos: BlockPos, pumping: &mut Pumping)
{
    if world.time() % pump::FOUNTAIN_EVERY == 0
    {
        pumping.m_charge -= 1;
    }

    let flowing: &str = "minecraft:flowing_water";
    let top: Option<i32> = (1..pump::FOUNTAIN_HEIGHT).rfind(|height|
    {
        let at: BlockPos = pos.offset(0, *height, 0);
        world.isAir(at) || world.is(at, flowing)
    });

    // TileEntityPump.fountain places flowing water one step from a source
    if let Some(height) = top
        && let Ok(water) = Identifier::parse(flowing)
    {
        let _ = world.setBlockState(pos.offset(0, height, 0), StateData::of(water).with("meta", 1));
    }
}

// TileEntityPump.updateEntity
fn pumpTick(world: World, pos: BlockPos, pumping: &mut Pumping)
{
    if pumping.m_energy > 0 && pumping.m_charge < pump::READY
    {
        pumping.m_energy -= 1;
        pumping.m_charge += 1;
        pumping.m_sound += 1;
        if pumping.m_sound % pump::HUM_EVERY == 0
        {
            world.playSound(dir::center(pos), &sound("machinePumpLoop"), 1.0, 1.0);
        }
    }

    if pumping.m_energy <= pump::CAPACITY
    {
        provide(&(world, pos), pump::FUEL, &mut pumping.m_energy, pump::CAPACITY);
    }

    if pumping.m_charge >= pump::READY
    {
        let below: BlockPos = pos.offset(0, -1, 0);
        let block: i32 = world.block(below);
        let liquid: Option<&str> = fluid(block);
        let _ = world.setTileData(pos, &*pumping);

        if !pumpHarvests(world, pos) && liquid == Some("minecraft:water")
        {
            fountain(world, pos, pumping);
        }
        else if let Some(liquid) = liquid.filter(|_| source(world, below))
        {
            let _ = world.clear(below);
            pumpThis(world, pos, liquid);
            *pumping = Pumping { m_energy: pumping.m_energy, m_sound: pumping.m_sound, ..world.tileData(pos) };
        }
    }

    setActive(world, pos, pumping.m_charge < pump::READY && pumping.m_energy > 0);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind
{
    Miner,
    Pump,
}

struct Digger
{
    m_kind: Kind,
}

static MINER: Digger = Digger { m_kind: Kind::Miner };
static PUMP: Digger = Digger { m_kind: Kind::Pump };

impl Digger
{
    fn energy(&self, world: World, pos: BlockPos) -> i32
    {
        match self.m_kind
        {
            Kind::Miner => world.tileData::<Dig>(pos).m_energy,
            Kind::Pump => world.tileData::<Pumping>(pos).m_energy,
        }
    }

    fn setEnergy(&self, world: World, pos: BlockPos, energy: i32)
    {
        match self.m_kind
        {
            Kind::Miner =>
            {
                let mut dig: Dig = world.tileData(pos);
                dig.m_energy = energy;
                let _ = world.setTileData(pos, &dig);
            }

            Kind::Pump =>
            {
                let mut pumping: Pumping = world.tileData(pos);
                pumping.m_energy = energy;
                let _ = world.setTileData(pos, &pumping);
            }
        }
    }

    fn rates(&self) -> (i32, i32)
    {
        match self.m_kind
        {
            Kind::Miner => (miner::CAPACITY, miner::INPUT),
            Kind::Pump => (pump::CAPACITY, pump::INPUT),
        }
    }
}

impl EnergyTile for Digger
{
    fn accepts(&self, _world: World, _pos: BlockPos, _face: i32) -> bool { true }

    fn sink(&self) -> bool { true }

    fn demands(&self, world: World, pos: BlockPos) -> bool
    {
        let (capacity, input): (i32, i32) = self.rates();
        self.energy(world, pos) < capacity - input
    }

    fn inject(&self, world: World, pos: BlockPos, _face: i32, amount: i32) -> i32
    {
        let (capacity, input): (i32, i32) = self.rates();
        if amount > input
        {
            net::overload(world, pos);
            return 0;
        }

        let energy: i32 = self.energy(world, pos) + amount;
        self.setEnergy(world, pos, energy.min(capacity));
        (energy - capacity).max(0)
    }
}

struct DiggerTile
{
    m_digger: &'static Digger,
}

#[tile_behavior]
impl TileBehavior for DiggerTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();

        match self.m_digger.m_kind
        {
            Kind::Miner =>
            {
                let before: Dig = ctx.tileData();
                let mut dig: Dig = before;
                Miner { m_world: world, m_pos: pos }.tick(&mut dig);
                if dig != before
                {
                    let _ = ctx.setTileData(&dig);
                }
            }

            Kind::Pump =>
            {
                let before: Pumping = ctx.tileData();
                let mut pumping: Pumping = before;
                pumpTick(world, pos, &mut pumping);
                if pumping != before
                {
                    let _ = ctx.setTileData(&pumping);
                }
            }
        }
    }

    fn energyDemand(&self, ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand>
    {
        let (_, input): (i32, i32) = self.m_digger.rates();
        self.m_digger.demands(ctx.world(), ctx.pos()).then(|| energy::demand(input)).flatten()
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let (capacity, input): (i32, i32) = self.m_digger.rates();
        let energy: i32 = self.m_digger.energy(ctx.world(), ctx.pos());
        let taken: i32 = (amount.clamp(0, i64::from(input)) as i32).min(capacity - energy).max(0);

        if !simulate
        {
            self.m_digger.setEnergy(ctx.world(), ctx.pos(), energy + taken);
        }

        i64::from(taken)
    }
}

// GuiMiner and GuiPump
struct DiggerScreen
{
    m_kind: Kind,
    m_pos: Option<BlockPos>,
}

mod layout
{
    use super::Bar;

    pub const MINER_ENERGY: Bar = Bar { m_at: [81, 41], m_uv: [176, 0], m_size: [14, 14] };
    pub const PUMP_ENERGY: Bar = Bar { m_at: [69, 36], m_uv: [176, 0], m_size: [14, 14] };
    pub const PUMP_CHARGE: Bar = Bar { m_at: [91, 31], m_uv: [176, 14], m_size: [17, 24] };
    pub const MINER_TITLE: i32 = 62;
    pub const PUMP_TITLE: i32 = 63;
}

impl Screen for DiggerScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        match self.m_kind
        {
            Kind::Miner =>
            {
                screen::labels(draw, &screen::name("miner"), layout::MINER_TITLE);
                let dig: Dig = screen::state(self.m_pos).unwrap_or_default();
                if dig.m_energy > 0
                {
                    let size: i32 = layout::MINER_ENERGY.m_size[1];
                    screen::rising(
                        draw,
                        "miner",
                        &layout::MINER_ENERGY,
                        (dig.m_energy * size / miner::CAPACITY).min(size),
                        0,
                    );
                }
            }

            Kind::Pump =>
            {
                screen::labels(draw, &screen::name("pump"), layout::PUMP_TITLE);
                let pumping: Pumping = screen::state(self.m_pos).unwrap_or_default();
                if pumping.m_energy > 0
                {
                    let size: i32 = layout::PUMP_ENERGY.m_size[1];
                    screen::rising(
                        draw,
                        "pump",
                        &layout::PUMP_ENERGY,
                        (pumping.m_energy * size / pump::CAPACITY).min(size),
                        0,
                    );
                }

                let size: i32 = layout::PUMP_CHARGE.m_size[1];
                screen::rising(
                    draw,
                    "pump",
                    &layout::PUMP_CHARGE,
                    (pumping.m_charge * size / pump::READY).min(size),
                    0,
                );
            }
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    // ContainerMiner and ContainerPump slot positions
    const MINER_SLOTS: [(i32, [i32; 2]); 4] =
        [(miner::FUEL, [81, 59]), (miner::SCANNER, [117, 22]), (miner::PIPES, [81, 22]), (miner::DRILL, [45, 22])];
    const PUMP_SLOTS: [(i32, [i32; 2]); 2] = [(pump::CONTAINER, [69, 17]), (pump::FUEL, [69, 53])];

    for (machine, digger, slots, sheet) in
        [(&MACHINES[0], &MINER, &MINER_SLOTS[..], "miner"), (&MACHINES[1], &PUMP, &PUMP_SLOTS[..], "pump")]
    {
        registerBlock(ctx, machine, Rc::new(DiggerTile { m_digger: digger }), true)?;
        energy::add(machine.m_name, digger);

        let mut builder: ScreenBuilder = screen::container(sheet);
        for (slot, [x, y]) in slots
        {
            builder = builder.slot(*slot, *x, *y);
        }

        ctx.screens().register(machine.m_name, builder, DiggerScreen { m_kind: digger.m_kind, m_pos: None })?;
    }

    Ok(())
}
