use super::{put, texture};
use crate::ids::{self, item};
use crate::math::look;
use b173_api::prelude::*;

// ItemTFTransformPowder.itemInteractionForEntity: a pinch turns a creature into its twin; beta lacks bats, cave
// spiders, blazes, villagers and witches, so the raven, swarm spider, wraith, redcap and druid pairs are left out
const PAIRS: [(&str, &str); 7] = [
    ("twilightforest:minotaur", "minecraft:pig_zombie"),
    ("twilightforest:wild_deer", "minecraft:cow"),
    ("twilightforest:bighorn_sheep", "minecraft:sheep"),
    ("twilightforest:wild_boar", "minecraft:pig"),
    ("twilightforest:hostile_wolf", "minecraft:wolf"),
    ("twilightforest:glacier_penguin", "minecraft:chicken"),
    ("twilightforest:hedge_spider", "minecraft:spider"),
];
const SPARKS: i32 = 30;

// the player whose hand the powder came from, the nearest one holding it
fn holder(world: World, at: [f64; 3]) -> Option<Player>
{
    world
        .players()
        .into_iter()
        .filter(|player| player.held().is_some_and(|stack| stack.item.is(item::TRANSFORM_POWDER)))
        .min_by(|a, b|
        {
            let reach =
                |player: &Player| player.info().map_or(f64::MAX, |info| crate::entities::ai::distance(info.pos, at));
            reach(a).total_cmp(&reach(b))
        })
}

struct Powder;

#[item_behavior]
impl ItemBehavior for Powder
{
    // ItemTFTransformPowder.onItemRightClick, a cloud of sparks where the powder would land
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        let Some(info) = player.info()
        else
        {
            return Use::Pass;
        };

        if world.isAuthority()
        {
            return Use::Pass;
        }

        let range: f64 = 2.0;
        let aim: [f64; 3] = look(info.yaw, info.pitch);
        let center: [f64; 3] = [0, 1, 2].map(|axis| info.pos[axis] + aim[axis] * range);

        for _ in 0..SPARKS
        {
            let at: [f64; 3] = center.map(|value| value + f64::from(ids::fraction(world)) * 2.0 - 1.0);
            crate::particles::spawn(world, "magicCrit", at, [0.0; 3]);
        }

        Use::Pass
    }

    fn onInteractEntity(&self, _stack: ItemStack, target: Entity)
    {
        let world: World = target.world;

        let Some(info) = target.info()
        else
        {
            return;
        };

        let kind: String = info.kind.as_ref().map(ToString::to_string).unwrap_or_default();

        let Some(twin) = PAIRS.iter().find_map(|(a, b)|
        {
            if *a == kind
            {
                Some(*b)
            }
            else if *b == kind
            {
                Some(*a)
            }
            else
            {
                None
            }
        })
        else
        {
            return;
        };

        for _ in 0..30
        {
            let jitter = || f64::from(ids::fraction(world) - 0.5);
            crate::particles::spawn(
                world,
                "largeexplode",
                [info.pos[0] + jitter(), info.pos[1] + jitter() + 1.0, info.pos[2] + jitter()],
                [0.0; 3],
            );
        }

        if !world.isAuthority()
        {
            return;
        }

        target.kill();
        crate::entities::spawnAt(world, twin, info.pos, info.yaw);

        if let Some(player) = holder(world, info.pos)
            && let Some(held) = player.held()
        {
            let left: Option<ItemStack> = (held.count > 1).then(|| ItemStack { count: held.count - 1, ..held });
            if let Err(error) = player.setHeld(left)
            {
                warn(&format!("transformation powder: {}", error.0));
            }
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    put(
        ctx,
        item::TRANSFORM_POWDER,
        ItemSpecBuilder::new().name("Transformation Powder").texture(&texture("transformpowder")).behavior(Powder),
    )
}
