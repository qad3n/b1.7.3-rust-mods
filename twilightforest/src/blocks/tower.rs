use super::{Cube, Kind, Look, base, drop, texture, tough, vanillaTexture};
use crate::colors;
use crate::names::{self, block};
use b173_api::prelude::*;

mod towerwood
{
    pub const HARDNESS: f32 = 40.0;
    pub const RESISTANCE: f32 = 10.0;
    pub const INFESTED_HARDNESS: f32 = 0.75;
}

mod device
{
    pub const HARDNESS: f32 = 10.0;
    pub const RESISTANCE: f32 = 35.0;
    pub const TRANSLUCENT_HARDNESS: f32 = 50.0;
    pub const TRANSLUCENT_RESISTANCE: f32 = 2000.0;
    pub const DEBRIS_HARDNESS: f32 = 0.3;
    pub const LOCKED_RESISTANCE: f32 = 6000000.0;
    pub const VANISH_RESISTANCE: f32 = 6000.0;
    pub const UNBREAKABLE: f32 = -1.0;
    // ticks a chain reaction waits between neighbours, BlockTFTowerDevice.getTickRateFor
    pub const DELAY: i32 = 15;
    pub const BUILT_DELAY: i32 = 10;
    pub const ACTIVE_BASE: i32 = 2;
    pub const ACTIVE_SPREAD: i32 = 5;
    pub const REAPPEAR_DELAY: i32 = 80;
    pub const BUILDER_DELAY: i32 = 4;
    // a lock block within two blocks keeps vanishing blocks shut
    pub const LOCK_RANGE: i32 = 2;
}

mod light
{
    pub const ACTIVE: f32 = 4.0 / 15.0;
    pub const ANTIBUILDER: f32 = 10.0 / 15.0;
    pub const FULL: f32 = 1.0;
}

const TOWERWOOD: [Cube; 4] = [
    tough(
        block::TOWERWOOD,
        "Towerwood Planks",
        Kind::Wood,
        Look::All("towerwood_planks"),
        towerwood::HARDNESS,
        towerwood::RESISTANCE,
    ),
    tough(
        block::ENCASED_TOWERWOOD,
        "Encased Towerwood",
        Kind::Wood,
        Look::All("towerwood_encased"),
        towerwood::HARDNESS,
        towerwood::RESISTANCE,
    ),
    tough(
        block::CRACKED_TOWERWOOD,
        "Cracked Towerwood",
        Kind::Wood,
        Look::All("towerwood_cracked"),
        towerwood::HARDNESS,
        towerwood::RESISTANCE,
    ),
    tough(
        block::MOSSY_TOWERWOOD,
        "Mossy Towerwood",
        Kind::Wood,
        Look::All("towerwood_mossy"),
        towerwood::HARDNESS,
        towerwood::RESISTANCE,
    ),
];

// every device state the tower blocks pass through, each its own block so hardness and light hold per state
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Device
{
    Reappearing,
    ReappearingActive,
    Vanishing,
    VanishingActive,
    Locked,
    Unlocked,
    Builder,
    BuilderActive,
    BuilderTimeout,
    Antibuilder,
    GhastTrap,
    GhastTrapActive,
    Reactor,
    ReactorActive,
    // the see-through half, BlockTFTowerTranslucent
    Ghost,
    GhostActive,
    Built,
    BuiltActive,
    Antibuilt,
    Debris,
    FalseGold,
    FalseDiamond,
}

struct DeviceRow
{
    m_device: Device,
    m_name: &'static str,
    m_label: &'static str,
    m_texture: &'static str,
    m_hardness: f32,
    m_resistance: f32,
    m_light: f32,
    // the block a broken device drops, BlockTFTowerDevice.damageDropped
    m_drop: Option<&'static str>,
    m_translucent: bool,
}

const fn row(
    device: Device,
    name: &'static str,
    label: &'static str,
    texture: &'static str,
    hardness: f32,
    resistance: f32,
    light: f32,
    drop: Option<&'static str>,
    translucent: bool,
) -> DeviceRow
{
    DeviceRow
    {
        m_device: device,
        m_name: name,
        m_label: label,
        m_texture: texture,
        m_hardness: hardness,
        m_resistance: resistance,
        m_light: light,
        m_drop: drop,
        m_translucent: translucent,
    }
}

const DEVICES: [DeviceRow; 22] = [
    row(
        Device::Reappearing,
        block::REAPPEARING,
        "Reappearing Block",
        "towerdev_reappearing_off",
        device::HARDNESS,
        device::RESISTANCE,
        0.0,
        Some(block::REAPPEARING),
        false,
    ),
    row(
        Device::ReappearingActive,
        "reappearing_block_active",
        "Active Reappearing Block",
        "towerdev_reappearing_on",
        device::UNBREAKABLE,
        device::RESISTANCE,
        light::ACTIVE,
        Some(block::REAPPEARING),
        false,
    ),
    row(
        Device::Vanishing,
        block::VANISHING,
        "Vanishing Block",
        "towerdev_vanish_off",
        device::UNBREAKABLE,
        device::VANISH_RESISTANCE,
        0.0,
        Some(block::VANISHING),
        false,
    ),
    row(
        Device::VanishingActive,
        "vanishing_block_active",
        "Active Vanishing Block",
        "towerdev_vanish_on",
        device::UNBREAKABLE,
        device::RESISTANCE,
        light::ACTIVE,
        Some(block::VANISHING),
        false,
    ),
    row(
        Device::Locked,
        block::LOCKED_VANISHING,
        "Locked Vanishing Block",
        "towerdev_lock_on",
        device::UNBREAKABLE,
        device::LOCKED_RESISTANCE,
        0.0,
        Some(block::LOCKED_VANISHING),
        false,
    ),
    row(
        Device::Unlocked,
        "unlocked_vanishing_block",
        "Unlocked Vanishing Block",
        "towerdev_lock_off",
        device::UNBREAKABLE,
        device::RESISTANCE,
        0.0,
        Some("unlocked_vanishing_block"),
        false,
    ),
    row(
        Device::Builder,
        block::BUILDER,
        "Carminite Builder",
        "towerdev_builder_off",
        device::HARDNESS,
        device::RESISTANCE,
        0.0,
        Some(block::BUILDER),
        false,
    ),
    row(
        Device::BuilderActive,
        "carminite_builder_active",
        "Active Carminite Builder",
        "towerdev_builder_on",
        device::HARDNESS,
        device::RESISTANCE,
        light::ACTIVE,
        Some(block::BUILDER),
        false,
    ),
    row(
        Device::BuilderTimeout,
        "carminite_builder_timeout",
        "Inactive Carminite Builder",
        "towerdev_builder_timeout",
        device::HARDNESS,
        device::RESISTANCE,
        0.0,
        Some(block::BUILDER),
        false,
    ),
    row(
        Device::Antibuilder,
        block::ANTIBUILDER,
        "Anti-builder",
        "towerdev_antibuilder",
        device::HARDNESS,
        device::RESISTANCE,
        light::ANTIBUILDER,
        None,
        false,
    ),
    row(
        Device::GhastTrap,
        block::GHAST_TRAP,
        "Ghast Trap",
        "towerdev_ghasttrap_off",
        device::HARDNESS,
        device::RESISTANCE,
        0.0,
        Some(block::GHAST_TRAP),
        false,
    ),
    row(
        Device::GhastTrapActive,
        "ghast_trap_active",
        "Active Ghast Trap",
        "towerdev_ghasttrap_on",
        device::HARDNESS,
        device::RESISTANCE,
        light::FULL,
        Some(block::GHAST_TRAP),
        false,
    ),
    row(
        Device::Reactor,
        block::REACTOR,
        "Carminite Reactor",
        "towerdev_reactor_off",
        device::HARDNESS,
        device::RESISTANCE,
        0.0,
        Some(block::REACTOR),
        false,
    ),
    row(
        Device::ReactorActive,
        "carminite_reactor_active",
        "Active Carminite Reactor",
        "towerdev_reactor_on",
        device::HARDNESS,
        device::RESISTANCE,
        light::FULL,
        Some(block::REACTOR),
        false,
    ),
    row(
        Device::Ghost,
        "reappearing_ghost",
        "Reappearing Block",
        "towerdev_reappearing_trace_off",
        device::TRANSLUCENT_HARDNESS,
        device::TRANSLUCENT_RESISTANCE,
        0.0,
        None,
        true,
    ),
    row(
        Device::GhostActive,
        "reappearing_ghost_active",
        "Active Reappearing Block",
        "towerdev_reappearing_trace_on",
        device::TRANSLUCENT_HARDNESS,
        device::TRANSLUCENT_RESISTANCE,
        0.0,
        None,
        true,
    ),
    row(
        Device::Built,
        block::BUILT,
        "Built Carminite Block",
        "towerdev_built_off",
        device::TRANSLUCENT_HARDNESS,
        device::TRANSLUCENT_RESISTANCE,
        0.0,
        None,
        true,
    ),
    row(
        Device::BuiltActive,
        "built_block_active",
        "Active Built Carminite Block",
        "towerdev_built_on",
        device::TRANSLUCENT_HARDNESS,
        device::TRANSLUCENT_RESISTANCE,
        0.0,
        None,
        true,
    ),
    row(
        Device::Antibuilt,
        block::ANTIBUILT,
        "Anti-built Block",
        "towerdev_antibuilt",
        device::DEBRIS_HARDNESS,
        device::TRANSLUCENT_RESISTANCE,
        0.0,
        None,
        true,
    ),
    row(
        Device::Debris,
        block::REACTOR_DEBRIS,
        "Carminite Reactor Debris",
        "minecraft:bricks",
        device::DEBRIS_HARDNESS,
        device::TRANSLUCENT_RESISTANCE,
        0.0,
        None,
        true,
    ),
    row(
        Device::FalseGold,
        block::FALSE_GOLD,
        "False Gold",
        "minecraft:gold_block",
        device::TRANSLUCENT_HARDNESS,
        device::TRANSLUCENT_RESISTANCE,
        0.0,
        None,
        true,
    ),
    row(
        Device::FalseDiamond,
        block::FALSE_DIAMOND,
        "False Diamond",
        "minecraft:diamond_block",
        device::TRANSLUCENT_HARDNESS,
        device::TRANSLUCENT_RESISTANCE,
        0.0,
        None,
        true,
    ),
];

fn deviceRow(device: Device) -> &'static DeviceRow
{
    DEVICES.iter().find(|row| row.m_device == device).expect("every device has a row")
}

pub(crate) fn deviceName(device: Device) -> &'static str { deviceRow(device).m_name }

fn deviceAt(world: World, pos: BlockPos) -> Option<Device>
{
    let block: i32 = world.block(pos);
    DEVICES.iter().find(|row| blockIs(block, row.m_name)).map(|row| row.m_device)
}

fn center(pos: BlockPos) -> [f64; 3] { crate::math::center(pos) }

fn click(world: World, pos: BlockPos, volume: f32, pitch: f32)
{
    world.playSound(center(pos), "random.click", volume, pitch);
}

// BlockTFTowerDevice.areNearbyLockBlocks
pub(crate) fn locked(world: World, pos: BlockPos) -> bool
{
    let lock: &str = deviceName(Device::Locked);
    let range: i32 = device::LOCK_RANGE;
    (-range..=range)
        .any(|dx| (-range..=range).any(|dy| (-range..=range).any(|dz| world.is(pos.offset(dx, dy, dz), lock))))
}

fn swap(world: World, pos: BlockPos, to: Device) { let _ = world.place(pos, deviceName(to)); }

fn delayFor(world: World, to: Device) -> i32
{
    match to
    {
        Device::VanishingActive | Device::ReappearingActive =>
        {
            device::ACTIVE_BASE + world.random(device::ACTIVE_SPREAD)
        }

        Device::BuiltActive => device::BUILT_DELAY,
        _ => device::DELAY,
    }
}

// BlockTFTowerDevice.changeToActiveVanishBlock
pub(crate) fn activate(world: World, pos: BlockPos, to: Device)
{
    swap(world, pos, to);
    world.playSound(center(pos), "random.pop", 0.3, 0.6);

    let _ = world.scheduleTick(pos, world.block(pos), delayFor(world, to));
}

// BlockTFTowerDevice.checkAndActivateVanishBlock
fn chain(world: World, pos: BlockPos)
{
    let Some(found) = deviceAt(world, pos)
    else
    {
        return;
    };

    match found
    {
        Device::Vanishing | Device::Unlocked if !locked(world, pos) => activate(world, pos, Device::VanishingActive),
        Device::Reappearing if !locked(world, pos) => activate(world, pos, Device::ReappearingActive),
        Device::Built => activate(world, pos, Device::BuiltActive),
        _ =>
        {}
    }
}

const NEIGHBORS: [[i32; 3]; 6] = [[-1, 0, 0], [1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];

fn chainAround(world: World, pos: BlockPos)
{
    for [dx, dy, dz] in NEIGHBORS
    {
        chain(world, pos.offset(dx, dy, dz));
    }
}

// BlockTFTowerDevice.unlockBlock, a tower key turns a lock
pub(crate) fn unlock(world: World, pos: BlockPos) -> bool
{
    if deviceAt(world, pos) != Some(Device::Locked)
    {
        return false;
    }

    swap(world, pos, Device::Unlocked);
    click(world, pos, 0.3, 0.6);
    true
}

fn reactorReady(world: World, pos: BlockPos) -> bool
{
    NEIGHBORS.iter().all(|[dx, dy, dz]| world.tagged(pos.offset(*dx, *dy, *dz), "c:ores/redstone"))
}

struct DeviceBlock
{
    m_device: Device,
}

impl DeviceBlock
{
    fn neighbor(&self, world: World, pos: BlockPos)
    {
        let powered: bool = world.isPowered(pos);

        match self.m_device
        {
            Device::Vanishing if powered && !locked(world, pos) => activate(world, pos, Device::VanishingActive),
            Device::Reappearing if powered && !locked(world, pos) => activate(world, pos, Device::ReappearingActive),

            Device::Builder if powered =>
            {
                swap(world, pos, Device::BuilderActive);
                click(world, pos, 0.3, 0.6);

                let _ = world.scheduleTick(pos, world.block(pos), device::BUILDER_DELAY);
            }

            Device::BuilderActive if !powered =>
            {
                swap(world, pos, Device::Builder);
                click(world, pos, 0.3, 0.6);

                let _ = world.scheduleTick(pos, world.block(pos), device::BUILDER_DELAY);
            }

            Device::BuilderTimeout if !powered => swap(world, pos, Device::Builder),

            Device::GhastTrap if powered && crate::tower::trapCharged(world, pos) =>
            {
                swap(world, pos, Device::GhastTrapActive);
                click(world, pos, 0.3, 0.6);
            }

            Device::Reactor if reactorReady(world, pos) => swap(world, pos, Device::ReactorActive),
            _ =>
            {}
        }
    }

    fn tick(&self, world: World, pos: BlockPos)
    {
        match self.m_device
        {
            Device::VanishingActive | Device::ReappearingActive | Device::BuiltActive =>
            {
                let ghost: bool = self.m_device == Device::ReappearingActive;
                if ghost
                {
                    swap(world, pos, Device::Ghost);

                    let _ = world.scheduleTick(pos, world.block(pos), device::REAPPEAR_DELAY);
                }
                else
                {
                    let _ = world.clear(pos);
                }
                world.playSound(center(pos), "random.pop", 0.3, 0.5);
                chainAround(world, pos);
            }

            Device::Ghost => activate(world, pos, Device::GhostActive),

            Device::GhostActive =>
            {
                swap(world, pos, Device::Reappearing);
                click(world, pos, 0.3, 0.5);
            }

            Device::BuilderActive if world.isPowered(pos) => crate::tower::startBuilding(world, pos),
            Device::Builder | Device::BuilderTimeout => chainAround(world, pos),
            _ =>
            {}
        }
    }
}

#[block_behavior]
impl BlockBehavior for DeviceBlock
{
    fn onAdded(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority() && self.m_device == Device::Builder && ctx.world().isPowered(ctx.pos())
        {
            swap(ctx.world(), ctx.pos(), Device::BuilderActive);
            click(ctx.world(), ctx.pos(), 0.3, 0.6);
        }
    }

    fn onActivated(&self, ctx: &mut BlockCtx, _player: Player) -> bool
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();

        let target: Device = match self.m_device
        {
            Device::Vanishing => Device::VanishingActive,
            Device::Reappearing => Device::ReappearingActive,
            _ => return false,
        };

        if !ctx.isAuthority()
        {
            return true;
        }

        if locked(world, pos)
        {
            click(world, pos, 1.0, 0.3);
        }
        else
        {
            activate(world, pos, target);
        }
        true
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if ctx.isAuthority()
        {
            self.neighbor(ctx.world(), ctx.pos());
        }
    }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority()
        {
            self.tick(ctx.world(), ctx.pos());
        }
    }

    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        if matches!(self.m_device, Device::VanishingActive | Device::ReappearingActive | Device::BuilderActive)
        {
            crate::particles::sparkle(ctx.world(), ctx.pos(), "reddust");
        }
    }
}

fn deviceSpec(row: &DeviceRow) -> Result<BlockSpecBuilder>
{
    let look: String = match row.m_texture.strip_prefix("minecraft:")
    {
        Some(sprite) => vanillaTexture(sprite),
        None => texture(row.m_texture),
    };
    let mut spec: BlockSpecBuilder = base("minecraft:block", row.m_label, Kind::Wood, row.m_hardness)
        .resistance(row.m_resistance)
        .light(row.m_light)
        .texture(&look)
        .behavior(DeviceBlock { m_device: row.m_device });

    if matches!(row.m_device, Device::GhastTrap | Device::GhastTrapActive)
    {
        let lid: &str = if row.m_device == Device::GhastTrap
        {
            "towerdev_ghasttraplid_off"
        }
        else
        {
            "towerdev_ghasttraplid_on"
        };

        spec = spec.faces(FaceTextures
        {
            top: Some(texture(lid)),
            bottom: Some(texture("towerwood_encased")),
            side: Some(look),
            front: None,
        });
    }

    if row.m_translucent
    {
        spec = spec
            .material("glass")
            .sound("metal")
            .model(BlockModelSpec { translucent: true, ..BlockModelSpec::default() });
    }

    if matches!(row.m_device, Device::Ghost | Device::GhostActive)
    {
        spec = spec.collision(Vec::new());
    }

    spec = match row.m_drop
    {
        Some(name) => spec.drops(vec![drop(name, vec![1], 1.0)?]),
        None => spec.drops(Vec::new()),
    };

    if let Some(tile) = crate::tower::tileFor(row.m_device)
    {
        spec = spec.tileEntity(tile);
    }

    Ok(spec)
}

// BlockTFTowerWood infested planks hatch a termite and drop nothing
struct Infested;

#[block_behavior]
impl BlockBehavior for Infested
{
    fn onDestroyedByPlayer(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority()
        {
            let _ = ctx.world().spawnEntity("tower_termite", center(ctx.pos()));
        }
    }

    fn colorMultiplier(&self, ctx: &mut BlockCtx) -> u32 { colors::towerwood(ctx.pos()) }
}

struct Shaded;

#[block_behavior]
impl BlockBehavior for Shaded
{
    fn colorMultiplier(&self, ctx: &mut BlockCtx) -> u32 { colors::towerwood(ctx.pos()) }
}

// BlockTFFireJet: smokers puff while lit, jets suck lava from below and burst into flame
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Jet
{
    Smoker,
    EncasedSmoker,
    EncasedSmokerOn,
    Idle,
    Popping,
    Flaming,
    EncasedIdle,
    EncasedPopping,
    EncasedFlaming,
}

struct JetRow
{
    m_jet: Jet,
    m_name: &'static str,
    m_label: &'static str,
    m_side: &'static str,
    m_top: &'static str,
    m_light: f32,
    m_drop: &'static str,
    m_tile: Option<&'static str>,
}

const JETS: [JetRow; 9] = [
    JetRow
    {
        m_jet: Jet::Smoker,
        m_name: block::SMOKER,
        m_label: "Smoking Block",
        m_side: "firejet_side",
        m_top: "firejet_top",
        m_light: 0.0,
        m_drop: block::SMOKER,
        m_tile: Some(crate::tower::SMOKER_TILE),
    },
    JetRow
    {
        m_jet: Jet::EncasedSmoker,
        m_name: block::ENCASED_SMOKER,
        m_label: "Encased Smoker",
        m_side: "towerdev_smoker_off",
        m_top: "towerdev_ghasttraplid_off",
        m_light: 0.0,
        m_drop: block::ENCASED_SMOKER,
        m_tile: None,
    },
    JetRow
    {
        m_jet: Jet::EncasedSmokerOn,
        m_name: "encased_smoker_on",
        m_label: "Encased Smoker",
        m_side: "towerdev_smoker_on",
        m_top: "towerdev_ghasttraplid_on",
        m_light: 0.0,
        m_drop: block::ENCASED_SMOKER,
        m_tile: Some(crate::tower::SMOKER_TILE),
    },
    JetRow
    {
        m_jet: Jet::Idle,
        m_name: block::FIRE_JET,
        m_label: "Fire Jet",
        m_side: "firejet_side",
        m_top: "firejet_top",
        m_light: 0.0,
        m_drop: block::FIRE_JET,
        m_tile: None,
    },
    JetRow
    {
        m_jet: Jet::Popping,
        m_name: "fire_jet_popping",
        m_label: "Popping Fire Jet",
        m_side: "firejet_side",
        m_top: "firejet_top",
        m_light: 0.0,
        m_drop: block::FIRE_JET,
        m_tile: Some(crate::tower::POPPING_TILE),
    },
    JetRow
    {
        m_jet: Jet::Flaming,
        m_name: "fire_jet_flaming",
        m_label: "Flaming Fire Jet",
        m_side: "firejet_side",
        m_top: "firejet_top",
        m_light: light::FULL,
        m_drop: block::FIRE_JET,
        m_tile: Some(crate::tower::FLAME_TILE),
    },
    JetRow
    {
        m_jet: Jet::EncasedIdle,
        m_name: block::ENCASED_FIRE_JET,
        m_label: "Encased Fire Jet",
        m_side: "towerdev_firejet_off",
        m_top: "towerdev_ghasttraplid_off",
        m_light: 0.0,
        m_drop: block::ENCASED_FIRE_JET,
        m_tile: None,
    },
    JetRow
    {
        m_jet: Jet::EncasedPopping,
        m_name: "encased_fire_jet_popping",
        m_label: "Encased Popping Fire Jet",
        m_side: "towerdev_firejet_on",
        m_top: "towerdev_ghasttraplid_on",
        m_light: 0.0,
        m_drop: block::ENCASED_FIRE_JET,
        m_tile: Some(crate::tower::POPPING_TILE),
    },
    JetRow
    {
        m_jet: Jet::EncasedFlaming,
        m_name: "encased_fire_jet_flaming",
        m_label: "Encased Flaming Fire Jet",
        m_side: "towerdev_firejet_on",
        m_top: "towerdev_ghasttraplid_on",
        m_light: light::FULL,
        m_drop: block::ENCASED_FIRE_JET,
        m_tile: Some(crate::tower::FLAME_TILE),
    },
];

pub(crate) fn jetName(jet: Jet) -> &'static str
{
    JETS.iter().find(|row| row.m_jet == jet).expect("every jet has a row").m_name
}

fn jetAt(world: World, pos: BlockPos) -> Option<Jet>
{
    let block: i32 = world.block(pos);
    JETS.iter().find(|row| blockIs(block, row.m_name)).map(|row| row.m_jet)
}

// what a popping or flaming jet turns into when its tile runs out, TileEntityTFPoppingJet and TileEntityTFFlameJet
pub(crate) fn nextJet(world: World, pos: BlockPos) -> Option<Jet>
{
    match jetAt(world, pos)?
    {
        Jet::Popping => Some(Jet::Flaming),
        Jet::Flaming => Some(Jet::Idle),
        Jet::EncasedPopping => Some(Jet::EncasedFlaming),
        Jet::EncasedFlaming => Some(Jet::EncasedIdle),
        _ => None,
    }
}

struct JetBlock
{
    m_jet: Jet,
}

fn stillLava(world: World, pos: BlockPos) -> bool { names::source(world, pos, "minecraft:lava") }

// BlockTFFireJet.findLavaAround tries the spot below then three random spots beside it
fn findLava(world: World, below: BlockPos) -> Option<BlockPos>
{
    let tries: i32 = 3;

    if stillLava(world, below)
    {
        return Some(below);
    }

    (0..tries).map(|_| below.offset(world.random(3) - 1, 0, world.random(3) - 1)).find(|pos| stillLava(world, *pos))
}

#[block_behavior]
impl BlockBehavior for JetBlock
{
    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority() || self.m_jet != Jet::Idle
        {
            return;
        }

        let world: World = ctx.world();
        if let Some(lava) = findLava(world, ctx.pos().offset(0, -1, 0))
        {
            let _ = world.clear(lava);
            let _ = names::handle(jetName(Jet::Popping)).place(world, ctx.pos(), false);
        }
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let powered: bool = world.isPowered(ctx.pos());

        let next: Option<Jet> = match self.m_jet
        {
            Jet::EncasedSmoker if powered => Some(Jet::EncasedSmokerOn),
            Jet::EncasedSmokerOn if !powered => Some(Jet::EncasedSmoker),
            Jet::EncasedIdle if powered => Some(Jet::EncasedPopping),
            _ => None,
        };
        if let Some(next) = next
        {
            let _ = world.place(ctx.pos(), jetName(next));

            click(world, ctx.pos(), 0.3, 0.6);
        }
    }

    fn colorMultiplier(&self, ctx: &mut BlockCtx) -> u32
    {
        match self.m_jet
        {
            Jet::Smoker | Jet::Idle | Jet::Popping | Jet::Flaming =>
            {
                crate::world::biomes::grassAround(ctx.world(), ctx.pos())
            }

            _ => 0xFFFFFF,
        }
    }
}

fn registerJets(ctx: &mut RegisterCtx) -> Result<()>
{
    let hardness: f32 = 1.5;

    for row in &JETS
    {
        let mut spec: BlockSpecBuilder = base("minecraft:block", row.m_label, Kind::Rock, hardness)
            .sound("wood")
            .light(row.m_light)
            .tickRandomly()
            .faces(FaceTextures
            {
                top: Some(texture(row.m_top)),
                bottom: Some(texture(
                    if row.m_name.starts_with("encased")
                    {
                        "towerwood_encased"
                    }
                    else
                    {
                        "firejet_side"
                    },
                )),
                side: Some(texture(row.m_side)),
                front: None,
            })
            .drops(vec![drop(row.m_drop, vec![1], 1.0)?])
            .behavior(JetBlock { m_jet: row.m_jet });

        if let Some(tile) = row.m_tile
        {
            spec = spec.tileEntity(tile);
        }

        super::put(ctx, row.m_name, spec)?;
    }

    Ok(())
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &TOWERWOOD
    {
        super::put(ctx, row.m_name, super::build(row).flammable(0, 1).behavior(Shaded))?;
    }

    super::put(
        ctx,
        block::INFESTED_TOWERWOOD,
        base("minecraft:block", "Infested Towerwood", Kind::Wood, towerwood::INFESTED_HARDNESS)
            .resistance(towerwood::RESISTANCE)
            .texture(&texture("towerwood_infested"))
            .flammable(0, 1)
            .drops(Vec::new())
            .behavior(Infested),
    )?;
    crate::tower::registerTiles(ctx)?;

    for row in &DEVICES
    {
        super::put(ctx, row.m_name, deviceSpec(row)?)?;
    }

    registerJets(ctx)
}
