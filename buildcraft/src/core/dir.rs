use b173_api::BlockPos;

// buildcraft.api.Orientations, whose order is the block face order: down, up, north, south, west, east
pub(crate) const COUNT: i32 = 6;
pub(crate) const UNKNOWN: i32 = 6;
pub(crate) const DOWN: i32 = 0;
pub(crate) const UP: i32 = 1;
pub(crate) const NORTH: i32 = 2;
pub(crate) const SOUTH: i32 = 3;
pub(crate) const WEST: i32 = 4;
pub(crate) const EAST: i32 = 5;

const OFFSETS: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

pub(crate) fn offset(face: i32) -> [i32; 3] { OFFSETS[face.rem_euclid(COUNT) as usize] }

pub(crate) fn step(pos: BlockPos, face: i32) -> BlockPos
{
    let [dx, dy, dz] = offset(face);
    pos.offset(dx, dy, dz)
}

// Orientations.reverse, which keeps Unknown
pub(crate) fn reverse(face: i32) -> i32
{
    if face == UNKNOWN
    {
        return UNKNOWN;
    }

    face ^ 1
}

pub(crate) fn all() -> std::ops::Range<i32> { 0..COUNT }

// Position.moveForwards on a double position
pub(crate) fn forward(pos: [f64; 3], face: i32, step: f64) -> [f64; 3]
{
    if face == UNKNOWN
    {
        return pos;
    }

    let [dx, dy, dz] = offset(face);
    [pos[0] + dx as f64 * step, pos[1] + dy as f64 * step, pos[2] + dz as f64 * step]
}

// Utils.get2dOrientation, the side of `to` that faces `from`
pub(crate) fn flat(from: [f64; 3], to: [f64; 3]) -> i32
{
    const QUARTER: f64 = 45.0;
    const HALF_TURN: f64 = 180.0;

    let dx: f64 = from[0] - to[0];
    let dz: f64 = from[2] - to[2];
    let angle: f64 = dz.atan2(dx).to_degrees() + HALF_TURN;

    match angle
    {
        angle if !(QUARTER..=QUARTER * 7.0).contains(&angle) => EAST,
        angle if angle < QUARTER * 3.0 => SOUTH,
        angle if angle < QUARTER * 5.0 => WEST,
        _ => NORTH,
    }
}

pub(crate) fn center(pos: BlockPos) -> [f64; 3]
{
    const HALF: f64 = 0.5;

    [pos.x as f64 + HALF, pos.y as f64 + HALF, pos.z as f64 + HALF]
}
