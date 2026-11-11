use crate::progress::{self, Gate};
use crate::world::features::{self, Feature};
use crate::world::structures;
use crate::{bosses, ids};
use b173_api::prelude::*;
use b173_api::terrain::structure::StructureBox;

// TFEventListener.isAreaProtected: until a player has the progress a feature asks for, its unconquered pieces turn
// away their pickaxe, keep their chests and levers shut and shelter the monsters inside

fn gate(world: World, x: i32, z: i32) -> Option<Gate>
{
    let (feature, _): (Feature, [i32; 2]) = features::nearest(world.seed(), x >> 4, z >> 4);
    features::row(feature).m_gate
}

// the box the player may not touch at this spot, None when they may
fn guarded(world: World, player: Player, pos: BlockPos) -> Option<StructureBox>
{
    if !crate::world::isTwilight(world) || !world.isAuthority()
    {
        return None;
    }

    let needed: Gate = gate(world, pos.x, pos.z)?;
    let center: [f64; 3] = ids::center(pos);
    if progress::has(player, needed) || bosses::conquered(world, center)
    {
        return None;
    }

    structures::protectedAt(world, pos.x, pos.y, pos.z)
}

fn refuse(event: &mut Event, pos: BlockPos)
{
    let (Some(world), Some(player)) = (event.world(), event.player())
    else
    {
        return;
    };

    if let Some(bounds) = guarded(world, player, pos)
    {
        event.cancel();
        crate::weather::showBox(player, bounds);

        let (feature, _): (Feature, [i32; 2]) = features::nearest(world.seed(), pos.x >> 4, pos.z >> 4);
        crate::hints::trySpawn(world, player, feature, pos);
    }
}

fn breaking(event: &mut Event)
{
    if let Some(pos) = event.pos()
    {
        refuse(event, pos);
    }
}

// TFEventListener.isBlockProtectedFromInteraction, chests, buttons, levers and the tower devices
fn activating(event: &mut Event)
{
    let EventData::Block { world, block, pos, .. } = event.data().clone()
    else
    {
        return;
    };

    let guardedBlocks: [i32; 3] = ["minecraft:chest", "minecraft:stone_button", "minecraft:lever"].map(ids::blockId);
    if guardedBlocks.contains(&block) || crate::tower::deviceBlock(World { dimension: world }, block)
    {
        refuse(event, pos);
    }
}

// TFEventListener.livingAttack, monsters inside a guarded piece shrug off the player
fn attacking(event: &mut Event)
{
    let EventData::Attack { entity, source: Some(source), .. } = event.data().clone()
    else
    {
        return;
    };

    let world: World = World { dimension: entity.world };
    let Some(player) = world.players().into_iter().find(|player| player.id == source)
    else
    {
        return;
    };

    let monster: bool = Entity { world, id: entity.id }.field(EntityField::AttackStrength) > 0.0;
    let pos: BlockPos =
        BlockPos::new(entity.pos[0].floor() as i32, entity.pos[1].floor() as i32, entity.pos[2].floor() as i32);

    if monster && guarded(world, player, pos).is_some()
    {
        event.cancel();
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<BlockBreak>(Priority::Normal, breaking)?;
    ctx.events().on::<BlockActivate>(Priority::Normal, activating)?;
    ctx.events().on::<LivingAttack>(Priority::Normal, attacking)?;
    Ok(())
}
