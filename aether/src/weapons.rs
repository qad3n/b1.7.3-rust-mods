use crate::entities::projectile::
{
    self, ENCHANTED_DART, FLAMING_ARROW, GOLDEN_DART, HAMMER, LIGHTNING_KNIFE, POISON_DART,
};
use crate::entities::{fraction, lookOf};
use b173_api::prelude::*;

mod weapon
{
    // ItemDartShooter, ItemNotchHammer and ItemLightningKnife
    pub const DART_SPEED: f64 = 1.5;
    pub const THROW_SPEED: f64 = 1.5;
    pub const SPREAD: f64 = 1.0;
    pub const MAIN_SLOTS: i32 = 36;
    // ItemElementalSword: flaming sets thirty ticks of fire, holy does fifteen more to the undead
    pub const FLAME_TICKS: f64 = 30.0;
    pub const HOLY_DAMAGE: i32 = 15;
    pub const PIG_SLAIN: i32 = 9999;
    pub const VAMPIRE_HEAL: i32 = 1;
    pub const HAMMER_WEAR: i32 = 1;
    pub const SHOOT_PITCH_LOW: f64 = 0.8;
    pub const SHOOT_PITCH_SPREAD: f64 = 0.4;
}

mod bow
{
    // ItemPhoenixBow.getMaxItemUseDuration, drawn until the use key is let go
    pub const USE_TICKS: i32 = 72000;
    // ItemPhoenixBow.getArrowVelocity is full after twenty ticks and a draw under a tenth looses nothing
    pub const FULL_DRAW: f64 = 20.0;
    pub const LEAST_DRAW: f64 = 0.1;
    // EntityArrow flies one and a half times the bow's velocity of twice the draw
    pub const SPEED: f64 = 3.0;
    pub const PITCH_LOW: f64 = 1.2;
    pub const PITCH_DRAW: f32 = 0.5;
}

// ItemPhoenixBow.getIcon, the pulling icon shown after these many ticks of drawing
const PULLS: [(i32, &str); 3] = [
    (0, "aether:item/weapons/bow_pulling_0"),
    (13, "aether:item/weapons/bow_pulling_1"),
    (17, "aether:item/weapons/bow_pulling_2"),
];

// a point a block ahead of the player's eyes along its look, where its shots leave from
fn aim(info: &EntityInfo) -> ([f64; 3], [f64; 3])
{
    let look: [f64; 3] = lookOf(info);
    let eyes: [f64; 3] = info.pos;
    (eyes, [eyes[0] + look[0], eyes[1] + look[1], eyes[2] + look[2]])
}

fn shootPitch(world: World) -> f32
{
    1.0 / (fraction(world) * weapon::SHOOT_PITCH_SPREAD + weapon::SHOOT_PITCH_LOW) as f32
}

// the first stack of an item in the player's main inventory
fn find(player: Player, name: &str) -> Option<(i32, ItemStack)>
{
    (0..weapon::MAIN_SLOTS)
        .find_map(|slot| player.slot(slot).filter(|stack| stack.item.is(name)).map(|stack| (slot, stack)))
}

// ItemDartShooter.consumeItem: the first stack of the shooter's dart in the main inventory loses one
fn consume(player: Player, name: &str) -> bool
{
    let Some((slot, stack)) = find(player, name)
    else
    {
        return false;
    };

    let _ = player.setSlot(slot, (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack }));

    true
}

// ItemPhoenixBow.getArrowVelocity, a second held is a full draw and short draws hardly fly
fn draw(held: i32) -> f64
{
    let seconds: f64 = f64::from(held) / bow::FULL_DRAW;
    ((seconds * seconds + seconds * 2.0) / 3.0).min(1.0)
}

pub(crate) struct DartShooter
{
    m_dart: &'static str,
}

#[item_behavior]
impl ItemBehavior for DartShooter
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        if !world.isAuthority()
        {
            return Use::Done(Some(stack));
        }
        let Some(info) = player.info().filter(|_| consume(player, self.m_dart))
        else
        {
            return Use::Done(Some(stack));
        };

        world.playSound(info.pos, "aether.projectile.dart_shooter.shoot", 1.0, shootPitch(world));

        let (from, toward) = aim(&info);
        projectile::fire(world, self.m_dart, from, toward, weapon::DART_SPEED, weapon::SPREAD, Some(player.id));
        Use::Done(Some(stack))
    }
}

// ItemNotchHammer: a shockwave hammer that wears with every swing
pub(crate) struct NotchHammer;

#[item_behavior]
impl ItemBehavior for NotchHammer
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        world.playSound(info.pos, "mob.ghast.fireball", 1.0, shootPitch(world));

        if world.isAuthority()
        {
            let (from, toward) = aim(&info);
            projectile::fire(world, HAMMER, from, toward, weapon::THROW_SPEED, weapon::SPREAD, Some(player.id));
        }

        Use::Done(Some(ItemStack { damage: stack.damage + weapon::HAMMER_WEAR, ..stack }))
    }
}

// ItemLightningKnife: thrown one at a time
pub(crate) struct LightningKnife;

#[item_behavior]
impl ItemBehavior for LightningKnife
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        world.playSound(info.pos, "random.bow", 1.0, shootPitch(world));

        if world.isAuthority()
        {
            let (from, toward) = aim(&info);
            projectile::fire(
                world,
                LIGHTNING_KNIFE,
                from,
                toward,
                weapon::THROW_SPEED,
                weapon::SPREAD,
                Some(player.id),
            );
        }

        Use::Done((stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack }))
    }
}

fn kindOf(target: Entity) -> String
{
    target.info().and_then(|info| info.kind).map_or_else(String::new, |kind| kind.path().to_owned())
}

// EntityLiving.isEntityUndead in b1.7.3: zombies, skeletons and zombie pigmen
const UNDEAD: [&str; 3] = ["zombie", "skeleton", "zombie_pigman"];
// ItemPigSlayer: anything pig-like by name
const PIGS: [&str; 3] = ["pig", "phyg", "taegore"];

#[derive(Clone, Copy)]
enum Blade
{
    Flaming,
    Lightning,
    Holy,
    Vampire,
    PigSlayer,
    CandyCane,
}

pub(crate) struct Sword
{
    m_blade: Blade,
}

#[item_behavior]
impl ItemBehavior for Sword
{
    fn onHitEntity(&self, _stack: ItemStack, target: Entity, attacker: i32) -> bool
    {
        let world: World = target.world;
        if !world.isAuthority()
        {
            return true;
        }

        match self.m_blade
        {
            Blade::Flaming => target.setField(EntityField::FireTicks, weapon::FLAME_TICKS),

            Blade::Lightning =>
            {
                if let Some(info) = target.info()
                {
                    let _ = world.lightning(info.pos);
                }
            }

            Blade::Holy =>
            {
                if UNDEAD.contains(&kindOf(target).as_str())
                {
                    crate::damage::hit(target, weapon::HOLY_DAMAGE, None, crate::damage::kind::DROWN);
                }
            }

            Blade::Vampire =>
            {
                let wielder: Entity = Entity { world, id: attacker };
                let health: i32 = wielder.field(EntityField::Health) as i32;
                if health > 0 && health < wielder.field(EntityField::MaxHealth) as i32
                {
                    wielder.setField(EntityField::Health, (health + weapon::VAMPIRE_HEAL) as f64);
                }
            }

            Blade::PigSlayer =>
            {
                let kind: String = kindOf(target);
                if PIGS.iter().any(|pig| kind.contains(pig))
                {
                    crate::damage::hit(target, weapon::PIG_SLAIN, Some(attacker), crate::damage::kind::MOB);
                }
            }

            Blade::CandyCane =>
            {
                if world.random(2) == 0
                    && let Some(info) = target.info()
                {
                    let _ = world.spawnItem(info.pos, ItemStack::of("candy_cane", 1));
                }
            }
        }

        true
    }
}

const BLADES: [(&str, Blade); 6] = [
    ("flaming_sword", Blade::Flaming),
    ("lightning_sword", Blade::Lightning),
    ("holy_sword", Blade::Holy),
    ("vampire_blade", Blade::Vampire),
    ("pig_slayer", Blade::PigSlayer),
    ("candy_cane_sword", Blade::CandyCane),
];

const SHOOTERS: [(&str, &str); 3] = [
    ("golden_dart_shooter", GOLDEN_DART),
    ("poison_dart_shooter", POISON_DART),
    ("enchanted_dart_shooter", ENCHANTED_DART),
];

// ItemPhoenixBow: a bow whose arrows fly burning, drawn while held and loosed on release with vanilla arrows
pub(crate) struct PhoenixBow;

#[item_behavior]
impl ItemBehavior for PhoenixBow
{
    // ItemPhoenixBow.onItemRightClick: drawing starts only with an arrow to loose
    fn onRightClick(&self, _world: World, player: Player, stack: ItemStack) -> Use
    {
        if find(player, "minecraft:arrow").is_none()
        {
            return Use::Done(Some(stack));
        }

        if let Err(error) = player.setItemInUse(bow::USE_TICKS)
        {
            warn(&format!("phoenix bow draw: {error}"));
        }

        Use::Done(Some(stack))
    }

    fn useDuration(&self, _stack: ItemStack) -> i32 { bow::USE_TICKS }

    fn useAction(&self, _stack: ItemStack) -> UseAction { UseAction::Bow }

    fn useIcon(&self, _stack: ItemStack, used: i32) -> Option<i32>
    {
        let (_, icon) = PULLS.iter().rev().find(|(after, _)| used > *after)?;
        b173_api::world::itemSlot(icon)
    }

    // ItemPhoenixBow.onPlayerStoppedUsing: the flaming arrow flies as fast as the bow was drawn
    fn onStoppedUsing(&self, world: World, player: Player, stack: ItemStack, left: i32) -> Use
    {
        let power: f64 = draw(bow::USE_TICKS - left);
        let Some(info) = player.info().filter(|_| power >= bow::LEAST_DRAW)
        else
        {
            return Use::Pass;
        };

        if !world.isAuthority()
        {
            return Use::Done(Some(stack));
        }
        if !consume(player, "minecraft:arrow")
        {
            return Use::Pass;
        }

        let (from, toward) = aim(&info);
        projectile::fire(world, FLAMING_ARROW, from, toward, bow::SPEED * power, weapon::SPREAD, Some(player.id));

        let pitch: f32 = 1.0 / (fraction(world) * weapon::SHOOT_PITCH_SPREAD + bow::PITCH_LOW) as f32
            + power as f32 * bow::PITCH_DRAW;
        world.playSound(info.pos, "random.bow", 1.0, pitch);
        Use::Done(Some(ItemStack { damage: stack.damage + 1, ..stack }))
    }
}

// no item spec names the pulling icons, asking for their slots now gives them room on the item sheet
pub(crate) fn reservePulls()
{
    for (_, icon) in PULLS
    {
        let _ = b173_api::world::itemSlot(icon);
    }
}

pub(crate) fn blade(name: &str) -> Option<Sword>
{
    BLADES.iter().find(|(blade, _)| *blade == name).map(|(_, kind)| Sword { m_blade: *kind })
}

pub(crate) fn shooter(name: &str) -> Option<DartShooter>
{
    SHOOTERS.iter().find(|(item, _)| *item == name).map(|(_, dart)| DartShooter { m_dart: dart })
}

pub(crate) fn hammer() -> NotchHammer { NotchHammer }

pub(crate) fn knife() -> LightningKnife { LightningKnife }
