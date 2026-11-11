use b173_api::BlockPos;

// the block face order: down, up, north, south, west, east
pub(crate) const COUNT: i32 = 6;
pub(crate) const DOWN: i32 = 0;
pub(crate) const UP: i32 = 1;
pub(crate) const NORTH: i32 = 2;
pub(crate) const SOUTH: i32 = 3;
pub(crate) const WEST: i32 = 4;
pub(crate) const EAST: i32 = 5;

const OFFSETS: [[i32; 3]; 6] = [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]];

// the facing values of block states, in face order
pub(crate) const NAMES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];

pub(crate) fn offset(face: i32) -> [i32; 3] { OFFSETS[face.rem_euclid(COUNT) as usize] }

pub(crate) fn step(pos: BlockPos, face: i32) -> BlockPos
{
    let [dx, dy, dz] = offset(face);
    pos.offset(dx, dy, dz)
}

pub(crate) fn reverse(face: i32) -> i32 { face ^ 1 }

pub(crate) fn name(face: i32) -> &'static str { NAMES[face.rem_euclid(COUNT) as usize] }

pub(crate) fn parse(name: &str) -> Option<i32> { NAMES.iter().position(|known| *known == name).map(|face| face as i32) }

pub(crate) fn all() -> std::ops::Range<i32> { 0..COUNT }

pub(crate) fn center(pos: BlockPos) -> [f64; 3]
{
    const HALF: f64 = 0.5;

    [pos.x as f64 + HALF, pos.y as f64 + HALF, pos.z as f64 + HALF]
}

// BlockMultiID.onBlockPlacedBy, the face toward the player from the yaw quarter they look along
pub(crate) fn fromYaw(yaw: f32) -> i32
{
    const QUARTERS: [i32; 4] = [NORTH, EAST, SOUTH, WEST];
    const TURN: f64 = 360.0;
    const ROUND: f64 = 0.5;

    let turns: f64 = f64::from(yaw) * QUARTERS.len() as f64 / TURN + ROUND;
    let quarter: usize = (turns.floor() as i32).rem_euclid(QUARTERS.len() as i32) as usize;
    QUARTERS[quarter]
}

// BlockElectric.onBlockPlacedBy also faces up or down when the player looks steeply
pub(crate) fn fromLook(yaw: f32, pitch: f32) -> i32
{
    const STEEP: i32 = 65;

    let pitch: i32 = pitch.round() as i32;

    match pitch
    {
        pitch if pitch >= STEEP => UP,
        pitch if pitch <= -STEEP => DOWN,
        _ => fromYaw(yaw),
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn yawQuartersFaceLikeBlockMultiId()
    {
        assert_eq!(fromYaw(0.0), NORTH);
    }
}
