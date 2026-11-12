pub(crate) mod generators;
pub(crate) mod luminator;
pub(crate) mod mining;
pub(crate) mod personal;
pub(crate) mod processing;
pub(crate) mod reactor;
pub(crate) mod screen;
pub(crate) mod special;
pub(crate) mod teleporter;
pub(crate) mod terraformer;

use crate::core::{FLAG, blockTexture, dir, own, sprites, stacks};
use crate::energy::net;
use b173_api::prelude::*;

// TileEntityBlock.facing and .active as block states, so the texture hooks and the client need no tile data
mod state
{
    pub const FACING: &str = "facing";
    pub const ACTIVE: &str = "active";
}

// a machine state's front face, down when it was never turned
pub(crate) fn facing(state: &StateData) -> i32 { state.get(state::FACING).and_then(dir::parse).unwrap_or(dir::DOWN) }

pub(crate) fn active(state: &StateData) -> bool { state.flag(state::ACTIVE) }

// the front face of the machine at pos
pub(crate) fn facingAt(world: World, pos: BlockPos) -> i32
{
    world.stateAt(pos).map_or(dir::DOWN, |state| facing(&state))
}

// TileEntityBlock.active changes redraw the block, as markBlocksDirty
pub(crate) fn setActive(world: World, pos: BlockPos, on: bool)
{
    let Some(state) = world.stateAt(pos)
    else
    {
        return;
    };

    if active(&state) == on
    {
        return;
    }

    if let Err(error) = world.setBlockState(pos, (*state).clone().with(state::ACTIVE, on))
    {
        warn(&format!("the machine at {pos:?} could not change its look: {error}"));
    }
}

pub(crate) fn setFacing(world: World, pos: BlockPos, face: i32)
{
    let Some(state) = world.stateAt(pos)
    else
    {
        return;
    };

    if let Err(error) = world.setBlockState(pos, (*state).clone().with(state::FACING, dir::name(face)))
    {
        warn(&format!("the machine at {pos:?} could not turn: {error}"));
    }
}

// how a block picks its sprite for a face: BlockMultiID, BlockElectric and the wind mill's BlockGenerator rule
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Faces
{
    Machine,
    Storage,
    Axis,
}

// a sprite row of fillSpriteSheet: bottom, top, side and front, then the same four while active
pub(crate) struct Look
{
    pub(crate) m_idle: [&'static str; 4],
    pub(crate) m_active: [&'static str; 4],
    pub(crate) m_faces: Faces,
}

impl Look
{
    fn reserve(&self)
    {
        for sprite in self.m_idle.iter().chain(self.m_active.iter())
        {
            sprites::terrain(&blockTexture(sprite));
        }
    }

    fn slot(&self, index: usize, on: bool) -> i32
    {
        let row: &[&'static str; 4] = if on { &self.m_active } else { &self.m_idle };
        sprites::terrain(&blockTexture(row[index]))
    }

    // BlockMultiID.getBlockTexture and its overrides
    fn world(&self, face: i32, front: i32, on: bool) -> i32
    {
        const BOTTOM: usize = 0;
        const SIDE: usize = 2;
        const FRONT: usize = 3;

        match self.m_faces
        {
            Faces::Storage if face == front => self.slot(FRONT, false),
            Faces::Machine | Faces::Storage if face < 2 => self.slot(BOTTOM + face as usize, on),
            Faces::Axis if face < 2 => self.slot(BOTTOM + face as usize, on),
            Faces::Machine if face == front => self.slot(FRONT, on),
            Faces::Axis if face >> 1 == front >> 1 => self.slot(FRONT, on),
            _ => self.slot(SIDE, on),
        }
    }

    // getBlockTextureFromSideAndMetadata for the item: the front faces south
    fn item(&self, face: i32) -> i32
    {
        const SIDE: usize = 2;
        const FRONT: usize = 3;

        match face
        {
            face if face < 2 => self.slot(face as usize, false),
            dir::SOUTH => self.slot(FRONT, false),
            _ => self.slot(SIDE, false),
        }
    }
}

// an item in hand that the block lets through to its own use instead of opening the screen
pub(crate) struct Machine
{
    pub(crate) m_name: &'static str,
    pub(crate) m_look: Look,
    pub(crate) m_hardness: f32,
    pub(crate) m_screen: bool,
    // TileEntityBlock.wrenchRate, the chance the wrench gives back the block rather than its drop
    pub(crate) m_wrench: f32,
    pub(crate) m_drop: &'static str,
    // an item tag whose held items the block lets through to their own use instead of opening its screen
    pub(crate) m_passes: Option<&'static str>,
    pub(crate) m_slots: i32,
}

struct MachineBlock
{
    m_machine: &'static Machine,
}

#[block_behavior]
impl BlockBehavior for MachineBlock
{
    fn texture(&self, face: i32, _meta: i32) -> i32 { self.m_machine.m_look.item(face) }

    // the facing state also turns the spec's faces, this hook draws the look IC2 gave each family instead
    fn faceTexture(&self, ctx: &mut BlockCtx, face: i32) -> Option<i32>
    {
        let state: Option<std::rc::Rc<StateData>> = ctx.stateAt();
        let front: i32 = state.as_ref().map_or(dir::DOWN, |state| facing(state));
        let on: bool = state.is_some_and(|state| active(&state));
        Some(self.m_machine.m_look.world(face, front, on))
    }

    fn onPlacedBy(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        let Some(info) = entity.info()
        else
        {
            return;
        };
        let face: i32 = match self.m_machine.m_look.m_faces
        {
            Faces::Storage => dir::fromLook(info.yaw, info.pitch),
            _ => dir::fromYaw(info.yaw),
        };
        setFacing(ctx.world(), ctx.pos(), face);
    }

    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        let transmitter: Option<ItemStack> = player.held().filter(|held| held.item.is("ic2:frequency_transmitter"));
        if self.m_machine.m_name == "teleporter"
            && let Some(transmitter) = transmitter
        {
            return ctx.isAuthority() && teleporter::link(ctx.world(), ctx.pos(), player, &transmitter);
        }

        if self.m_machine.m_name == "nuclear_reactor"
        {
            let chamber: bool = player.held().is_some_and(|held| held.item.is("ic2:reactor_chamber"));
            if !chamber && ctx.isAuthority()
            {
                reactor::openScreen(ctx.world(), ctx.pos(), player);
            }

            return !chamber;
        }

        if matches!(self.m_machine.m_name, "personal_safe" | "trade_o_mat")
        {
            return personal::activate(ctx, player, self.m_machine.m_name);
        }

        if !self.m_machine.m_screen
        {
            return false;
        }

        let passes: bool =
            self.m_machine.m_passes.is_some_and(|tag| player.held().is_some_and(|held| held.item.tagged(tag)));
        if passes
        {
            return false;
        }

        if ctx.isAuthority()
            && let Err(error) = player.openScreen(self.m_machine.m_name, Some(ctx.pos()), Vec::new())
        {
            warn(&format!("could not open the {} screen: {error}", self.m_machine.m_name));
        }

        true
    }

    fn onClicked(&self, ctx: &mut BlockCtx, player: Player)
    {
        let transmitter: Option<ItemStack> = player.held().filter(|held| held.item.is("ic2:frequency_transmitter"));
        if self.m_machine.m_name == "teleporter"
            && ctx.isAuthority()
            && let Some(transmitter) = transmitter
        {
            teleporter::tune(ctx.world(), ctx.pos(), player, transmitter);
        }
    }

    fn onAdded(&self, ctx: &mut BlockCtx) { net::changed(ctx.world()); }

    fn onRemoved(&self, ctx: &mut BlockCtx) { net::changed(ctx.world()); }

    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        if self.m_machine.m_name == "nuclear_reactor"
        {
            reactor::smoke(ctx.world(), ctx.pos(), ctx.pos());
            return;
        }

        screen::flames(self.m_machine.m_name, ctx);
    }
}

pub(crate) fn find(name: &str) -> Option<&'static Machine>
{
    generators::MACHINES
        .iter()
        .chain(processing::MACHINES.iter())
        .chain(special::MACHINES.iter())
        .chain(mining::MACHINES.iter())
        .chain(std::iter::once(&terraformer::MACHINE))
        .chain(personal::MACHINES.iter())
        .chain(std::iter::once(&teleporter::MACHINE))
        .chain(std::iter::once(&reactor::MACHINE))
        .chain(crate::energy::storage::MACHINES.iter())
        .find(|row| own(row.m_name) == name)
}

// IWrenchableBlock.wrench for the BlockMultiID family: the failed roll falls back to idDropped with the block's own
// metadata, which every family maps back to the block itself, so the machine always comes back with its inventory
pub(crate) fn dismantle(world: World, pos: BlockPos, machine: &Machine)
{
    stacks::drop(world, pos, &ItemStack::of(own(machine.m_name).as_str(), 1));

    for slot in 0..machine.m_slots
    {
        if let Some(stack) = world.tileSlot(pos, slot)
        {
            stacks::drop(world, pos, &stack);
            stacks::set(world, pos, slot, None);
        }
    }

    let _ = world.clear(pos);
    net::changed(world);
}

pub(crate) fn registerBlock(
    ctx: &mut RegisterCtx,
    machine: &'static Machine,
    tile: impl TileBehavior + 'static,
    sync: bool,
) -> Result<()>
{
    machine.m_look.reserve();
    let tile: Tile = Tile::new(tile).inventory(machine.m_slots).ticks();
    let tile: Tile = if sync { tile.synced() } else { tile };
    ctx.blocks().register(
        machine.m_name,
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("metal")
            .hardness(machine.m_hardness)
            .texture(&blockTexture(machine.m_look.m_idle[2]))
            .tool(ToolClass::Pickaxe, 0)
            .tile(tile)
            .property(Property::values(state::FACING, &dir::NAMES))
            .property(Property::values(state::ACTIVE, &FLAG))
            .drops(vec![DropSpec
            {
                item: Identifier::parse(&own(machine.m_drop))?,
                count: vec![1],
                meta: 0,
                chance: 1.0,
            }])
            .tag("ic2:machines")
            .behavior(MachineBlock { m_machine: machine }),
    )?;
    Ok(())
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    processing::register(ctx)?;
    special::register(ctx)?;
    mining::register(ctx)?;
    terraformer::register(ctx)?;
    personal::register(ctx)?;
    teleporter::register(ctx)?;
    luminator::register(ctx)?;
    reactor::register(ctx)?;
    generators::register(ctx)
}
