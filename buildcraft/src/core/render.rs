use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// RenderEntityBlock and the laser boxes of Utils.createLaser, drawn as tile pose ops from terrain sprites
const TERRAIN: &str = "/terrain.png";
const SHEET_TILES: f32 = 16.0;

mod shade
{
    // RenderBlocks' face light: bottom, top, north and south, west and east
    pub const BOTTOM: f32 = 0.5;
    pub const TOP: f32 = 1.0;
    pub const SOUTH_NORTH: f32 = 0.8;
    pub const EAST_WEST: f32 = 0.6;
}

mod laser
{
    // Utils.createLaser puts a 0.1 thick beam through the middle of the block line it joins
    pub const THICKNESS: f64 = 0.1;
    pub const INSET: f64 = 0.45;
    pub const HALF: f64 = 0.5;
}

// the sprites lasers and boxes draw with, a slot per Utils laser kind
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Laser
{
    Red,
    Blue,
    Stripes,
}

impl Laser
{
    fn sprite(self) -> &'static str
    {
        match self
        {
            Laser::Red => "buildcraft:block/laser_red",
            Laser::Blue => "buildcraft:block/laser_blue",
            Laser::Stripes => "buildcraft:block/laser_stripes",
        }
    }
}

thread_local!
{
    static UVS: RefCell<HashMap<&'static str, [f32; 4]>> = RefCell::new(HashMap::new());
}

// a terrain sprite's corners on the sheet, looked up once since slots never move after loading
pub(crate) fn uv(sprite: &'static str) -> [f32; 4]
{
    if let Some(uv) = UVS.with(|uvs| uvs.borrow().get(sprite).copied())
    {
        return uv;
    }

    let slot: i32 = b173_api::world::terrainSlot(sprite).unwrap_or(0);
    let column: f32 = (slot % SHEET_TILES as i32) as f32;
    let row: f32 = (slot / SHEET_TILES as i32) as f32;
    let uv: [f32; 4] =
        [column / SHEET_TILES, row / SHEET_TILES, (column + 1.0) / SHEET_TILES, (row + 1.0) / SHEET_TILES];
    UVS.with(|uvs| uvs.borrow_mut().insert(sprite, uv));
    uv
}

thread_local!
{
    static SLOTS: RefCell<HashMap<&'static str, i32>> = RefCell::new(HashMap::new());
}

// a terrain sprite's slot for the texture hooks, looked up once
pub(crate) fn slot(sprite: &'static str) -> i32
{
    if let Some(slot) = SLOTS.with(|slots| slots.borrow().get(sprite).copied())
    {
        return slot;
    }

    let slot: i32 = b173_api::world::terrainSlot(sprite).unwrap_or(0);
    SLOTS.with(|slots| slots.borrow_mut().insert(sprite, slot));
    slot
}

// WorldProvider.generateLightBrightnessTable for the overworld, a light level as a draw brightness
pub(crate) fn brightness(light: i32) -> f32
{
    const LEVELS: f32 = 15.0;
    const FLOOR: f32 = 0.05;
    const CURVE: f32 = 3.0;

    let dark: f32 = 1.0 - light.clamp(0, LEVELS as i32) as f32 / LEVELS;
    (1.0 - dark) / (dark * CURVE + 1.0) * (1.0 - FLOOR) + FLOOR
}

pub(crate) fn lightAt(world: World, pos: BlockPos) -> f32 { brightness(world.light(pos)) }

pub(crate) struct Scene
{
    pub(crate) m_ops: Vec<ModelOp>,
}

impl Scene
{
    pub(crate) fn new() -> Self
    {
        Scene { m_ops: vec![ModelOp::Push, ModelOp::Lighting(false), ModelOp::Texture(TERRAIN.to_owned())] }
    }

    pub(crate) fn finish(mut self) -> Pose
    {
        self.m_ops.extend([ModelOp::Color([1.0; 4]), ModelOp::Lighting(true), ModelOp::Pop]);
        Pose { pre: self.m_ops, model: Vec::new() }
    }

    fn face(&mut self, light: f32, shade: f32, quads: Vec<[f32; 5]>)
    {
        let level: f32 = light * shade;
        self.m_ops.push(ModelOp::Color([level, level, level, 1.0]));
        self.m_ops.push(ModelOp::Quads(quads));
    }

    // quads drawn as given, lit evenly
    pub(crate) fn quads(&mut self, quads: Vec<[f32; 5]>, light: f32)
    {
        self.m_ops.push(ModelOp::Color([light, light, light, 1.0]));
        self.m_ops.push(ModelOp::Quads(quads));
    }

    // a box inside one block cell, its sprite cropped to the box as RenderBlocks crops a smaller block
    pub(crate) fn cube(&mut self, min: [f32; 3], max: [f32; 3], sprite: [f32; 4], light: f32)
    {
        let [x0, y0, z0] = min;
        let [x1, y1, z1] = max;
        let [u0, v0, u1, v1] = sprite;
        let width: f32 = u1 - u0;
        let height: f32 = v1 - v0;
        let cell: [f32; 3] = [x0.floor(), y0.floor(), z0.floor()];
        let local = |value: f32, axis: usize| (value - cell[axis]).clamp(0.0, 1.0);
        let u = |value: f32, axis: usize| u0 + local(value, axis) * width;
        let v = |value: f32, axis: usize| v0 + local(value, axis) * height;
        let flip = |value: f32, axis: usize| v1 - local(value, axis) * height;

        self.face(
            light,
            shade::BOTTOM,
            vec![
                [x0, y0, z1, u(x0, 0), v(z1, 2)],
                [x0, y0, z0, u(x0, 0), v(z0, 2)],
                [x1, y0, z0, u(x1, 0), v(z0, 2)],
                [x1, y0, z1, u(x1, 0), v(z1, 2)],
            ],
        );
        self.face(
            light,
            shade::TOP,
            vec![
                [x1, y1, z1, u(x1, 0), v(z1, 2)],
                [x1, y1, z0, u(x1, 0), v(z0, 2)],
                [x0, y1, z0, u(x0, 0), v(z0, 2)],
                [x0, y1, z1, u(x0, 0), v(z1, 2)],
            ],
        );
        self.face(
            light,
            shade::SOUTH_NORTH,
            vec![
                [x0, y1, z0, u(x0, 0), flip(y1, 1)],
                [x1, y1, z0, u(x1, 0), flip(y1, 1)],
                [x1, y0, z0, u(x1, 0), flip(y0, 1)],
                [x0, y0, z0, u(x0, 0), flip(y0, 1)],
                [x0, y1, z1, u(x0, 0), flip(y1, 1)],
                [x0, y0, z1, u(x0, 0), flip(y0, 1)],
                [x1, y0, z1, u(x1, 0), flip(y0, 1)],
                [x1, y1, z1, u(x1, 0), flip(y1, 1)],
            ],
        );
        self.face(
            light,
            shade::EAST_WEST,
            vec![
                [x0, y1, z1, u(z1, 2), flip(y1, 1)],
                [x0, y1, z0, u(z0, 2), flip(y1, 1)],
                [x0, y0, z0, u(z0, 2), flip(y0, 1)],
                [x0, y0, z1, u(z1, 2), flip(y0, 1)],
                [x1, y0, z1, u(z1, 2), flip(y0, 1)],
                [x1, y0, z0, u(z0, 2), flip(y0, 1)],
                [x1, y1, z0, u(z0, 2), flip(y1, 1)],
                [x1, y1, z1, u(z1, 2), flip(y1, 1)],
            ],
        );
    }

    // RenderEntityBlock: a stretched box drawn one block cell at a time so its sprite tiles
    pub(crate) fn stretched(&mut self, origin: [f64; 3], size: [f64; 3], sprite: [f32; 4], light: f32)
    {
        let cells = |axis: usize| 0..=(size[axis].max(0.0).floor() as i32);

        for i in cells(0)
        {
            for j in cells(1)
            {
                for k in cells(2)
                {
                    let base: [i32; 3] = [i, j, k];
                    let extent: [f64; 3] = std::array::from_fn(|axis| (size[axis] - f64::from(base[axis])).min(1.0));
                    if extent.iter().any(|value| *value <= 0.0)
                    {
                        continue;
                    }

                    let min: [f32; 3] = std::array::from_fn(|axis| (origin[axis] + f64::from(base[axis])) as f32);
                    let max: [f32; 3] = std::array::from_fn(|axis| min[axis] + extent[axis] as f32);
                    self.cubeIn(min, max, sprite, light);
                }
            }
        }
    }

    // a cell of a stretched box keeps the sprite from its own lower corner, as each cell draws a whole block
    fn cubeIn(&mut self, min: [f32; 3], max: [f32; 3], sprite: [f32; 4], light: f32)
    {
        let ops: usize = self.m_ops.len();

        self.cube([0.0; 3], std::array::from_fn(|axis| max[axis] - min[axis]), sprite, light);

        for op in &mut self.m_ops[ops..]
        {
            if let ModelOp::Quads(quads) = op
            {
                for vertex in quads.iter_mut()
                {
                    for axis in 0..3
                    {
                        vertex[axis] += min[axis];
                    }
                }
            }
        }
    }

    // Utils.createLaser between two block positions, relative to the drawing tile
    pub(crate) fn laser(&mut self, from: [f64; 3], to: [f64; 3], kind: Laser, light: f32)
    {
        if from == to
        {
            return;
        }

        let mut origin: [f64; 3] = from;
        let mut size: [f64; 3] = std::array::from_fn(|axis| to[axis] - from[axis]);
        let along: usize = (0..3).find(|axis| size[*axis] != 0.0).unwrap_or(0);

        for axis in 0..3
        {
            if axis == along
            {
                origin[axis] += laser::HALF;
            }
            else
            {
                origin[axis] += laser::INSET;
                size[axis] = laser::THICKNESS;
            }
        }

        self.stretched(origin, size, uv(kind.sprite()), light);
    }

    // Utils.createLaserBox, the twelve edges of a box
    pub(crate) fn laserBox(&mut self, min: [f64; 3], max: [f64; 3], kind: Laser, light: f32)
    {
        let corner = |x: bool, y: bool, z: bool| -> [f64; 3]
        {
            [if x { max[0] } else { min[0] }, if y { max[1] } else { min[1] }, if z { max[2] } else { min[2] }]
        };
        let points: [[f64; 3]; 8] = std::array::from_fn(|index| corner(index & 1 != 0, index & 2 != 0, index & 4 != 0));
        const EDGES: [(usize, usize); 12] =
            [(0, 1), (0, 2), (2, 3), (1, 3), (4, 5), (4, 6), (5, 7), (6, 7), (0, 4), (1, 5), (2, 6), (3, 7)];

        for (start, end) in EDGES
        {
            self.laser(points[start], points[end], kind, light);
        }
    }

    pub(crate) fn item(&mut self, at: [f32; 3], stack: &ItemStack, light: f32)
    {
        let Ok(stack) = stack.toRaw()
        else
        {
            return;
        };

        self.m_ops.push(ModelOp::Push);
        self.m_ops.push(ModelOp::Translate(at));
        self.m_ops.push(ModelOp::Item { stack, brightness: light });
        self.m_ops.push(ModelOp::Pop);
        self.m_ops.push(ModelOp::Texture(TERRAIN.to_owned()));
    }
}

// a world position relative to the tile that draws it
pub(crate) fn relative(pos: BlockPos, world: [f64; 3]) -> [f64; 3]
{
    [world[0] - pos.x as f64, world[1] - pos.y as f64, world[2] - pos.z as f64]
}

pub(crate) fn corner(pos: BlockPos) -> [f64; 3] { [pos.x as f64, pos.y as f64, pos.z as f64] }

// mod_BuildCraftCore.RenderInvBlock draws a pipe model item as a quarter to three quarters wide column
pub(crate) fn column(texture: Option<&str>) -> ItemModelSpec
{
    const FROM: [f32; 3] = [4.0, 0.0, 4.0];
    const TO: [f32; 3] = [12.0, 16.0, 12.0];

    let cuboid: Cuboid =
        Cuboid { from: FROM, to: TO, texture: texture.map(str::to_owned), metas: Vec::new(), connect: None };
    ItemModelSpec { cuboids: vec![cuboid], ..ItemModelSpec::default() }
}

// a whole block as a model cuboid, for blocks drawn by their tile that still collide as a cube
pub(crate) fn fullCuboid() -> Cuboid
{
    const UNITS: f32 = 16.0;

    Cuboid { from: [0.0; 3], to: [UNITS; 3], texture: None, metas: Vec::new(), connect: None }
}
