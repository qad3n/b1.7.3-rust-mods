use super::{put, texture};
use crate::entities::shots::{self, ARROW, ENDER_ARROW, ICE_ARROW, SEEKER_ARROW};
use crate::ids::item;
use crate::math::look;
use b173_api::prelude::*;

// ItemTFBowBase and its triple, seeker, ice and ender bows; beta's bow fires on the click, so do these, at full draw
mod bow
{
    pub const DURABILITY: i32 = 384;
    pub const SPEED: f64 = 3.0;
    pub const SPREAD: f64 = 1.0;
    // the triple bow's side arrows climb and dip 0.15 a tick
    pub const SIDE_LIFT: f64 = 0.15;
    pub const SLOTS: i32 = 36;
}

#[derive(Clone, Copy)]
struct Bow
{
    m_arrow: &'static str,
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
        let aim: [f64; 3] = look(info.yaw, info.pitch);
        let from: [f64; 3] = info.pos;
        let toward: [f64; 3] = [0, 1, 2].map(|axis| from[axis] + aim[axis]);
        let mut fired: Vec<Entity> =
            shots::fire(world, self.m_arrow, from, toward, bow::SPEED, bow::SPREAD, Some(player.id))
                .into_iter()
                .collect();
        if self.m_triple
        {
            for lift in [bow::SIDE_LIFT, -bow::SIDE_LIFT]
            {
                if let Some(side) =
                    shots::fire(world, self.m_arrow, from, toward, bow::SPEED, bow::SPREAD, Some(player.id))
                    && let Some(motion) = side.info().map(|side| side.motion)
                {
                    let _ = side.setMotion([motion[0], motion[1] + lift, motion[2]]);
                    fired.push(side);
                }
            }
        }
        if !fired.is_empty()
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
        (item::TRIPLE_BOW, "Tri-bow", "triplebow_standby", Bow { m_arrow: ARROW, m_triple: true }),
        (item::SEEKER_BOW, "Seeker Bow", "seekerbow_standby", Bow { m_arrow: SEEKER_ARROW, m_triple: false }),
        (item::ICE_BOW, "Ice Bow", "icebow_standby", Bow { m_arrow: ICE_ARROW, m_triple: false }),
        (item::ENDER_BOW, "Ender Bow", "enderbow_standby", Bow { m_arrow: ENDER_ARROW, m_triple: false }),
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
