use crate::entities::floating_block;
use b173_api::prelude::*;

mod tool
{
    // ItemHolystoneTool drops a shard on six rolls in a hundred
    pub const SHARD_ROLL: i32 = 100;
    pub const SHARD_BELOW: i32 = 6;
    // ItemGravititeTool wears four for every block it lifts
    pub const LIFT_WEAR: i32 = 4;
    // ItemZaniteTool.level, the speed for each band of wear, the most worn band fastest
    pub const ZANITE_LEVELS: [f32; 5] = [2.0, 4.0, 6.0, 8.0, 12.0];
    pub const ZANITE_BANDS: [i32; 4] = [50, 110, 200, 239];
}

// ItemHolystoneTool.onBlockDestroyed
struct Holystone;

#[item_behavior]
impl ItemBehavior for Holystone
{
    fn onBlockDestroyed(&self, _stack: ItemStack, _block: i32, pos: BlockPos, player: Player) -> bool
    {
        let Some(info) = player.info()
        else
        {
            return true;
        };
        let world: World = World { dimension: info.world };
        if world.isAuthority() && world.random(tool::SHARD_ROLL) < tool::SHARD_BELOW
        {
            let _ = world.spawnItem([pos.x as f64, pos.y as f64, pos.z as f64], ItemStack::of("ambrosium_shard", 1));
        }
        true
    }
}

// ItemGravititeTool.onItemUse: a block the tool could mine floats up when there is room above it
struct Gravitite;

#[item_behavior]
impl ItemBehavior for Gravitite
{
    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        let block: i32 = world.block(pos);
        let info: BlockInfo = World::blockInfo(block);
        if block == 0
            || info.hardness < 0.0
            || !world.isAir(pos.offset(0, 1, 0))
            || !world.tileData::<Vec<u8>>(pos).is_empty()
        {
            return Use::Pass;
        }
        if world.isAuthority()
        {
            floating_block::lift(world, pos);
        }
        Use::Done(Some(ItemStack { damage: stack.damage + tool::LIFT_WEAR, ..stack }))
    }
}

// ItemZaniteTool.calculateIncrease: the more worn the zanite, the faster it digs what it is made for
fn zaniteSpeed(event: &mut Event)
{
    let Some(held) = event.player().and_then(Player::held).filter(|held| {
        held.item.name().is_some_and(|name| name.namespace() == "aether" && name.path().starts_with("zanite_"))
    })
    else
    {
        return;
    };
    let Some(limit) = held.item.name().and_then(|name| crate::items::toolUses(name.path()))
    else
    {
        return;
    };
    let left: i32 = limit - held.damage;
    let band: usize = tool::ZANITE_BANDS
        .iter()
        .position(|edge| left <= *edge)
        .map_or(0, |index| tool::ZANITE_LEVELS.len() - 1 - index);
    if let EventData::BreakSpeed { speed, .. } = event.dataMut()
        && *speed > 1.0
    {
        *speed = tool::ZANITE_LEVELS[band];
    }
}

pub fn holystone() -> impl ItemBehavior { Holystone }

pub fn gravitite() -> impl ItemBehavior { Gravitite }

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<PlayerBreakSpeed>(Priority::Normal, zaniteSpeed)
}
