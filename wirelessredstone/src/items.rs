use crate::blocks::{self, Kind};
use crate::screens;
use b173_api::prelude::*;
use b173_api::world::{Player, World};

pub(crate) const REMOTE: &str = "remote";
pub(crate) const CONFIGURATOR: &str = "power_configurator";
pub(crate) const SNIFFER: &str = "sniffer";
pub(crate) const TRIANGULATOR: &str = "triangulator";
// the remote and triangulator frequency on their stack
pub(crate) const FREQ: &str = "wirelessredstone:freq";

// both tools wear out after 64 uses as their setMaxDamage(64) gave
const USES: i32 = 64;

// the source rounded the player position into block coordinates for a use in the air
fn standing(player: Player) -> Option<BlockPos>
{
    let [x, y, z] = player.info()?.pos;
    Some(BlockPos { x: x.round() as i32, y: y.round() as i32, z: z.round() as i32 })
}

fn open(player: Player, screen: &str, pos: Option<BlockPos>)
{
    if let Err(error) = player.openScreen(screen, pos, Vec::new())
    {
        warn(&format!("open {screen} screen: {error:?}"));
    }
}

// Item.damageItem through the held stack, the reply hands the worn stack back to the game
fn wear(player: Player) -> Use
{
    if let Err(error) = player.damageHeld(1)
    {
        warn(&format!("wear held tool: {error:?}"));
    }

    Use::Done(player.held())
}

// ItemRedstoneWirelessRemote: every use opens the gui at the spot and costs one use
struct Remote;

impl Remote
{
    fn useAt(world: World, player: Player, stack: ItemStack, pos: Option<BlockPos>) -> Use
    {
        if !world.isAuthority()
        {
            return Use::Done(Some(stack));
        }

        open(player, screens::REMOTE, pos);
        wear(player)
    }
}

#[item_behavior]
impl ItemBehavior for Remote
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        Self::useAt(world, player, stack, standing(player))
    }

    fn onUseOnBlock(&self, world: World, player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        Self::useAt(world, player, stack, Some(pos))
    }
}

// ItemRedstoneWirelessPowerDirector: only a receiver opens its gui, at the cost of one use
struct Configurator;

#[item_behavior]
impl ItemBehavior for Configurator
{
    fn onUseOnBlock(&self, world: World, player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        if blocks::kindAt(world, pos) != Some(Kind::Receiver)
        {
            return Use::Pass;
        }

        if !world.isAuthority()
        {
            return Use::Done(Some(stack));
        }

        open(player, screens::CONFIGURATOR, Some(pos));
        wear(player)
    }
}

// ItemRedstoneWirelessSniffer
struct Sniffer;

#[item_behavior]
impl ItemBehavior for Sniffer
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        if world.isAuthority()
        {
            open(player, screens::SNIFFER, None);
        }

        Use::Done(Some(stack))
    }
}

// ItemRedstoneWirelessTriangulator: the gui sets the frequency the needle follows
struct Triangulator;

impl Triangulator
{
    fn useAt(world: World, player: Player, stack: ItemStack) -> Use
    {
        if world.isAuthority()
        {
            open(player, screens::TRIANGULATOR, None);
        }

        Use::Done(Some(stack))
    }
}

#[item_behavior]
impl ItemBehavior for Triangulator
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use { Self::useAt(world, player, stack) }

    fn onUseOnBlock(&self, world: World, player: Player, stack: ItemStack, _pos: BlockPos, _face: i32) -> Use
    {
        Self::useAt(world, player, stack)
    }
}

fn tool(texture: &str, uses: Option<i32>) -> ItemSpecBuilder
{
    let mut spec: ItemSpecBuilder = ItemSpecBuilder::new().texture(texture).maxStack(1);

    if let Some(uses) = uses
    {
        spec = spec.maxDamage(uses);
    }

    // isFull3D, held upright like a tool
    spec.m_spec.full3d = Some(true);
    spec
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.items().register(REMOTE, tool("wirelessredstone:item/remote", Some(USES)).behavior(Remote))?;
    ctx.items()
        .register(CONFIGURATOR, tool("wirelessredstone:item/power_configurator", Some(USES)).behavior(Configurator))?;
    ctx.items().register(SNIFFER, tool("wirelessredstone:item/sniffer", None).behavior(Sniffer))?;
    ctx.items().register(TRIANGULATOR, tool("wirelessredstone:item/triangulator", None).behavior(Triangulator))?;
    Ok(())
}
