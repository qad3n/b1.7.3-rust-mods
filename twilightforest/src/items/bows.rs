use super::{put, texture};
use crate::entities::arrows::{self, ICE, PLAIN, SEEKER, SIDE};
use crate::names::item;
use b173_api::prelude::*;

// ItemTFBowBase and its triple, seeker, ice and ender bows, drawn while the use key is held and loosed on release
mod bow
{
    pub const DURABILITY: i32 = 384;
    // ticks to a full draw and the least draw that looses an arrow
    pub const FULL_DRAW: f64 = 20.0;
    pub const LEAST_DRAW: f64 = 0.1;
    // a full draw, EntityArrow's 1.5 times the bow's velocity of 2
    pub const SPEED: f64 = 3.0;
    // ItemTFSeekerBow halves the velocity
    pub const SEEKER_SPEED: f64 = 1.5;
    // the triple bow's side arrows climb and dip 0.15 a tick and start 0.025 above and below
    pub const SIDE_LIFT: f64 = 0.15;
    pub const SIDE_RISE: f64 = 0.025;
    pub const SLOTS: i32 = 36;
}

// the three pulling icons of a bow sheet by its name
macro_rules! pulls
{
    ($bow:literal) =>
    {
        [
            concat!("twilightforest:item/", $bow, "_pulling_0"),
            concat!("twilightforest:item/", $bow, "_pulling_1"),
            concat!("twilightforest:item/", $bow, "_pulling_2"),
        ]
    };
}

#[derive(Clone, Copy)]
struct Bow
{
    m_arrow: &'static str,
    m_speed: f64,
    m_triple: bool,
    // ItemTFBowBase's pulling icons, a little, half and fully drawn
    m_pulls: [&'static str; 3],
}

// ItemTFBowBase.getIcon, the pull shown after these many ticks of drawing
const PULL_AFTER: [i32; 3] = [0, 13, 17];

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

fn hasArrow(player: Player) -> bool
{
    (0..bow::SLOTS).any(|slot| player.slot(slot).is_some_and(|stack| stack.item.is("minecraft:arrow")))
}

// ItemBow's draw, a second held is a full draw and short draws hardly fly
fn draw(held: i32) -> f64
{
    let seconds: f64 = f64::from(held) / bow::FULL_DRAW;
    ((seconds * seconds + seconds * 2.0) / 3.0).min(1.0)
}

#[item_behavior]
impl ItemBehavior for Bow
{
    // ItemBow.onItemRightClick, drawing starts only with an arrow to loose
    fn onRightClick(&self, _world: World, player: Player, _stack: ItemStack) -> Use
    {
        if !hasArrow(player)
        {
            return Use::Done(player.held());
        }

        super::magic::holdInUse(player, super::magic::held::LONGEST)
    }

    fn useDuration(&self, _stack: ItemStack) -> i32 { super::magic::held::LONGEST }

    fn useAction(&self, _stack: ItemStack) -> UseAction { UseAction::Bow }

    fn useIcon(&self, _stack: ItemStack, used: i32) -> Option<i32>
    {
        let pull: usize = PULL_AFTER.iter().rposition(|after| used > *after)?;
        b173_api::world::itemSlot(self.m_pulls[pull])
    }

    // ItemTFBowBase.onPlayerStoppedUsing and ItemTFTripleBow's, the arrows fly as fast as the bow was drawn
    fn onStoppedUsing(&self, world: World, player: Player, _stack: ItemStack, left: i32) -> Use
    {
        let power: f64 = draw(super::magic::held::LONGEST - left);
        let Some(info) = player.info().filter(|_| power >= bow::LEAST_DRAW)
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

        let speed: f64 = self.m_speed * power;
        let critical: bool = power == 1.0;
        let mut fired: bool = arrows::shoot(world, self.m_arrow, &info, speed, 0.0, 0.0, critical).is_some();

        if self.m_triple
        {
            for side in [1.0, -1.0]
            {
                let (lift, rise): (f64, f64) = (side * bow::SIDE_LIFT, side * bow::SIDE_RISE);
                fired |= arrows::shoot(world, SIDE, &info, speed, lift, rise, critical).is_some();
            }
        }

        if fired
        {
            let pitch: f32 = 1.0 / (crate::math::fraction(world) * 0.4 + 1.2) + power as f32 * 0.5;
            world.playSound(info.pos, "random.bow", 1.0, pitch);
            super::magic::wearHeld(player, 1);
        }

        Use::Done(player.held())
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let rows: [(&str, &str, &str, Bow); 4] = [
        (
            item::TRIPLE_BOW,
            "Tri-bow",
            "triplebow_standby",
            Bow { m_arrow: PLAIN, m_speed: bow::SPEED, m_triple: true, m_pulls: pulls!("triplebow") },
        ),
        (
            item::SEEKER_BOW,
            "Seeker Bow",
            "seekerbow_standby",
            Bow { m_arrow: SEEKER, m_speed: bow::SEEKER_SPEED, m_triple: false, m_pulls: pulls!("seekerbow") },
        ),
        (
            item::ICE_BOW,
            "Ice Bow",
            "icebow_standby",
            Bow { m_arrow: ICE, m_speed: bow::SPEED, m_triple: false, m_pulls: pulls!("icebow") },
        ),
        // ItemTFEnderBow fires a plain arrow, the swap happens in the hurt listener
        (
            item::ENDER_BOW,
            "Ender Bow",
            "enderbow_standby",
            Bow { m_arrow: PLAIN, m_speed: bow::SPEED, m_triple: false, m_pulls: pulls!("enderbow") },
        ),
    ];

    for (name, label, look, behavior) in rows
    {
        // no item spec names the pulling icons, asking for their slots now gives them room on the item sheet
        for pull in behavior.m_pulls
        {
            let _ = b173_api::world::itemSlot(pull);
        }

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
