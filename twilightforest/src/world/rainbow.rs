use super::trees::{Builder, Placed, leaves, log};
use b173_api::terrain::{Random, math};

// WorldGenBigTree, drawn in twilight oak with rainbow leaves for TFGenLargeRainboak or in plain beta oak
const OTHER_AXES: [usize; 6] = [2, 0, 0, 1, 2, 1];

struct BigTree<'a>
{
    m_builder: &'a Builder<'a>,
    m_random: Random,
    m_base: [i32; 3],
    m_limit: i32,
    m_height: i32,
    m_leafReach: i32,
    m_nodes: Vec<[i32; 4]>,
    m_vanilla: bool,
}

mod shape
{
    pub const ATTENUATION: f64 = 0.618;
    pub const SLOPE: f64 = 0.381;
    pub const WIDTH: f64 = 1.0;
    pub const DENSITY: f64 = 1.0;
    pub const LIMIT: i32 = 12;
    pub const LEAF_REACH: i32 = 4;
}

impl BigTree<'_>
{
    // beta logs have no orientation so a vanilla tree keeps every log upright
    fn wood(&self, bits: i32) -> Placed
    {
        if self.m_vanilla
        {
            (self.m_builder.m_world.id("minecraft:log"), 0)
        }
        else
        {
            log(bits)
        }
    }

    fn leaf(&self) -> Placed
    {
        if self.m_vanilla
        {
            (self.m_builder.m_world.id("minecraft:leaves"), 0)
        }
        else
        {
            leaves(3)
        }
    }

    // WorldGenBigTree.checkBlockLine, how far a line runs before something other than air or leaves blocks it
    fn clearRun(&self, from: [i32; 3], to: [i32; 3]) -> i32
    {
        let delta: [i32; 3] = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let lead: usize = (0..3).fold(0, |best, axis|
        {
            if delta[axis].abs() > delta[best].abs()
            {
                axis
            }
            else
            {
                best
            }
        });
        if delta[lead] == 0
        {
            return -1;
        }
        let (first, second): (usize, usize) = (OTHER_AXES[lead], OTHER_AXES[lead + 3]);
        let step: i32 = if delta[lead] > 0 { 1 } else { -1 };
        let slopeFirst: f64 = f64::from(delta[first]) / f64::from(delta[lead]);
        let slopeSecond: f64 = f64::from(delta[second]) / f64::from(delta[lead]);
        let end: i32 = delta[lead] + step;
        let mut offset: i32 = 0;
        let leaves: i32 = self.m_builder.m_world.id("minecraft:leaves");
        while offset != end
        {
            let mut at: [i32; 3] = [0; 3];
            at[lead] = from[lead] + offset;
            at[first] = math::floor(f64::from(from[first]) + f64::from(offset) * slopeFirst);
            at[second] = math::floor(f64::from(from[second]) + f64::from(offset) * slopeSecond);
            let there: i32 = self.m_builder.block(at[0], at[1], at[2]);
            if there != 0 && there != leaves
            {
                break;
            }
            offset += step;
        }
        if offset == end { -1 } else { offset.abs() }
    }

    fn layerSize(&self, layer: i32) -> f32
    {
        if f64::from(layer) < f64::from(self.m_limit) * 0.3
        {
            return -1.618;
        }
        let half: f32 = self.m_limit as f32 / 2.0;
        let offset: f32 = half - layer as f32;
        let size: f32 = if offset == 0.0
        {
            half
        }
        else if offset.abs() >= half
        {
            0.0
        }
        else
        {
            (half * half - offset * offset).sqrt()
        };
        size * 0.5
    }

    // WorldGenBigTree.generateLeafNodeList
    fn nodes(&mut self)
    {
        self.m_height = ((f64::from(self.m_limit) * shape::ATTENUATION) as i32).min(self.m_limit - 1);
        let perLayer: i32 = ((1.382 + (shape::DENSITY * f64::from(self.m_limit) / 13.0).powi(2)) as i32).max(1);
        let mut y: i32 = self.m_base[1] + self.m_limit - self.m_leafReach;
        let top: i32 = self.m_base[1] + self.m_height;
        let mut layer: i32 = y - self.m_base[1];
        self.m_nodes = vec![[self.m_base[0], y, self.m_base[2], top]];
        y -= 1;
        while layer >= 0
        {
            let size: f32 = self.layerSize(layer);
            if size >= 0.0
            {
                for _ in 0..perLayer
                {
                    let reach: f64 = shape::WIDTH * f64::from(size) * (f64::from(self.m_random.nextFloat()) + 0.328);
                    let turn: f64 = f64::from(self.m_random.nextFloat()) * 2.0 * std::f64::consts::PI;
                    let nx: i32 = math::floor(reach * turn.sin() + f64::from(self.m_base[0]) + 0.5);
                    let nz: i32 = math::floor(reach * turn.cos() + f64::from(self.m_base[2]) + 0.5);
                    let node: [i32; 3] = [nx, y, nz];
                    if self.clearRun(node, [nx, y + self.m_leafReach, nz]) != -1
                    {
                        continue;
                    }
                    let distance: f64 = f64::from((self.m_base[0] - nx).pow(2) + (self.m_base[2] - nz).pow(2)).sqrt();
                    let drop: f64 = distance * shape::SLOPE;
                    let baseY: i32 = if f64::from(y) - drop > f64::from(top)
                    {
                        top
                    }
                    else
                    {
                        (f64::from(y) - drop) as i32
                    };
                    if self.clearRun([self.m_base[0], baseY, self.m_base[2]], node) == -1
                    {
                        self.m_nodes.push([nx, y, nz, baseY]);
                    }
                }
            }
            y -= 1;
            layer -= 1;
        }
    }

    // WorldGenBigTree.genTreeLayer, a disc of leaves at one height
    fn leafLayer(&self, center: [i32; 3], radius: f32, leaf: Placed)
    {
        let reach: i32 = (f64::from(radius) + 0.618) as i32;
        for a in -reach..=reach
        {
            for b in -reach..=reach
            {
                let distance: f64 = (f64::from(a.abs()) + 0.5).powi(2) + (f64::from(b.abs()) + 0.5).powi(2);
                if distance > f64::from(radius * radius)
                {
                    continue;
                }
                let (x, z): (i32, i32) = (center[0] + a, center[2] + b);
                let there: i32 = self.m_builder.block(x, center[1], z);
                if there == 0 || crate::ids::leaves(there) || !self.m_builder.m_world.info(there).opaque
                {
                    self.m_builder.set(x, center[1], z, leaf);
                }
            }
        }
    }

    fn leaves(&self)
    {
        let leaf: Placed = self.leaf();
        for node in &self.m_nodes
        {
            for dy in 0..self.m_leafReach
            {
                let size: f32 = if dy == 0 || dy == self.m_leafReach - 1
                {
                    2.0
                }
                else
                {
                    3.0
                };
                self.leafLayer([node[0], node[1] + dy, node[2]], size, leaf);
            }
        }
    }

    // WorldGenBigTree.placeBlockLine, logs along a line turned to face the way it runs
    fn logLine(&self, from: [i32; 3], to: [i32; 3])
    {
        let delta: [i32; 3] = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let lead: usize = (0..3).fold(0, |best, axis|
        {
            if delta[axis].abs() > delta[best].abs()
            {
                axis
            }
            else
            {
                best
            }
        });
        if delta[lead] == 0
        {
            return;
        }
        let (first, second): (usize, usize) = (OTHER_AXES[lead], OTHER_AXES[lead + 3]);
        let step: i32 = if delta[lead] > 0 { 1 } else { -1 };
        let slopeFirst: f64 = f64::from(delta[first]) / f64::from(delta[lead]);
        let slopeSecond: f64 = f64::from(delta[second]) / f64::from(delta[lead]);
        let end: i32 = delta[lead] + step;
        let mut offset: i32 = 0;
        while offset != end
        {
            let mut at: [i32; 3] = [0; 3];
            at[lead] = math::floor(f64::from(from[lead] + offset) + 0.5);
            at[first] = math::floor(f64::from(from[first]) + f64::from(offset) * slopeFirst + 0.5);
            at[second] = math::floor(f64::from(from[second]) + f64::from(offset) * slopeSecond + 0.5);
            let (dx, dz): (i32, i32) = ((at[0] - from[0]).abs(), (at[2] - from[2]).abs());
            let widest: i32 = dx.max(dz);
            let bits: i32 = if widest == 0
            {
                0
            }
            else if dx == widest
            {
                4
            }
            else
            {
                8
            };
            self.m_builder.set(at[0], at[1], at[2], self.wood(bits));
            offset += step;
        }
    }

    fn validLocation(&mut self) -> bool
    {
        if !self.m_builder.soilBelow(self.m_base[0], self.m_base[1], self.m_base[2])
        {
            return false;
        }
        let top: [i32; 3] = [self.m_base[0], self.m_base[1] + self.m_limit - 1, self.m_base[2]];
        let run: i32 = self.clearRun(self.m_base, top);
        if run == -1
        {
            return true;
        }
        let shortest: i32 = 6;
        if run < shortest
        {
            return false;
        }
        self.m_limit = run;
        true
    }
}

pub fn large(builder: &Builder, random: &Random, x: i32, y: i32, z: i32, vanilla: bool) -> bool
{
    let own: Random = Random::withSeed(random.nextLong());
    let limit: i32 = 5 + own.nextIntBounded(shape::LIMIT);
    let mut tree: BigTree = BigTree
    {
        m_builder: builder,
        m_random: own,
        m_base: [x, y, z],
        m_limit: limit,
        m_height: 0,
        m_leafReach: shape::LEAF_REACH,
        m_nodes: Vec::new(),
        m_vanilla: vanilla,
    };
    if !tree.validLocation()
    {
        return false;
    }
    tree.nodes();
    tree.leaves();
    tree.logLine(tree.m_base, [x, y + tree.m_height, z]);
    let floor: f64 = f64::from(tree.m_limit) * 0.2;
    for node in tree.m_nodes.clone()
    {
        if f64::from(node[3] - tree.m_base[1]) >= floor
        {
            tree.logLine([x, node[3], z], [node[0], node[1], node[2]]);
        }
    }
    true
}
