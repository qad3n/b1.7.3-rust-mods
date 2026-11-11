use super::{put, texture};
use crate::ids::{self, block, item};
use crate::math::look;
use b173_api::math::{cos, floor, sin};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// the twilight tools that act on the world, ItemTFOreMeter through ItemTFTowerKey
mod reach
{
    // ItemTFCrumbleHorn and ItemTFPeacockFan act on a four block box three blocks ahead of the eyes
    pub const AHEAD: f64 = 3.0;
    pub const RADIUS: f64 = 2.0;
    // ItemTFOreMagnet pulls along a line this long
    pub const MAGNET: f64 = 32.0;
    pub const VEIN: usize = 24;
    // ItemTFLampOfCinders burns thorns this far around the player
    pub const LAMP: i32 = 4;
    // ItemTFOreMeter counts this many chunks around the player's
    pub const METER: i32 = 3;
}

mod wear
{
    pub const HORN: i32 = 1024;
    pub const FAN: i32 = 1024;
    pub const LAMP: i32 = 1024;
    pub const MAGNET: i32 = 12;
}

mod odds
{
    // ItemTFCrumbleHorn crumbles one block in five, and harvests soft blocks one in twenty
    pub const CRUMBLE: i32 = 5;
    pub const HARVEST: i32 = 20;
    // ItemTFPeacockFan picks a flower one time in three
    pub const PICK: i32 = 3;
}

fn eyeBox(info: &EntityInfo) -> ([i32; 3], [i32; 3])
{
    let aim: [f64; 3] = look(info.yaw, info.pitch);
    // a beta player's position is already at its eyes
    let center: [f64; 3] = [0, 1, 2].map(|axis| info.pos[axis] + aim[axis] * reach::AHEAD);
    (center.map(|axis| floor(axis - reach::RADIUS)), center.map(|axis| floor(axis + reach::RADIUS)))
}

fn cells(low: [i32; 3], high: [i32; 3]) -> impl Iterator<Item = BlockPos>
{
    (low[0]..=high[0]).flat_map(move |x|
    {
        (low[1]..=high[1]).flat_map(move |y| (low[2]..=high[2]).map(move |z| BlockPos::new(x, y, z)))
    })
}

pub(crate) fn wearHeld(player: Player, amount: i32)
{
    if amount <= 0
    {
        return;
    }

    if let Err(error) = player.damageHeld(amount)
    {
        warn(&format!("held item wear: {}", error.0));
    }
}

fn swap(world: World, pos: BlockPos, block: i32, meta: i32)
{
    let was: i32 = world.block(pos);

    if let Err(error) = world.setBlock(pos, block, meta)
    {
        warn(&format!("block change at {pos:?}: {}", error.0));
        return;
    }

    if let Err(error) = world.breakEffect(pos, was, world.meta(pos))
    {
        warn(&format!("break effect at {pos:?}: {}", error.0));
    }
}

// ItemTFCrumbleHorn.crumbleBlock, a sound blast that wears stone down to rubble, beta repeats the use while held
fn crumble(world: World, pos: BlockPos) -> i32
{
    let here: i32 = world.block(pos);
    let rolls: [(&str, &str); 3] = [
        ("minecraft:stone", "minecraft:cobblestone"),
        (block::MAZESTONE_BRICK, block::CRACKED_MAZESTONE),
        ("minecraft:cobblestone", "minecraft:gravel"),
    ];

    let mut cost: i32 = 0;

    for (from, to) in rolls
    {
        if here == ids::blockId(from) && world.random(odds::CRUMBLE) == 0
        {
            swap(world, pos, ids::blockId(to), 0);
            cost += 1;
        }
    }

    let soft: bool = here == ids::blockId("minecraft:gravel") || here == ids::blockId("minecraft:dirt");
    if soft && world.random(odds::HARVEST) == 0
    {
        let meta: i32 = world.meta(pos);

        if let Err(error) = world.setBlock(pos, 0, 0)
        {
            warn(&format!("crumble at {pos:?}: {}", error.0));
            return cost;
        }

        if let Err(error) = world.dropBlockItems(pos, here, meta)
        {
            warn(&format!("crumble drops at {pos:?}: {}", error.0));
        }

        if let Err(error) = world.breakEffect(pos, here, meta)
        {
            warn(&format!("break effect at {pos:?}: {}", error.0));
        }

        cost += 1;
    }

    cost
}

// the held items used by holding the use key, ItemTFCrumbleHorn, ItemTFLampOfCinders, ItemTFOreMagnet and the rest
pub(crate) mod held
{
    pub(crate) const LONGEST: i32 = 72000;
    // ItemTFPeacockFan stays raised for a second after each fan
    pub(crate) const FAN: i32 = 20;
    // the horn crumbles every fifth tick once ten have passed, the lamp and magnet act on release past a short hold
    pub(crate) const HORN_EVERY: i32 = 5;
    pub(crate) const HORN_WARMUP: i32 = 10;
    pub(crate) const LAMP_HOLD: i32 = 12;
    pub(crate) const MAGNET_HOLD: i32 = 10;
}

// starts holding the item in use for ticks and keeps the stack in hand
pub(crate) fn holdInUse(player: Player, ticks: i32) -> Use
{
    if let Err(error) = player.setItemInUse(ticks)
    {
        warn(&format!("hold item in use: {}", error.0));
    }

    Use::Done(player.held())
}

// EntityPlayer.stopUsingItem, as when a charged out item lets go by itself
pub(crate) fn letGo(player: Player)
{
    if let Err(error) = player.stopUsingItem()
    {
        warn(&format!("stop using item: {}", error.0));
    }
}

struct CrumbleHorn;

fn bleat(world: World, info: &EntityInfo)
{
    let (volume, pitch): (f32, f32) = (1.0, 0.8);
    world.playSound(info.pos, "mob.sheep", volume, pitch);
}

#[item_behavior]
impl ItemBehavior for CrumbleHorn
{
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        if let Some(info) = player.info()
        {
            bleat(world, &info);
        }

        holdInUse(player, held::LONGEST)
    }

    fn useDuration(&self, _stack: ItemStack) -> i32 { held::LONGEST }

    fn useAction(&self, _stack: ItemStack) -> UseAction { UseAction::Bow }

    // ItemTFCrumbleHorn.onUsingTick, a blast of crumbling every fifth tick of the hold
    fn onUsingTick(&self, world: World, player: Player, _stack: ItemStack, left: i32)
    {
        if left <= held::HORN_WARMUP || left % held::HORN_EVERY != 0 || !world.isAuthority()
        {
            return;
        }

        let Some(info) = player.info()
        else
        {
            return;
        };

        let (low, high): ([i32; 3], [i32; 3]) = eyeBox(&info);
        let crumbled: i32 = cells(low, high).map(|pos| crumble(world, pos)).sum();
        if crumbled > 0
        {
            wearHeld(player, crumbled);
        }

        bleat(world, &info);
    }
}

thread_local!
{
    // ticks left on the jump boost a fan gust in the air gives, the source's Potion.jump
    static GUSTED: RefCell<HashMap<i32, i64>> = RefCell::new(HashMap::new());
}

const GUST_TICKS: i64 = 45;
const FLOWERS: [&str; 2] = ["minecraft:red_flower", "minecraft:yellow_flower"];

// ItemTFPeacockFan, a gust that blows flowers and loose things away or lifts the player in the air
struct PeacockFan;

impl PeacockFan
{
    fn gust(world: World, player: Player, info: &EntityInfo)
    {
        let entity: Entity = Entity { world, id: player.id };
        let now: i64 = world.time();
        let boosted: bool = GUSTED.with(|gusted| gusted.borrow().get(&player.id).is_some_and(|until| *until > now));
        if boosted
        {
            return;
        }

        GUSTED.with(|gusted| gusted.borrow_mut().insert(player.id, now + GUST_TICKS));

        let (push, lift): (f64, f64) = (3.0, 1.5);
        let _ = entity.setMotion([info.motion[0] * push, lift, info.motion[2] * push]);

        entity.setField(EntityField::FallDistance, 0.0);
    }

    fn fan(world: World, info: &EntityInfo)
    {
        let (low, high): ([i32; 3], [i32; 3]) = eyeBox(info);
        for pos in cells(low, high)
        {
            let here: i32 = world.block(pos);
            if FLOWERS.iter().any(|name| here == ids::blockId(name)) && world.random(odds::PICK) == 0
            {
                if let Err(error) = world.dropBlockItems(pos, here, 0)
                {
                    warn(&format!("flower drops at {pos:?}: {}", error.0));
                }

                swap(world, pos, 0, 0);
            }
        }

        let force: f64 = 2.0;
        let aim: [f64; 3] = look(info.yaw, info.pitch);
        let span: f64 = reach::RADIUS * 2.0;
        let center: [f64; 3] = [0, 1, 2].map(|axis| (f64::from(low[axis]) + f64::from(high[axis])) / 2.0);
        for other in world.entitiesNear(center, span)
        {
            if other.id == info.id
            {
                continue;
            }

            let _ = Entity { world, id: other.id }.setMotion(aim.map(|axis| axis * force));
        }
    }
}

#[item_behavior]
impl ItemBehavior for PeacockFan
{
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        let grounded: bool = Entity { world, id: player.id }.flag(EntityField::OnGround);
        if !grounded
        {
            Self::gust(world, player, &info);
        }
        else if world.isAuthority()
        {
            Self::fan(world, &info);
            wearHeld(player, 1);
        }
        else
        {
            let (low, high): ([i32; 3], [i32; 3]) = eyeBox(&info);
            let aim: [f64; 3] = look(info.yaw, info.pitch);
            let puffs: i32 = 30;
            for _ in 0..puffs
            {
                let at: [f64; 3] = [0, 1, 2].map(|axis|
                {
                    f64::from(low[axis]) + f64::from(ids::fraction(world)) * f64::from(high[axis] - low[axis])
                });
                world.particle("explode", at, aim);
            }
        }
        let volume: f32 = 1.0 + ids::fraction(world);
        let pitch: f32 = ids::fraction(world) * 0.7 + 0.3;
        world.playSound(info.pos, "random.breath", volume, pitch);
        holdInUse(player, held::FAN)
    }

    fn useDuration(&self, _stack: ItemStack) -> i32 { held::FAN }

    fn useAction(&self, _stack: ItemStack) -> UseAction { UseAction::Block }
}

fn thorny(world: World, pos: BlockPos) -> bool
{
    let here: i32 = world.block(pos);
    here == ids::blockId(block::BROWN_THORNS) || here == ids::blockId(block::GREEN_THORNS)
}

// ItemTFLampOfCinders.burnBlock, thorns of either kind burn to ash and keep their axis
fn burn(world: World, pos: BlockPos) -> bool
{
    if !thorny(world, pos)
    {
        return false;
    }

    if let Err(error) = world.setBlock(pos, ids::blockId(block::BURNT_THORNS), world.meta(pos))
    {
        warn(&format!("burn thorns at {pos:?}: {}", error.0));
    }

    true
}

const LAMP_SOUND: &str = "mob.ghast.fireball";

// ItemTFLampOfCinders, held to charge and let go to burn every thorn around
struct LampOfCinders;

#[item_behavior]
impl ItemBehavior for LampOfCinders
{
    fn onRightClick(&self, _world: World, player: Player, stack: ItemStack) -> Use
    {
        if stack.damage >= b173_api::maxDamage(&stack.item)
        {
            return Use::Done(player.held());
        }

        holdInUse(player, held::LONGEST)
    }

    fn useDuration(&self, _stack: ItemStack) -> i32 { held::LONGEST }

    fn useAction(&self, _stack: ItemStack) -> UseAction { UseAction::Bow }

    // ItemTFLampOfCinders.onPlayerStoppedUsing, a hold past twelve ticks burns
    fn onStoppedUsing(&self, world: World, player: Player, stack: ItemStack, left: i32) -> Use
    {
        let charged: bool =
            held::LONGEST - left > held::LAMP_HOLD && stack.damage + 1 < b173_api::maxDamage(&stack.item);

        let Some(info) = player.info().filter(|_| charged)
        else
        {
            return Use::Pass;
        };

        let center: [i32; 3] = info.pos.map(floor);

        if world.isAuthority()
        {
            let (volume, pitch): (f32, f32) = (1.5, 0.8);
            world.playSound(info.pos, LAMP_SOUND, volume, pitch);

            for pos in cells(center.map(|axis| axis - reach::LAMP), center.map(|axis| axis + reach::LAMP))
            {
                burn(world, pos);
            }

            crate::progress::grant(player, crate::progress::Gate::Troll);
        }

        let puffs: i32 = 6;
        for _ in 0..puffs
        {
            let at: [f64; 3] = [
                f64::from(center[0] + world.random(reach::LAMP) - world.random(reach::LAMP)) + 0.5,
                f64::from(center[1] + world.random(2)) + 0.5,
                f64::from(center[2] + world.random(reach::LAMP) - world.random(reach::LAMP)) + 0.5,
            ];
            world.particle("flame", at, [0.0; 3]);
            world.particle("smoke", at, [0.0; 3]);
        }

        Use::Done(player.held())
    }

    fn onUseOnBlock(&self, world: World, player: Player, _stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        if !burn(world, pos)
        {
            return Use::Pass;
        }

        let (volume, pitch): (f32, f32) = (0.5, 1.5);
        world.playSound(ids::center(pos), LAMP_SOUND, volume, pitch);

        let spread: f32 = 0.75;

        for _ in 0..10
        {
            let at: [f64; 3] = [pos.x, pos.y, pos.z]
                .map(|axis| f64::from(axis as f32 + 0.5 + (ids::fraction(world) - ids::fraction(world)) * spread));
            world.particle("smoke", at, [0.0; 3]);
            world.particle("flame", at, [0.0; 3]);
        }

        Use::Done(player.held())
    }
}

// ItemTFMagicBeans, a winding beanstalk from uberous soil up to the cloud islands
struct MagicBeans;

mod stalk
{
    pub const CLIMB: i32 = 100;
    // the troll cave clouds this port moves under the beta ceiling, the stalk reaches past them to the top
    pub const CLOUDS: i32 = 120;
    pub const ABOVE_CLOUDS: i32 = 25;
    pub const THICKNESS: f32 = 2.5;
    pub const TAPER: i32 = 5;
}

impl MagicBeans
{
    fn place(world: World, pos: BlockPos) -> bool
    {
        let info: BlockInfo = World::blockInfo(world.block(pos));
        let here: i32 = world.block(pos);
        if here != 0 && !info.replaceable && info.opaque && !ids::leaves(here) && !ids::holdsLeaves(here)
        {
            return false;
        }

        let _ = world.setBlock(pos, ids::blockId(block::HUGE_STALK), 0);

        true
    }

    fn leaf(world: World, pos: BlockPos)
    {
        let here: i32 = world.block(pos);
        if here == 0 || !World::blockInfo(here).opaque
        {
            let _ = world.setBlock(pos, ids::blockId(block::BEANSTALK_LEAVES), 0);
        }
    }

    fn leaves(world: World, pos: BlockPos)
    {
        let _ = world.setBlock(pos, ids::blockId(block::HUGE_STALK), 0);

        for dx in -1..=1
        {
            for dz in -1..=1
            {
                Self::leaf(world, pos.offset(dx, -1, dz));
                Self::leaf(world, pos.offset(dx, 1, dz));
            }
        }

        for dx in -2..=2i32
        {
            for dz in -2..=2i32
            {
                if dx.abs() != 2 || dz.abs() != 2
                {
                    Self::leaf(world, pos.offset(dx, 0, dz));
                }
            }
        }
    }

    // ItemTFMagicBeans.makeHugeStalk
    fn grow(world: World, x: i32, z: i32, low: i32, high: i32)
    {
        let offset: i32 = world.random(100);
        let curl: f32 = ids::fraction(world) * 0.25 + 0.125;
        let swell: f32 = ids::fraction(world) * 0.25 + 0.125;
        let phase = |y: i32, scale: f32| (y + offset) as f32 * scale;
        let start: f32 = 4.0 + sin(phase(low, swell)) * 3.0;
        let x: i32 = (x as f32 - sin(phase(low, curl)) * start) as i32;
        let z: i32 = (z as f32 - cos(phase(low, curl)) * start) as i32;
        let mut nextLeaf: i32 = low + 10 + world.random(20);
        let mut clear: bool = true;
        let mut y: i32 = low;
        while y < high && clear
        {
            let radius: f32 = 5.0 + sin(phase(y, swell)) * 2.5;
            let cx: f32 = x as f32 + sin(phase(y, curl)) * radius;
            let cz: f32 = z as f32 + cos(phase(y, curl)) * radius;

            let thickness: f32 = if high - y < stalk::TAPER
            {
                stalk::THICKNESS * (high - y) as f32 / stalk::TAPER as f32
            }
            else
            {
                stalk::THICKNESS
            };
            let reach: f32 = radius + thickness;
            for dx in floor(f64::from(x as f32 - reach))..(x as f32 + reach).ceil() as i32
            {
                for dz in floor(f64::from(z as f32 - reach))..(z as f32 + reach).ceil() as i32
                {
                    let (ox, oz): (f32, f32) = (dx as f32 - cx, dz as f32 - cz);
                    if ox * ox + oz * oz < thickness * thickness
                    {
                        clear &= Self::place(world, BlockPos::new(dx, y, dz));
                    }
                }
            }

            if y == nextLeaf
            {
                let lx: i32 = (x as f32 + sin(phase(y, curl)) * reach) as i32;
                let lz: i32 = (z as f32 + cos(phase(y, curl)) * reach) as i32;
                Self::leaves(world, BlockPos::new(lx, y, lz));
                nextLeaf = y + 5 + world.random(10);
            }

            y += 1;
        }
    }
}

#[item_behavior]
impl ItemBehavior for MagicBeans
{
    fn onUseOnBlock(&self, world: World, _player: Player, _stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        let top: i32 = crate::world::terrain::height();
        let high: i32 = (pos.y + stalk::CLIMB).max(stalk::CLOUDS + stalk::ABOVE_CLOUDS).min(top);
        if pos.y >= high || world.block(pos) != ids::blockId(block::UBEROUS_SOIL)
        {
            return Use::Pass;
        }

        if world.isAuthority()
        {
            Self::grow(world, pos.x, pos.z, pos.y + 1, high);
        }

        Use::Done(None)
    }
}

// ItemTFTowerKey, turns a locked vanishing block and is used up
struct TowerKey;

#[item_behavior]
impl ItemBehavior for TowerKey
{
    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        if !world.isAuthority() || !crate::blocks::tower::unlock(world, pos)
        {
            return Use::Pass;
        }

        let left: ItemStack = ItemStack { count: stack.count - 1, ..stack };
        Use::Done((left.count > 0).then_some(left))
    }
}

// ItemTFOreMagnet.isOre, anything whose name says ore plus the liveroot
fn ore(block: i32) -> bool
{
    if block == 0
    {
        return false;
    }

    block == ids::blockId(block::LIVEROOT)
        || nameOf(RegistryKind::Block, block).is_some_and(|name| name.path().contains("ore"))
}

// ItemTFOreMagnet.isReplaceable, the ground an ore vein may be pulled into
fn ground(block: i32) -> bool
{
    ["minecraft:dirt", "minecraft:grass", "minecraft:gravel", "minecraft:stone"]
        .iter()
        .any(|name| block == ids::blockId(name))
}

fn vein(world: World, pos: BlockPos, kind: (i32, i32), found: &mut Vec<BlockPos>)
{
    if found.contains(&pos) || found.len() >= reach::VEIN || (world.block(pos), world.meta(pos)) != kind
    {
        return;
    }

    found.push(pos);

    for [dx, dy, dz] in ids::NEIGHBORS
    {
        vein(world, pos.offset(dx, dy, dz), kind, found);
    }
}

fn magnet(world: World, from: [f64; 3], aim: [f64; 3]) -> i32
{
    let to: [f64; 3] = [0, 1, 2].map(|axis| from[axis] + aim[axis] * reach::MAGNET);
    pull(world, from.map(floor), to.map(floor))
}

// ItemTFOreMagnet.doMagnet, the first ore on the line comes to the first ground the line passed
pub(crate) fn pull(world: World, from: [i32; 3], to: [i32; 3]) -> i32
{
    let mut base: Option<BlockPos> = None;
    let mut found: Option<BlockPos> = None;

    for [x, y, z] in crate::world::trees::line(from, to)
    {
        let pos: BlockPos = BlockPos::new(x, y, z);
        let here: i32 = world.block(pos);
        if base.is_none() && ground(here)
        {
            base = Some(pos);
        }

        if ore(here)
        {
            found = Some(pos);
            break;
        }
    }

    let (Some(base), Some(found)) = (base, found)
    else
    {
        return 0;
    };

    let kind: (i32, i32) = (world.block(found), world.meta(found));
    let mut blocks: Vec<BlockPos> = Vec::new();
    vein(world, found, kind, &mut blocks);

    let offset: [i32; 3] = [base.x - found.x, base.y - found.y, base.z - found.z];
    let stone: i32 = ids::blockId("minecraft:stone");
    let mut moved: i32 = 0;

    for pos in blocks
    {
        let target: BlockPos = pos.offset(offset[0], offset[1], offset[2]);
        let there: i32 = world.block(target);
        if there != 0 && !ground(there)
        {
            continue;
        }

        let _ = world.setBlockQuiet(pos, stone, 0);

        world.markBlockForRender(pos);

        let _ = world.setBlockQuiet(target, kind.0, kind.1);

        world.markBlockForRender(target);
        moved += 1;
    }

    moved
}

mod magnet
{
    pub const FLICKER: i32 = 20;
    pub const GLOW: &str = "twilightforest:item/oremagnet1";
    pub const BRIGHT: &str = "twilightforest:item/oremagnet2";
}

// ItemTFOreMagnet, pulls the nearest ore vein out of the rock toward the player, trying eight aims around the look
struct OreMagnet;

const AIMS: [[f32; 2]; 9] = [
    [0.0, 0.0],
    [10.0, 0.0],
    [10.0, 10.0],
    [0.0, 10.0],
    [-10.0, 10.0],
    [-10.0, 0.0],
    [-10.0, -10.0],
    [0.0, -10.0],
    [10.0, -10.0],
];

#[item_behavior]
impl ItemBehavior for OreMagnet
{
    fn onRightClick(&self, _world: World, player: Player, _stack: ItemStack) -> Use { holdInUse(player, held::LONGEST) }

    fn useDuration(&self, _stack: ItemStack) -> i32 { held::LONGEST }

    fn useAction(&self, _stack: ItemStack) -> UseAction { UseAction::Bow }

    // ItemTFOreMagnet.getIcon, it glows after ten ticks and flickers once a second has passed
    fn useIcon(&self, _stack: ItemStack, used: i32) -> Option<i32>
    {
        let glow: &str = match used
        {
            _ if used >= magnet::FLICKER && (used >> 2) % 2 == 0 => magnet::BRIGHT,
            _ if used > held::MAGNET_HOLD => magnet::GLOW,
            _ => return None,
        };
        b173_api::world::itemSlot(glow)
    }

    // ItemTFOreMagnet.onPlayerStoppedUsing, a hold past ten ticks pulls on release
    fn onStoppedUsing(&self, world: World, player: Player, _stack: ItemStack, left: i32) -> Use
    {
        let held: bool = held::LONGEST - left > held::MAGNET_HOLD;
        let Some(info) = player.info().filter(|_| held && world.isAuthority())
        else
        {
            return Use::Pass;
        };

        let moved: i32 = AIMS
            .iter()
            .map(|[yaw, pitch]| magnet(world, info.pos, look(info.yaw + yaw, info.pitch + pitch)))
            .find(|moved| *moved > 0)
            .unwrap_or(0);

        if moved > 0
        {
            wearHeld(player, moved);
            world.playSound(info.pos, "mob.endermen.portal", 1.0, 1.0);
        }

        Use::Done(player.held())
    }
}

// ItemTFOreMeter.countOreInArea, the 2.4.3 meter counts but never speaks, the older meters' report is restored
struct OreMeter;

const METERED: [(&str, &str); 11] = [
    ("minecraft:stone", "Stone"),
    ("minecraft:dirt", "Dirt"),
    ("minecraft:gravel", "Gravel"),
    ("minecraft:coal_ore", "Coal"),
    ("minecraft:iron_ore", "Iron"),
    ("minecraft:gold_ore", "Gold"),
    ("minecraft:diamond_ore", "Diamond"),
    ("minecraft:lapis_ore", "Lapis"),
    ("minecraft:redstone_ore", "Redstone"),
    (block::ROOT, "Roots"),
    (block::LIVEROOT, "Liveroot"),
];

#[item_behavior]
impl ItemBehavior for OreMeter
{
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        let Some(info) = player.info().filter(|_| world.isAuthority())
        else
        {
            return Use::Pass;
        };

        let ids: Vec<i32> = METERED.iter().map(|(name, _)| ids::blockId(name)).collect();
        let mut counts: Vec<i32> = vec![0; ids.len()];
        let (chunkX, chunkZ): (i32, i32) = (floor(info.pos[0]) >> 4, floor(info.pos[2]) >> 4);
        let top: i32 = crate::world::terrain::height();

        for cx in chunkX - reach::METER..=chunkX + reach::METER
        {
            for cz in chunkZ - reach::METER..=chunkZ + reach::METER
            {
                for x in cx * 16..cx * 16 + 16
                {
                    for z in cz * 16..cz * 16 + 16
                    {
                        for y in 0..top
                        {
                            let here: i32 = world.block(BlockPos::new(x, y, z));
                            if let Some(index) = ids.iter().position(|id| *id == here)
                            {
                                counts[index] += 1;
                            }
                        }
                    }
                }
            }
        }

        let total: i32 = counts.iter().sum::<i32>().max(1);

        player.chat(&format!("Ore Meter! Radius {}", reach::METER));

        for ((_, label), count) in METERED.iter().zip(&counts).skip(3)
        {
            let percent: f32 = *count as f32 / total as f32 * 100.0;
            player.chat(&format!("{label} - {count} ({percent:.2}%)"));
        }

        Use::Done(player.held())
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let worn = |label: &str, look: &str, durability: i32|
    {
        ItemSpecBuilder::new().name(label).texture(&texture(look)).maxStack(1).maxDamage(durability)
    };

    put(
        ctx,
        item::ORE_METER,
        ItemSpecBuilder::new().name("Ore Meter [WIP]").texture(&texture("oremeter")).maxStack(1).behavior(OreMeter),
    )?;

    // no item spec names the glowing magnet icons, asking for their slots now gives them room on the item sheet
    for glow in [magnet::GLOW, magnet::BRIGHT]
    {
        let _ = b173_api::world::itemSlot(glow);
    }

    put(ctx, item::ORE_MAGNET, worn("Ore Magnet", "oremagnet", wear::MAGNET).behavior(OreMagnet))?;
    put(ctx, item::CRUMBLE_HORN, worn("Crumble Horn", "crumblehorn", wear::HORN).behavior(CrumbleHorn))?;
    put(ctx, item::PEACOCK_FAN, worn("Peacock Feather Fan", "peacockfan", wear::FAN).behavior(PeacockFan))?;
    put(ctx, item::LAMP_OF_CINDERS, worn("Lamp of Cinders", "lampofcinders", wear::LAMP).behavior(LampOfCinders))?;
    put(
        ctx,
        item::MAGIC_BEANS,
        ItemSpecBuilder::new().name("Magic Beans").texture(&texture("magicbeans")).behavior(MagicBeans),
    )?;
    put(
        ctx,
        item::TOWER_KEY,
        ItemSpecBuilder::new().name("Tower Key").texture(&texture("towerkey")).behavior(TowerKey),
    )?;
    Ok(())
}
