use super::trees::{Builder, Placed, Woods, core, line, translate};
use crate::names::block;
use b173_api::prelude::*;
use b173_api::terrain::Random;

// TFGenHollowTree, a giant hollow oak with a ladder up its middle, a full crown and roots
mod hollow
{
    pub const HEIGHT_BASE: i32 = 32;
    pub const HEIGHT_SPREAD: i32 = 64;
    pub const DIAMETER_SPREAD: i32 = 4;
    pub const LEAF_DUNGEON_ODDS: i32 = 8;
    // beta's world ends at 128, the source's 256 tall trees are cut down to fit under it
    pub const CEILING: i32 = 127;
}

// one ring of branches around the trunk, TFGenHollowTree.buildBranchRing
struct Ring
{
    m_height: i32,
    m_heightVar: i32,
    m_length: i32,
    m_tilt: f64,
    m_min: i32,
    m_max: i32,
    m_size: i32,
    m_leafy: bool,
}

const fn ring(height: i32, heightVar: i32, length: i32, tilt: f64, min: i32, max: i32, size: i32, leafy: bool) -> Ring
{
    Ring
    {
        m_height: height,
        m_heightVar: heightVar,
        m_length: length,
        m_tilt: tilt,
        m_min: min,
        m_max: max,
        m_size: size,
        m_leafy: leafy,
    }
}

struct Hollow<'a>
{
    m_builder: &'a Builder<'a>,
    m_random: &'a Random,
    m_origin: [i32; 3],
    m_diameter: i32,
    m_woods: Woods,
}

impl Hollow<'_>
{
    fn smallBranch(&self, src: [i32; 3], length: f64, angle: f64, tilt: f64, leafy: bool)
    {
        let dest: [i32; 3] = translate(src, length, angle, tilt);
        self.m_builder.drawLine(src, dest, self.m_woods.m_branch);

        if leafy
        {
            let radius: i32 = self.m_random.nextIntBounded(2) + 1;
            self.m_builder.leafBlob(dest[0], dest[1], dest[2], radius, self.m_woods.m_leaf);
        }
    }

    fn medBranch(&self, src: [i32; 3], length: f64, angle: f64, tilt: f64, leafy: bool)
    {
        let dest: [i32; 3] = translate(src, length, angle, tilt);
        self.m_builder.drawLine(src, dest, self.m_woods.m_branch);

        if leafy
        {
            self.m_builder.leafBlob(dest[0], dest[1], dest[2], 2, self.m_woods.m_leaf);
        }

        let shoots: i32 = self.m_random.nextIntBounded(2) + 1;
        let step: f64 = 0.8 / f64::from(shoots);

        for index in 0..=shoots
        {
            let turn: f64 = step * f64::from(index) - 0.4;
            let out: f64 = self.m_random.nextDouble() * 0.8 + 0.2;
            let lean: f64 = self.m_random.nextDouble() * 0.75 + 0.15;
            let start: [i32; 3] = translate(src, length * out, angle, tilt);
            self.smallBranch(start, length * 0.4, angle + turn, tilt * lean, leafy);
        }
    }

    fn largeBranch(&self, src: [i32; 3], length: f64, angle: f64, tilt: f64, leafy: bool)
    {
        let dest: [i32; 3] = translate(src, length, angle, tilt);
        self.m_builder.drawLine(src, dest, self.m_woods.m_branch);

        let braces: i32 = self.m_random.nextIntBounded(3);
        for index in 0..=braces
        {
            let offset: [i32; 3] = [
                if index & 2 == 0 { 1 } else { 0 },
                if index & 1 == 0 { 1 } else { -1 },
                if index & 2 == 0 { 0 } else { 1 },
            ];
            self.m_builder.drawLine(
                [src[0] + offset[0], src[1] + offset[1], src[2] + offset[2]],
                dest,
                self.m_woods.m_branch,
            );
        }

        if leafy
        {
            self.m_builder.leafBlob(dest[0], dest[1] + 1, dest[2], 3, self.m_woods.m_leaf);
        }

        let mediums: i32 = self.m_random.nextIntBounded((length / 6.0) as i32) + self.m_random.nextIntBounded(2) + 1;
        for index in 0..=mediums
        {
            let out: f64 = self.m_random.nextDouble() * 0.3 + 0.3;
            let turn: f64 = self.m_random.nextDouble() * 0.225 * if index & 1 == 0 { 1.0 } else { -1.0 };
            let start: [i32; 3] = translate(src, length * out, angle, tilt);
            self.medBranch(start, length * 0.6, angle + turn, tilt, leafy);
        }

        let smalls: i32 = self.m_random.nextIntBounded(2) + 1;
        for index in 0..=smalls
        {
            let out: f64 = self.m_random.nextDouble() * 0.25 + 0.25;
            let turn: f64 = self.m_random.nextDouble() * 0.25 * if index & 1 == 0 { 1.0 } else { -1.0 };
            let start: [i32; 3] = translate(src, length * out, angle, tilt);
            self.smallBranch(start, (length * 0.3).max(2.0), angle + turn, tilt, leafy);
        }

        if self.m_random.nextIntBounded(hollow::LEAF_DUNGEON_ODDS) == 0
        {
            self.leafDungeon([dest[0], dest[1] + 1, dest[2]]);
        }
    }

    // TFGenHollowTree.makeLeafDungeon, a leafy ball hiding a swarm spider spawner and a chest
    fn leafDungeon(&self, at: [i32; 3])
    {
        let [x, y, z] = at;
        self.m_builder.leafBlob(x, y, z, 4, self.m_woods.m_leaf);
        self.m_builder.blob(x, y, z, 3, self.m_woods.m_branch);
        self.m_builder.blob(x, y, z, 2, (0, 0));

        let world: World = self.m_builder.m_world.m_world;
        if let Err(error) = world.setSpawner(BlockPos::new(x, y + 1, z), crate::entities::SWARM_SPIDER)
        {
            b173_api::warn(&format!("leaf dungeon spawner at {x} {y} {z}: {}", error.0));
        }

        let side: [[i32; 2]; 4] = [[0, 1], [-1, 0], [0, -1], [1, 0]];
        let [dx, dz] = side[self.m_random.nextIntBounded(4) as usize];
        crate::treasure::place(
            self.m_builder.m_world,
            self.m_random,
            crate::treasure::Table::TreeCache,
            x + dx * 2,
            y - 1,
            z + dz * 2,
        );
    }

    // TFGenHollowTree.makeRoot, wood while it is in the open and roots once it goes underground
    fn root(&self, height: i32, length: f64, angle: f64, tilt: f64)
    {
        let [x, y, z] = self.m_origin;
        let src: [i32; 3] = translate([x, y + height, z], f64::from(self.m_diameter), angle, 0.5);
        let dest: [i32; 3] = translate(src, length, angle, tilt);
        let mut above: bool = true;

        for [rx, ry, rz] in line(src, dest)
        {
            let placed: Placed = if above && self.m_builder.airAround(rx, ry, rz)
            {
                self.m_woods.m_branch
            }
            else
            {
                above = false;
                self.m_woods.m_root
            };
            self.m_builder.set(rx, ry, rz, placed);
            self.m_builder.set(rx, ry - 1, rz, placed);
        }
    }

    fn branchFrom(&self, height: i32) -> impl Fn(f64) -> [i32; 3] + '_
    {
        let [x, y, z] = self.m_origin;
        move |angle: f64| translate([x, y + height, z], f64::from(self.m_diameter), angle, 0.5)
    }

    fn ring(&self, ring: &Ring)
    {
        let count: i32 = self.m_random.nextIntBounded(ring.m_max - ring.m_min) + ring.m_min;
        let rotation: f64 = 1.0 / f64::from(count + 1);
        let offset: f64 = self.m_random.nextDouble();

        for index in 0..=count
        {
            let height: i32 = if ring.m_heightVar > 0
            {
                ring.m_height - ring.m_heightVar + self.m_random.nextIntBounded(2 * ring.m_heightVar)
            }
            else
            {
                ring.m_height
            };
            let angle: f64 = f64::from(index) * rotation + offset;
            let start: [i32; 3] = self.branchFrom(height)(angle);
            let length: f64 = f64::from(ring.m_length);

            match ring.m_size
            {
                1 => self.medBranch(start, length, angle, ring.m_tilt, ring.m_leafy),
                2 => self.largeBranch(start, length, angle, ring.m_tilt, ring.m_leafy),

                3 =>
                {
                    // the source's switch falls through from roots into small branches
                    self.root(height, length, angle, ring.m_tilt);
                    self.smallBranch(start, length, angle, ring.m_tilt, ring.m_leafy);
                }

                _ => self.smallBranch(start, length, angle, ring.m_tilt, ring.m_leafy),
            }
        }
    }

    fn reach(dx: i32, dz: i32) -> i32
    {
        (f64::from(dx.abs().max(dz.abs())) + f64::from(dx.abs().min(dz.abs())) * 0.5) as i32
    }

    // the buried part of TFGenHollowTree.buildTrunk, bark where air touches it and roots elsewhere
    fn base(&self)
    {
        let [x, y, z] = self.m_origin;
        let diameter: i32 = self.m_diameter;
        let hollowRadius: i32 = diameter / 2;

        for dx in -diameter..=diameter
        {
            for dz in -diameter..=diameter
            {
                let reach: i32 = Self::reach(dx, dz);
                if reach > diameter
                {
                    continue;
                }

                for dy in -4..0
                {
                    let (bx, by, bz): (i32, i32, i32) = (dx + x, dy + y, dz + z);
                    let placed: Placed = if self.m_builder.airAround(bx, by, bz)
                    {
                        if reach > hollowRadius
                        {
                            self.m_woods.m_trunk
                        }
                        else
                        {
                            self.m_woods.m_branch
                        }
                    }
                    else
                    {
                        self.m_woods.m_root
                    };
                    self.m_builder.set(bx, by, bz, placed);
                }
            }
        }
    }

    // TFGenHollowTree.buildTrunk, a round shell of wood with a ladder up its inside where the source hangs vines
    fn trunk(&self, height: i32)
    {
        let [x, y, z] = self.m_origin;
        let diameter: i32 = self.m_diameter;
        let hollowRadius: i32 = diameter / 2;

        self.base();
        // beta has no vines, a ladder on the same inner wall climbs the same way
        let ladderOnEastWall: Placed = super::palette::meta("minecraft:ladder", 4);

        for dx in -diameter..=diameter
        {
            for dz in -diameter..=diameter
            {
                let reach: i32 = Self::reach(dx, dz);

                for dy in 0..=height
                {
                    if reach <= diameter && reach > hollowRadius
                    {
                        self.m_builder.set(dx + x, dy + y, dz + z, self.m_woods.m_trunk);
                    }

                    if reach == hollowRadius && dx == hollowRadius
                    {
                        self.m_builder.set(dx + x, dy + y, dz + z, ladderOnEastWall);
                    }
                }
            }
        }
    }

    // TFGenHollowStump.buildTrunk, a ragged ring of bark a few blocks high
    fn stumpTrunk(&self)
    {
        let [x, y, z] = self.m_origin;
        let diameter: i32 = self.m_diameter;
        let hollowRadius: i32 = diameter / 2;

        self.base();

        for dx in -diameter..=diameter
        {
            for dz in -diameter..=diameter
            {
                let height: i32 = 2 + self.m_random.nextIntBounded(3) + self.m_random.nextIntBounded(2);
                let reach: i32 = Self::reach(dx, dz);
                if reach > diameter || reach <= hollowRadius
                {
                    continue;
                }

                for dy in 0..=height
                {
                    self.m_builder.set(dx + x, dy + y, dz + z, self.m_woods.m_trunk);
                }
            }
        }
    }

    fn critter(&self, name: &'static str, height: i32, angle: f64, fallback: i32)
    {
        let [x, y, z] = self.m_origin;
        let at: [i32; 3] = translate([x, y + height, z], f64::from(self.m_diameter + 1), angle, 0.5);
        let turn: f64 = angle % 1.0;
        let meta: i32 = if !(0.125..=0.875).contains(&turn)
        {
            3
        }
        else if turn <= 0.375
        {
            1
        }
        else if turn <= 0.625
        {
            4
        }
        else if turn <= 0.875
        {
            2
        }
        else
        {
            fallback
        };

        if self.m_builder.isAir(at[0], at[1], at[2])
        {
            let facing: &'static str = crate::blocks::CRITTER_FACINGS[meta as usize];
            self.m_builder.set(at[0], at[1], at[2], super::palette::with(name, "facing", facing));
        }
    }

    fn critters(&self, name: &'static str, height: i32, fallback: i32)
    {
        let count: i32 = self.m_random.nextIntBounded(3 * self.m_diameter) + 5;
        for _ in 0..=count
        {
            let at: i32 = (f64::from(height) * self.m_random.nextDouble() * 0.9) as i32 + height / 10;
            let angle: f64 = self.m_random.nextDouble();
            self.critter(name, at, angle, fallback);
        }
    }

    fn crown(&self, height: i32, radius: i32, spread: i32, sizes: [i32; 4])
    {
        let rings: [Ring; 4] = [
            ring(height - radius, 0, radius, 0.35, spread, spread + 2, sizes[0], true),
            ring(height - radius / 2, 0, radius, 0.28, spread, spread + 2, sizes[1], true),
            ring(height, 0, radius, 0.15, 2, 4, sizes[2], true),
            ring(height, 0, radius / 2, 0.05, spread, spread + 2, sizes[3], true),
        ];

        for ring in &rings
        {
            self.ring(ring);
        }
    }
}

fn clearCrown(builder: &Builder, origin: [i32; 3], height: i32, radius: i32) -> bool
{
    for dx in -radius..=radius
    {
        for dz in -radius..=radius
        {
            for dy in height - radius..=height + radius
            {
                let there: i32 = builder.block(origin[0] + dx, origin[1] + dy, origin[2] + dz);
                if there != 0 && !blockIs(there, "minecraft:leaves")
                {
                    return false;
                }
            }
        }
    }

    true
}

pub(crate) fn tree(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let mut height: i32 = random.nextIntBounded(hollow::HEIGHT_SPREAD) + hollow::HEIGHT_BASE;
    let diameter: i32 = random.nextIntBounded(hollow::DIAMETER_SPREAD) + 1;
    let crownRadius: i32 = diameter * 4 + 8;
    height = height.min(hollow::CEILING - y - crownRadius / 2);
    if y < 1 || height < hollow::HEIGHT_BASE / 2
    {
        return false;
    }
    if !clearCrown(builder, [x, y, z], height, crownRadius) || !builder.soilBelow(x, y, z)
    {
        return false;
    }

    let hollow: Hollow = Hollow
    {
        m_builder: builder,
        m_random: random,
        m_origin: [x, y, z],
        m_diameter: diameter,
        m_woods: Woods::plain(0, 0),
    };
    hollow.trunk(height);
    hollow.critters(block::FIREFLY, height, 0);
    hollow.critters(block::CICADA, height, 1);
    hollow.crown(height, diameter * 4 + 4, diameter + 2, [2, 1, 2, 1]);

    let branches: i32 = random.nextIntBounded(3) + 3;
    for _ in 0..=branches
    {
        let at: i32 = (f64::from(height) * random.nextDouble() * 0.9) as i32 + height / 10;
        let angle: f64 = random.nextDouble();
        let start: [i32; 3] = hollow.branchFrom(at)(angle);
        hollow.smallBranch(start, 4.0, angle, 0.35, true);
    }

    hollow.ring(&ring(3, 2, 6, 0.75, 3, 5, 3, false));
    hollow.ring(&ring(1, 2, 8, 0.9, 3, 5, 3, false));
    true
}

// TFGenTreeOfTime, a small hollow tree of timewood with the clock set into its trunk
pub(crate) fn time(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let height: i32 = 8;
    let diameter: i32 = 1;
    if y < 1 || y + height + diameter > hollow::CEILING || !builder.soilBelow(x, y, z)
    {
        return false;
    }

    let hollow: Hollow = Hollow
    {
        m_builder: builder,
        m_random: random,
        m_origin: [x, y, z],
        m_diameter: diameter,
        m_woods: Woods::magic(0),
    };
    hollow.trunk(height);

    let radius: i32 = 4;
    let rings: [Ring; 4] = [
        ring(height - radius, 0, radius, 0.35, 1, 3, 1, true),
        ring(height - radius / 2, 0, radius, 0.28, 1, 3, 1, true),
        ring(height, 0, radius, 0.15, 2, 4, 0, true),
        ring(height, 0, radius / 2, 0.05, 1, 3, 0, true),
    ];

    for ring in &rings
    {
        hollow.ring(ring);
    }

    hollow.ring(&ring(1, 0, 12, 0.75, 3, 5, 3, false));
    hollow.ring(&ring(1, 2, 18, 0.9, 3, 5, 3, false));
    builder.set(x - 1, y + 2, z, core(0));
    true
}

// TFGenHollowStump, the stub of a hollow tree on its roots
pub(crate) fn stump(builder: &Builder, random: &Random, x: i32, y: i32, z: i32) -> bool
{
    let radius: i32 = random.nextIntBounded(2) + 2;
    if !super::scatter::suitable(builder.m_world, x - radius, y, z - radius, [2 * radius, 6, 2 * radius])
    {
        return false;
    }

    let stump: Hollow = Hollow
    {
        m_builder: builder,
        m_random: random,
        m_origin: [x, y, z],
        m_diameter: radius,
        m_woods: Woods::plain(0, 0),
    };
    stump.stumpTrunk();
    stump.ring(&ring(3, 2, 6, 0.75, 3, 5, 3, false));
    stump.ring(&ring(1, 2, 8, 0.9, 3, 5, 3, false));
    true
}
