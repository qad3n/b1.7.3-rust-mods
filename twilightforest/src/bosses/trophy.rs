use crate::models::{Program, combine, tf_hydra_head, tf_lich, tf_naga, tf_snow_queen};
use crate::names::{self, block, item};
use crate::progress::{self, Gate};
use b173_api::math::{distance, floor};
use b173_api::prelude::*;
use std::rc::Rc;

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
    // the turn of the boss faces, the latent state waits for a trophy and lights the faces, a spare property keeps
    // the stored metadata the source's so the model's cuboids still pick by it
    pub const TURNS: i32 = 4;
    pub const LATENT: i32 = 8;
    pub const TURN_VALUES: [&str; 4] = ["0", "1", "2", "3"];
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

fn lit(meta: i32) -> bool { meta >= pedestal::LATENT }

// any state but the plain unturned one, which the pedestal treats as placed by a player
fn guarded(state: Option<Rc<StateData>>) -> bool
{
    state.is_some_and(|state| state.flag("latent") || state.flag("spare") || state.get("turn") != Some("0"))
}

fn trophyOnTop(world: World, pos: BlockPos) -> bool { world.is(pos.offset(0, 1, 0), BLOCK) }

// BlockTFTrophyPedestal.updateTick once a trophy sits on it, players are rewarded before the check so the first
// worthy visit opens it instead of needing a second trophy placement
fn crown(world: World, pos: BlockPos)
{
    let center: [f64; 3] = crate::math::center(pos);
    let near: Vec<Player> = world
        .players()
        .into_iter()
        .filter(|player| player.info().is_some_and(|info| distance(info.pos, center) <= pedestal::REACH))
        .collect();

    for player in &near
    {
        progress::grant(*player, Gate::Trophy);
    }

    if !names::flag(world.stateAt(pos), "latent")
    {
        return;
    }

    if near.iter().any(|player| progress::has(*player, Gate::Trophy))
    {
        open(world, pos);
    }

    for player in near.iter().filter(|player| !progress::has(**player, Gate::Trophy))
    {
        player.chat("You are unworthy.");
    }
}

// BlockTFTrophyPedestal.doPedestalEffect, only the latent shields the access chamber lays under its floor vanish
fn open(world: World, pos: BlockPos)
{
    let reach: i32 = pedestal::SHIELDS;
    for x in -reach..=reach
    {
        for y in -reach..=reach
        {
            for z in -reach..=reach
            {
                let at: BlockPos = pos.offset(x, y, z);
                let (shield, meta): (i32, i32) = (world.block(at), world.meta(at));
                let state: Option<Rc<StateData>> = stateOf(shield, meta);
                let latent: bool = state.is_some_and(|state| state.get("facing") == Some(crate::blocks::SHIELD_LATENT));
                if blockIs(shield, block::SHIELD) && latent
                {
                    let _ = world.clear(at);
                    let _ = world.breakEffect(at, shield, meta);
                }
            }
        }
    }

    let _ = world.setProperty(pos, "latent", false);

    world.playSound(crate::math::center(pos), "mob.zombie.infect", 4.0, 0.1);
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

        let quarter: i32 = floor(f64::from(info.yaw) * 4.0 / 360.0 + 0.5) & 3;
        let turn: usize = [0, 1, 3, 2][quarter as usize];
        if let Err(error) = ctx.world().setProperty(ctx.pos(), "turn", pedestal::TURN_VALUES[turn])
        {
            warn(&format!("trophy pedestal at {:?}: {}", ctx.pos(), error.0));
        }
    }

    // BlockTFTrophyPedestal.getPlayerRelativeBlockHardness, any meta but the plain first one cannot be broken
    fn strength(&self, ctx: &mut BlockCtx, _player: Player) -> Option<f32> { guarded(ctx.stateAt()).then_some(0.0) }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32)
    {
        let (world, pos): (World, BlockPos) = (ctx.world(), ctx.pos());
        if ctx.isAuthority() && guarded(ctx.stateAt()) && trophyOnTop(world, pos)
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
            let lit: bool = lit(meta);

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
    ctx.blocks().states(
        block::TROPHY_PEDESTAL,
        BlockStatesSpec
        {
            properties: vec![
                PropertySpec::new("latent", &["false", "true"]),
                PropertySpec::new("spare", &["false", "true"]),
                PropertySpec::new("turn", &pedestal::TURN_VALUES),
            ],
            ..BlockStatesSpec::default()
        },
    )
}

// BlockTFTrophy and TileEntityTFTrophy: the block's meta says where it sits, 1 on the floor and 2 to 5 on a wall
// facing out of that face, the tile keeps which boss and the floor turn in sixteenths
mod head
{
    pub const FLOOR: i32 = 1;
    // the face a trophy sits against in BlockSkull metadata order, the model's cuboids pick by that metadata
    pub const FACINGS: [&str; 6] = ["down", "up", "north", "south", "west", "east"];
    pub const TURNS: i32 = 16;
    pub const FULL_TURN: f32 = 360.0;
    pub const TILE: &str = "trophy_head";
    // RenderBlockTFPedestal's skull bounds, BlockSkull.setBlockBoundsBasedOnState
    pub const BOUNDS: [[[f32; 3]; 2]; 5] = [
        [[4.0, 0.0, 4.0], [12.0, 8.0, 12.0]],
        [[4.0, 4.0, 8.0], [12.0, 12.0, 16.0]],
        [[4.0, 4.0, 0.0], [12.0, 12.0, 8.0]],
        [[8.0, 4.0, 4.0], [16.0, 12.0, 12.0]],
        [[0.0, 4.0, 4.0], [8.0, 12.0, 12.0]],
    ];
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Head
{
    m_kind: u8,
    m_turn: u8,
}

struct TrophyItem
{
    m_kind: i32,
}

#[item_behavior]
impl ItemBehavior for TrophyItem
{
    // ItemTFTrophy.onItemUse, set down on top of or against the side of a solid block, never under one
    fn onUseOnBlock(&self, world: World, player: Player, stack: ItemStack, pos: BlockPos, face: i32) -> Use
    {
        let offset: [i32; 3] = crate::world::structures::facing(face);
        let at: BlockPos = pos.offset(offset[0], offset[1], offset[2]);
        let solid: bool = World::blockInfo(world.block(pos)).solid;
        if face == 0 || !solid || !World::blockInfo(world.block(at)).replaceable && world.block(at) != 0
        {
            return Use::Pass;
        }

        if world.isAuthority()
        {
            let yaw: f32 = player.info().map_or(0.0, |info| info.yaw);

            let turn: i32 = if face == head::FLOOR
            {
                floor(f64::from(yaw * head::TURNS as f32 / head::FULL_TURN + 0.5)) & (head::TURNS - 1)
            }
            else
            {
                0
            };
            let _ = world.setBlockState(at, names::state(BLOCK).with("facing", head::FACINGS[face as usize]));
            let state: Head = Head { m_kind: self.m_kind as u8, m_turn: turn as u8 };
            if let Err(error) = world.setTileData(at, &state)
            {
                warn(&format!("trophy at {at:?}: {}", error.0));
            }
        }

        let left: i32 = stack.count - 1;
        Use::Done((left > 0).then_some(ItemStack { count: left, ..stack }))
    }
}

// BlockTFTrophy drops the trophy of its boss
struct TrophyBlock;

fn dropHead(ctx: &mut BlockCtx)
{
    if !ctx.isAuthority()
    {
        return;
    }
    let state: Head = ctx.tileData();
    let Some(trophy) = TROPHIES.get(usize::from(state.m_kind))
    else
    {
        return;
    };

    let name: String = format!("twilightforest:{}", trophy.m_item);
    if let Err(error) = ctx.world().spawnItem(crate::math::center(ctx.pos()), ItemStack::of(name.as_str(), 1))
    {
        warn(&format!("trophy drop: {}", error.0));
    }
}

#[block_behavior]
impl BlockBehavior for TrophyBlock
{
    fn onDestroyedByPlayer(&self, ctx: &mut BlockCtx) { dropHead(ctx) }

    fn onDestroyedByExplosion(&self, ctx: &mut BlockCtx) { dropHead(ctx) }
}

// the five heads share one model, each kind's parts start where the one before it ends
fn kindStarts() -> [u16; 5]
{
    thread_local!
    {
        static STARTS: std::cell::OnceCell<[u16; 5]> = const { std::cell::OnceCell::new() };
    }

    STARTS.with(|starts|
    {
        *starts.get_or_init(||
        {
            let sizes: [usize; 5] = [
                tf_hydra_head::model().parts.len(),
                tf_naga::model().parts.len(),
                tf_lich::model().parts.len(),
                super::ur_ghast::model().parts.len(),
                tf_snow_queen::model().parts.len(),
            ];

            let mut first: u16 = 0;

            sizes.map(|size|
            {
                let start: u16 = first;

                first += size as u16;
                start
            })
        })
    })
}

fn headModel() -> EntityModelSpec
{
    combine(vec![
        tf_hydra_head::model(),
        tf_naga::model(),
        tf_lich::model(),
        super::ur_ghast::model(),
        tf_snow_queen::model(),
    ])
}

// TileEntityTFTrophyRenderer, each boss head turned and lifted as the source draws it on a floor or a wall
struct HeadTile;

#[tile_behavior]
impl TileBehavior for HeadTile
{
    fn pose(&self, ctx: &mut BlockCtx, partial: f32, ticks: i32) -> Pose
    {
        let state: Head = ctx.tileData();
        let facing: Option<usize> = names::pick(ctx.stateAt(), "facing", &head::FACINGS);
        let floor: bool = facing == Some(head::FLOOR as usize);

        let turn: f32 = match facing.map(|facing| head::FACINGS[facing])
        {
            Some("north") => 0.0,
            Some("south") => 180.0,
            Some("west") => 270.0,
            Some("east") => 90.0,
            _ => f32::from(state.m_turn) * head::FULL_TURN / head::TURNS as f32,
        };

        let starts: [u16; 5] = kindStarts();
        let mut body: Program = Program::new();
        body.translate(0.5, 0.5, 0.5);

        let face = |body: &mut Program, texture: &str|
        {
            body.texture(texture).scale(1.0, -1.0, -1.0).rotate(turn, 0.0, 1.0, 0.0).rotate(180.0, 0.0, 1.0, 0.0);
        };

        match state.m_kind
        {
            0 =>
            {
                body.scale(0.25, 0.25, 0.25);
                face(&mut body, "twilightforest:model/hydra4");
                body.translate(0.0, if floor { 1.0 } else { 0.0 }, 1.5);
                // ModelTFHydraHead.openMouthForTrophy
                let mouth: f32 = if floor { 0.0 } else { 0.25 };
                let first: u16 = starts[0];
                body.part(
                    first + tf_hydra_head::HEAD,
                    [-mouth * std::f32::consts::PI / 12.0, 0.0, 0.0],
                    tf_hydra_head::PIVOTS[0],
                );
                body.part(
                    first + tf_hydra_head::JAW,
                    [mouth * std::f32::consts::PI / 3.0, 0.0, 0.0],
                    tf_hydra_head::PIVOTS[1],
                );
                body.part(first + tf_hydra_head::FRILL, tf_hydra_head::ROTATIONS[2], tf_hydra_head::PIVOTS[2]);
            }

            1 =>
            {
                body.translate(0.0, -0.125, 0.0).scale(0.25, 0.25, 0.25);
                face(&mut body, "twilightforest:model/nagahead");
                body.translate(0.0, if floor { 1.0 } else { 0.0 }, if floor { 0.0 } else { 1.0 });
                body.scale(2.0, 2.0, 2.0).rest(starts[1] + tf_naga::HEAD);
            }

            2 =>
            {
                body.translate(0.0, 1.0, 0.0);
                face(&mut body, "twilightforest:model/twilightlich64");
                body.translate(0.0, if floor { 1.75 } else { 1.5 }, if floor { 0.0 } else { 0.24 });
                body.rest(starts[2] + tf_lich::HEAD).rest(starts[2] + tf_lich::HEADWEAR);
            }

            3 =>
            {
                body.translate(0.0, 1.0, 0.0).scale(0.5, 0.5, 0.5);
                face(&mut body, "twilightforest:model/towerboss");
                body.translate(0.0, 1.0, 0.0);
                super::ur_ghast::drawBody(&mut body, starts[3], ticks as f32 + partial, [0.0, 0.0]);
            }

            _ =>
            {
                body.translate(0.0, 1.0, 0.0);
                face(&mut body, "twilightforest:model/snowqueen");
                body.translate(0.0, if floor { 1.5 } else { 1.25 }, if floor { 0.0 } else { 0.24 });
                body.rest(starts[4] + tf_snow_queen::HEAD).rest(starts[4] + tf_snow_queen::HEADWEAR);
            }
        }

        Pose { pre: Vec::new(), model: body.take() }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    registerPedestal(ctx)?;
    // the bounds of each placement, drawn with an empty sprite since the tile draws the head
    let cuboids: Vec<Cuboid> = head::BOUNDS
        .iter()
        .enumerate()
        .map(|(index, [from, to])| Cuboid
        {
            from: *from,
            to: *to,
            texture: Some("twilightforest:block/blank".to_owned()),
            metas: vec![head::FLOOR + index as i32],
            ..Cuboid::default()
        })
        .collect();
    let spec: TileEntitySpec = TileEntitySpec { inventory: 0, ticks: false, sync: true, behavior: None };
    ctx.tileEntities().register(head::TILE, spec, Some(std::rc::Rc::new(HeadTile)))?;
    ctx.tileEntities().model(head::TILE, headModel())?;
    ctx.blocks().register(
        BLOCK,
        BlockSpecBuilder::template("minecraft:block")
            .name("Trophy")
            .material("circuits")
            .texture("twilightforest:block/blank")
            .hardness(1.0)
            .model(BlockModelSpec { cuboids, translucent: true, ..BlockModelSpec::default() })
            .tileEntity(head::TILE)
            .drops(Vec::new())
            .behavior(TrophyBlock),
    )?;
    ctx.blocks().states(
        BLOCK,
        BlockStatesSpec { properties: vec![PropertySpec::new("facing", &head::FACINGS)], ..BlockStatesSpec::default() },
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
