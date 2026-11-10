use super::{put, texture};
use crate::entities::arrows::{self, ICE, PLAIN, SEEKER, SIDE};
use crate::ids::item;
use b173_api::prelude::*;

// ItemTFBowBase and its triple, seeker, ice and ender bows; beta's bow fires on the click, so do these, at full draw
mod bow
{
    pub const DURABILITY: i32 = 384;
    // a full draw, EntityArrow's 1.5 times the bow's velocity of 2
    pub const SPEED: f64 = 3.0;
    // ItemTFSeekerBow halves the velocity
    pub const SEEKER_SPEED: f64 = 1.5;
    // the triple bow's side arrows climb and dip 0.15 a tick and start 0.025 above and below
    pub const SIDE_LIFT: f64 = 0.15;
    pub const SIDE_RISE: f64 = 0.025;
    pub const SLOTS: i32 = 36;
}

#[derive(Clone, Copy)]
struct Bow
{
    m_arrow: &'static str,
    m_speed: f64,
    m_triple: bool,
}

// InventoryPlayer.consumeInventoryItem for one arrow
fn takeArrow(player: Player) -> bool
{
    let Some((slot, stack)) = (0..bow::SLOTS)
        .find_map(|slot| player.slot(slot).filter(|stack| stack.item.is("minecraft:arrow")).map(|stack| (slot, stack)))
    else
    {
        return false;
    };

    let left: Option<ItemStack> = (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack });
    if let Err(error) = player.setSlot(slot, left)
    {
        warn(&format!("bow arrow: {}", error.0));
        return false;
    }
    true
}

#[item_behavior]
impl ItemBehavior for Bow
{
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        if !world.isAuthority()
        {
            return Use::Done(player.held());
        }
        if !takeArrow(player)
        {
            return Use::Pass;
        }
        let mut fired: bool = arrows::shoot(world, self.m_arrow, &info, self.m_speed, 0.0, 0.0).is_some();
        if self.m_triple
        {
            for side in [1.0, -1.0]
            {
                let (lift, rise): (f64, f64) = (side * bow::SIDE_LIFT, side * bow::SIDE_RISE);
                fired |= arrows::shoot(world, SIDE, &info, self.m_speed, lift, rise).is_some();
            }
        }
        if fired
        {
            let pitch: f32 = 1.0 / (crate::ids::fraction(world) * 0.4 + 1.2) + 0.5;
            world.playSound(info.pos, "random.bow", 1.0, pitch);
            super::magic::wearHeld(player, 1);
        }
        Use::Done(player.held())
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let rows: [(&str, &str, &str, Bow); 4] = [
        (item::TRIPLE_BOW, "Tri-bow", "triplebow_standby", Bow { m_arrow: PLAIN, m_speed: bow::SPEED, m_triple: true }),
        (
            item::SEEKER_BOW,
            "Seeker Bow",
            "seekerbow_standby",
            Bow { m_arrow: SEEKER, m_speed: bow::SEEKER_SPEED, m_triple: false },
        ),
        (item::ICE_BOW, "Ice Bow", "icebow_standby", Bow { m_arrow: ICE, m_speed: bow::SPEED, m_triple: false }),
        // ItemTFEnderBow fires a plain arrow, the swap happens in the hurt listener
        (
            item::ENDER_BOW,
            "Ender Bow",
            "enderbow_standby",
            Bow { m_arrow: PLAIN, m_speed: bow::SPEED, m_triple: false },
        ),
    ];
    for (name, label, look, behavior) in rows
    {
        put(
            ctx,
            name,
            ItemSpecBuilder::new()
                .name(label)
                .texture(&texture(look))
                .maxStack(1)
                .maxDamage(bow::DURABILITY)
                .behavior(behavior),
        )?;
    }
    Ok(())
}
