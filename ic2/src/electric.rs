use crate::core::{itemTexture, own, sprites};
use b173_api::prelude::*;

// IChargeableItem: an item that stores EU in its damage, 1 is full and the last point empty
pub(crate) struct Chargeable
{
    pub(crate) m_name: &'static str,
    pub(crate) m_tier: i32,
    pub(crate) m_ratio: i32,
    pub(crate) m_transfer: i32,
    pub(crate) m_rechargeable: bool,
}

const fn charge(name: &'static str, tier: i32, ratio: i32, transfer: i32) -> Chargeable
{
    Chargeable { m_name: name, m_tier: tier, m_ratio: ratio, m_transfer: transfer, m_rechargeable: true }
}

// every IC2 item that takes a charge, batteries first
pub(crate) const CHARGEABLE: [Chargeable; 21] = [
    charge("re_battery", 1, 1, 100),
    charge("energy_crystal", 2, 10, 250),
    charge("lapotron_crystal", 3, 100, 600),
    charge("mining_drill", 1, 50, 100),
    charge("diamond_drill", 1, 80, 80),
    charge("chainsaw", 1, 50, 100),
    charge("od_scanner", 1, 50, 100),
    charge("ov_scanner", 2, 50, 250),
    charge("mining_laser", 2, 100, 250),
    charge("nano_saber", 2, 10, 250),
    charge("nano_saber_active", 2, 10, 250),
    charge("electric_jetpack", 2, 3, 60),
    charge("batpack", 1, 2, 100),
    charge("nano_helmet", 2, 160, 160),
    charge("nano_chestplate", 2, 160, 160),
    charge("nano_leggings", 2, 160, 160),
    charge("nano_boots", 2, 160, 160),
    charge("quantum_helmet", 3, 1000, 1000),
    charge("quantum_chestplate", 3, 1000, 1000),
    charge("quantum_leggings", 3, 1000, 1000),
    charge("quantum_boots", 3, 1000, 1000),
];

// the batteries machines draw from, ItemBattery
const BATTERIES: usize = 3;
const FULL: i32 = 1;
// ItemBattery.setMaxDamage and the emptied damage it checks for
const BATTERY_DAMAGE: i32 = 10002;
const BATTERY_EMPTY: i32 = BATTERY_DAMAGE - FULL;
const DISCHARGED: &str = "re_battery_empty";
const CHARGED: &str = "re_battery";
const ELECTRIC_WRENCH: &str = "electric_wrench";

mod wrench
{
    // ItemToolWrenchElectric.giveEnergyTo: 50 EU a point, five points at most per charge
    pub const PER_POINT: i32 = 50;
    pub const MAX_GAIN: i32 = 5;
}

fn row(stack: &ItemStack) -> Option<&'static Chargeable>
{
    CHARGEABLE.iter().find(|row| stack.item.is(&own(row.m_name)))
}

pub(crate) fn isBattery(stack: &ItemStack) -> bool
{
    CHARGEABLE[..BATTERIES].iter().any(|row| stack.item.is(&own(row.m_name)))
}

pub(crate) fn chargeable(stack: &ItemStack) -> bool
{
    row(stack).is_some() || stack.item.is(&own(DISCHARGED)) || stack.item.is(&own(ELECTRIC_WRENCH))
}

// ElectricItem.giveEnergyTo and its copies: the offer cut to the transfer rate and the room left, in whole points
fn fill(row: &Chargeable, stack: &mut ItemStack, offer: i32, tier: i32) -> i32
{
    if !row.m_rechargeable || tier < row.m_tier || stack.damage == FULL
    {
        return 0;
    }

    let need: i32 = (stack.damage - FULL) * row.m_ratio;
    let mut given: i32 = if row.m_transfer != 0
    {
        offer.min(row.m_transfer)
    }
    else
    {
        offer
    };
    given = given.min(need);
    given -= given.rem_euclid(row.m_ratio);
    stack.damage -= given / row.m_ratio;
    given
}

// IChargeableItem.giveEnergyTo for any IC2 item, the stack may change into its charged form; the EU it took
pub(crate) fn give(stack: &mut ItemStack, offer: i32, tier: i32) -> i32
{
    if stack.item.is(&own(ELECTRIC_WRENCH))
    {
        return giveWrench(stack, offer, tier);
    }

    // ItemBatteryDischarged turns into an empty RE-Battery that takes the charge, one at a time
    if stack.item.is(&own(DISCHARGED))
    {
        if offer <= 0 || stack.count != 1
        {
            return 0;
        }

        *stack = ItemStack::of(own(CHARGED).as_str(), 1).withDamage(BATTERY_EMPTY);
    }

    row(stack).map_or(0, |row| fill(row, stack, offer, tier))
}

fn giveWrench(stack: &mut ItemStack, offer: i32, tier: i32) -> i32
{
    if tier < 1
    {
        return 0;
    }

    let gain: i32 = (offer / wrench::PER_POINT).min((stack.damage - FULL).min(wrench::MAX_GAIN)).max(0);
    stack.damage -= gain;
    gain * wrench::PER_POINT
}

// ItemBattery.getEnergyFrom: a battery gives up to its transfer rate, an emptied RE-Battery turns discharged
pub(crate) fn take(stack: &mut ItemStack, tier: i32) -> i32
{
    let Some(row) = CHARGEABLE[..BATTERIES].iter().find(|row| stack.item.is(&own(row.m_name)))
    else
    {
        return 0;
    };

    let last: i32 = BATTERY_EMPTY;
    if tier < row.m_tier || stack.damage == last
    {
        return 0;
    }

    let mut energy: i32 = (last - stack.damage) * row.m_ratio;
    if row.m_transfer != 0 && row.m_transfer < energy
    {
        energy = row.m_transfer;
    }

    energy -= energy.rem_euclid(row.m_ratio);
    stack.damage += energy / row.m_ratio;

    if row.m_name == CHARGED && stack.damage == last
    {
        *stack = ItemStack::of(own(DISCHARGED).as_str(), 1);
    }

    energy
}

// ElectricItem.use and ItemElectricTool.use: wears the charge, false when it runs dry
pub(crate) fn use_(stack: &mut ItemStack, points: i32) -> bool
{
    let max: i32 = maxDamage(&stack.item);
    if stack.damage + points > max - FULL
    {
        stack.damage = max - FULL;
        return false;
    }

    stack.damage += points;
    true
}

pub(crate) fn charged(stack: &ItemStack) -> bool { stack.damage < maxDamage(&stack.item) - FULL }

// ItemBattery.getIconFromDamage: five pictures from empty to full
struct BatteryIcon
{
    m_name: &'static str,
}

mod icon
{
    pub const STEPS: [i32; 4] = [7501, 5001, 2501, 1];
}

thread_local!
{
    // ItemBattery.soundTicker, the hum while an RE-Battery is held down
    static HUM: std::cell::Cell<i32> = const { std::cell::Cell::new(0) };
}

#[item_behavior]
impl ItemBehavior for BatteryIcon
{
    fn icon(&self, damage: i32) -> i32
    {
        let level: usize = icon::STEPS.iter().filter(|step| damage <= **step).count();
        sprites::item(&itemTexture(&format!("{}_{level}", self.m_name)))
    }

    // ItemBattery.onItemRightClick: an RE-Battery held down tops up the electric things in the hotbar
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        const HOTBAR: i32 = 9;
        const HUM_EVERY: i32 = 32;

        if self.m_name != CHARGED
        {
            return Use::Pass;
        }

        let ticks: i32 = HUM.with(|hum|
        {
            hum.set(hum.get() + 1);
            hum.get()
        });
        if ticks % HUM_EVERY == 0
            && let Some(info) = player.info()
        {
            world.playSound(info.pos, &crate::core::sound("battery"), 1.0, 1.0);
        }

        let mut energy: i32 = BATTERY_EMPTY - stack.damage;
        let mut skipped: bool = false;

        for slot in 0..HOTBAR
        {
            let Some(mut other) = player.slot(slot)
            else
            {
                continue;
            };

            // the battery in hand never charges itself
            let itself: bool = !skipped && other == stack;
            skipped |= itself;
            if itself || !chargeable(&other)
            {
                continue;
            }

            energy -= give(&mut other, energy, 1);
            energy -= give(&mut other, energy, 1);
            let _ = player.setSlot(slot, Some(other));

            if energy <= 0
            {
                break;
            }
        }

        if energy <= 0
        {
            return Use::Done(Some(ItemStack::of(own(DISCHARGED).as_str(), 1)));
        }

        Use::Done(Some(ItemStack { damage: BATTERY_EMPTY - energy, ..stack }))
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    const LEVELS: i32 = 5;
    const DISCHARGED_STACK: i32 = 16;

    for row in &CHARGEABLE[..BATTERIES]
    {
        for level in 0..LEVELS
        {
            sprites::item(&itemTexture(&format!("{}_{level}", row.m_name)));
        }

        ctx.items().register(
            row.m_name,
            ItemSpecBuilder::new()
                .texture(&itemTexture(&format!("{}_4", row.m_name)))
                .maxStack(1)
                .maxDamage(BATTERY_DAMAGE)
                .tag("ic2:batteries")
                .behavior(BatteryIcon { m_name: row.m_name }),
        )?;
    }

    ctx.items().register(
        DISCHARGED,
        ItemSpecBuilder::new().texture(&itemTexture("re_battery_0")).maxStack(DISCHARGED_STACK),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests
{
    use super::*;

    fn stack(name: &str, damage: i32) -> ItemStack { ItemStack::of(own(name).as_str(), 1).withDamage(damage) }

    #[test]
    fn anEmptyReBatteryTakesUpToItsTransferRate()
    {
        let mut battery: ItemStack = stack("re_battery", BATTERY_EMPTY);
        assert_eq!(give(&mut battery, 32, 1), 32);
        assert_eq!(battery.damage, BATTERY_EMPTY - 32);
        assert_eq!(give(&mut battery, 512, 1), 100);
    }

    #[test]
    fn crystalsNeedAHigherTierAndChargeInWholePoints()
    {
        let mut crystal: ItemStack = stack("energy_crystal", BATTERY_EMPTY);
        assert_eq!(give(&mut crystal, 512, 1), 0);
        assert_eq!(give(&mut crystal, 512, 2), 250);
        assert_eq!(crystal.damage, BATTERY_EMPTY - 25);

        let mut lapotron: ItemStack = stack("lapotron_crystal", BATTERY_EMPTY);
        assert_eq!(give(&mut lapotron, 99, 3), 0);
    }

    #[test]
    fn aFullItemTakesNothing()
    {
        let mut drill: ItemStack = stack("mining_drill", FULL);
        assert_eq!(give(&mut drill, 100, 3), 0);
    }

    #[test]
    fn aDischargedBatteryBecomesAChargingReBatteryOneAtATime()
    {
        let mut single: ItemStack = stack("re_battery_empty", 0);
        assert_eq!(give(&mut single, 40, 1), 40);
        assert!(single.item.is("ic2:re_battery"));
        assert_eq!(single.damage, BATTERY_EMPTY - 40);

        let mut pile: ItemStack = ItemStack::of("ic2:re_battery_empty", 2);
        assert_eq!(give(&mut pile, 40, 1), 0);
    }

    #[test]
    fn machinesDrawABatteryAtItsTransferRateUntilItIsDischarged()
    {
        let mut battery: ItemStack = stack("re_battery", FULL);
        assert_eq!(take(&mut battery, 1), 100);
        assert_eq!(battery.damage, FULL + 100);

        let mut last: ItemStack = stack("re_battery", BATTERY_EMPTY - 1);
        assert_eq!(take(&mut last, 1), 1);
        assert!(last.item.is("ic2:re_battery_empty"));

        let mut crystal: ItemStack = stack("energy_crystal", FULL);
        assert_eq!(take(&mut crystal, 1), 0);
        assert_eq!(take(&mut crystal, 2), 250);
    }

    #[test]
    fn theElectricWrenchTakesFiftyEuAPointFiveAtMost()
    {
        let mut wrench: ItemStack = stack("electric_wrench", 100);
        assert_eq!(give(&mut wrench, 120, 1), 100);
        assert_eq!(wrench.damage, 98);
        assert_eq!(give(&mut wrench, 1000, 1), 250);
        assert_eq!(give(&mut wrench, 1000, 0), 0);
    }
}
