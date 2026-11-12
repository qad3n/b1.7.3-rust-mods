mod pump;
mod quarry;
mod tank;
mod well;
mod workbench;

use crate::core::{AIR, dir, render, state};
use b173_api::prelude::*;

pub(crate) const FRAME: &str = "buildcraft:frame";
pub(crate) const MINING_PIPE: &str = "buildcraft:mining_pipe";
pub(crate) const MINING: &str = "mining.enabled";

// the sprites a facing machine shows, BlockQuarry, BlockMiningWell and BlockBuilder pick them by face and metadata
pub(crate) struct Faces
{
    pub(crate) m_front: &'static str,
    pub(crate) m_top: &'static str,
    pub(crate) m_side: &'static str,
    pub(crate) m_bottom: Option<&'static str>,
    pub(crate) m_back: Option<&'static str>,
}

impl Faces
{
    // the texture hooks name sprites no block spec does, the atlas makes room for them while registering
    pub(crate) fn reserve(&self)
    {
        for sprite in
            [Some(self.m_front), Some(self.m_top), Some(self.m_side), self.m_bottom, self.m_back].into_iter().flatten()
        {
            render::slot(sprite);
        }
    }

    // the item faces down and shows its front to the south, a placed machine shows it toward its facing
    pub(crate) fn sprite(&self, face: i32, block: &StateData) -> Option<String>
    {
        let facing: i32 = block.get(state::FACING).and_then(dir::named).unwrap_or(dir::UNKNOWN);
        let sprite: &'static str = match face
        {
            face if face == dir::SOUTH && facing == dir::DOWN => self.m_front,
            dir::UP => self.m_top,
            dir::DOWN if self.m_bottom.is_some() => self.m_bottom.unwrap_or(self.m_side),
            face if face == facing => self.m_front,
            face if facing != dir::UNKNOWN && dir::reverse(facing) == face && self.m_back.is_some() =>
            {
                self.m_back.unwrap_or(self.m_side)
            }
            _ => self.m_side,
        };
        Some(sprite.to_owned())
    }
}

// BlockQuarry, BlockMiningWell and BlockBuilder face away from the player who placed them
pub(crate) fn facePlacer(ctx: &mut BlockCtx, entity: Entity)
{
    let Some(info) = entity.info()
    else
    {
        return;
    };

    let toward: i32 = dir::flat(info.pos, render::corner(ctx.pos()));
    state::set(ctx.world(), ctx.pos(), state::FACING, dir::name(dir::reverse(toward)));
}

// a facing machine's named states, in the face order BuildCraft stored as metadata
pub(crate) fn facing() -> Property { Property::values(state::FACING, &dir::NAMES) }

// MangoPackUtils.isBlockUnbreakable, missing from the BuildCraft zips, read as Utils.unbreakableBlock's bedrock and
// lava plus any block that cannot be mined
pub(crate) fn unbreakable(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    let listed: bool = blockIs(block, "minecraft:bedrock") || blockTagged(block, "minecraft:lava");
    !blockIs(block, AIR) && (listed || World::blockInfo(block).hardness < 0.0)
}

// Utils.softBlock: air, water and unknown blocks are open space
pub(crate) fn soft(world: World, pos: BlockPos) -> bool
{
    let block: i32 = world.block(pos);
    blockIs(block, AIR) || blockTagged(block, "minecraft:water")
}

fn cuboid(from: [f32; 3], to: [f32; 3], texture: &str, connect: Option<i32>) -> Cuboid
{
    Cuboid { from, to, texture: Some(texture.to_owned()), metas: Vec::new(), connect }
}

// BlockFrame joins only other frames, drawn with the pipe model
struct FrameBlock;

#[block_behavior]
impl BlockBehavior for FrameBlock
{
    fn connections(&self, ctx: &mut BlockCtx) -> u8
    {
        let world: World = ctx.world();
        dir::all().filter(|face| world.is(dir::step(ctx.pos(), *face), FRAME)).fold(0, |mask, face| mask | 1 << face)
    }
}

fn registerFrames(ctx: &mut RegisterCtx) -> Result<()>
{
    const FRAME_HARDNESS: f32 = 0.5;
    const CORE: [f32; 3] = [4.0; 3];
    const FAR: [f32; 3] = [12.0; 3];
    const PIPE_FROM: [f32; 3] = [4.0, 0.0, 4.0];
    const PIPE_TO: [f32; 3] = [12.0, 16.0, 12.0];

    let frame: &str = "buildcraft:block/frame";
    let mut cuboids: Vec<Cuboid> = vec![cuboid(CORE, FAR, frame, None)];
    cuboids.extend(dir::all().map(|face|
    {
        let (from, to) = crate::transport::arm(face);
        cuboid(from, to, frame, Some(face))
    }));

    ctx.blocks().register(
        "frame",
        BlockSpecBuilder::template("minecraft:block")
            .material("glass")
            .sound("glass")
            .hardness(FRAME_HARDNESS)
            .texture(frame)
            .opacity(0)
            .drops(Vec::new())
            .tag(crate::machines::CONNECTORS)
            .model(BlockModelSpec { cuboids, ..BlockModelSpec::default() })
            .itemModel(render::column(Some(frame)))
            .behavior(FrameBlock),
    )?;

    // BlockPlainPipe, the mining well's shaft, dropping nothing
    let shaft: &str = "buildcraft:block/mining_pipe";
    ctx.blocks().register(
        "mining_pipe",
        BlockSpecBuilder::template("minecraft:block")
            .material("glass")
            .sound("glass")
            .texture(shaft)
            .opacity(0)
            .drops(Vec::new())
            .tag(crate::machines::CONNECTORS)
            .model(BlockModelSpec
            {
                cuboids: vec![cuboid(PIPE_FROM, PIPE_TO, shaft, None)],
                ..BlockModelSpec::default()
            })
            .itemModel(render::column(Some(shaft))),
    )?;
    Ok(())
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.config().options([Setting::bool(MINING, true)
        .label("Mining")
        .comment("allows crafting the mining well and the quarry")
        .restart()])?;

    registerFrames(ctx)?;
    well::register(ctx)?;
    quarry::register(ctx)?;
    workbench::register(ctx)?;
    tank::register(ctx)?;
    pump::register(ctx)
}

pub(crate) fn quarryActive(world: World, pos: BlockPos) -> bool { quarry::active(world, pos) }

pub(crate) fn wellActive(world: World, pos: BlockPos) -> bool { well::active(world, pos) }

pub(crate) fn mining() -> bool { configValue(MINING) != false.to_string() }
