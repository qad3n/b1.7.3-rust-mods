use super::gen_world::GenWorld;
use super::trees::{self, Tree};
use crate::names::{Pick, block};
use b173_api::prelude::*;
use b173_api::terrain::{Random, math};

mod cloud
{
    pub const GOLDEN_AMOUNT: i32 = 4;
    pub const GOLDEN_CHANCE: i32 = 50;
    pub const GOLDEN_BASE: i32 = 96;
    pub const BLUE_AMOUNT: i32 = 8;
    pub const BLUE_CHANCE: i32 = 26;
    pub const BLUE_BASE: i32 = 32;
    pub const COLD_AMOUNT: i32 = 16;
    pub const COLD_CHANCE: i32 = 14;
    pub const COLD_BASE: i32 = 64;
    pub const BASE_SPREAD: i32 = 64;
    pub const RISE: i32 = 64;
    pub const WIDTH: i32 = 3;
    pub const HEIGHT: i32 = 2;
    pub const REACH: i32 = 4;
}

mod ore
{
    pub const DIRT_CHANCE: i32 = 3;
    // block size, veins per chunk, highest y
    pub const DIRT: (i32, i32, i32) = (32, 20, 128);
    pub const ICESTONE: (i32, i32, i32) = (16, 10, 128);
    pub const AMBROSIUM: (i32, i32, i32) = (16, 15, 128);
    pub const ZANITE: (i32, i32, i32) = (8, 12, 64);
    pub const GRAVITITE: (i32, i32, i32) = (6, 8, 32);
    pub const RISE: i32 = 2;
}

mod feature
{
    pub const CRYSTAL_ISLAND_CHANCE: i32 = 37;
    pub const ISLAND_BASE: i32 = 32;
    pub const ISLAND_SPREAD: i32 = 64;
    pub const TREES: i32 = 3;
    pub const OAK_CHANCE: i32 = 30;
    pub const DUNGEON_OAKS: i32 = 25;
    pub const HOLIDAY_TREE_CHANCE: i32 = 15;
    // AetherBiomeDecorator draws tall grass heights over twice the 1.7 world, the patches then sink to the ground
    pub const GRASS_PATCHES: i32 = 10;
    pub const GRASS_HEIGHT: i32 = 512;
    pub const GRASS_TRIES: i32 = 128;
    pub const LAKE_CHANCE: i32 = 10;
    pub const LAKE_SPREAD: i32 = 256;
    pub const PLANT_TRIES: i32 = 64;
    pub const PLANT_PATCHES: i32 = 2;
    pub const PLANT_SPREAD: i32 = 8;
    pub const PLANT_RISE: i32 = 4;
    pub const HEIGHT: i32 = 128;
    // decoration lands in the chunk diagonal from the populated one, as vanilla populate does
    pub const OFFSET: i32 = 8;
}

const CHUNK: i32 = 16;

struct Decorator<'a>
{
    m_world: &'a GenWorld,
    m_random: &'a Random,
    m_x: i32,
    m_z: i32,
}

impl Decorator<'_>
{
    fn nextInt(&self, bound: i32) -> i32 { self.m_random.nextIntBounded(bound) }

    fn shouldSpawn(&self, chance: i32) -> bool { self.nextInt(chance) == 0 }

    // AetherBiomeDecorator.generateClouds, the height argument draws before the chance as the Java call evaluates it
    fn clouds(&self, block: &'static str, amount: i32, chance: i32, base: i32)
    {
        let height: i32 = self.nextInt(cloud::BASE_SPREAD) + base;

        if self.shouldSpawn(chance)
        {
            let x: i32 = self.m_x + self.nextInt(CHUNK);
            let y: i32 = self.nextInt(height);
            let z: i32 = self.m_z + self.nextInt(CHUNK);
            cloudPuffs(self.m_world, self.m_random, block, amount, x, y, z);
        }
    }

    // AetherBiomeDecorator.spawnOre
    fn ore(&self, block: &'static str, (size, veins, top): (i32, i32, i32))
    {
        for _ in 0..veins
        {
            let x: i32 = self.m_x + self.nextInt(CHUNK);
            let y: i32 = self.nextInt(top);
            let z: i32 = self.m_z + self.nextInt(CHUNK);
            minable(self.m_world, self.m_random, block, size, x, y, z);
        }
    }

    // AetherBiomeDecorator.generateFoilage
    fn plants(&self, block: &'static str)
    {
        for _ in 0..feature::PLANT_PATCHES
        {
            let x: i32 = self.m_x + self.nextInt(CHUNK) + feature::OFFSET;
            let y: i32 = self.nextInt(feature::HEIGHT);
            let z: i32 = self.m_z + self.nextInt(CHUNK) + feature::OFFSET;
            plantPatch(self.m_world, self.m_random, block, x, y, z);
        }
    }

    // AetherBiomeDecorator.decorate's crystal island
    fn crystalIsland(&self)
    {
        if self.shouldSpawn(feature::CRYSTAL_ISLAND_CHANCE)
        {
            let y: i32 = self.nextInt(feature::ISLAND_SPREAD) + feature::ISLAND_BASE;
            crystalIsland(self.m_world, self.m_random, self.m_x + feature::OFFSET, y, self.m_z + feature::OFFSET);
        }
    }

    fn aetherDirt(&self)
    {
        if self.shouldSpawn(ore::DIRT_CHANCE)
        {
            self.ore(block::AETHER_DIRT, ore::DIRT);
        }
    }

    fn trees(&self)
    {
        for _ in 0..feature::TREES
        {
            let x: i32 = self.m_x + self.nextInt(CHUNK) + feature::OFFSET;
            let z: i32 = self.m_z + self.nextInt(CHUNK) + feature::OFFSET;
            let y: i32 = self.m_world.height(x, z);

            let tree: Tree = if self.shouldSpawn(feature::OAK_CHANCE)
            {
                Tree::GoldenOak
            }
            else
            {
                Tree::Skyroot
            };
            tree.generate(self.m_world, self.m_random, x, y, z);
        }
    }

    // AetherConfig.shouldLoadHolidayContent: the Java draws its chance only when the trees are on
    fn holidayTrees(&self)
    {
        if crate::config::enabled(crate::config::CHRISTMAS_CONTENT) && self.shouldSpawn(feature::HOLIDAY_TREE_CHANCE)
        {
            let x: i32 = self.m_x + feature::OFFSET;
            let z: i32 = self.m_z + feature::OFFSET;
            super::holiday::holidayTree(self.m_world, self.m_random, x, self.m_world.height(x, z), z);
        }
    }

    // AetherConfig.tallgrassEnabled
    fn tallGrass(&self)
    {
        if !crate::config::enabled(crate::config::TALL_GRASS)
        {
            return;
        }

        for _ in 0..feature::GRASS_PATCHES
        {
            let x: i32 = self.m_x + self.nextInt(CHUNK) + feature::OFFSET;
            let z: i32 = self.m_z + self.nextInt(CHUNK) + feature::OFFSET;
            let y: i32 = self.nextInt(feature::GRASS_HEIGHT);
            grassPatch(self.m_world, self.m_random, x, y, z);
        }
    }

    fn dungeonOaks(&self)
    {
        for _ in 0..feature::DUNGEON_OAKS
        {
            let x: i32 = self.m_x + self.nextInt(CHUNK);
            let z: i32 = self.m_z + self.nextInt(CHUNK);
            let y: i32 = self.m_world.height(x, z);
            trees::dungeonOakTree(self.m_world, self.m_random, x, y, z);
        }
    }

    fn lakes(&self)
    {
        if self.shouldSpawn(feature::LAKE_CHANCE)
        {
            let x: i32 = self.m_x + self.nextInt(CHUNK) + feature::OFFSET;
            let y: i32 = self.nextInt(feature::LAKE_SPREAD);
            let z: i32 = self.m_z + self.nextInt(CHUNK) + feature::OFFSET;
            super::lakes::waterLake(self.m_world, self.m_random, x, y, z);
        }
    }

    // ChunkProviderAether.populate's silver and gold dungeons, before the decorator
    fn dungeons(&self)
    {
        let world: World = self.m_world.m_world;
        let chunk: ChunkInfo =
            ChunkInfo { world: world.dimension, x: self.m_x / CHUNK, z: self.m_z / CHUNK, seed: world.seed() };
        super::dungeons::structures(self.m_world, self.m_random, &chunk);
    }

    // ChunkProviderAether.populate's bronze dungeon, after the decorator
    fn bronze(&self) { super::dungeons::bronze(self.m_world, self.m_random, self.m_x, self.m_z); }
}

// one step of ChunkProviderAether.populate, a named feature placed in this order on the chunk's random, so the
// loader lists every Aether decoration and players can turn one off; holiday trees and tall grass follow their
// config options, off as in the Java config
struct Decoration
{
    m_name: &'static str,
    m_step: FeatureStep,
    m_run: fn(&Decorator),
}

const DECORATIONS: [Decoration; 19] = [
    Decoration { m_name: "dungeons", m_step: FeatureStep::SurfaceStructures, m_run: |decorator| decorator.dungeons() },
    Decoration
    {
        m_name: "golden_aerclouds",
        m_step: FeatureStep::RawGeneration,
        m_run: |decorator|
        {
            decorator.clouds(block::GOLDEN_AERCLOUD, cloud::GOLDEN_AMOUNT, cloud::GOLDEN_CHANCE, cloud::GOLDEN_BASE)
        },
    },
    Decoration
    {
        m_name: "blue_aerclouds",
        m_step: FeatureStep::RawGeneration,
        m_run: |decorator|
        {
            decorator.clouds(block::BLUE_AERCLOUD, cloud::BLUE_AMOUNT, cloud::BLUE_CHANCE, cloud::BLUE_BASE)
        },
    },
    Decoration
    {
        m_name: "cold_aerclouds",
        m_step: FeatureStep::RawGeneration,
        m_run: |decorator|
        {
            decorator.clouds(block::COLD_AERCLOUD, cloud::COLD_AMOUNT, cloud::COLD_CHANCE, cloud::COLD_BASE)
        },
    },
    Decoration
    {
        m_name: "crystal_islands",
        m_step: FeatureStep::SurfaceStructures,
        m_run: |decorator| decorator.crystalIsland(),
    },
    Decoration
    {
        m_name: "aether_dirt",
        m_step: FeatureStep::UndergroundOres,
        m_run: |decorator| decorator.aetherDirt(),
    },
    Decoration
    {
        m_name: "white_flowers",
        m_step: FeatureStep::VegetalDecoration,
        m_run: |decorator| decorator.plants(block::WHITE_FLOWER),
    },
    Decoration
    {
        m_name: "purple_flowers",
        m_step: FeatureStep::VegetalDecoration,
        m_run: |decorator| decorator.plants(block::PURPLE_FLOWER),
    },
    Decoration
    {
        m_name: "icestone",
        m_step: FeatureStep::UndergroundOres,
        m_run: |decorator| decorator.ore(block::ICESTONE, ore::ICESTONE),
    },
    Decoration
    {
        m_name: "ambrosium_ore",
        m_step: FeatureStep::UndergroundOres,
        m_run: |decorator| decorator.ore(block::AMBROSIUM_ORE, ore::AMBROSIUM),
    },
    Decoration
    {
        m_name: "zanite_ore",
        m_step: FeatureStep::UndergroundOres,
        m_run: |decorator| decorator.ore(block::ZANITE_ORE, ore::ZANITE),
    },
    Decoration
    {
        m_name: "gravitite_ore",
        m_step: FeatureStep::UndergroundOres,
        m_run: |decorator| decorator.ore(block::GRAVITITE_ORE, ore::GRAVITITE),
    },
    Decoration
    {
        m_name: "berry_bushes",
        m_step: FeatureStep::VegetalDecoration,
        m_run: |decorator| decorator.plants(block::BERRY_BUSH),
    },
    Decoration { m_name: "trees", m_step: FeatureStep::VegetalDecoration, m_run: |decorator| decorator.trees() },
    Decoration
    {
        m_name: "holiday_trees",
        m_step: FeatureStep::VegetalDecoration,
        m_run: |decorator| decorator.holidayTrees(),
    },
    Decoration
    {
        m_name: "dungeon_oaks",
        m_step: FeatureStep::VegetalDecoration,
        m_run: |decorator| decorator.dungeonOaks(),
    },
    Decoration
    {
        m_name: "tall_grass",
        m_step: FeatureStep::VegetalDecoration,
        m_run: |decorator| decorator.tallGrass(),
    },
    Decoration { m_name: "lakes", m_step: FeatureStep::Lakes, m_run: |decorator| decorator.lakes() },
    Decoration
    {
        m_name: "bronze_dungeons",
        m_step: FeatureStep::UndergroundStructures,
        m_run: |decorator| decorator.bronze(),
    },
];

fn place(run: fn(&Decorator), ctx: &FeatureCtx) -> bool
{
    let world: GenWorld = GenWorld::new(ctx.world);
    run(&Decorator { m_world: &world, m_random: ctx.random, m_x: ctx.origin.x, m_z: ctx.origin.z });
    true
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for decoration in &DECORATIONS
    {
        let run: fn(&Decorator) = decoration.m_run;
        let feature: FeatureBuilder = FeatureBuilder::custom(decoration.m_step).dimension(super::DIMENSION)?.placed();
        ctx.worldgen().custom(decoration.m_name, feature, move |placed| place(run, placed))?;
    }

    Ok(())
}

// AetherGenClouds.generate, every loop bound draws a fresh random on each check as the Java condition does
fn cloudPuffs(world: &GenWorld, random: &Random, block: &'static str, amount: i32, x: i32, y: i32, z: i32)
{
    let (mut xIn, mut yIn, mut zIn): (i32, i32, i32) = (x, y + random.nextIntBounded(cloud::RISE), z);

    for _ in 0..amount
    {
        let offsetY: bool = random.nextBoolean();

        xIn += random.nextIntBounded(2);

        yIn += if offsetY
        {
            random.nextIntBounded(3) - 1
        }
        else
        {
            0
        };
        zIn += random.nextIntBounded(2);

        let mut x1: i32 = xIn;
        while x1 < xIn + random.nextIntBounded(2) + cloud::WIDTH
        {
            let mut y1: i32 = yIn;
            while y1 < yIn + random.nextIntBounded(1) + cloud::HEIGHT
            {
                let mut z1: i32 = zIn;
                while z1 < zIn + random.nextIntBounded(2) + cloud::WIDTH
                {
                    if world.isAir(x1, y1, z1)
                        && (x1 - xIn).abs() + (y1 - yIn).abs() + (z1 - zIn).abs()
                            < cloud::REACH + random.nextIntBounded(2)
                    {
                        world.set(x1, y1, z1, block);
                    }

                    z1 += 1;
                }

                y1 += 1;
            }

            x1 += 1;
        }
    }
}

// AetherGenMinable.generate, WorldGenMinable restricted to holystone
fn minable(world: &GenWorld, random: &Random, block: &'static str, size: i32, x: i32, y: i32, z: i32)
{
    let angle: f32 = random.nextFloat() * std::f32::consts::PI;
    let reach = |wave: f32| (wave * size as f32 / 8.0) as f64;
    let x0: f64 = (x + feature::OFFSET) as f64 + reach(math::sin(angle));
    let x1: f64 = (x + feature::OFFSET) as f64 - reach(math::sin(angle));
    let z0: f64 = (z + feature::OFFSET) as f64 + reach(math::cos(angle));
    let z1: f64 = (z + feature::OFFSET) as f64 - reach(math::cos(angle));
    let y0: f64 = (y + random.nextIntBounded(3) + ore::RISE) as f64;
    let y1: f64 = (y + random.nextIntBounded(3) + ore::RISE) as f64;

    for l in 0..=size
    {
        let centerX: f64 = x0 + (x1 - x0) * l as f64 / size as f64;
        let centerY: f64 = y0 + (y1 - y0) * l as f64 / size as f64;
        let centerZ: f64 = z0 + (z1 - z0) * l as f64 / size as f64;
        let spread: f64 = random.nextDouble() * size as f64 / 16.0;
        let bulge: f64 = (math::sin(l as f32 * std::f32::consts::PI / size as f32) + 1.0) as f64;
        let width: f64 = bulge * spread + 1.0;
        let height: f64 = bulge * spread + 1.0;
        let fromX: i32 = math::floor(centerX - width / 2.0);
        let fromY: i32 = math::floor(centerY - height / 2.0);
        let fromZ: i32 = math::floor(centerZ - width / 2.0);
        let toX: i32 = math::floor(centerX + width / 2.0);
        let toY: i32 = math::floor(centerY + height / 2.0);
        let toZ: i32 = math::floor(centerZ + width / 2.0);

        for bx in fromX..=toX
        {
            let dx: f64 = (bx as f64 + 0.5 - centerX) / (width / 2.0);
            if dx * dx >= 1.0
            {
                continue;
            }

            for by in fromY..=toY
            {
                let dy: f64 = (by as f64 + 0.5 - centerY) / (height / 2.0);
                if dx * dx + dy * dy >= 1.0
                {
                    continue;
                }

                for bz in fromZ..=toZ
                {
                    let dz: f64 = (bz as f64 + 0.5 - centerZ) / (width / 2.0);
                    if dx * dx + dy * dy + dz * dz < 1.0 && world.is(bx, by, bz, block::HOLYSTONE)
                    {
                        world.set(bx, by, bz, block);
                    }
                }
            }
        }
    }
}

// AetherGenFoilage.generate
fn plantPatch(world: &GenWorld, random: &Random, block: &'static str, x: i32, y: i32, z: i32)
{
    for _ in 0..feature::PLANT_TRIES
    {
        let plantX: i32 =
            x + random.nextIntBounded(feature::PLANT_SPREAD) - random.nextIntBounded(feature::PLANT_SPREAD);
        let plantY: i32 = y + random.nextIntBounded(feature::PLANT_RISE) - random.nextIntBounded(feature::PLANT_RISE);
        let plantZ: i32 =
            z + random.nextIntBounded(feature::PLANT_SPREAD) - random.nextIntBounded(feature::PLANT_SPREAD);

        if world.isAir(plantX, plantY, plantZ) && world.tagged(plantX, plantY - 1, plantZ, crate::tags::SOIL)
        {
            world.set(plantX, plantY, plantZ, block);
        }
    }
}

// WorldGenTallGrass.generate: the patch sinks through air and leaves to the ground, then grass takes root around it
fn grassPatch(world: &GenWorld, random: &Random, x: i32, y: i32, z: i32)
{
    const TALL_GRASS: Pick = Pick::with("minecraft:tallgrass", &[("meta", "1")]);

    let open = |y: i32| world.isAir(x, y, z) || world.tagged(x, y, z, "minecraft:leaves");
    let mut ground: i32 = y.min(feature::HEIGHT - 1);
    while ground > 0 && open(ground)
    {
        ground -= 1;
    }

    for _ in 0..feature::GRASS_TRIES
    {
        let grassX: i32 =
            x + random.nextIntBounded(feature::PLANT_SPREAD) - random.nextIntBounded(feature::PLANT_SPREAD);
        let grassY: i32 =
            ground + random.nextIntBounded(feature::PLANT_RISE) - random.nextIntBounded(feature::PLANT_RISE);
        let grassZ: i32 =
            z + random.nextIntBounded(feature::PLANT_SPREAD) - random.nextIntBounded(feature::PLANT_SPREAD);

        if world.isAir(grassX, grassY, grassZ) && world.tagged(grassX, grassY - 1, grassZ, crate::tags::SOIL)
        {
            world.put(grassX, grassY, grassZ, TALL_GRASS);
        }
    }
}

// AetherGenFloatingIsland.generate, a tiny grass island holding a crystal tree
fn crystalIsland(world: &GenWorld, random: &Random, j: i32, k: i32, l: i32) -> bool
{
    let clear: bool = (j - 6..j + 12).all(|x| (k - 6..k + 17).all(|y| (l - 6..l + 12).all(|z| world.isAir(x, y, z))));
    if k + 11 > feature::HEIGHT || !clear
    {
        return false;
    }

    let crystal = ||
    {
        if random.nextIntBounded(3) == 0
        {
            block::CRYSTAL_FRUIT_LEAVES
        }
        else
        {
            block::CRYSTAL_LEAVES
        }
    };

    let cross = |y: i32, radius: i32, block: &dyn Fn() -> &'static str|
    {
        for z in -radius..=radius
        {
            world.set(j, y, l + z, block());
        }

        for x in -radius..=radius
        {
            world.set(j + x, y, l, block());
        }
    };

    let ring = |y: i32, block: &dyn Fn() -> &'static str|
    {
        for x in -1..2
        {
            for z in (-1..=1).rev()
            {
                if x != 0 || z != 0
                {
                    world.set(j + x, y, l + z, block());
                }
            }
        }

        world.set(j + 1, y, l + 1, block());
        world.set(j - 1, y, l - 1, block());
    };

    let holystone = || block::HOLYSTONE;
    let grass = || block::AETHER_GRASS;
    let log = || block::SKYROOT_LOG;

    world.set(j, k, l + 1, block::HOLYSTONE);

    for x in -1..2
    {
        world.set(x + j, k, l, block::HOLYSTONE);
    }

    cross(k + 1, 2, &holystone);
    ring(k + 1, &holystone);
    cross(k + 2, 2, &grass);
    ring(k + 2, &grass);
    world.set(j, k + 10, l, crystal());
    crossSkippingCenter(world, j, k + 5, l, 1, &log);
    cross(k + 5, 2, &crystal);

    for x in [-1, 1]
    {
        world.set(j + x, k + 5, l - 2, crystal());
    }

    for x in [-1, 1]
    {
        world.set(j + x, k + 5, l + 2, crystal());
    }

    for z in [-1, 1]
    {
        world.set(j - 2, k + 5, l + z, crystal());
    }

    for z in [-1, 1]
    {
        world.set(j + 2, k + 5, l + z, crystal());
    }

    ring(k + 5, &crystal);
    cross(k + 6, 2, &crystal);
    ring(k + 6, &crystal);
    crossSkippingCenter(world, j, k + 6, l, 1, &crystal);
    crossSkippingCenter(world, j, k + 7, l, 1, &crystal);
    crossSkippingCenter(world, j, k + 8, l, 1, &log);
    cross(k + 8, 2, &crystal);
    ring(k + 8, &crystal);
    crossSkippingCenter(world, j, k + 9, l, 1, &crystal);

    for y in k + 3..=k + 9
    {
        world.set(j, y, l, block::SKYROOT_LOG);
    }

    true
}

fn crossSkippingCenter(world: &GenWorld, j: i32, y: i32, l: i32, radius: i32, block: &dyn Fn() -> &'static str)
{
    for z in (-radius..=radius).filter(|z| *z != 0)
    {
        world.set(j, y, l + z, block());
    }

    for x in (-radius..=radius).filter(|x| *x != 0)
    {
        world.set(j + x, y, l, block());
    }
}

// ChunkProviderAether.populate
pub(crate) fn populate(chunkInfo: ChunkInfo)
{
    let world: World = World { dimension: chunkInfo.world };
    let x: i32 = chunkInfo.x * CHUNK;
    let z: i32 = chunkInfo.z * CHUNK;
    let random: Random = Random::withSeed(chunkInfo.seed);
    let k: i64 = random.nextLong() / 2 * 2 + 1;
    let l: i64 = random.nextLong() / 2 * 2 + 1;
    random.setSeed((x as i64).wrapping_mul(k).wrapping_add((z as i64).wrapping_mul(l)) ^ chunkInfo.seed);

    for decoration in &DECORATIONS
    {
        world.placeFeature(decoration.m_name, BlockPos::new(x, 0, z), &random);
    }
}
