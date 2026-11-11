use crate::effects::Effect;
use b173_api::prelude::*;
use b173_api::terrain::Random;
use std::cell::{Cell, RefCell};
use std::collections::HashSet;

// EntityPlayer.health starts at twenty half hearts, life shards add two each
pub(crate) const BASE_HEALTH: i32 = 20;
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
    // entities the Aether poisoned by dimension and id, followed until the loader's effect runs out
    static POISONED: RefCell<HashSet<(i32, i32)>> = RefCell::new(HashSet::new());
    // PotionInebriation keeps one push and one turn for every poisoned entity
    static DRIFT: Cell<(f64, f64)> = const { Cell::new((0.0, 0.0)) };
    static GAUSSIAN: Random = Random::withSeed(0);
}

fn poisoned(entity: Entity) -> bool { crate::effects::has(entity, Effect::Inebriation) }

pub(crate) fn poison(entity: Entity)
{
    if !entity.world.isAuthority()
    {
        return;
    }

    crate::effects::give(entity, Effect::Inebriation, inebriation::TICKS);
    POISONED.with(|poisoned| poisoned.borrow_mut().insert((entity.world.dimension, entity.id)));
}

// the remedy bucket and the white apple cure it
pub(crate) fn cure(entity: Entity)
{
    if !entity.world.isAuthority()
    {
        return;
    }

    crate::effects::remove(entity, Effect::Inebriation);
    POISONED.with(|poisoned| poisoned.borrow_mut().remove(&(entity.world.dimension, entity.id)));
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

// the poisoned player's own client moves and turns it, a server cannot steer a player
fn localPoison()
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
    if poisoned(entity)
    {
        distract(entity, &info);
    }
}

// PotionInebriation.performEffect for every entity the Aether poisoned, the loader's effect deals the damage
fn tick()
{
    let current: Vec<(i32, i32)> = POISONED.with(|poisoned| poisoned.borrow().iter().copied().collect());
    for (dimension, id) in current
    {
        let entity: Entity = Entity { world: World { dimension }, id };
        let living: Option<EntityInfo> = entity.info().filter(|info| info.alive && info.health.unwrap_or(0) > 0);
        let Some(info) = living.filter(|_| poisoned(entity))
        else
        {
            POISONED.with(|poisoned| poisoned.borrow_mut().remove(&(dimension, id)));
            continue;
        };

        if info.player.is_none()
        {
            distract(entity, &info);
        }

        spit(entity, &info);
    }
}

mod armor
{
    // InventoryPlayer keeps boots, leggings, chestplate and helmet after the 36 main slots
    pub const FIRST_SLOT: i32 = 36;
    pub const PIECES: [&str; 4] = ["boots", "leggings", "chestplate", "helmet"];
    pub const GLOVES: usize = 6;
    // AbilityArmor: the gravitite jump, liquid strokes and the phoenix set burning out in water
    pub const GRAVITITE_JUMP: f64 = 1.0;
    pub const WATER_STROKE: f64 = 0.03;
    pub const QUENCH_EVERY: f64 = 5.0;
    // PlayerAether.onUpdate sends a player who falls below the Aether to the overworld sky
    pub const FALL_OUT: f64 = -12.0;
    pub const OVERWORLD: i32 = 0;
    pub const SKY: f64 = 127.0;
}

fn armorPiece(player: Player, index: usize) -> Option<ItemStack> { player.slot(armor::FIRST_SLOT + index as i32) }

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
    if swimmer && entity.flag(EntityField::InWater)
    {
        // AetherLiquidMovement repeats the stroke vanilla already takes in water
        let forward: f64 = entity.field(EntityField::MoveForward);
        let strafe: f64 = entity.field(EntityField::MoveStrafe);
        let yaw: f64 = (info.yaw as f64).to_radians();
        let push: [f64; 2] = [strafe * yaw.cos() - forward * yaw.sin(), forward * yaw.cos() + strafe * yaw.sin()];
        let _ = entity.setMotion([
            info.motion[0] + push[0] * armor::WATER_STROKE,
            info.motion[1],
            info.motion[2] + push[1] * armor::WATER_STROKE,
        ]);
    }
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

    ctx.events().on::<TickPost>(Priority::Normal, |_|
    {
        tick();

        if isClient()
        {
            localPoison();
        }
    })?;

    Ok(())
}
