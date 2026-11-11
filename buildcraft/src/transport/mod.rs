mod diamond;
mod liquids;
mod obsidian;
mod pickup;
mod pipe;
mod render;
mod travel;

use crate::core::dir;
use b173_api::prelude::*;
pub(crate) use pipe::toRandomPipe;
use std::rc::Rc;

pub(crate) const ALWAYS_CONNECT: &str = "pipes.always_connect";
pub(crate) const EXCLUSION: &str = "wooden_pipe.exclusion";

// the pipes of BuildCraftTransport in their order there
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind
{
    Wooden,
    Stone,
    Iron,
    Golden,
    Diamond,
    Obsidian,
    Cobblestone,
}

pub(crate) struct Pipe
{
    pub(crate) m_kind: Kind,
    pub(crate) m_name: &'static str,
    m_material: &'static str,
    m_sound: &'static str,
    m_ingredient: &'static str,
    // the block a pipe of the same kind refuses to join, as BlockWoodenPipe.isPipeConnected and friends
    m_refuses: Option<Kind>,
}

pub(crate) const PIPES: [Pipe; 7] = [
    Pipe
    {
        m_kind: Kind::Wooden,
        m_name: "wooden_pipe",
        m_material: "wood",
        m_sound: "wood",
        m_ingredient: "minecraft:planks",
        m_refuses: Some(Kind::Wooden),
    },
    Pipe
    {
        m_kind: Kind::Stone,
        m_name: "stone_pipe",
        m_material: "rock",
        m_sound: "stone",
        m_ingredient: "minecraft:stone",
        m_refuses: Some(Kind::Cobblestone),
    },
    Pipe
    {
        m_kind: Kind::Iron,
        m_name: "iron_pipe",
        m_material: "iron",
        m_sound: "metal",
        m_ingredient: "minecraft:iron_ingot",
        m_refuses: None,
    },
    Pipe
    {
        m_kind: Kind::Golden,
        m_name: "golden_pipe",
        m_material: "iron",
        m_sound: "metal",
        m_ingredient: "minecraft:gold_ingot",
        m_refuses: Some(Kind::Golden),
    },
    Pipe
    {
        m_kind: Kind::Diamond,
        m_name: "diamond_pipe",
        m_material: "iron",
        m_sound: "metal",
        m_ingredient: "minecraft:diamond",
        m_refuses: None,
    },
    Pipe
    {
        m_kind: Kind::Obsidian,
        m_name: "obsidian_pipe",
        m_material: "iron",
        m_sound: "metal",
        m_ingredient: "minecraft:obsidian",
        m_refuses: Some(Kind::Obsidian),
    },
    Pipe
    {
        m_kind: Kind::Cobblestone,
        m_name: "cobblestone_pipe",
        m_material: "rock",
        m_sound: "stone",
        m_ingredient: "minecraft:cobblestone",
        m_refuses: Some(Kind::Stone),
    },
];

mod shape
{
    // ModLoader's pipe model: a quarter to three quarters core and an arm to each joined face
    pub const MIN: f32 = 4.0;
    pub const MAX: f32 = 12.0;
    pub const EDGE: f32 = 16.0;
    pub const HARDNESS: f32 = 0.5;
}

// the gold pipe keeps its power in its metadata so the model shows it, BlockGoldenPipe reads it at draw time
pub(crate) const GOLD_POWERED: u8 = 1;
// wooden pipe metadata past the six faces means it pulls from nowhere
pub(crate) const NO_SOURCE: u8 = 6;

pub(crate) fn byBlock(block: i32) -> Option<&'static Pipe>
{
    PIPES.iter().find(|pipe| blockId(&format!("buildcraft:{}", pipe.m_name)) == Some(block))
}

pub(crate) fn blockId(name: &str) -> Option<i32> { crate::core::ids::block(name) }

pub(crate) fn kindAt(world: World, pos: BlockPos) -> Option<Kind> { byBlock(world.block(pos)).map(|pipe| pipe.m_kind) }

pub(crate) fn isPipe(world: World, pos: BlockPos) -> bool { kindAt(world, pos).is_some() }

fn alwaysConnect() -> bool { configValue(ALWAYS_CONNECT) == true.to_string() }

// BlockPipe.isPipeConnected: a pipe joins pipes, inventories, fluid holders and BuildCraft machines
fn pipeJoins(world: World, kind: Kind, other: BlockPos, face: i32) -> bool
{
    if let Some(otherKind) = kindAt(world, other)
    {
        let pipe: &Pipe = PIPES.iter().find(|pipe| pipe.m_kind == kind).expect("every kind has a row");
        return alwaysConnect() || pipe.m_refuses != Some(otherKind);
    }

    let reachable: i32 = capability::ITEMS | capability::FLUIDS;
    world.capabilities(other, dir::reverse(face)) & reachable != 0 || crate::machines::isMachine(world, other)
}

// a pipe's own rule toward a face, for a pipe on that side asking to join it
pub(crate) fn pipeAccepts(world: World, pos: BlockPos, face: i32) -> bool
{
    kindAt(world, pos).is_some_and(|kind| pipeJoins(world, kind, dir::step(pos, face), face))
}

// Utils.checkPipesConnections from a pipe toward a face: the pipe's own rule, then the neighbour's if it has one
pub(crate) fn joins(world: World, pos: BlockPos, face: i32) -> bool
{
    let Some(kind) = kindAt(world, pos)
    else
    {
        return false;
    };
    let other: BlockPos = dir::step(pos, face);

    pipeJoins(world, kind, other, face) && crate::machines::acceptsPipe(world, other, pos)
}

fn cuboid(from: [f32; 3], to: [f32; 3], texture: &str, metas: Vec<i32>, connect: Option<i32>) -> Cuboid
{
    Cuboid { from, to, texture: Some(texture.to_owned()), metas, connect }
}

// an arm from the core to one face, the box the model and collision share
pub(crate) fn arm(face: i32) -> ([f32; 3], [f32; 3])
{
    let mut from: [f32; 3] = [shape::MIN; 3];
    let mut to: [f32; 3] = [shape::MAX; 3];
    let axis: usize = match face
    {
        dir::DOWN | dir::UP => 1,
        dir::NORTH | dir::SOUTH => 2,
        _ => 0,
    };

    if face % 2 == 0
    {
        from[axis] = 0.0;
        to[axis] = shape::MIN;
    }
    else
    {
        from[axis] = shape::MAX;
        to[axis] = shape::EDGE;
    }

    (from, to)
}

fn texture(name: &str) -> String { format!("buildcraft:block/{name}") }

// each arm's sprite: IBlockPipe.getTextureForConnection by face and metadata
fn arms(kind: Kind, name: &str) -> Vec<Cuboid>
{
    let mut cuboids: Vec<Cuboid> = Vec::new();
    let plain: String = texture(name);
    let all: Vec<i32> = (0..16).collect();

    for face in dir::all()
    {
        let (from, to) = arm(face);
        let others: Vec<i32> = all.iter().copied().filter(|meta| *meta != face).collect();

        match kind
        {
            Kind::Wooden =>
            {
                cuboids.push(cuboid(from, to, &texture("wooden_pipe_plain"), vec![face], Some(face)));
                cuboids.push(cuboid(from, to, &plain, others, Some(face)));
            }

            Kind::Iron =>
            {
                cuboids.push(cuboid(from, to, &plain, vec![face], Some(face)));
                cuboids.push(cuboid(from, to, &texture("iron_pipe_plain"), others, Some(face)));
            }

            Kind::Diamond =>
            {
                const FACES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];
                cuboids.push(cuboid(
                    from,
                    to,
                    &texture(&format!("diamond_pipe_{}", FACES[face as usize])),
                    Vec::new(),
                    Some(face),
                ));
            }

            Kind::Golden =>
            {
                cuboids.push(cuboid(
                    from,
                    to,
                    &texture("golden_pipe_powered"),
                    vec![i32::from(GOLD_POWERED)],
                    Some(face),
                ));
                cuboids.push(cuboid(from, to, &plain, vec![0], Some(face)));
            }

            _ => cuboids.push(cuboid(from, to, &plain, Vec::new(), Some(face))),
        }
    }

    cuboids
}

fn model(kind: Kind, name: &str) -> BlockModelSpec
{
    let core: [f32; 3] = [shape::MIN; 3];
    let far: [f32; 3] = [shape::MAX; 3];
    let mut cuboids: Vec<Cuboid> = match kind
    {
        Kind::Golden => vec![
            cuboid(core, far, &texture("golden_pipe_powered"), vec![i32::from(GOLD_POWERED)], None),
            cuboid(core, far, &texture(name), vec![0], None),
        ],
        _ => vec![cuboid(core, far, &texture(name), Vec::new(), None)],
    };
    cuboids.extend(arms(kind, name));

    BlockModelSpec { cuboids, ..BlockModelSpec::default() }
}

struct PipeBlock
{
    m_kind: Kind,
}

#[block_behavior]
impl BlockBehavior for PipeBlock
{
    fn connections(&self, ctx: &mut BlockCtx) -> u8
    {
        let world: World = ctx.world();
        dir::all().filter(|face| joins(world, ctx.pos(), *face)).fold(0, |mask, face| mask | 1 << face)
    }

    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool { pipe::activated(self.m_kind, ctx, player) }

    fn onPlaced(&self, ctx: &mut BlockCtx, _face: i32) { pipe::placed(self.m_kind, ctx); }

    fn onNeighborChanged(&self, ctx: &mut BlockCtx, _neighbor: i32) { pipe::neighborChanged(self.m_kind, ctx); }

    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        if self.m_kind == Kind::Obsidian
        {
            pipe::collided(ctx, entity);
        }
    }
}

fn registerPipe(ctx: &mut RegisterCtx, row: &'static Pipe) -> Result<()>
{
    let tile: Rc<dyn TileBehavior> = Rc::new(pipe::PipeTile { m_kind: row.m_kind });
    let slots: i32 = if row.m_kind == Kind::Diamond
    {
        diamond::SLOTS
    }
    else
    {
        0
    };
    ctx.tileEntities().register(
        row.m_name,
        TileEntitySpec { inventory: slots, ticks: true, sync: false, behavior: None },
        Some(tile),
    )?;
    ctx.tileEntities().model(row.m_name, EntityModelSpec::default())?;

    ctx.blocks().register(
        row.m_name,
        BlockSpecBuilder::template("minecraft:block")
            .material(row.m_material)
            .sound(row.m_sound)
            .hardness(shape::HARDNESS)
            .texture(&texture(row.m_name))
            .opacity(0)
            .model(model(row.m_kind, row.m_name))
            .tileEntity(row.m_name)
            .tag("buildcraft:pipes")
            .behavior(PipeBlock { m_kind: row.m_kind }),
    )?;

    ctx.blocks().itemModel(row.m_name, crate::core::render::column(Some(&texture(row.m_name))))?;

    // BuildCraftTransport: eight pipes from two ingredient blocks around a glass pane
    ctx.recipes()
        .shaped(row.m_name, ItemStack::of(format!("buildcraft:{}", row.m_name).as_str(), 8))
        .pattern(["   ", "PGP", "   "])
        .key('P', row.m_ingredient)
        .key('G', "minecraft:glass")
        .build()
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.config().options([
        Setting::bool(ALWAYS_CONNECT, false)
            .label("Pipes always connect")
            .comment("set to true to deactivate pipe connection rules, false by default"),
        Setting::text(EXCLUSION, "")
            .label("Wooden pipe exclusions")
            .comment("blocks wooden pipes never pull from, by identifier, separated by commas"),
    ])?;

    for row in &PIPES
    {
        registerPipe(ctx, row)?;
    }

    diamond::register(ctx)?;
    pickup::register(ctx)?;
    pipe::registerChannels(ctx)
}
