use super::magic::{held, holdInUse};
use super::{put, texture};
use crate::entities::shots::{self, MOONWORM_SHOT};
use crate::ids::{self, block, item};
use crate::math::look;
use b173_api::prelude::*;

// ItemTFMoonwormQueen: sets moonworms on the face clicked, or drawn and let go fires one that sticks where it lands
mod queen
{
    pub const CHARGES: i32 = 256;
    // a fired moonworm costs two charges and needs two left after it
    pub const SHOT_COST: i32 = 2;
    pub const SHOT_SPEED: f64 = 1.5;
    // ItemTFMoonwormQueen.onPlayerStoppedUsing fires only after more than twelve ticks of drawing
    pub const DRAW: i32 = 12;
}

// BlockTorch metadata by the face the moonworm clings to, the bottom face holds none
const FACE_META: [Option<i32>; 6] = [None, Some(5), Some(4), Some(3), Some(2), Some(1)];

// BlockTFMoonworm placement on a face of the block at pos
pub(crate) fn attach(world: World, pos: BlockPos, face: i32) -> bool
{
    let Some(meta) = FACE_META.get(face as usize).copied().flatten()
    else
    {
        return false;
    };

    let step: [i32; 3] = crate::world::structures::facing(face);
    let at: BlockPos = pos.offset(step[0], step[1], step[2]);
    if world.block(at) != 0 || !World::blockInfo(world.block(pos)).solid
    {
        return false;
    }

    let placed: bool = world.setBlock(at, ids::blockId(block::MOONWORM), meta).is_ok();
    if placed
    {
        world.playSound(ids::center(at), "mob.slime", 0.5, 0.8);
    }

    placed
}

struct MoonwormQueen;

#[item_behavior]
impl ItemBehavior for MoonwormQueen
{
    fn onUseOnBlock(&self, world: World, player: Player, stack: ItemStack, pos: BlockPos, face: i32) -> Use
    {
        if stack.damage >= queen::CHARGES || world.block(pos) == ids::blockId(block::MOONWORM)
        {
            return Use::Pass;
        }

        if world.isAuthority() && attach(world, pos, face)
        {
            super::magic::wearHeld(player, 1);
        }

        Use::Done(player.held())
    }

    fn onRightClick(&self, _world: World, player: Player, stack: ItemStack) -> Use
    {
        if stack.damage >= queen::CHARGES
        {
            return Use::Pass;
        }

        holdInUse(player, held::LONGEST)
    }

    fn useDuration(&self, _stack: ItemStack) -> i32 { held::LONGEST }

    fn useAction(&self, _stack: ItemStack) -> UseAction { UseAction::Bow }

    // ItemTFMoonwormQueen.onPlayerStoppedUsing, a long enough draw fires a moonworm
    fn onStoppedUsing(&self, world: World, player: Player, stack: ItemStack, left: i32) -> Use
    {
        let drawn: bool = held::LONGEST - left > queen::DRAW;
        let Some(info) = player.info().filter(|_| drawn && stack.damage + 1 < queen::CHARGES)
        else
        {
            return Use::Pass;
        };

        if world.isAuthority()
        {
            let aim: [f64; 3] = look(info.yaw, info.pitch);
            let toward: [f64; 3] = [0, 1, 2].map(|axis| info.pos[axis] + aim[axis]);
            if shots::fire(world, MOONWORM_SHOT, info.pos, toward, queen::SHOT_SPEED, 1.0, Some(player.id)).is_some()
            {
                super::magic::wearHeld(player, queen::SHOT_COST);
                world.playSound(info.pos, "mob.slime", 1.0, 1.0);
            }
        }

        Use::Done(player.held())
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let spec: ItemSpecBuilder = ItemSpecBuilder::new()
        .name("Moonworm Queen")
        .texture(&texture("moonwormqueen"))
        .maxStack(1)
        .maxDamage(queen::CHARGES)
        .behavior(MoonwormQueen);
    put(ctx, item::MOONWORM_QUEEN, spec)
}
