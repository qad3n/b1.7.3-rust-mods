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

// BlockTFTrophyPedestal
mod pedestal
{
    // players within 16 blocks are rewarded or warned, latent shields within 5 vanish
    pub const REACH: f64 = 16.0;
    pub const SHIELDS: i32 = 5;
    pub const HARDNESS: f32 = 2.0;
    pub const RESISTANCE: f32 = 2000.0;
    // the low two bits turn the boss faces, the latent bit waits for a trophy and lights the faces
    pub const TURNS: i32 = 4;
    pub const LATENT: i32 = 8;
    // RenderBlockTFPedestal's base, column and cap in pixels
    pub const BOXES: [[[f32; 3]; 2]; 3] = [
        [[1.0, 0.0, 1.0], [15.0, 3.0, 15.0]],
        [[2.0, 3.0, 2.0], [14.0, 13.0, 14.0]],
        [[1.0, 13.0, 1.0], [15.0, 16.0, 15.0]],
    ];
    // the faces sit on a thin skin over a slightly smaller core so every side can carry its own sprite
    pub const SKIN: f32 = 0.02;
    // the boss on each side before turning, north south west east
    pub const FACES: [&str; 4] = ["naga", "lich", "hydra", "urghast"];
}

fn texture(name: &str) -> String { format!("twilightforest:item/{name}") }

fn latent(meta: i32) -> bool { meta & pedestal::LATENT != 0 }

fn trophyOnTop(world: World, pos: BlockPos) -> bool { world.block(pos.offset(0, 1, 0)) == ids::blockId(BLOCK) }

// BlockTFTrophyPedestal.updateTick once a trophy sits on it, players are rewarded before the check so the first
// worthy visit opens it instead of needing a second trophy placement
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
    for player in &near
    {
        progress::grant(*player, Gate::Trophy);
    }

    let meta: i32 = world.meta(pos);
    if !latent(meta)
    {
        return;
    }

    if near.iter().any(|player| progress::has(*player, Gate::Trophy))
    {
        open(world, pos, meta);
    }
    for player in near.iter().filter(|player| !progress::has(**player, Gate::Trophy))
    {
        player.chat("You are unworthy.");
    }
}

// BlockTFTrophyPedestal.doPedestalEffect, only the latent shields the access chamber lays under its floor vanish
fn open(world: World, pos: BlockPos, meta: i32)
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
                if world.block(at) == shield && world.meta(at) == crate::blocks::SHIELD_LATENT
                {
                    let _ = world.setBlock(at, 0, 0);
                    let _ = world.breakEffect(at, shield, crate::blocks::SHIELD_LATENT);
                }
            }
        }
    }
    let _ = world.setBlock(pos, ids::blockId(block::TROPHY_PEDESTAL), meta & !pedestal::LATENT);
    world.playSound(ids::center(pos), "mob.zombie.infect", 4.0, 0.1);
}

struct Pedestal;

#[block_behavior]
impl BlockBehavior for Pedestal
{
    // BlockTFTrophyPedestal.onBlockPlacedBy, the naga side faces the player
    fn onPlacedBy(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        let Some(info) = entity.info()
        else
        {
            return;
        };

        let quarter: i32 = (f64::from(info.yaw) * 4.0 / 360.0 + 0.5).floor() as i32 & 3;
        let turn: i32 = [0, 1, 3, 2][quarter as usize];
        ctx.setMeta((turn | i32::from(ctx.meta()) & pedestal::LATENT) as u8);
    }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        let (world, pos): (World, BlockPos) = (ctx.world(), ctx.pos());
        if ctx.isAuthority() && ctx.meta() > 0 && trophyOnTop(world, pos)
        {
            let _ = world.scheduleTick(pos, ctx.block(), 1);
        }
    }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority() && trophyOnTop(ctx.world(), ctx.pos())
        {
            crown(ctx.world(), ctx.pos());
        }
    }
}

fn blockTexture(name: &str, lit: bool) -> String
{
    let suffix: &str = if lit { "_active" } else { "" };
    format!("twilightforest:block/pedestal_{name}{suffix}")
}

// one thin cuboid over a box's face, a step of -1 or 1 along an axis picks the face
fn skin(from: [f32; 3], to: [f32; 3], axis: usize, step: i32) -> ([f32; 3], [f32; 3])
{
    let (mut low, mut high): ([f32; 3], [f32; 3]) = (from, to);
    match step
    {
        1 => low[axis] = to[axis] - pedestal::SKIN,
        _ => high[axis] = from[axis] + pedestal::SKIN,
    }
    (low, high)
}

// RenderBlockTFPedestal's three boxes, each face with the sprite BlockTFTrophyPedestal.getIcon gives it
fn pedestalModel() -> Vec<Cuboid>
{
    let sides: [(usize, i32); 4] = [(2, -1), (2, 1), (0, -1), (0, 1)];
    let mut cuboids: Vec<Cuboid> = Vec::new();
    for [from, to] in pedestal::BOXES
    {
        let core: Cuboid = Cuboid
        {
            from: [from[0] + pedestal::SKIN, from[1], from[2] + pedestal::SKIN],
            to: [to[0] - pedestal::SKIN, to[1] - pedestal::SKIN, to[2] - pedestal::SKIN],
            texture: Some(blockTexture("top", false)),
            ..Cuboid::default()
        };
        cuboids.push(core);
        for meta in 0..pedestal::LATENT * 2
        {
            let lit: bool = latent(meta);
            if meta % pedestal::LATENT == 0
            {
                let (low, high): ([f32; 3], [f32; 3]) = skin(from, to, 1, 1);
                let plates: Vec<i32> = (meta..meta + pedestal::LATENT).collect();
                cuboids.push(Cuboid
                {
                    from: low,
                    to: high,
                    texture: Some(blockTexture("top", lit)),
                    metas: plates,
                    ..Cuboid::default()
                });
            }
            for (side, (axis, step)) in sides.into_iter().enumerate()
            {
                let face: &str =
                    pedestal::FACES[(side + (meta & pedestal::TURNS - 1) as usize) % pedestal::FACES.len()];
                let (low, high): ([f32; 3], [f32; 3]) = skin(from, to, axis, step);
                cuboids.push(Cuboid
                {
                    from: low,
                    to: high,
                    texture: Some(blockTexture(face, lit)),
                    metas: vec![meta],
                    ..Cuboid::default()
                });
            }
        }
    }
    cuboids
}

fn registerPedestal(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.blocks().register(
        block::TROPHY_PEDESTAL,
        BlockSpecBuilder::template("minecraft:block")
            .name("Trophy Pedestal")
            .material("rock")
            .sound("stone")
            .tool(ToolClass::Pickaxe, 0)
            .hardness(pedestal::HARDNESS)
            .resistance(pedestal::RESISTANCE)
            .texture(&blockTexture("top", false))
            .model(BlockModelSpec { cuboids: pedestalModel(), ..BlockModelSpec::default() })
            .behavior(Pedestal),
    )?;
    Ok(())
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
        }
        let left: i32 = stack.count - 1;
        Use::Done((left > 0).then_some(ItemStack { count: left, ..stack }))
    }
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    registerPedestal(ctx)?;
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
