use crate::core::{dir, render, state};
use b173_api::prelude::*;

// BlockMarker's placement on any full block face, ceilings included, and mod_BuildCraftCore.renderMarkerWithMeta
pub(crate) const SPRITE: &str = "buildcraft:block/marker";
const INVISIBLE: &str = "buildcraft:block/laser_transparent";

// the face of the block a landmark leans on, its states in BlockMarker's metadata order
pub(crate) const SUPPORT: &str = "support";
const SUPPORTS: [&str; 6] = ["up", "west", "east", "north", "south", "down"];
// onBlockAdded's search order
const ADDED_ORDER: [i32; 6] = [dir::WEST, dir::EAST, dir::NORTH, dir::SOUTH, dir::DOWN, dir::UP];

mod shape
{
    // getSelectedBoundingBoxFromPool: a 0.3 wide stick 0.65 long from the support, in model units
    pub const NEAR: f32 = 16.0 * (0.5 - 0.15);
    pub const FAR: f32 = 16.0 * (0.5 + 0.15);
    pub const LENGTH: f32 = 16.0 * 0.65;
    pub const EDGE: f32 = 16.0;
    // renderMarkerWithMeta's half width and the top quad's uv inset
    pub const HALF: f64 = 0.0625;
    pub const TOP_U: f64 = 0.02734375;
    pub const TOP_V: f64 = 0.0234375;
    pub const TILE: f64 = 15.99;
    pub const SHEET: f64 = 256.0;
    pub const TILES: i32 = 16;
}

// air answers with the default block info, which is not opaque
fn supported(world: World, pos: BlockPos) -> bool { World::blockInfo(world.block(pos)).opaque }

fn holds(world: World, pos: BlockPos, support: i32) -> bool { supported(world, dir::step(pos, support)) }

fn lean(world: World, pos: BlockPos, support: i32) { state::set(world, pos, SUPPORT, dir::name(support)); }

pub(crate) fn register(ctx: &mut RegisterCtx, block: &str) -> Result<()>
{
    state::register(ctx, block, SUPPORT, &SUPPORTS)?;
    ctx.blocks().stateModel(block, model())
}

// canPlaceBlockAt: any neighbour drawn as a full block
pub(crate) fn canPlace(world: World, pos: BlockPos) -> bool
{
    dir::all().any(|face| supported(world, dir::step(pos, face)))
}

// onBlockPlaced: the clicked face wins when it holds
pub(crate) fn placed(ctx: &mut BlockCtx, face: i32)
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    let support: i32 = dir::reverse(face.clamp(dir::DOWN, dir::EAST));

    if holds(world, pos, support)
    {
        lean(world, pos, support);
    }
}

// onBlockAdded: the first support in BlockMarker's order, gone when there is none
pub(crate) fn added(ctx: &mut BlockCtx)
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    if !ctx.isAuthority()
    {
        return;
    }

    if let Some(support) = ADDED_ORDER.iter().copied().find(|support| holds(world, pos, *support))
    {
        lean(world, pos, support);
        return;
    }

    crate::core::stacks::breakBlock(world, pos);
}

// onNeighborBlockChange: a marker whose support went away drops
pub(crate) fn neighborChanged(ctx: &mut BlockCtx)
{
    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    if !ctx.isAuthority()
    {
        return;
    }

    let support: i32 = state::face(ctx.stateAt(), SUPPORT).unwrap_or(dir::UP);
    if !canPlace(world, pos) || !holds(world, pos, support)
    {
        crate::core::stacks::breakBlock(world, pos);
    }
}

// the selection stick per support, drawn invisibly so only the tile's marker model shows
fn model() -> StateModelSpec
{
    let boxes: [([f32; 3], [f32; 3]); 6] = [
        ([shape::NEAR, shape::EDGE - shape::LENGTH, shape::NEAR], [shape::FAR, shape::EDGE, shape::FAR]),
        ([0.0, shape::NEAR, shape::NEAR], [shape::LENGTH, shape::FAR, shape::FAR]),
        ([shape::EDGE - shape::LENGTH, shape::NEAR, shape::NEAR], [shape::EDGE, shape::FAR, shape::FAR]),
        ([shape::NEAR, shape::NEAR, 0.0], [shape::FAR, shape::FAR, shape::LENGTH]),
        ([shape::NEAR, shape::NEAR, shape::EDGE - shape::LENGTH], [shape::FAR, shape::FAR, shape::EDGE]),
        ([shape::NEAR, 0.0, shape::NEAR], [shape::FAR, shape::LENGTH, shape::FAR]),
    ];

    let cuboids: Vec<StateCuboid> = boxes
        .into_iter()
        .zip(SUPPORTS)
        .flat_map(|((from, to), support)|
        {
            let cuboid: Cuboid =
                Cuboid { from, to, texture: Some(INVISIBLE.to_owned()), metas: Vec::new(), connect: None };
            state::cuboids(&cuboid, state::when(SUPPORT, &SUPPORTS, |value| value == support))
        })
        .collect();
    StateModelSpec { cuboids }
}

// rotateFace, the four corners of a face shifted one along
fn rotate(face: &mut Face)
{
    for row in face.iter_mut()
    {
        row.rotate_left(1);
    }
}

fn turned(base: Face, turns: usize) -> Face
{
    let mut face: Face = base;
    (0..turns).for_each(|_| rotate(&mut face));
    face
}

// a face's four corners as x, y and z rows, one per support or None where it draws none
type Face = [[f64; 4]; 3];
type Faces = [Option<Face>; 6];

// initializeMarkerMatrix: the cross pieces' corners for each support
fn frames() -> (Faces, Faces, Faces)
{
    let x: Face = [[-0.0625; 4], [1.0, 0.0, 0.0, 1.0], [-0.5, -0.5, 0.5, 0.5]];
    let z: Face = [[-0.5, -0.5, 0.5, 0.5], [1.0, 0.0, 0.0, 1.0], [0.0625; 4]];
    let y: Face = [[-0.5, -0.5, 0.5, 0.5], [-0.0625; 4], [0.5, -0.5, -0.5, 0.5]];

    let frontX: Faces = [Some(turned(x, 2)), None, None, Some(turned(x, 3)), Some(turned(x, 1)), Some(x)];
    let frontZ: Faces = [Some(turned(z, 2)), Some(turned(z, 3)), Some(turned(z, 1)), None, None, Some(z)];
    let frontY: Faces = [None, Some(turned(y, 3)), Some(turned(y, 1)), Some(y), Some(turned(y, 2)), None];
    (frontX, frontZ, frontY)
}

// renderMarkerWithMeta: the glowing tip and two crossed planes, in block space from the lower corner
pub(crate) fn quads(support: i32) -> Vec<[f32; 5]>
{
    let slot: i32 = render::slot(SPRITE);
    let left: f64 = f64::from((slot & (shape::TILES - 1)) * shape::TILES) / shape::SHEET;
    let top: f64 = f64::from((slot / shape::TILES) * shape::TILES) / shape::SHEET;
    let right: f64 = left + shape::TILE / shape::SHEET;
    let bottom: f64 = top + shape::TILE / shape::SHEET;
    let (tipU, tipV): (f64, f64) = (left + shape::TOP_U, top + shape::TOP_V);
    let (x, y, z): (f64, f64, f64) = (0.5, 0.0, 0.5);
    let s: f64 = shape::HALF;
    let mut out: Vec<[f64; 5]> = Vec::new();

    let tip: [[f64; 3]; 4] = match support
    {
        dir::DOWN => [
            [x - s, y + 0.5 + s, z - s],
            [x - s, y + 0.5 + s, z + s],
            [x + s, y + 0.5 + s, z + s],
            [x + s, y + 0.5 + s, z - s],
        ],
        dir::UP => [
            [x + s, y + 0.5 - s, z - s],
            [x + s, y + 0.5 - s, z + s],
            [x - s, y + 0.5 - s, z + s],
            [x - s, y + 0.5 - s, z - s],
        ],
        dir::EAST => [
            [x - s, y + 0.5 - s, z - s],
            [x - s, y + 0.5 - s, z + s],
            [x - s, y + 0.5 + s, z + s],
            [x - s, y + 0.5 + s, z - s],
        ],
        dir::WEST => [
            [x + s, y + 0.5 + s, z - s],
            [x + s, y + 0.5 + s, z + s],
            [x + s, y + 0.5 - s, z + s],
            [x + s, y + 0.5 - s, z - s],
        ],
        dir::NORTH => [
            [x - s, y + 0.5 - s, z + s],
            [x + s, y + 0.5 - s, z + s],
            [x + s, y + 0.5 + s, z + s],
            [x - s, y + 0.5 + s, z + s],
        ],
        _ => [
            [x - s, y + 0.5 + s, z - s],
            [x + s, y + 0.5 + s, z - s],
            [x + s, y + 0.5 - s, z - s],
            [x - s, y + 0.5 - s, z - s],
        ],
    };
    out.extend(tip.into_iter().map(|[px, py, pz]| [px, py, pz, tipU, tipV]));

    let uvs: [[f64; 2]; 4] = [[left, top], [left, bottom], [right, bottom], [right, top]];
    let (frontX, frontZ, frontY) = frames();
    let index: usize = SUPPORTS.iter().position(|name| *name == dir::name(support)).unwrap_or(0);

    if let Some(face) = frontX[index]
    {
        out.extend((0..4).map(|i| [x + face[0][i], y + face[1][i], z + face[2][i], uvs[i][0], uvs[i][1]]));
        out.extend((0..4).rev().map(|i| [x - face[0][i], y + face[1][i], z + face[2][i], uvs[i][0], uvs[i][1]]));
    }

    if let Some(face) = frontZ[index]
    {
        out.extend((0..4).map(|i| [x + face[0][i], y + face[1][i], z + face[2][i], uvs[i][0], uvs[i][1]]));
        out.extend((0..4).rev().map(|i| [x + face[0][i], y + face[1][i], z - face[2][i], uvs[i][0], uvs[i][1]]));
    }

    if let Some(face) = frontY[index]
    {
        out.extend((0..4).map(|i| [x + face[0][i], y + 0.5 + face[1][i], z + face[2][i], uvs[i][0], uvs[i][1]]));
        out.extend((0..4).rev().map(|i| [x + face[0][i], y + 0.5 - face[1][i], z + face[2][i], uvs[i][0], uvs[i][1]]));
    }

    out.into_iter().map(|vertex| vertex.map(|value| value as f32)).collect()
}
