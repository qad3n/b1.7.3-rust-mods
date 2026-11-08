use crate::entities::projectile::{
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

// ItemDartShooter.consumeItem: the first stack of the shooter's dart in the main inventory loses one
fn consume(player: Player, name: &str) -> bool
{
    for slot in 0..weapon::MAIN_SLOTS
    {
        let Some(stack) = player.slot(slot).filter(|stack| stack.item.is(name))
        else
        {
            continue;
        };
        let _ = player.setSlot(slot, (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack }));
        return true;
    }
    false
}

pub struct DartShooter
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
pub struct NotchHammer;

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
pub struct LightningKnife;

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

pub struct Sword
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
                    target.damage(weapon::HOLY_DAMAGE, None);
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
                    target.damage(weapon::PIG_SLAIN, Some(attacker));
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

// ItemPhoenixBow: a bow whose arrows fly burning, it uses vanilla arrows from the inventory
pub struct PhoenixBow;

#[item_behavior]
impl ItemBehavior for PhoenixBow
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
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
            return Use::Done(Some(stack));
        }
        world.playSound(info.pos, "random.bow", 1.0, shootPitch(world));
        let (from, toward) = aim(&info);
        projectile::fire(world, FLAMING_ARROW, from, toward, weapon::DART_SPEED, weapon::SPREAD, Some(player.id));
        Use::Done(Some(ItemStack { damage: stack.damage + 1, ..stack }))
    }
}

pub fn blade(name: &str) -> Option<Sword>
{
    BLADES.iter().find(|(blade, _)| *blade == name).map(|(_, kind)| Sword { m_blade: *kind })
}

pub fn shooter(name: &str) -> Option<DartShooter>
{
    SHOOTERS.iter().find(|(item, _)| *item == name).map(|(_, dart)| DartShooter { m_dart: dart })
}

pub fn hammer() -> NotchHammer { NotchHammer }

pub fn knife() -> LightningKnife { LightningKnife }
