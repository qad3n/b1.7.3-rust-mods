use super::generate::GenWorld;
use crate::ids::{self, block};
use b173_api::prelude::*;
use b173_api::terrain::Random;

// a block and its metadata as the generators place it
pub type Placed = (i32, i32);

#[derive(Clone, Copy, PartialEq)]
pub enum Tree
{
    SmallOak,
    Canopy,
    Mangrove,
    DarkCanopy,
    Hollow,
    Time,
    Transformation,
    Miners,
    Sorting,
    Rainbow,
}

// TF log metadata keeps the wood in the low bits and the orientation in bits 4 and 8, 12 is all bark
mod orient
{
    pub const UPRIGHT: i32 = 0;
    pub const X: i32 = 4;
    pub const Z: i32 = 8;
    pub const BARK: i32 = 12;
}

const PLAIN_WOODS: [&str; 4] = [block::OAK_LOG, block::CANOPY_LOG, block::MANGROVE_LOG, block::DARKWOOD_LOG];
const MAGIC_WOODS: [&str; 4] = [block::TIMEWOOD, block::TRANSWOOD, block::MINEWOOD, block::SORTINGWOOD];
const PLAIN_LEAVES: [&str; 4] =
    [block::OAK_LEAVES, block::CANOPY_LEAVES, block::MANGROVE_LEAVES, block::RAINBOW_LEAVES];
const MAGIC_LEAVES: [&str; 4] = [block::TIME_LEAVES, block::TRANS_LEAVES, block::MINE_LEAVES, block::SORT_LEAVES];
const CORES: [&str; 4] = [block::TIME_CORE, block::TRANS_CORE, block::MINE_CORE, block::SORT_CORE];

// TF log metadata to this port's block and axis state
fn logFrom(names: &[&str; 4], meta: i32) -> Placed
{
    let axis: i32 = match meta & orient::BARK
    {
        orient::X => 1,
        orient::Z => 2,
        orient::BARK => 3,
        _ => 0,
    };
    (ids::blockId(names[(meta & 3) as usize]), axis)
}

pub fn log(meta: i32) -> Placed { logFrom(&PLAIN_WOODS, meta) }

pub fn magicLog(meta: i32) -> Placed { logFrom(&MAGIC_WOODS, meta) }

pub fn leaves(meta: i32) -> Placed { (ids::blockId(PLAIN_LEAVES[(meta & 3) as usize]), 0) }

pub fn magicLeaves(meta: i32) -> Placed { (ids::blockId(MAGIC_LEAVES[(meta & 3) as usize]), 0) }

pub fn core(kind: i32) -> Placed { (ids::blockId(CORES[(kind & 3) as usize]), 0) }

pub fn root() -> Placed { (ids::blockId(block::ROOT), 0) }

// the woods a TF tree generator draws with, TFTreeGenerator treeBlock branchBlock leafBlock rootBlock
#[derive(Clone, Copy)]
pub struct Woods
{
    pub m_trunk: Placed,
    pub m_branch: Placed,
    pub m_leaf: Placed,
    pub m_root: Placed,
}

impl Woods
{
    pub fn plain(wood: i32, leaf: i32) -> Woods
    {
        Woods { m_trunk: log(wood), m_branch: log(wood | orient::BARK), m_leaf: leaves(leaf), m_root: root() }
    }

    pub fn magic(wood: i32) -> Woods
    {
        Woods
        {
            m_trunk: magicLog(wood),
            m_branch: magicLog(wood | orient::BARK),
            m_leaf: magicLeaves(wood),
            m_root: root(),
        }
    }
}

// TFGenerator.translate, a point some distance away at an angle around and a tilt down from up
pub fn translate(from: [i32; 3], distance: f64, angle: f64, tilt: f64) -> [i32; 3]
{
    let around: f64 = angle * 2.0 * std::f64::consts::PI;
    let down: f64 = tilt * std::f64::consts::PI;
    let round = |value: f64| (value + 0.5).floor() as i32;
    [
        from[0] + round(around.sin() * down.sin() * distance),
        from[1] + round(down.cos() * distance),
        from[2] + round(around.cos() * down.sin() * distance),
    ]
}

// TFGenerator.getBresehnamArrayCoords, every block on a 3D line including both ends
pub fn line(from: [i32; 3], to: [i32; 3]) -> Vec<[i32; 3]>
{
    let delta: [i32; 3] = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let step: [i32; 3] = delta.map(|value| if value < 0 { -1 } else { 1 });
    let length: [i32; 3] = delta.map(i32::abs);
    // the axis that changes most leads, the other two follow on error terms
    let lead: usize = if length[0] >= length[1] && length[0] >= length[2]
    {
        0
    }
    else if length[1] >= length[0] && length[1] >= length[2]
    {
        1
    }
    else
    {
        2
    };
    let (first, second): (usize, usize) = match lead
    {
        0 => (1, 2),
        1 => (0, 2),
        _ => (1, 0),
    };
    let mut pixel: [i32; 3] = from;
    let mut points: Vec<[i32; 3]> = Vec::with_capacity(length[lead] as usize + 1);
    let doubled: [i32; 3] = length.map(|value| value << 1);
    let mut errorFirst: i32 = doubled[first] - length[lead];
    let mut errorSecond: i32 = doubled[second] - length[lead];
    for _ in 0..length[lead]
    {
        points.push(pixel);
        if errorFirst > 0
        {
            pixel[first] += step[first];
            errorFirst -= doubled[lead];
        }
        if errorSecond > 0
        {
            pixel[second] += step[second];
            errorSecond -= doubled[lead];
        }
        errorFirst += doubled[first];
        errorSecond += doubled[second];
        pixel[lead] += step[lead];
    }
    points.push(pixel);
    points
}

// TFGenerator.drawBlob distance, longest axis plus a half and a quarter of the others
fn blobDistance(dx: i32, dy: i32, dz: i32) -> i32
{
    let mut axes: [i32; 3] = [dx, dy, dz];
    axes.sort_unstable();
    axes[2] + (axes[1] >> 1) + (axes[0] >> 2)
}

// the world a tree is drawn into, quiet during generation and notifying neighbours when a sapling grows
pub struct Builder<'a>
{
    pub m_world: &'a GenWorld,
    pub m_notify: bool,
}

impl Builder<'_>
{
    pub fn set(&self, x: i32, y: i32, z: i32, placed: Placed)
    {
        if self.m_notify
        {
            self.m_world.setNotify(x, y, z, placed.0, placed.1);
        }
        else
        {
            self.m_world.setMeta(x, y, z, placed.0, placed.1);
        }
    }

    pub fn block(&self, x: i32, y: i32, z: i32) -> i32 { self.m_world.block(x, y, z) }

    pub fn isAir(&self, x: i32, y: i32, z: i32) -> bool { self.m_world.isAir(x, y, z) }

    // TFGenerator.putLeafBlock, leaves go only where Block.canBeReplacedByLeaves says
    pub fn putLeaf(&self, x: i32, y: i32, z: i32, placed: Placed)
    {
        let there: i32 = self.block(x, y, z);
        if there == 0 || !self.m_world.info(there).opaque || ids::leaves(there)
        {
            self.set(x, y, z, placed);
        }
    }

    pub fn drawLine(&self, from: [i32; 3], to: [i32; 3], placed: Placed)
    {
        for [x, y, z] in line(from, to)
        {
            self.set(x, y, z, placed);
        }
    }

    // TFGenerator.makeLeafCircle, the hack trims the 3,3 corner off the round
    pub fn leafCircle(&self, x: i32, y: i32, z: i32, radius: i32, placed: Placed, hack: bool)
    {
        for dx in 0..=radius
        {
            for dz in 0..=radius
            {
                let mut distance: i32 = dx.max(dz) + (dx.min(dz) >> 1);
                if hack && dx == 3 && dz == 3
                {
                    distance = 6;
                }
                if distance > radius
                {
                    continue;
                }
                for [sx, sz] in [[1, 1], [1, -1], [-1, 1], [-1, -1]]
                {
                    self.putLeaf(x + dx * sx, y, z + dz * sz, placed);
                }
            }
        }
    }

    // TFGenerator.makeLeafCircle2, a circle around a 2x2 trunk
    pub fn leafCircle2(&self, x: i32, y: i32, z: i32, radius: i32, placed: Placed)
    {
        for dx in 0..=radius
        {
            for dz in 0..=radius
            {
                if dx * dx + dz * dz > radius * radius
                {
                    continue;
                }
                self.putLeaf(x + 1 + dx, y, z + 1 + dz, placed);
                self.putLeaf(x + 1 + dx, y, z - dz, placed);
                self.putLeaf(x - dx, y, z + 1 + dz, placed);
                self.putLeaf(x - dx, y, z - dz, placed);
            }
        }
    }

    fn eachBlob(&self, x: i32, y: i32, z: i32, radius: i32, mut visit: impl FnMut(i32, i32, i32))
    {
        for dx in 0..=radius
        {
            for dy in 0..=radius
            {
                for dz in 0..=radius
                {
                    if blobDistance(dx, dy, dz) > radius
                    {
                        continue;
                    }
                    for [sx, sy, sz] in [
                        [1, 1, 1],
                        [1, 1, -1],
                        [-1, 1, 1],
                        [-1, 1, -1],
                        [1, -1, 1],
                        [1, -1, -1],
                        [-1, -1, 1],
                        [-1, -1, -1],
                    ]
                    {
                        visit(x + dx * sx, y + dy * sy, z + dz * sz);
                    }
                }
            }
        }
    }

    pub fn leafBlob(&self, x: i32, y: i32, z: i32, radius: i32, placed: Placed)
    {
        self.eachBlob(x, y, z, radius, |bx, by, bz| self.putLeaf(bx, by, bz, placed));
    }

    pub fn blob(&self, x: i32, y: i32, z: i32, radius: i32, placed: Placed)
    {
        self.eachBlob(x, y, z, radius, |bx, by, bz| self.set(bx, by, bz, placed));
    }

    // TFGenerator.hasAirAround, any of the four sides or the top open
    pub fn airAround(&self, x: i32, y: i32, z: i32) -> bool
    {
        [[1, 0, 0], [-1, 0, 0], [0, 0, 1], [0, 0, -1], [0, 1, 0]]
            .iter()
            .any(|[dx, dy, dz]| self.isAir(x + dx, y + dy, z + dz))
    }

    // TFGenerator.isNearSolid
    fn nearSolid(&self, x: i32, y: i32, z: i32) -> bool
    {
        [[1, 0], [-1, 0], [0, 1], [0, -1]].iter().any(|[dx, dz]| self.m_world.isSolid(x + dx, y, z + dz))
    }

    // TFTreeGenerator.canRootGrowIn, roots fill air only beside something solid and never break bedrock
    pub fn rootFits(&self, x: i32, y: i32, z: i32) -> bool
    {
        let there: i32 = self.block(x, y, z);
        if there == 0
        {
            return self.nearSolid(x, y, z);
        }
        there != self.m_world.id("minecraft:bedrock")
            && there != self.m_world.id("minecraft:obsidian")
            && there != self.m_world.id(block::SHIELD)
    }

    pub fn placeRoot(&self, x: i32, y: i32, z: i32, placed: Placed)
    {
        if self.rootFits(x, y, z)
        {
            self.set(x, y, z, placed);
        }
    }

    // TFTreeGenerator.buildRoot, a root reaching down and out from below the trunk
    pub fn buildRoot(&self, x: i32, y: i32, z: i32, offset: f64, index: i32, woods: &Woods)
    {
        let start: [i32; 3] = [x, y - index - 2, z];
        let reach: f64 = 5.0;
        let tilt: f64 = 0.8;
        let end: [i32; 3] = translate(start, reach, 0.3 * f64::from(index) + offset, tilt);
        for [rx, ry, rz] in line(start, end)
        {
            self.placeRoot(rx, ry, rz, woods.m_root);
        }
    }

    // a firefly on one of four sides of the trunk, TFGenCanopyTree.addFirefly
    pub fn firefly(&self, x: i32, y: i32, z: i32, height: i32, angle: f64)
    {
        let firefly: Placed = (self.m_world.id(block::FIREFLY), 0);
        match (angle * 4.0) as i32
        {
            0 => self.set(x + 1, y + height, z, firefly),
            1 => self.set(x - 1, y + height, z, firefly),
            2 => self.set(x, y + height, z + 1, firefly),
            3 => self.set(x, y + height, z - 1, firefly),
            _ =>
            {}
        }
    }

    // a trunk foot, log where air can reach it and root where buried
    pub fn foot(&self, x: i32, y: i32, z: i32, woods: &Woods)
    {
        let placed: Placed = if self.airAround(x, y, z)
        {
            woods.m_trunk
        }
        else
        {
            woods.m_root
        };
        self.set(x, y, z, placed);
    }

    pub fn soilBelow(&self, x: i32, y: i32, z: i32) -> bool
    {
        let below: i32 = self.block(x, y - 1, z);
        below == self.m_world.id("minecraft:grass") || below == self.m_world.id("minecraft:dirt")
    }
}

const TOP: i32 = 128;

mod small
{
    use super::*;

    // TFGenSmallTwilightOak, a vanilla style little oak with twilight wood
    pub fn oak(builder: &Builder, random: &Random, x: i32, y: i32, z: i32, woods: &Woods, minimum: i32) -> bool
    {
        let height: i32 = random.nextIntBounded(3) + minimum;
        if y < 1 || y + height + 1 > TOP
        {
            return false;
        }
        for cy in y..=y + 1 + height
        {
            let width: i32 = if cy == y
            {
                0
            }
            else if cy >= y + 1 + height - 2
            {
                2
            }
            else
            {
                1
            };
            for cx in x - width..=x + width
            {
                for cz in z - width..=z + width
                {
                    let there: i32 = builder.block(cx, cy, cz);
                    let clear: bool = there == 0 || ids::leaves(there) || ids::soil(there) || ids::holdsLeaves(there);
                    if !clear
                    {
                        return false;
                    }
                }
            }
        }
        if !builder.soilBelow(x, y, z) || y >= TOP - height - 1
        {
            return false;
        }
        builder.set(x, y - 1, z, (builder.m_world.id("minecraft:dirt"), 0));
        let crown: i32 = 3;
        for cy in y - crown + height..=y + height
        {
            let layer: i32 = cy - (y + height);
            let width: i32 = 1 - layer / 2;
            for tx in x - width..=x + width
            {
                for tz in z - width..=z + width
                {
                    let corner: bool = (tx - x).abs() == width && (tz - z).abs() == width;
                    if corner && (random.nextIntBounded(2) == 0 || layer == 0)
                    {
                        continue;
                    }
                    builder.putLeaf(tx, cy, tz, woods.m_leaf);
                }
            }
        }
        for dy in 0..height
        {
            let there: i32 = builder.block(x, y + dy, z);
            if there == 0 || ids::leaves(there)
            {
                builder.set(x, y + dy, z, woods.m_trunk);
            }
        }
        true
    }

    // TFGenSortingTree, a stubby tree around its engine
    pub fn sorting(builder: &Builder, x: i32, y: i32, z: i32) -> bool
    {
        let woods: Woods = Woods::magic(3);
        if !builder.soilBelow(x, y, z) || y >= TOP - 12
        {
            return false;
        }
        for dy in 0..4
        {
            builder.set(x, y + dy, z, woods.m_trunk);
        }
        for level in [2, 3]
        {
            for lx in -1..=1_i32
            {
                for ly in -1..=1_i32
                {
                    for lz in -1..=1_i32
                    {
                        if ly == 0 || lx.abs() + lz.abs() <= 1
                        {
                            builder.putLeaf(x + lx, y + level + ly, z + lz, woods.m_leaf);
                        }
                    }
                }
            }
        }
        builder.set(x, y + 1, z, core(3));
        true
    }

    // TFGenMinersTree, a pole with two drooping arms and the minewood core near the foot
    pub fn miners(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let woods: Woods = Woods::magic(2);
        if !builder.soilBelow(x, y, z) || y >= TOP - 12
        {
            return false;
        }
        for dy in 0..10
        {
            builder.set(x, y + dy, z, woods.m_branch);
        }
        let arm: [(i32, i32, bool); 5] = [(9, 1, true), (9, 2, false), (8, 3, false), (7, 4, false), (6, 5, false)];
        for side in [1, -1]
        {
            for (dy, dz, bushy) in arm
            {
                let (bx, by, bz): (i32, i32, i32) = (x, y + dy, z + dz * side);
                builder.set(bx, by, bz, woods.m_branch);
                for lx in -1..=1_i32
                {
                    for ly in -1..=1_i32
                    {
                        for lz in -1..=1_i32
                        {
                            if bushy || ly == 0 || lx == 0
                            {
                                builder.putLeaf(bx + lx, by + ly, bz + lz, woods.m_leaf);
                            }
                        }
                    }
                }
            }
        }
        builder.set(x, y + 1, z, core(2));
        let _ = builder.m_world.m_world.scheduleTick(BlockPos::new(x, y + 1, z), core(2).0, 20);
        builder.foot(x, y - 1, z, &woods);
        let roots: i32 = 3 + random.nextIntBounded(2);
        let offset: f64 = random.nextDouble();
        for index in 0..roots
        {
            builder.buildRoot(x, y, z, offset, index, &woods);
        }
        true
    }
}

mod canopy
{
    use super::*;

    pub struct Shape
    {
        pub m_minimum: i32,
        pub m_firstFive: i32,
        pub m_secondFive: i32,
    }

    pub const TALL: Shape = Shape { m_minimum: 20, m_firstFive: 3, m_secondFive: 8 };

    fn height(random: &Random, shape: &Shape) -> i32
    {
        let mut height: i32 = shape.m_minimum;
        if shape.m_firstFive != i32::MAX && random.nextIntBounded(shape.m_firstFive) == 0
        {
            height += random.nextIntBounded(5);
            if random.nextIntBounded(shape.m_secondFive) == 0
            {
                height += random.nextIntBounded(5);
            }
        }
        height
    }

    fn clampEnd(end: &mut [i32; 3], x: i32, z: i32, limit: i32)
    {
        end[0] = end[0].clamp(x - limit, x + limit);
        end[2] = end[2].clamp(z - limit, z + limit);
    }

    fn bark4(builder: &Builder, end: [i32; 3], placed: Placed)
    {
        for [dx, dz] in [[1, 0], [-1, 0], [0, 1], [0, -1]]
        {
            builder.set(end[0] + dx, end[1], end[2] + dz, placed);
        }
    }

    // TFGenCanopyTree.buildBranch
    fn branch(
        builder: &Builder,
        random: &Random,
        origin: [i32; 3],
        height: i32,
        length: f64,
        angle: f64,
        tilt: f64,
        trunk: bool,
        woods: &Woods,
    )
    {
        let [x, y, z] = origin;
        let start: [i32; 3] = [x, y + height, z];
        let mut end: [i32; 3] = translate(start, length, angle, tilt);
        clampEnd(&mut end, x, z, 4);
        builder.drawLine(start, end, if trunk { woods.m_trunk } else { woods.m_branch });
        if trunk
        {
            builder.firefly(x, y, z, 3 + random.nextIntBounded(7), random.nextDouble());
        }
        builder.leafCircle(end[0], end[1] - 1, end[2], 3, woods.m_leaf, true);
        builder.leafCircle(end[0], end[1], end[2], 4, woods.m_leaf, true);
        builder.leafCircle(end[0], end[1] + 1, end[2], 2, woods.m_leaf, true);
        bark4(builder, end, woods.m_branch);
    }

    // TFGenCanopyTree.generate, a tall trunk with a few long boughs ending in flat leaf pads
    pub fn tree(builder: &Builder, random: &Random, x: i32, y: i32, z: i32, woods: &Woods, shape: &Shape) -> bool
    {
        if !builder.soilBelow(x, y, z) || y >= TOP - 12
        {
            return false;
        }
        let height: i32 = height(random, shape);
        branch(builder, random, [x, y, z], 0, f64::from(height), 0.0, 0.0, true, woods);
        let boughs: i32 = 3 + random.nextIntBounded(2);
        let offset: f32 = random.nextFloat();
        let reach: f64 = 9.0;
        for index in 0..boughs
        {
            branch(
                builder,
                random,
                [x, y, z],
                height - 10 + index,
                reach,
                0.3 * f64::from(index) + f64::from(offset),
                0.2,
                false,
                woods,
            );
        }
        builder.foot(x, y - 1, z, woods);
        let roots: i32 = 3 + random.nextIntBounded(2);
        let offset: f32 = random.nextFloat();
        for index in 0..roots
        {
            builder.buildRoot(x, y, z, f64::from(offset), index, woods);
        }
        true
    }

    // TFGenCanopyOak, a 2x2 trunk feathered with many short branches
    pub fn oak(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let woods: Woods = Woods::plain(0, 0);
        if !builder.soilBelow(x, y, z) || y >= TOP - 12
        {
            return false;
        }
        let height: i32 = height(random, &TALL);
        for dy in 0..height
        {
            for [dx, dz] in [[0, 0], [1, 0], [0, 1], [1, 1]]
            {
                builder.set(x + dx, y + dy, z + dz, woods.m_trunk);
            }
        }
        builder.leafBlob(x, y + height, z, 3, woods.m_leaf);
        let branches: i32 = 12 + random.nextIntBounded(9);
        let mut angle: f32 = random.nextFloat();
        for index in 0..branches
        {
            let tilt: f32 = 0.15 + random.nextFloat() * 0.35;
            let start: [i32; 3] = [x, y + height - 10 + index / 2, z];
            let mut end: [i32; 3] = translate(start, 5.0, f64::from(angle), f64::from(tilt));
            clampEnd(&mut end, x, z, 5);
            builder.drawLine(start, end, woods.m_branch);
            builder.leafBlob(end[0], end[1], end[2], 2, woods.m_leaf);
            bark4(builder, end, woods.m_branch);
            angle += random.nextFloat() * 0.4;
            if angle > 1.0
            {
                angle -= 1.0;
            }
        }
        for [dx, dz] in [[0, 0], [1, 0], [0, 1], [1, 1]]
        {
            builder.foot(x + dx, y - 1, z + dz, &woods);
            let roots: i32 = 1 + random.nextIntBounded(2);
            let offset: f32 = random.nextFloat();
            for index in 0..roots
            {
                builder.buildRoot(x + dx, y, z + dz, f64::from(offset), index, &woods);
            }
        }
        true
    }

    // TFGenCanopyMushroom, a giant mushroom with boughs, drawn with mushroom blocks beta lacks so huge mushgloom stands in
    pub fn mushroom(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
    {
        let mut height: i32 = 12;
        if random.nextIntBounded(3) == 0
        {
            height += random.nextIntBounded(5);
            if random.nextIntBounded(8) == 0
            {
                height += random.nextIntBounded(5);
            }
        }
        if !builder.soilBelow(x, y, z) || y >= TOP - height - 1
        {
            return false;
        }
        let red: bool = random.nextIntBounded(3) == 0;
        let cap: Placed = (
            builder.m_world.id(
                if red
                {
                    block::HUGE_RED_MUSHROOM
                }
                else
                {
                    block::HUGE_BROWN_MUSHROOM
                },
            ),
            0,
        );
        let stem: Placed = (cap.0, 1);
        let stalk = |height: i32, length: f64, angle: f64, tilt: f64, trunk: bool|
        {
            let start: [i32; 3] = [x, y + height, z];
            let mut end: [i32; 3] = translate(start, length, angle, tilt);
            clampEnd(&mut end, x, z, 4);
            if start[0] == end[0] && start[2] == end[2]
            {
                builder.drawLine(start, [end[0], end[1] - 1, end[2]], stem);
            }
            else
            {
                builder.drawLine(start, [end[0], start[1], end[2]], stem);
                builder.drawLine([end[0], start[1] + 1, end[2]], [end[0], end[1] - 1, end[2]], stem);
            }
            if trunk
            {
                builder.firefly(x, y, z, 3 + random.nextIntBounded(7), random.nextDouble());
            }
            builder.leafCircle(end[0], end[1], end[2], 4, cap, true);
        };
        stalk(0, f64::from(height), 0.0, 0.0, true);
        let boughs: i32 = 3 + random.nextIntBounded(2);
        let offset: f64 = random.nextDouble();
        for index in 0..boughs
        {
            stalk(height - 5 + index, 9.0, 0.3 * f64::from(index) + offset, 0.2, false);
        }
        true
    }
}

mod swamp
{
    use super::*;

    fn clampEnd(end: &mut [i32; 3], x: i32, z: i32)
    {
        end[0] = end[0].clamp(x - 4, x + 4);
        end[2] = end[2].clamp(z - 4, z + 4);
    }

    // TFGenMangroveTree.buildBranch
    fn branch(
        builder: &Builder,
        random: &Random,
        origin: [i32; 3],
        height: i32,
        length: f64,
        angle: f64,
        tilt: f64,
        trunk: bool,
        woods: &Woods,
    )
    {
        let [x, y, z] = origin;
        let start: [i32; 3] = [x, y + height, z];
        let mut end: [i32; 3] = translate(start, length, angle, tilt);
        clampEnd(&mut end, x, z);
        builder.drawLine(start, end, if trunk { woods.m_trunk } else { woods.m_branch });
        let size: i32 = 2 + random.nextIntBounded(3);
        if size > 2
        {
            for [dx, dz] in [[1, 0], [-1, 0], [0, 1], [0, -1]]
            {
                builder.set(end[0] + dx, end[1], end[2] + dz, woods.m_branch);
            }
        }
        builder.leafCircle(end[0], end[1] - 1, end[2], size - 1, woods.m_leaf, false);
        builder.leafCircle(end[0], end[1], end[2], size, woods.m_leaf, false);
        builder.leafCircle(end[0], end[1] + 1, end[2], size - 2, woods.m_leaf, false);
    }

    // TFGenMangroveTree.buildRoot, stilt roots that stay wood above ground
    fn stilt(builder: &Builder, origin: [i32; 3], angle: f64, tilt: f64, woods: &Woods)
    {
        let start: [i32; 3] = [origin[0], origin[1] + 5, origin[2]];
        let length: f64 = 12.0;
        let end: [i32; 3] = translate(start, length, angle, tilt);
        let mut above: bool = true;
        for [rx, ry, rz] in line(start, end)
        {
            if above && builder.airAround(rx, ry, rz)
            {
                builder.set(rx, ry, rz, woods.m_branch);
                builder.set(rx, ry - 1, rz, woods.m_branch);
            }
            else
            {
                builder.placeRoot(rx, ry, rz, woods.m_root);
                builder.placeRoot(rx, ry - 1, rz, woods.m_root);
                above = false;
            }
        }
    }

    // TFGenMangroveTree, a tree on stilts in the water
    pub fn mangrove(builder: &Builder, random: &Random, x: i32, y: i32, z: i32, onWater: bool) -> bool
    {
        let woods: Woods = Woods::plain(2, 2);
        let wet: bool = builder.block(x, y - 1, z) == builder.m_world.id("minecraft:water");
        if (onWater && !wet) || y >= 109
        {
            return false;
        }
        branch(builder, random, [x, y, z], 5, f64::from(6 + random.nextIntBounded(3)), 0.0, 0.0, true, &woods);
        let boughs: i32 = random.nextIntBounded(3);
        let offset: f64 = random.nextDouble();
        for index in 0..boughs
        {
            branch(
                builder,
                random,
                [x, y, z],
                7 + index,
                f64::from(6 + random.nextIntBounded(2)),
                0.3 * f64::from(index) + offset,
                0.25,
                false,
                &woods,
            );
        }
        let roots: i32 = 3 + random.nextIntBounded(2);
        let offset: f64 = random.nextDouble();
        for index in 0..roots
        {
            let tilt: f64 = 0.75 + random.nextDouble() * 0.1;
            stilt(builder, [x, y, z], 0.4 * f64::from(index) + offset, tilt, &woods);
        }
        builder.firefly(x, y, z, 5 + random.nextIntBounded(5), random.nextDouble());
        true
    }
}

mod dark
{
    use super::*;

    fn leafAround(builder: &Builder, x: i32, y: i32, z: i32, leaf: Placed)
    {
        let size: i32 = 4;
        if !builder.airAround(x, y, z)
        {
            return;
        }
        builder.leafCircle(x, y - 1, z, size, leaf, false);
        builder.leafCircle(x, y, z, size + 1, leaf, false);
        builder.leafCircle(x, y + 1, z, size, leaf, false);
        builder.leafCircle(x, y + 2, z, size - 2, leaf, false);
    }

    // TFGenDarkCanopyTree, a short trunk carrying four boughs of dark leaves
    pub fn canopy(builder: &Builder, random: &Random, x: i32, mut y: i32, z: i32) -> bool
    {
        let woods: Woods = Woods
        {
            m_trunk: log(3),
            m_branch: log(15),
            m_leaf: (builder.m_world.id(block::DARK_LEAVES), 0),
            m_root: root(),
        };
        let mut found: bool = false;
        let mut dy: i32 = y;
        while dy >= super::super::terrain::SEA_LEVEL
        {
            let under: i32 = builder.block(x, dy - 1, z);
            if ids::soil(under)
            {
                found = true;
                y = dy;
                break;
            }
            if builder.m_world.isOpaque(x, dy - 1, z) && !ids::leaves(under)
            {
                break;
            }
            dy -= 1;
        }
        if !found
        {
            return false;
        }
        let woody = |bx: i32, bz: i32| ids::holdsLeaves(builder.block(bx, y, bz));
        if woody(x + 1, z) || woody(x - 1, z) || woody(x, z + 1) || woody(x, z - 1)
        {
            return false;
        }
        let height: i32 = 6 + random.nextIntBounded(5);
        builder.drawLine([x, y, z], [x, y + height, z], woods.m_trunk);
        leafAround(builder, x, y + height, z, woods.m_leaf);
        let boughs: i32 = 4;
        let offset: f64 = f64::from(random.nextFloat());
        for index in 0..boughs
        {
            let start: [i32; 3] = [x, y + height - 3 - boughs + index / 2, z];
            let length: f64 = f64::from(10 + random.nextIntBounded(4));
            let mut end: [i32; 3] = translate(start, length, 0.23 * f64::from(index) + offset, 0.23);
            end[0] = end[0].clamp(x - 4, x + 4);
            end[2] = end[2].clamp(z - 4, z + 4);
            builder.drawLine(start, end, woods.m_branch);
            leafAround(builder, end[0], end[1], end[2], woods.m_leaf);
        }
        builder.foot(x, y - 1, z, &woods);
        let roots: i32 = 3 + random.nextIntBounded(2);
        let offset: f64 = random.nextDouble();
        for index in 0..roots
        {
            builder.buildRoot(x, y, z, offset, index, &woods);
        }
        true
    }
}

pub fn soil(block: i32) -> bool { ids::soil(block) }

// BlockTFSapling.growTree, the sapling steps aside for its tree and comes back when the tree does not fit
pub fn growSapling(world: World, pos: BlockPos, sapling: i32, tree: Tree)
{
    let generator: GenWorld = GenWorld::new(world);
    let builder: Builder = Builder { m_world: &generator, m_notify: true };
    let random: Random = Random::withSeed(world.time() ^ i64::from(pos.x) << 20 ^ i64::from(pos.z));
    let meta: i32 = world.meta(pos);
    let _ = world.setBlockQuiet(pos, 0, 0);
    let grown: bool = grow(&builder, &random, tree, pos.x, pos.y, pos.z);
    if !grown
    {
        let _ = world.setBlockQuiet(pos, sapling, meta);
    }
}

pub fn grow(builder: &Builder, random: &Random, tree: Tree, x: i32, y: i32, z: i32) -> bool
{
    let rare: i32 = 7;
    match tree
    {
        Tree::SmallOak => small::oak(builder, random, x, y, z, &Woods::plain(0, 0), 4),
        Tree::Rainbow if random.nextIntBounded(rare) == 0 => super::rainbow::large(builder, random, x, y, z, false),
        Tree::Rainbow => small::oak(builder, random, x, y, z, &Woods::plain(0, 3), 4),
        Tree::Canopy => canopy::tree(builder, random, x, y, z, &Woods::plain(1, 1), &canopy::TALL),
        Tree::Mangrove => swamp::mangrove(builder, random, x, y, z, false),
        Tree::DarkCanopy => dark::canopy(builder, random, x, y, z),
        Tree::Hollow => super::hollow::tree(builder, random, x, y, z),
        Tree::Time => super::hollow::time(builder, random, x, y, z),
        Tree::Transformation => transformation(builder, random, x, y, z),
        Tree::Miners => small::miners(builder, random, x, y, z),
        Tree::Sorting => small::sorting(builder, x, y, z),
    }
}

// TFGenTreeOfTransformation, a short canopy tree of transwood with its heart three up
fn transformation(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let shape: canopy::Shape = canopy::Shape { m_minimum: 11, m_firstFive: i32::MAX, m_secondFive: i32::MAX };
    if !canopy::tree(builder, random, x, y, z, &Woods::magic(1), &shape)
    {
        return false;
    }
    builder.set(x, y + 3, z, core(1));
    true
}

// the decorator's own trees, which grow without notifying neighbours
pub fn twilightOak(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    small::oak(builder, random, x, y, z, &Woods::plain(0, 0), 4)
}

pub fn smallRainbow(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    small::oak(builder, random, x, y, z, &Woods::plain(0, 3), 4)
}

pub fn canopyTree(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    canopy::tree(builder, random, x, y, z, &Woods::plain(1, 1), &canopy::TALL)
}

pub fn canopyOak(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    canopy::oak(builder, random, x, y, z)
}

pub fn canopyMushroom(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    canopy::mushroom(builder, random, x, y, z)
}

pub fn mangrove(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    swamp::mangrove(builder, random, x, y, z, true)
}

pub fn darkCanopy(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    dark::canopy(builder, random, x, y, z)
}
