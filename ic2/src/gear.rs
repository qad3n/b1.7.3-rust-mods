use crate::core::{itemTexture, own, sound};
use crate::electric;
use crate::items::{armor, wear};
use crate::keys::{self, Key};
use b173_api::prelude::*;

// the armor slots of the player's inventory, boots first as InventoryPlayer.armorInventory
mod slot
{
    pub const FEET: i32 = 36;
    pub const LEGS: i32 = 37;
    pub const CHEST: i32 = 38;
    pub const HEAD: i32 = 39;
    pub const ALL: [i32; 4] = [FEET, LEGS, CHEST, HEAD];
}

// one IC2 armor piece: its slot on the body, its layer sheet and its durability or charge
struct Piece
{
    m_name: &'static str,
    m_slot: usize,
    m_layer: &'static str,
    m_durability: i32,
}

// ItemArmorRubBoots, ItemArmorJetpack, ItemArmorBatpack, ItemArmorNanoSuit and ItemArmorQuantumSuit
const PIECES: [Piece; 12] = [
    Piece { m_name: "rubber_boots", m_slot: 3, m_layer: "rubber", m_durability: 64 },
    Piece { m_name: "jetpack", m_slot: 1, m_layer: "jetpack", m_durability: 18002 },
    Piece { m_name: "electric_jetpack", m_slot: 1, m_layer: "jetpack", m_durability: 10002 },
    Piece { m_name: "batpack", m_slot: 1, m_layer: "batpack", m_durability: 30002 },
    Piece { m_name: "nano_helmet", m_slot: 0, m_layer: "nano", m_durability: 252 },
    Piece { m_name: "nano_chestplate", m_slot: 1, m_layer: "nano", m_durability: 252 },
    Piece { m_name: "nano_leggings", m_slot: 2, m_layer: "nano", m_durability: 252 },
    Piece { m_name: "nano_boots", m_slot: 3, m_layer: "nano", m_durability: 252 },
    Piece { m_name: "quantum_helmet", m_slot: 0, m_layer: "quantum", m_durability: 1002 },
    Piece { m_name: "quantum_chestplate", m_slot: 1, m_layer: "quantum", m_durability: 1002 },
    Piece { m_name: "quantum_leggings", m_slot: 2, m_layer: "quantum", m_durability: 1002 },
    Piece { m_name: "quantum_boots", m_slot: 3, m_layer: "quantum", m_durability: 1002 },
];

fn worn(player: Player, slot: i32, name: &str) -> Option<ItemStack>
{
    player.slot(slot).filter(|stack| stack.item.is(&own(name)))
}

pub(crate) fn isNano(stack: &ItemStack) -> bool
{
    PIECES
        .iter()
        .filter(|piece| piece.m_layer == "nano" || piece.m_layer == "quantum")
        .any(|piece| stack.item.is(&own(piece.m_name)))
}

fn isQuantum(stack: &ItemStack) -> bool
{
    PIECES.iter().filter(|piece| piece.m_layer == "quantum").any(|piece| stack.item.is(&own(piece.m_name)))
}

fn charged(stack: &ItemStack) -> bool { stack.damage < maxDamage(&stack.item) - 1 }

fn put(player: Player, slot: i32, stack: Option<ItemStack>)
{
    if let Err(error) = player.setSlot(slot, stack)
    {
        warn(&format!("could not update worn IC2 armor: {error}"));
    }
}

// ItemArmorRubBoots.multiplyFall: shortens the fall a thing has built up
pub(crate) fn multiplyFall(entity: Entity, factor: f32)
{
    let fall: f64 = entity.field(EntityField::FallDistance);
    entity.setField(EntityField::FallDistance, fall * f64::from(factor));
    if let Some(store) = STORED.get(entity)
    {
        crate::core::store(&STORED, entity, &Some(store * factor));
    }
}

// ItemArmorBatpack.useBatpackOn: tops up a held electric tool from a worn batpack
pub(crate) fn batpackCharge(player: Player, tool: &mut ItemStack)
{
    const RATIO: i32 = 2;
    const TIER: i32 = 1;

    let Some(pack) = worn(player, slot::CHEST, "batpack")
    else
    {
        return;
    };

    if !electric::chargeable(tool)
    {
        return;
    }

    let last: i32 = maxDamage(&pack.item) - 1;
    let mut energy: i32 = (last - pack.damage) * RATIO;
    energy -= electric::give(tool, energy, TIER);
    put(player, slot::CHEST, Some(ItemStack { damage: last - energy / RATIO, ..pack }));
}

// ItemArmorRubBoots.fallStorage and ItemArmorQuantumSuit.fallStorage per player
static STORED: EntityState<Option<f32>> = EntityState::new("fall_stored");
// ItemArmorJetpack.hoverMode and the last hover key state per player, the mode lasts through death
static HOVER: EntityState<(bool, bool)> = EntityState::new("hover").kept();
// ItemArmorQuantumSuit.speedTicker and jumpCharge per player
static QUANTUM: EntityState<Option<(i32, f32)>> = EntityState::new("quantum");
const QUANTUM_START: (i32, f32) = (0, 1.0);

fn quantum(entity: Entity) -> (i32, f32) { QUANTUM.get(entity).unwrap_or(QUANTUM_START) }

mod fall
{
    pub const SAFE: f32 = 3.0;
    pub const PER_WEAR: i32 = 2;
    pub const HURT_SHARE: i32 = 4;
    pub const QUANTUM_HARD: i32 = 8;
}

// ItemArmorRubBoots.absorbFalling and ItemArmorQuantumSuit.absorbFalling: each block of fall goes into a store the
// boots pay for when the player lands
fn absorbFall(player: Player, entity: Entity, quantum: bool)
{
    let mut fallen: f32 = entity.field(EntityField::FallDistance) as f32;
    let stored: f32 = STORED.get(entity).unwrap_or(0.0);
    if fallen < 1.0 && stored == 0.0
    {
        return;
    }

    let mut store: f32 = stored;
    if store > 0.0 && entity.flag(EntityField::InWater)
    {
        store = 0.0;
    }

    if fallen >= 1.0
    {
        fallen -= 1.0;
        entity.setField(EntityField::FallDistance, f64::from(fallen));
        store += 1.0;
    }

    if entity.flag(EntityField::OnGround)
    {
        if store >= fall::SAFE
        {
            let hard: i32 = (store - fall::SAFE).ceil() as i32;
            let boots: Option<ItemStack> = player.slot(slot::FEET);

            if quantum
            {
                if hard > fall::QUANTUM_HARD
                {
                    put(player, slot::FEET, boots.and_then(|boots| wear(boots, 1)));
                }
            }
            else
            {
                let damage: i32 = (hard + 1) / fall::PER_WEAR;
                put(player, slot::FEET, boots.and_then(|boots| wear(boots, damage)));
                if damage >= fall::HURT_SHARE
                {
                    entity.damage(damage / fall::HURT_SHARE, None);
                }
            }
        }

        store = 0.0;
    }

    crate::core::store(&STORED, entity, &Some(store));
}

mod jet
{
    pub const FUEL_POWER: f32 = 1.0;
    pub const ELECTRIC_POWER: f32 = 0.7;
    pub const FUEL_FADE: f32 = 0.2;
    pub const ELECTRIC_FADE: f32 = 0.05;
    pub const PUSH: f32 = 0.15;
    pub const HOVER_PUSH: f32 = 0.5;
    pub const ELECTRIC_PUSH: f32 = 0.15;
    pub const FORWARD: f32 = 0.4;
    pub const STRAFE_FACTOR: f32 = 0.02;
    pub const CEILING: f64 = 128.0;
    pub const ELECTRIC_CEILING: f64 = 100.0;
    pub const THIN_AIR: f64 = 25.0;
    pub const LIFT: f64 = 0.2;
    pub const TOP_SPEED: f64 = 0.6;
    pub const HOVER_FALL: f64 = -0.1;
    pub const HOVER_RISE: f64 = 0.1;
    pub const BURN: i32 = 9;
    pub const HOVER_BURN: i32 = 6;
    pub const ELECTRIC_SAVING: i32 = 2;
    pub const SAFE_FALL: f64 = -0.35;
    pub const HUM_EVERY: i32 = 16;
}

fn hovering(player: Player) -> bool { keys::entityOf(player).is_some_and(|entity| HOVER.get(entity).0) }

// ItemArmorJetpack.toggleHover on the hover key's press
fn toggleHover(player: Player, world: World)
{
    let pressed: bool = keys::held(player, Key::Hover);
    let entity: Entity = Entity { world, id: player.id };
    let (mut on, was): (bool, bool) = HOVER.get(entity);
    let flipped: bool = pressed && !was;
    if flipped
    {
        on = !on;
    }

    crate::core::store(&HOVER, entity, &(on, pressed));

    if flipped && world.isAuthority()
    {
        let state: &str = if hovering(player)
        {
            "activated"
        }
        else
        {
            "deactivated"
        };
        player.chat(&format!("Hover Mode {state}."));
    }
}

fn jetpack(player: Player) -> Option<(ItemStack, bool)>
{
    let pack: ItemStack = player.slot(slot::CHEST)?;
    let electric: bool = pack.item.is("ic2:electric_jetpack");
    (electric || pack.item.is("ic2:jetpack")).then_some((pack, electric))
}

fn thrusting(player: Player, entity: Entity, electric: bool) -> bool
{
    let falling: bool = entity.info().is_some_and(|info| info.motion[1] < jet::SAFE_FALL);
    keys::held(player, Key::Jump) || electric && hovering(player) && falling
}

// ItemArmorJetpack.useJetpack: the lift, on the player's own client
fn thrust(player: Player, entity: Entity, info: &EntityInfo, pack: &ItemStack, electric: bool)
{
    let last: f32 = (maxDamage(&pack.item) - 1) as f32;
    if pack.damage as f32 + 1.0 >= last + 1.0
    {
        return;
    }

    let (mut power, fade): (f32, f32) = if electric
    {
        (jet::ELECTRIC_POWER, jet::ELECTRIC_FADE)
    }
    else
    {
        (jet::FUEL_POWER, jet::FUEL_FADE)
    };
    if pack.damage as f32 + last * fade >= last
    {
        power *= (last - pack.damage as f32) / (last * fade);
    }

    let hover: bool = hovering(player);
    if keys::held(player, Key::Forward)
    {
        let mut push: f32 = if hover { jet::HOVER_PUSH } else { jet::PUSH };
        if electric
        {
            push += jet::ELECTRIC_PUSH;
        }

        let forward: f32 = power * push * 2.0;
        if forward > 0.0
        {
            let _ = entity.accelerate(0.0, jet::FORWARD * forward, jet::STRAFE_FACTOR);
        }
    }

    let ceiling: f64 = if electric
    {
        jet::ELECTRIC_CEILING
    }
    else
    {
        jet::CEILING
    };
    let height: f64 = info.pos[1].min(ceiling);
    if height > ceiling - jet::THIN_AIR
    {
        power = (f64::from(power) * ((ceiling - height) / jet::THIN_AIR)) as f32;
    }

    let motion: [f64; 3] = entity.info().map_or(info.motion, |info| info.motion);
    let before: f64 = motion[1];
    let mut rise: f64 = (motion[1] + jet::LIFT * f64::from(power)).min(jet::TOP_SPEED);

    if hover
    {
        let cap: f64 = if electric && keys::held(player, Key::Jump)
        {
            jet::HOVER_RISE
        }
        else
        {
            jet::HOVER_FALL
        };
        if rise > cap
        {
            rise = if before > cap { before } else { cap };
        }
    }

    let _ = entity.setMotion([motion[0], rise, motion[2]]);
    if rise > jet::SAFE_FALL
    {
        entity.setField(EntityField::FallDistance, 0.0);
    }
}

// the jetpack's fuel or charge, worn down on the server
fn burnJetpack(player: Player, pack: ItemStack, electric: bool)
{
    let mut burn: i32 = if hovering(player)
    {
        jet::HOVER_BURN
    }
    else
    {
        jet::BURN
    };
    if electric
    {
        burn -= jet::ELECTRIC_SAVING;
    }

    let last: i32 = maxDamage(&pack.item) - 1;
    put(player, slot::CHEST, Some(ItemStack { damage: (pack.damage + burn).min(last), ..pack }));
}

mod quantum
{
    pub const AIR_LOW: f64 = 100.0;
    pub const AIR_GAIN: f64 = 200.0;
    pub const RUN: f64 = 0.1;
    pub const RUN_EVERY: i32 = 10;
    pub const SPEED: f32 = 0.25;
    pub const SWIM_SPEED: f32 = 0.1;
    pub const SWIM_LIFT: f64 = 0.1;
    pub const LEAP: f64 = 3.5;
    pub const JUMP: f64 = 0.3;
    pub const JUMP_FADE: f32 = 0.75;
}

// the movement half of ItemArmorQuantumSuit.performQuantum, on the player's own client
fn quantumMoves(player: Player, entity: Entity, info: &EntityInfo)
{
    let suit: bool = keys::held(player, Key::Suit);
    let (_, mut jump): (i32, f32) = quantum(entity);
    let wet: bool = entity.flag(EntityField::InWater);
    let ground: bool = entity.flag(EntityField::OnGround);
    let mut motion: [f64; 3] = info.motion;

    let legs: bool = worn(player, slot::LEGS, "quantum_leggings").is_some_and(|legs| charged(&legs));
    let running: bool = ground && motion[0].abs() + motion[2].abs() > quantum::RUN || wet;
    if legs && running && suit && keys::held(player, Key::Forward)
    {
        let speed: f32 = if wet
        {
            quantum::SWIM_SPEED
        }
        else
        {
            quantum::SPEED
        };
        if wet && keys::held(player, Key::Jump)
        {
            motion[1] += quantum::SWIM_LIFT;
            let _ = entity.setMotion(motion);
        }

        let _ = entity.accelerate(0.0, 1.0, speed);
        motion = entity.info().map_or(motion, |info| info.motion);
    }

    let Some(boots) = worn(player, slot::FEET, "quantum_boots")
    else
    {
        return;
    };

    if ground && jump < 1.0 && charged(&boots)
    {
        jump = 1.0;
    }

    if motion[1] >= 0.0 && jump > 0.0 && !wet
    {
        if keys::held(player, Key::Jump) && suit
        {
            if jump == 1.0
            {
                motion[0] *= quantum::LEAP;
                motion[2] *= quantum::LEAP;
            }

            motion[1] += f64::from(jump) * quantum::JUMP;
            jump *= quantum::JUMP_FADE;
            let _ = entity.setMotion(motion);
        }
        else if jump < 1.0
        {
            jump = 0.0;
        }
    }

    let (ticker, _): (i32, f32) = quantum(entity);
    crate::core::store(&QUANTUM, entity, &Some((ticker, jump)));
}

// the wear half of performQuantum and the rest of mod_IC2.OnTickInGame, on the server
fn serverArmor(player: Player, ticks: i64)
{
    let Some(info) = player.info()
    else
    {
        return;
    };
    let world: World = World { dimension: info.world };
    let entity: Entity = Entity { world, id: player.id };

    toggleHover(player, world);

    if let Some((pack, electric)) = jetpack(player).filter(|_| !entity.flag(EntityField::InWater))
        && thrusting(player, entity, electric)
        && charged(&pack)
    {
        burnJetpack(player, pack, electric);
        if ticks % i64::from(jet::HUM_EVERY) == 0
        {
            world.playSound(info.pos, &sound("jetpackLoop"), 1.0, 1.0);
        }
    }

    if worn(player, slot::FEET, "rubber_boots").is_some()
    {
        absorbFall(player, entity, false);
    }

    if let Some(helmet) = player.slot(slot::HEAD).filter(|helmet| isQuantum(helmet) && charged(helmet))
        && entity.field(EntityField::Air) < quantum::AIR_LOW
    {
        entity.setField(EntityField::Air, entity.field(EntityField::Air) + quantum::AIR_GAIN);
        put(player, slot::HEAD, wear(helmet, 1));
    }

    if player.slot(slot::CHEST).is_some_and(|chest| isQuantum(&chest))
    {
        entity.setField(EntityField::FireTicks, 0.0);
    }

    if let Some(legs) = worn(player, slot::LEGS, "quantum_leggings").filter(charged)
        && keys::held(player, Key::Suit)
        && keys::held(player, Key::Forward)
    {
        let (ticker, jump): (i32, f32) = quantum(entity);
        let ticker: i32 = ticker + 1;
        let running: bool = ticker >= quantum::RUN_EVERY;
        let next: i32 = if running { 0 } else { ticker };
        crate::core::store(&QUANTUM, entity, &Some((next, jump)));

        if running
        {
            put(player, slot::LEGS, wear(legs, 1));
        }
    }

    if let Some(boots) = worn(player, slot::FEET, "quantum_boots")
    {
        let jumping: bool = keys::held(player, Key::Jump) && keys::held(player, Key::Suit);
        if jumping && entity.flag(EntityField::OnGround) && charged(&boots)
        {
            put(player, slot::FEET, wear(boots.clone(), 1));
        }

        if charged(&boots)
        {
            absorbFall(player, entity, true);
        }
        else
        {
            crate::core::store(&STORED, entity, &Some(0.0));
        }
    }
}

fn localArmor()
{
    let Some((player, info)) = localPlayer().and_then(|player| player.info().map(|info| (player, info)))
    else
    {
        return;
    };
    let world: World = World { dimension: info.world };
    let entity: Entity = Entity { world, id: player.id };

    if !world.isAuthority()
    {
        toggleHover(player, world);
    }

    if let Some((pack, electric)) = jetpack(player).filter(|_| !entity.flag(EntityField::InWater))
        && thrusting(player, entity, electric)
    {
        thrust(player, entity, &info, &pack, electric);
    }

    quantumMoves(player, entity, &info);
}

// ItemArmorNanoSuit.getProperties through EntityPlayer.damageEntity: each special piece in turn may take the hit,
// wearing every piece; only a reduction of a full 25 points removes it, and plain armor stops counting
fn absorbHit(event: &mut Event)
{
    const FULL: f64 = 25.0;
    const QUANTUM_SHARE: [i32; 2] = [2, 3];

    let Some(info) = event.entity().and_then(|entity| entity.info()).filter(|info| info.player.is_some())
    else
    {
        return;
    };
    let player: Player = Player { id: info.id };
    let EventData::Attack { amount, .. } = event.data().clone()
    else
    {
        return;
    };

    if !slot::ALL.iter().any(|slot| player.slot(*slot).is_some_and(|stack| isNano(&stack)))
    {
        return;
    }

    let initial: i32 = amount;
    let mut left: i32 = amount;

    for armorSlot in slot::ALL
    {
        if !player.slot(armorSlot).is_some_and(|stack| isNano(&stack)) || left != initial
        {
            continue;
        }

        let mut damage: i32 = left;
        if player.slot(slot::CHEST).is_some_and(|chest| isQuantum(&chest) && charged(&chest))
        {
            damage = damage * QUANTUM_SHARE[0] / QUANTUM_SHARE[1];
        }

        let mut reduce: f64 = 0.0;
        for (index, other) in slot::ALL.iter().enumerate().rev()
        {
            let Some(piece) = player.slot(*other)
            else
            {
                continue;
            };
            let reduction: f64 = f64::from(crate::items::ARMOR_REDUCE[slot::ALL.len() - 1 - index]);

            if isNano(&piece)
            {
                let last: i32 = maxDamage(&piece.item) - 1;
                if piece.damage + damage >= last
                {
                    reduce += 1.0;
                    put(player, *other, Some(ItemStack { damage: last, ..piece.clone() }));
                }
                else
                {
                    reduce += reduction + 1.0;
                    put(player, *other, wear(piece.clone(), damage));
                }

                if isQuantum(&piece)
                {
                    reduce += 1.0;
                }

                continue;
            }

            let max: i32 = maxDamage(&piece.item);
            if max > 0
            {
                reduce += f64::from(max - piece.damage) / f64::from(max) * reduction;
            }

            put(player, *other, wear(piece, damage));
        }

        let removed: i32 = (reduce / FULL).min(1.0) as i32 * damage;
        left -= removed;
    }

    if let EventData::Attack { amount, .. } = event.dataMut()
    {
        *amount = left.max(0);
    }

    if left <= 0
    {
        event.cancel();
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for piece in &PIECES
    {
        let spec: ArmorSpec = armor(piece.m_slot, piece.m_durability, piece.m_layer);
        ctx.items().register(
            piece.m_name,
            ItemSpecBuilder::new().texture(&itemTexture(piece.m_name)).maxDamage(piece.m_durability).armor(spec),
        )?;
    }

    ctx.events().on::<LivingHurt>(Priority::Normal, absorbHit)?;
    ctx.events().on::<TickPost>(Priority::Normal, |_|
    {
        if isClient()
        {
            localArmor();
        }

        for player in allPlayers()
        {
            let world: Option<World> = player.info().map(|info| World { dimension: info.world });
            if let Some(world) = world.filter(|world| world.isAuthority())
            {
                serverArmor(player, world.time());
            }
        }
    })
}
