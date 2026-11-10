use crate::ids::{self, block, item};
use crate::progress::{self, Gate};
use b173_api::prelude::*;

// ItemTFTrophy and BlockTFTrophy: boss trophies placed as small heads, and BlockTFTrophyPedestal, which opens the
// knight stronghold for players worthy of it when a trophy sits on top
const BLOCK: &str = "trophy";

struct Trophy
{
    m_item: &'static str,
    m_label: &'static str,
    m_texture: &'static str,
}

// BlockTFTrophy metadata order
const TROPHIES: [Trophy; 5] = [
    Trophy { m_item: item::HYDRA_TROPHY, m_label: "Hydra Trophy", m_texture: "hydratrophy" },
    Trophy { m_item: item::NAGA_TROPHY, m_label: "Naga Trophy", m_texture: "nagatrophy" },
    Trophy { m_item: item::LICH_TROPHY, m_label: "Lich Trophy", m_texture: "lichtrophy" },
    Trophy { m_item: item::UR_GHAST_TROPHY, m_label: "Ur-ghast Trophy", m_texture: "urghasttrophy" },
    Trophy { m_item: item::SNOW_QUEEN_TROPHY, m_label: "Snow Queen Trophy", m_texture: "snowqueentrophy" },
];

mod pedestal
{
    // players within 16 blocks are rewarded or warned, shields within 5 vanish
    pub const REACH: f64 = 16.0;
    pub const SHIELDS: i32 = 5;
    pub const ACTIVE: i32 = 1;
}

fn texture(name: &str) -> String { format!("twilightforest:item/{name}") }

// BlockTFTrophyPedestal.updateTick once a trophy sits on it
fn crown(world: World, pos: BlockPos)
{
    let center: [f64; 3] = ids::center(pos);
    let near: Vec<Player> = world
        .players()
        .into_iter()
        .filter(|player|
        {
            player.info().is_some_and(|info| crate::entities::ai::distance(info.pos, center) <= pedestal::REACH)
        })
        .collect();
    if world.meta(pos) != pedestal::ACTIVE
    {
        if near.iter().any(|player| progress::has(*player, Gate::Trophy))
        {
            open(world, pos);
        }
        for player in near.iter().filter(|player| !progress::has(**player, Gate::Trophy))
        {
            player.chat("You are unworthy.");
        }
    }
    for player in near
    {
        progress::grant(player, Gate::Trophy);
    }
}

// BlockTFTrophyPedestal.doPedestalEffect
fn open(world: World, pos: BlockPos)
{
    let shield: i32 = ids::blockId(block::SHIELD);
    let reach: i32 = pedestal::SHIELDS;
    for x in -reach..=reach
    {
        for y in -reach..=reach
        {
            for z in -reach..=reach
            {
                let at: BlockPos = pos.offset(x, y, z);
                if world.block(at) == shield
                {
                    let meta: i32 = world.meta(at);
                    let _ = world.setBlock(at, 0, 0);
                    let _ = world.breakEffect(at, shield, meta);
                }
            }
        }
    }
    let _ = world.setBlock(pos, ids::blockId(block::TROPHY_PEDESTAL), pedestal::ACTIVE);
    world.playSound(ids::center(pos), "mob.zombie.infect", 4.0, 0.1);
}

struct TrophyItem
{
    m_kind: i32,
}

#[item_behavior]
impl ItemBehavior for TrophyItem
{
    // ItemTFTrophy.onItemUse, set down on the face clicked
    fn onUseOnBlock(&self, world: World, _player: Player, stack: ItemStack, pos: BlockPos, face: i32) -> Use
    {
        let offset: [i32; 3] = crate::world::structures::facing(face);
        let at: BlockPos = pos.offset(offset[0], offset[1], offset[2]);
        if world.block(at) != 0
        {
            return Use::Pass;
        }

        if world.isAuthority()
        {
            let _ = world.setBlock(at, ids::blockId(BLOCK), self.m_kind);
            if world.block(at.offset(0, -1, 0)) == ids::blockId(block::TROPHY_PEDESTAL)
            {
                crown(world, at.offset(0, -1, 0));
            }
        }
        let left: i32 = stack.count - 1;
        Use::Done((left > 0).then_some(ItemStack { count: left, ..stack }))
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let head: [[f32; 3]; 2] = [[4.0, 0.0, 4.0], [12.0, 8.0, 12.0]];
    let cuboids: Vec<Cuboid> = TROPHIES
        .iter()
        .enumerate()
        .map(|(kind, trophy)| Cuboid
        {
            from: head[0],
            to: head[1],
            texture: Some(texture(trophy.m_texture)),
            metas: vec![kind as i32],
            ..Cuboid::default()
        })
        .collect();
    ctx.blocks().register(
        BLOCK,
        BlockSpecBuilder::template("minecraft:block")
            .name("Trophy")
            .material("circuits")
            .texture(&texture(TROPHIES[0].m_texture))
            .hardness(1.0)
            .model(BlockModelSpec { cuboids, translucent: true, ..BlockModelSpec::default() })
            .collision(vec![Cuboid { from: head[0], to: head[1], ..Cuboid::default() }])
            .drops(Vec::new()),
    )?;
    for (kind, trophy) in TROPHIES.iter().enumerate()
    {
        let spec: ItemSpecBuilder = ItemSpecBuilder::new()
            .name(trophy.m_label)
            .texture(&texture(trophy.m_texture))
            .behavior(TrophyItem { m_kind: kind as i32 });
        ctx.items().register(trophy.m_item, spec)?;
    }
    Ok(())
}
