use crate::effects::Effect;
use b173_api::prelude::*;
use b173_api::terrain::Random;
use std::cell::Cell;
use std::rc::Rc;

// each life shard adds two half hearts to beta's twenty
pub(crate) const SHARD_HEALTH: i32 = 2;

mod inebriation
{
    // EffectInebriation as darts, needles and poison buckets give it
    pub const TICKS: i32 = 500;
    // PotionInebriation.distractEntity: a gaussian push and turn, each smoothed into the last
    pub const PUSH: f64 = 0.1;
    pub const TURN: f64 = std::f64::consts::FRAC_PI_4;
    pub const PUSH_KEEP: f64 = 0.8;
    pub const TURN_KEEP: f64 = 0.875;
    pub const SPIT_HEIGHT: f64 = 0.8;
    pub const SPITS: i32 = 2;
    // red dye, item 351 metadata 1, whose icon bits the poisoned spit
    pub const SPIT: &str = "iconcrack_351_1";
}

thread_local!
{
    // PotionInebriation keeps one push and one turn for every poisoned entity
    static DRIFT: Cell<(f64, f64)> = const { Cell::new((0.0, 0.0)) };
    static GAUSSIAN: Random = Random::withSeed(0);
}

pub(crate) fn poison(entity: Entity)
{
    if !entity.world.isAuthority()
    {
        return;
    }

    crate::effects::give(entity, Effect::Inebriation, inebriation::TICKS);
}

// the remedy bucket and the white apple cure it, PlayerAether.setCured tints the cured player's view for a while
pub(crate) fn cure(entity: Entity, ticks: i32)
{
    crate::hud::cured(Player { id: entity.id }, ticks);

    if !entity.world.isAuthority()
    {
        return;
    }

    crate::effects::remove(entity, Effect::Inebriation);
}

// PotionInebriation.distractEntity: the push sways the walk, the turn shakes the view
fn distract(entity: Entity, info: &EntityInfo)
{
    let gaussian: f64 = GAUSSIAN.with(Random::nextGaussian);
    let (push, turn) = DRIFT.get();
    let push: f64 = (1.0 - inebriation::PUSH_KEEP) * inebriation::PUSH * gaussian + inebriation::PUSH_KEEP * push;
    let turn: f64 = (1.0 - inebriation::TURN_KEEP) * inebriation::TURN * gaussian + inebriation::TURN_KEEP * turn;
    DRIFT.set((push, turn));

    let _ = entity.setMotion([info.motion[0] + push, info.motion[1], info.motion[2] + push]);

    entity.setField(EntityField::Yaw, f64::from(info.yaw) + turn);
    entity.setField(EntityField::Pitch, f64::from(info.pitch) + turn);
}

fn spit(entity: Entity, info: &EntityInfo)
{
    let height: f64 = entity.field(EntityField::Height);
    let mouth: [f64; 3] =
        [info.pos[0], entity.field(EntityField::Bottom) + height * inebriation::SPIT_HEIGHT, info.pos[2]];

    for _ in 0..inebriation::SPITS
    {
        entity.world.particle(inebriation::SPIT, mouth, [0.0; 3]);
    }
}

// PotionInebriation.performEffect, the effect's own interval deals the damage
struct Inebriation;

#[effect_behavior]
impl EffectBehavior for Inebriation
{
    // the server sways mobs and spits for everyone, a player's own client sways that player since a server cannot
    // steer it
    fn onTick(&self, entity: Entity, _level: i32, _ticks: i32)
    {
        let Some(info) = entity.info().filter(|info| info.alive && info.health.unwrap_or(0) > 0)
        else
        {
            return;
        };

        let authority: bool = entity.world.isAuthority();
        if !authority || info.player.is_none()
        {
            distract(entity, &info);
        }

        if authority
        {
            spit(entity, &info);
        }
    }
}

pub(crate) fn inebriation() -> Rc<dyn EffectBehavior> { Rc::new(Inebriation) }

mod armor
{
    // InventoryPlayer keeps boots, leggings, chestplate and helmet after the 36 main slots
    pub const FIRST_SLOT: i32 = 36;
    pub const PIECES: [&str; 4] = ["boots", "leggings", "chestplate", "helmet"];
    pub const GLOVES: usize = 6;
    // AbilityArmor: the gravitite jump, liquid strokes and the phoenix set burning out in water
    pub const GRAVITITE_JUMP: f64 = 1.0;
    pub const WATER_STROKE: f64 = 0.03;
    pub const LAVA_STROKE: f64 = 0.06;
    // AetherLiquidMovement.negativeDifference eases each movement input this much toward rest
    pub const EASE: f64 = 0.1;
    // Entity.moveFlying ignores inputs this small
    pub const STILL: f64 = 0.01;
    pub const QUENCH_EVERY: f64 = 5.0;
    // PlayerAether.onUpdate sends a player who falls below the Aether to the overworld sky
    pub const FALL_OUT: f64 = -12.0;
    pub const OVERWORLD: i32 = 0;
    pub const SKY: f64 = 127.0;
}

fn armorPiece(player: Player, index: usize) -> Option<ItemStack> { player.slot(armor::FIRST_SLOT + index as i32) }

// InventoryAccessories.wearingArmor for the boots
pub(crate) fn wearingBoots(player: Player, name: &str) -> bool
{
    armorPiece(player, 0).is_some_and(|stack| stack.item.is(name))
}

// isWearingGravititeSet and friends: the four pieces and the matching gloves
pub(crate) fn wearingSet(player: Player, set: &str) -> bool
{
    let gloves: bool = crate::accessories::Worn::of(player)
        .stack(armor::GLOVES)
        .is_some_and(|stack| stack.item.is(&format!("{set}_gloves")));
    gloves
        && armor::PIECES.iter().enumerate().all(|(index, piece)|
        {
            armorPiece(player, index).is_some_and(|stack| stack.item.is(&format!("{set}_{piece}")))
        })
}

// AbilityArmor.damagePhoenixArmor: phoenix pieces wear in water and cool into obsidian
fn quench(player: Player)
{
    for (index, piece) in armor::PIECES.iter().enumerate()
    {
        let Some(stack) = armorPiece(player, index).filter(|stack| stack.item.is(&format!("phoenix_{piece}")))
        else
        {
            continue;
        };

        let worn: i32 = stack.damage + 1;
        let limit: i32 = crate::items::armorUses("phoenix", index).unwrap_or(i32::MAX);
        let replaced: ItemStack = if worn > limit
        {
            ItemStack::of(format!("obsidian_{piece}").as_str(), 1)
        }
        else
        {
            ItemStack { damage: worn, ..stack }
        };
        let _ = player.setSlot(armor::FIRST_SLOT + index as i32, Some(replaced));
    }
}

fn serverArmor(player: Player)
{
    let Some(info) = player.info()
    else
    {
        return;
    };

    let world: World = World { dimension: info.world };
    let entity: Entity = Entity { world, id: player.id };

    if Some(info.world) == crate::portal::aetherDimension() && info.pos[1] < armor::FALL_OUT
    {
        let _ = player.teleport(armor::OVERWORLD, [info.pos[0], armor::SKY, info.pos[2]]);

        return;
    }

    crate::gear::openFalling(player, &info);

    if entity.flag(EntityField::InWater) && entity.field(EntityField::TicksExisted) % armor::QUENCH_EVERY == 0.0
    {
        quench(player);
    }

    if wearingSet(player, "phoenix")
    {
        entity.setField(EntityField::FireTicks, 0.0);
        world.particle(
            "flame",
            [
                info.pos[0] + (crate::entities::fraction(world) - 0.5) / 2.5,
                info.pos[1] + (crate::entities::fraction(world) - 0.5) / 2.5,
                info.pos[2] + (crate::entities::fraction(world) - 0.5) / 1.5,
            ],
            [0.0; 3],
        );
    }
}

thread_local!
{
    static JUMP_BOOSTED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

// the movement half of AbilityArmor on the player's own client
fn localArmor()
{
    let Some(player) = localPlayer()
    else
    {
        return;
    };
    let Some(info) = player.info()
    else
    {
        return;
    };

    let entity: Entity = Entity { world: World { dimension: info.world }, id: player.id };
    let jumping: bool = entity.flag(EntityField::Jumping);
    if wearingSet(player, "gravitite") && jumping && !JUMP_BOOSTED.with(std::cell::Cell::get)
    {
        let _ = entity.setMotion([info.motion[0], armor::GRAVITITE_JUMP, info.motion[2]]);

        JUMP_BOOSTED.with(|boosted| boosted.set(true));
    }

    if !jumping && entity.flag(EntityField::OnGround)
    {
        JUMP_BOOSTED.with(|boosted| boosted.set(false));
    }

    let swimmer: bool = wearingSet(player, "neptune") || wearingSet(player, "phoenix");
    if !swimmer
    {
        return;
    }

    // AetherLiquidMovement repeats the stroke vanilla already takes, once in water and once more in lava
    let strafe: f64 = ease(entity.field(EntityField::MoveStrafe));
    let forward: f64 = ease(entity.field(EntityField::MoveForward));
    for (liquid, scale) in [(EntityField::InWater, armor::WATER_STROKE), (EntityField::InLava, armor::LAVA_STROKE)]
    {
        if !entity.flag(liquid)
        {
            continue;
        }

        let motion: [f64; 3] = entity.info().map_or(info.motion, |info| info.motion);
        let push: [f64; 2] = stroke(info.yaw, strafe, forward, scale);
        let _ = entity.setMotion([motion[0] + push[0], motion[1], motion[2] + push[1]]);
    }
}

// AetherLiquidMovement.negativeDifference
fn ease(input: f64) -> f64
{
    if input == 0.0
    {
        0.0
    }
    else
    {
        input - input.signum() * armor::EASE
    }
}

// Entity.moveFlying: the horizontal push of one stroke along the look
fn stroke(yaw: f32, strafe: f64, forward: f64, scale: f64) -> [f64; 2]
{
    let length: f64 = strafe.hypot(forward);
    if length < armor::STILL
    {
        return [0.0; 2];
    }

    let factor: f64 = scale / length.max(1.0);
    let (strafe, forward): (f64, f64) = (strafe * factor, forward * factor);
    let (sin, cos): (f64, f64) = f64::from(yaw).to_radians().sin_cos();
    [strafe * cos - forward * sin, forward * cos + strafe * sin]
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<TickPost>(Priority::Normal, |_|
    {
        if isClient()
        {
            localArmor();
        }

        for player in allPlayers()
        {
            if player.info().is_some_and(|info| World { dimension: info.world }.isAuthority())
            {
                serverArmor(player);
            }
        }
    })?;

    Ok(())
}
