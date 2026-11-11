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
    // ItemZaniteSword.level, its attack for each band of wear, beside the source's iron sword the port registers it as
    pub const SWORD_LEVELS: [i32; 5] = [3, 4, 5, 6, 7];
    pub const IRON_SWORD: i32 = 6;
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

// ItemZaniteTool.calculateIncrease: the band of wear a zanite stack is in, the most worn band last
fn zaniteBand(held: &ItemStack) -> Option<usize>
{
    let name: Identifier =
        held.item.name().filter(|name| name.namespace() == "aether" && name.path().starts_with("zanite_"))?;
    let limit: i32 = crate::items::toolUses(name.path())?;
    let left: i32 = limit - held.damage;
    let band: usize = tool::ZANITE_BANDS
        .iter()
        .position(|edge| left <= *edge)
        .map_or(0, |index| tool::ZANITE_LEVELS.len() - 1 - index);
    Some(band)
}

// ItemZaniteSword.getAttributeModifiers: the more worn the sword, the harder a player's own blow with it lands
fn zaniteHit(event: &mut Event)
{
    let EventData::Attack { source: Some(source), .. } = event.data().clone()
    else
    {
        return;
    };
    let Some(band) =
        Player { id: source }.held().filter(|held| held.item.is("zanite_sword")).as_ref().and_then(zaniteBand)
    else
    {
        return;
    };
    if !crate::damage::melee()
    {
        return;
    }

    if let EventData::Attack { amount, .. } = event.dataMut()
    {
        *amount += tool::SWORD_LEVELS[band] - tool::IRON_SWORD;
    }
}

// ItemZaniteTool.calculateIncrease: the more worn the zanite, the faster it digs what it is made for
fn zaniteSpeed(event: &mut Event)
{
    let Some(band) = event.player().and_then(Player::held).as_ref().and_then(zaniteBand)
    else
    {
        return;
    };

    if let EventData::BreakSpeed { speed, .. } = event.dataMut()
        && *speed > 1.0
    {
        *speed = tool::ZANITE_LEVELS[band];
    }
}

pub(crate) fn holystone() -> impl ItemBehavior { Holystone }

pub(crate) fn gravitite() -> impl ItemBehavior { Gravitite }

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<PlayerBreakSpeed>(Priority::Normal, zaniteSpeed)?;
    ctx.events().on::<LivingHurt>(Priority::Normal, zaniteHit)
}
