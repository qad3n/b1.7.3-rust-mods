use super::biomes::{self, Biome};
use super::flora::{self, Decor, Flower, Grass, Grow};
use super::generate::GenWorld;
use super::trees::Builder;
use super::{features, ruins, scatter, trees, vanilla};
use crate::ids::block;
use b173_api::prelude::*;
use b173_api::terrain::Random;

mod height
{
    // TFWorld.CHUNKHEIGHT, the source still draws its random heights over 256 blocks
    pub const SOURCE: i32 = 256;
    pub const SEA: i32 = 31;
    // roots and torchberries start hanging from this height
    pub const ROOTS: i32 = 64;
    // TFBiomeDecorator places its ruins only below this
    pub const RUINS: i32 = 75;
    // BiomeDecorator draws spring heights under these
    pub const WATER_SPRINGS: i32 = 248;
    pub const LAVA_SPRINGS: i32 = 240;
    pub const CAVE_SPRINGS: i32 = 24;
    pub const CAVE_FLOOR: i32 = 4;
    // the dark forest decorator draws heights over the old 128 block world
    pub const DARK: i32 = 128;
}

mod chunk
{
    pub const SIZE: i32 = 16;
    // decorations land half a chunk over so they spill into loaded neighbours
    pub const EDGE: i32 = 8;
}

// how many of each decoration one chunk tries, from the source decorators
mod counts
{
    pub const SAND: i32 = 3;
    pub const REEDS: i32 = 10;
    pub const WATER_SPRINGS: i32 = 50;
    pub const LAVA_SPRINGS: i32 = 20;
    pub const PLANT_ROOTS: i32 = 12;
    pub const WOOD_ROOTS: i32 = 20;
    pub const CAVE_SPRINGS: i32 = 50;
    pub const TORCHBERRIES: i32 = 3;
    pub const DUNGEONS: i32 = 8;
    pub const DEAD_BUSHES: i32 = 8;
    pub const LAMPS: i32 = 30;
    pub const TROLL_ROOTS: i32 = 24;
    pub const THORNS: i32 = 128;
    pub const PENGUINS: i32 = 10;
    pub const VINES: i32 = 50;
    pub const ENCHANTED_VINES: i32 = 20;
    pub const LILY_PADS: i32 = 25;
    pub const WATER_LILIES: i32 = 2;
    pub const DOUBLE_PLANTS: i32 = 7;
    pub const ENCHANTED_PLANTS: i32 = 20;
}

// the source's one in n odds
mod odds
{
    pub const RUIN: i32 = 6;
    pub const EXTRA_TREE: i32 = 10;
    pub const PUMPKIN: i32 = 32;
    pub const BROWN_MUSHROOM: i32 = 4;
    pub const RED_MUSHROOM: i32 = 8;
    pub const DARK_BROWN: i32 = 8;
    pub const DARK_RED: i32 = 16;
    pub const MUSHGLOOM: i32 = 24;
    pub const LAKE: i32 = 4;
    pub const LAVA_LAKE: i32 = 32;
    pub const SURFACE_LAVA: i32 = 10;
    pub const SMOKER: i32 = 4;
    pub const PENGUINS: i32 = 4;
    pub const LAMPPOST: i32 = 4;
}

// WorldGenDoublePlant kinds
mod double
{
    pub const LILAC: i32 = 1;
    pub const GRASS: i32 = 2;
    pub const FERN: i32 = 3;
    pub const ROSE_BUSH: i32 = 4;
    pub const PEONY: i32 = 5;
}

// one chunk's decoration, the source's BiomeDecorator fields
struct Decorator<'a>
{
    m_world: &'a GenWorld,
    m_builder: Builder<'a>,
    m_random: &'a Random,
    m_biome: Biome,
    m_decor: &'static Decor,
    m_x: i32,
    m_z: i32,
}

impl Decorator<'_>
{
    // a random column in the chunk offset by eight, as every decorator picks its spots
    fn column(&self) -> (i32, i32)
    {
        (
            self.m_x + self.m_random.nextIntBounded(chunk::SIZE) + chunk::EDGE,
            self.m_z + self.m_random.nextIntBounded(chunk::SIZE) + chunk::EDGE,
        )
    }

    fn height(&self, x: i32, z: i32) -> i32 { self.m_world.height(x, z) }

    // BiomeDecorator.nextInt as Forge patches it, no draw for a bound of one or less
    fn below(&self, bound: i32) -> i32
    {
        if bound <= 1
        {
            0
        }
        else
        {
            self.m_random.nextIntBounded(bound)
        }
    }

    fn id(&self, name: &'static str) -> i32 { self.m_world.id(name) }

    // TFBiomeDecorator.genDecorations
    fn decorate(&self)
    {
        if self.m_random.nextIntBounded(odds::RUIN) == 0
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = self.height(x, z);
            if y < height::RUINS
            {
                ruins::generate(self.m_world, self.m_random, x, y, z);
            }
        }
        self.canopy();
        for _ in 0..self.m_decor.m_mangroves
        {
            let (x, z): (i32, i32) = self.column();
            trees::mangrove(&self.m_builder, self.m_random, x, self.height(x, z), z);
        }
        for _ in 0..self.m_decor.m_lakes
        {
            let (x, z): (i32, i32) = self.column();
            vanilla::lake(self.m_world, self.m_random, x, self.height(x, z), z, self.id("minecraft:water"));
        }
        if self.m_random.nextFloat() <= self.m_decor.m_lava
        {
            let (x, z): (i32, i32) = self.column();
            vanilla::lake(self.m_world, self.m_random, x, self.height(x, z), z, self.id("minecraft:lava"));
        }
        for _ in 0..self.m_decor.m_mycelium
        {
            let (x, z): (i32, i32) = self.column();
            let size: i32 = 5;
            scatter::groundBlob(self.m_world, self.m_random, x, self.height(x, z), z, self.id("minecraft:dirt"), size);
        }
        self.vanilla();
        self.underground();
    }

    fn canopy(&self)
    {
        // the shared canopy follows the config, the dark forest and the off switches keep their own
        let coverage: f32 = if self.m_decor.m_canopy == flora::CANOPY
        {
            crate::config::canopy()
        }
        else
        {
            self.m_decor.m_canopy
        };
        let whole: i32 = coverage as i32;
        let count: i32 = whole + i32::from(self.m_random.nextFloat() < coverage - whole as f32);
        for _ in 0..count
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = self.height(x, z);
            let alternate: bool =
                self.m_decor.m_altCanopy > 0.0 && self.m_random.nextFloat() <= self.m_decor.m_altCanopy;
            match (self.m_biome, alternate)
            {
                // the savanna's alternate canopy is TFGenNoTree
                (Biome::OakSavanna, true) => false,
                (Biome::OakSavanna, false) => trees::canopyOak(&self.m_builder, self.m_random, x, y, z),
                (_, true) => trees::canopyMushroom(&self.m_builder, self.m_random, x, y, z),
                (_, false) => trees::canopyTree(&self.m_builder, self.m_random, x, y, z),
            };
        }
    }

    fn grow(&self, kind: Grow, x: i32, y: i32, z: i32)
    {
        let (world, random): (&GenWorld, &Random) = (self.m_world, self.m_random);
        match kind
        {
            Grow::Oak | Grow::Swamp => vanilla::oak(world, random, x, y, z),
            Grow::Birch => vanilla::birch(world, random, x, y, z),
            Grow::BigOak => super::rainbow::large(&self.m_builder, random, x, y, z, true),
            Grow::Shrub => vanilla::shrub(world, random, x, y, z),
            Grow::Taiga1 => vanilla::taiga1(world, random, x, y, z),
            Grow::Taiga2 | Grow::MegaPine => vanilla::taiga2(world, random, x, y, z),
            Grow::LargeWinter => super::winter::tree(&self.m_builder, random, x, y, z),
            Grow::SmallRainbow => trees::smallRainbow(&self.m_builder, random, x, y, z),
            Grow::LargeRainbow => super::rainbow::large(&self.m_builder, random, x, y, z, false),
        };
    }

    // BiomeDecorator.genDecorations of 1.7 with the plants beta lacks swapped for their nearest kin
    fn vanilla(&self)
    {
        let random: &Random = self.m_random;
        self.ores();
        let (sand, gravel, clay, dirt, grass): (i32, i32, i32, i32, i32) = (
            self.id("minecraft:sand"),
            self.id("minecraft:gravel"),
            self.id("minecraft:clay"),
            self.id("minecraft:dirt"),
            self.id("minecraft:grass"),
        );
        for _ in 0..counts::SAND
        {
            let (x, z): (i32, i32) = self.column();
            vanilla::disc(self.m_world, random, [x, self.m_world.topSolid(x, z), z], sand, 7, 2, &[dirt, grass]);
        }
        for _ in 0..self.m_decor.m_clay
        {
            let (x, z): (i32, i32) = self.column();
            vanilla::disc(self.m_world, random, [x, self.m_world.topSolid(x, z), z], clay, 4, 1, &[dirt, clay]);
        }
        let (x, z): (i32, i32) = self.column();
        vanilla::disc(self.m_world, random, [x, self.m_world.topSolid(x, z), z], gravel, 6, 2, &[dirt, grass]);
        let trees: i32 = self.m_decor.m_trees + i32::from(random.nextIntBounded(odds::EXTRA_TREE) == 0);
        for _ in 0..trees
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = self.height(x, z);
            self.grow(flora::tree(self.m_biome, random), x, y, z);
        }
        for _ in 0..self.m_decor.m_bigMushrooms
        {
            let (x, z): (i32, i32) = self.column();
            vanilla::bigMushroom(self.m_world, random, x, self.height(x, z), z);
        }
        self.flowers();
        for _ in 0..self.m_decor.m_grass
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = self.below(self.height(x, z) * 2);
            self.grass(flora::grass(self.m_biome, random), x, y, z);
        }
        for _ in 0..self.m_decor.m_deadBush
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = self.below(self.height(x, z) * 2);
            vanilla::deadBush(self.m_world, random, x, y, z);
        }
        for _ in 0..self.m_decor.m_waterlily
        {
            let (x, z): (i32, i32) = self.column();
            let mut y: i32 = self.below(self.height(x, z) * 2);
            while y > 0 && self.m_world.isAir(x, y - 1, z)
            {
                y -= 1;
            }
            vanilla::waterlily(random, x, y, z);
        }
        self.mushrooms();
        for _ in 0..self.m_decor.m_reeds + counts::REEDS
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = self.below(self.height(x, z) * 2);
            vanilla::reeds(self.m_world, random, x, y, z);
        }
        if random.nextIntBounded(odds::PUMPKIN) == 0
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = self.below(self.height(x, z) * 2);
            vanilla::pumpkins(self.m_world, random, x, y, z);
        }
        if self.m_decor.m_springs
        {
            self.springs();
        }
    }

    fn flowers(&self)
    {
        let random: &Random = self.m_random;
        for _ in 0..self.m_decor.m_flowers
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = self.below(self.height(x, z) + 32);
            let name: &'static str = match flora::flower(self.m_biome, random, x, z)
            {
                Flower::Rose => "minecraft:red_flower",
                Flower::Dandelion => "minecraft:yellow_flower",
            };
            vanilla::flowers(self.m_world, random, x, y, z, self.id(name), vanilla::lit);
        }
    }

    fn grass(&self, kind: Grass, x: i32, y: i32, z: i32)
    {
        let tallgrass: i32 = self.id("minecraft:tallgrass");
        match kind
        {
            Grass::Grass => vanilla::tallGrass(self.m_world, self.m_random, x, y, z, (tallgrass, 1), vanilla::lit),
            Grass::Fern => vanilla::tallGrass(self.m_world, self.m_random, x, y, z, (tallgrass, 2), vanilla::lit),
            Grass::Twilight(plant) =>
            {
                scatter::plantPatch(self.m_world, self.m_random, x, y, z, plant);
                true
            }
        };
    }

    fn mushroom(&self, red: bool, x: i32, y: i32, z: i32)
    {
        let name: &'static str = if red
        {
            "minecraft:red_mushroom"
        }
        else
        {
            "minecraft:brown_mushroom"
        };
        let brightest: i32 = 13;
        let dim = |world: &GenWorld, bx: i32, by: i32, bz: i32|
        {
            world.m_world.light(BlockPos::new(bx, by, bz)) < brightest && world.isOpaque(bx, by - 1, bz)
        };
        vanilla::flowers(self.m_world, self.m_random, x, y, z, self.id(name), dim);
    }

    fn mushrooms(&self)
    {
        let random: &Random = self.m_random;
        for _ in 0..self.m_decor.m_mushrooms
        {
            if random.nextIntBounded(odds::BROWN_MUSHROOM) == 0
            {
                let (x, z): (i32, i32) = self.column();
                self.mushroom(false, x, self.height(x, z), z);
            }
            if random.nextIntBounded(odds::RED_MUSHROOM) == 0
            {
                let (x, z): (i32, i32) = self.column();
                let y: i32 = self.below(self.height(x, z) * 2);
                self.mushroom(true, x, y, z);
            }
        }
        for (red, odds) in [(false, odds::BROWN_MUSHROOM), (true, odds::RED_MUSHROOM)]
        {
            if random.nextIntBounded(odds) == 0
            {
                let (x, z): (i32, i32) = self.column();
                let y: i32 = self.below(self.height(x, z) * 2);
                self.mushroom(red, x, y, z);
            }
        }
    }

    // the water and lava springs of BiomeDecorator.genDecorations
    fn springs(&self)
    {
        let random: &Random = self.m_random;
        let (water, lava): (i32, i32) = (self.id("minecraft:flowing_water"), self.id("minecraft:flowing_lava"));
        for _ in 0..counts::WATER_SPRINGS
        {
            let x: i32 = self.m_x + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
            let y: i32 = random.nextIntBounded(random.nextIntBounded(height::WATER_SPRINGS) + 8);
            let z: i32 = self.m_z + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
            vanilla::spring(self.m_world, x, y, z, water);
        }
        for _ in 0..counts::LAVA_SPRINGS
        {
            let x: i32 = self.m_x + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
            let y: i32 =
                random.nextIntBounded(random.nextIntBounded(random.nextIntBounded(height::LAVA_SPRINGS) + 8) + 8);
            let z: i32 = self.m_z + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
            vanilla::spring(self.m_world, x, y, z, lava);
        }
    }

    // BiomeDecorator.generateOres
    fn ores(&self)
    {
        let rows: [(&'static str, i32, i32, i32); 7] = [
            ("minecraft:dirt", 20, 32, 256),
            ("minecraft:gravel", 10, 32, 256),
            ("minecraft:coal_ore", 20, 16, 128),
            ("minecraft:iron_ore", 20, 8, 64),
            ("minecraft:gold_ore", 2, 8, 32),
            ("minecraft:redstone_ore", 8, 7, 16),
            ("minecraft:diamond_ore", 1, 7, 16),
        ];
        let random: &Random = self.m_random;
        for (name, count, size, top) in rows
        {
            let block: i32 = self.id(name);
            for _ in 0..count
            {
                let x: i32 = self.m_x + random.nextIntBounded(chunk::SIZE);
                let y: i32 = random.nextIntBounded(top);
                let z: i32 = self.m_z + random.nextIntBounded(chunk::SIZE);
                vanilla::ore(self.m_world, random, x, y, z, block, size);
            }
        }
        let (center, spread): (i32, i32) = (16, 16);
        let x: i32 = self.m_x + random.nextIntBounded(chunk::SIZE);
        let y: i32 = random.nextIntBounded(spread) + random.nextIntBounded(spread) + center - spread;
        let z: i32 = self.m_z + random.nextIntBounded(chunk::SIZE);
        vanilla::ore(self.m_world, random, x, y, z, self.id("minecraft:lapis_ore"), 6);
    }

    // TFBiomeDecorator.decorateUnderground
    fn underground(&self)
    {
        let random: &Random = self.m_random;
        for _ in 0..counts::PLANT_ROOTS
        {
            let (x, z): (i32, i32) = self.column();
            scatter::plantRoots(self.m_world, random, x, height::ROOTS, z);
        }
        for _ in 0..counts::WOOD_ROOTS
        {
            let x: i32 = self.m_x + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
            let y: i32 = random.nextIntBounded(height::ROOTS);
            let z: i32 = self.m_z + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
            scatter::woodRoots(self.m_world, random, x, y, z);
        }
        if self.m_decor.m_springs
        {
            let water: i32 = self.id("minecraft:flowing_water");
            for _ in 0..counts::CAVE_SPRINGS
            {
                let x: i32 = self.m_x + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
                let y: i32 = random.nextIntBounded(height::CAVE_SPRINGS) + height::CAVE_FLOOR;
                let z: i32 = self.m_z + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
                vanilla::spring(self.m_world, x, y, z, water);
            }
        }
        for _ in 0..counts::TORCHBERRIES
        {
            let (x, z): (i32, i32) = self.column();
            scatter::torchBerries(self.m_world, random, x, height::ROOTS, z);
        }
    }

    // TFDarkForestBiomeDecorator.decorateChunk, the dark canopy and its sparse undergrowth
    fn dark(&self)
    {
        let random: &Random = self.m_random;
        let whole: i32 = self.m_decor.m_canopy as i32;
        let count: i32 = whole + i32::from(random.nextFloat() < self.m_decor.m_canopy - whole as f32);
        for _ in 0..count
        {
            let (x, z): (i32, i32) = self.column();
            trees::darkCanopy(&self.m_builder, random, x, self.height(x, z), z);
        }
        for _ in 0..self.m_decor.m_trees
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = self.ground(x, z, &[]);
            self.grow(flora::tree(self.m_biome, random), x, y, z);
        }
        for _ in 0..self.m_decor.m_deadBush
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = random.nextIntBounded(height::DARK);
            scatter::plants(self.m_world, random, x, y, z, block::DRIED_BUSH, counts::DEAD_BUSHES);
        }
        for _ in 0..self.m_decor.m_deadBush
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = random.nextIntBounded(height::DARK);
            scatter::plantPatch(self.m_world, random, x, y, z, block::FOREST_GRASS);
        }
        for _ in 0..self.m_decor.m_mushrooms
        {
            for (red, odds) in [(false, odds::DARK_BROWN), (true, odds::DARK_RED)]
            {
                if random.nextIntBounded(odds) == 0
                {
                    let (x, z): (i32, i32) = self.column();
                    self.mushroom(red, x, self.ground(x, z, &[]), z);
                }
            }
            if random.nextIntBounded(odds::MUSHGLOOM) == 0
            {
                let (x, z): (i32, i32) = self.column();
                scatter::plantPatch(self.m_world, random, x, self.ground(x, z, &[]), z, block::MUSHGLOOM);
            }
        }
        for (red, odds) in [(false, odds::BROWN_MUSHROOM), (true, odds::RED_MUSHROOM)]
        {
            if random.nextIntBounded(odds) == 0
            {
                let (x, z): (i32, i32) = self.column();
                self.mushroom(red, x, random.nextIntBounded(height::DARK), z);
            }
        }
        if random.nextIntBounded(odds::PUMPKIN) == 0
        {
            let (x, z): (i32, i32) = self.column();
            vanilla::pumpkins(self.m_world, random, x, self.ground(x, z, &[]), z);
        }
    }

    // getGroundLevel of the firefly forest, dark forest and thornlands, the first grass above the sea
    fn ground(&self, x: i32, z: i32, extra: &[&'static str]) -> i32
    {
        let grass: i32 = self.id("minecraft:grass");
        let earth: Vec<i32> = ["minecraft:dirt", "minecraft:stone", "minecraft:gravel"]
            .iter()
            .chain(extra)
            .map(|name| self.id(name))
            .collect();
        let mut last: i32 = height::SEA;
        for y in height::SEA..super::terrain::height() - 1
        {
            let block: i32 = self.m_world.block(x, y, z);
            if block == grass
            {
                return y + 1;
            }
            if earth.contains(&block)
            {
                last = y + 1;
            }
        }
        last
    }

    fn doublePlants(&self, kind: i32, count: i32)
    {
        for _ in 0..count
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = self.m_random.nextIntBounded(self.height(x, z) + 32);
            vanilla::doublePlant(self.m_world, self.m_random, x, y, z, kind);
        }
    }

    fn vines(&self, count: i32)
    {
        for _ in 0..count
        {
            let (x, z): (i32, i32) = self.column();
            vanilla::vines(self.m_world, self.m_random, x, height::SEA, z);
        }
    }

    // each biome's func_76728_a, its own extras around the shared decorator
    fn biome(&self, decorated: bool)
    {
        let random: &Random = self.m_random;
        match self.m_biome
        {
            Biome::DenseForest => self.doublePlants(double::FERN, counts::DOUBLE_PLANTS),
            Biome::EnchantedForest =>
            {
                self.vines(counts::ENCHANTED_VINES);
                self.doublePlants(double::FERN, counts::ENCHANTED_PLANTS);
            }
            Biome::OakSavanna => self.doublePlants(double::GRASS, counts::DOUBLE_PLANTS),
            Biome::FireflyForest => self.fireflyFlowers(),
            Biome::Highlands => self.highlands(),
            _ =>
            {}
        }
        match self.m_biome
        {
            Biome::DarkForest | Biome::DarkForestCenter =>
            {
                if decorated
                {
                    self.dark();
                }
                self.underground();
                self.ores();
            }
            _ if decorated => self.decorate(),
            _ =>
            {
                self.underground();
                self.ores();
            }
        }
        match self.m_biome
        {
            Biome::Swamp =>
            {
                self.vines(counts::VINES);
                for _ in 0..counts::LILY_PADS
                {
                    let (x, z): (i32, i32) = self.column();
                    scatter::lilyPads(self.m_world, random, x, height::SEA, z);
                }
                for _ in 0..counts::WATER_LILIES
                {
                    let (x, z): (i32, i32) = self.column();
                    scatter::waterLilies(self.m_world, random, x, height::SEA, z);
                }
            }
            Biome::FireSwamp if decorated =>
            {
                self.vines(counts::VINES);
                if random.nextIntBounded(odds::SMOKER) == 0
                {
                    let (x, z): (i32, i32) = self.column();
                    scatter::fireJet(self.m_world, random, x, height::SEA, z, block::SMOKER);
                }
                let (x, z): (i32, i32) = self.column();
                scatter::fireJet(self.m_world, random, x, height::SEA, z, block::FIRE_JET);
            }
            Biome::Glacier if random.nextIntBounded(odds::PENGUINS) == 0 =>
            {
                let (x, z): (i32, i32) = self.column();
                scatter::critters(self.m_world, random, x, z, crate::entities::PENGUIN, counts::PENGUINS);
            }
            Biome::Thornlands =>
            {
                let extra: [&'static str; 4] =
                    ["minecraft:sandstone", "minecraft:sand", "minecraft:clay", block::SOLID_DEADROCK];
                for _ in 0..counts::THORNS
                {
                    let (x, z): (i32, i32) = self.column();
                    let y: i32 = self.ground(x, z, &extra);
                    scatter::thorns(self.m_world, random, x, y, z);
                }
            }
            Biome::FireflyForest => self.fireflyLights(),
            _ =>
            {}
        }
    }

    fn fireflyFlowers(&self)
    {
        let random: &Random = self.m_random;
        let cycles: i32 = random.nextIntBounded(3) - 1;
        for _ in 0..cycles
        {
            let kind: i32 = [double::LILAC, double::ROSE_BUSH, double::PEONY][random.nextIntBounded(3) as usize];
            self.doublePlants(kind, 1);
        }
    }

    // TFBiomeFireflyForest.decorate after the shared decorator, mushglooms lamps and lampposts
    fn fireflyLights(&self)
    {
        let random: &Random = self.m_random;
        if random.nextIntBounded(odds::MUSHGLOOM) == 0
        {
            let (x, z): (i32, i32) = self.column();
            scatter::plantPatch(self.m_world, random, x, self.ground(x, z, &[]), z, block::MUSHGLOOM);
        }
        for _ in 0..counts::LAMPS
        {
            let (x, z): (i32, i32) = self.column();
            let y: i32 = height::SEA + random.nextIntBounded(height::SOURCE - height::SEA);
            scatter::hangingLamp(self.m_world, x, y, z);
        }
        if random.nextIntBounded(odds::LAMPPOST) == 0
        {
            let (x, z): (i32, i32) = self.column();
            scatter::lamppost(self.m_world, random, x, self.ground(x, z, &[]), z);
        }
        if random.nextIntBounded(odds::PUMPKIN) == 0
        {
            let (x, z): (i32, i32) = self.column();
            vanilla::pumpkins(self.m_world, random, x, self.ground(x, z, &[]), z);
        }
    }

    // TFBiomeHighlands.decorate before the shared decorator
    fn highlands(&self)
    {
        let random: &Random = self.m_random;
        let boulders: i32 = random.nextIntBounded(2);
        for _ in 0..boulders
        {
            let (x, z): (i32, i32) = self.column();
            vanilla::boulder(self.m_world, random, x, self.height(x, z), z, self.id("minecraft:mossy_cobblestone"));
        }
        self.doublePlants(double::FERN, counts::DOUBLE_PLANTS);
        let (x, z): (i32, i32) = self.column();
        let y: i32 = random.nextIntBounded(height::ROOTS);
        scatter::plantPatch(self.m_world, random, x, y, z, block::MUSHGLOOM);
        for _ in 0..counts::TROLL_ROOTS
        {
            let (x, z): (i32, i32) = self.column();
            scatter::trollRoots(self.m_world, random, x, height::ROOTS, z);
        }
    }
}

// ChunkProviderTwilightForest.populate, the chunk seed then every decoration of the chunk
pub fn populate(chunk: ChunkInfo)
{
    let world: GenWorld = GenWorld::new(World { dimension: chunk.world });
    let random: Random = Random::withSeed(chunk.seed);
    let xFactor: i64 = random.nextLong() / 2 * 2 + 1;
    let zFactor: i64 = random.nextLong() / 2 * 2 + 1;
    random.setSeed(
        i64::from(chunk.x).wrapping_mul(xFactor).wrapping_add(i64::from(chunk.z).wrapping_mul(zFactor)) ^ chunk.seed,
    );
    let (mapX, mapZ): (i32, i32) = (chunk.x * 16, chunk.z * 16);
    let biome: Biome = biomes::at(chunk.seed, mapX + 16, mapZ + 16);
    let decor: &'static Decor = flora::decor(biome);
    // ChunkProviderTwilightForest.populate, a structure crossing the chunk keeps lakes out, a nearby feature also the decorations
    let built: bool = super::structures::generate(&world, &random, chunk.seed, chunk.x, chunk.z);
    super::structures::hollowTrees(&world, &random, chunk.seed, chunk.x, chunk.z);
    let decorated: bool = features::row(features::nearest(chunk.seed, chunk.x, chunk.z).0).m_decorated;
    let open: bool = decorated && !built;
    let column = ||
    {
        (
            mapX + random.nextIntBounded(chunk::SIZE) + chunk::EDGE,
            mapZ + random.nextIntBounded(chunk::SIZE) + chunk::EDGE,
        )
    };
    if open && random.nextIntBounded(odds::LAKE) == 0 && decor.m_springs
    {
        let (x, z): (i32, i32) = column();
        let y: i32 = random.nextIntBounded(height::SOURCE);
        vanilla::lake(&world, &random, x, y, z, world.id("minecraft:water"));
    }
    if open && random.nextIntBounded(odds::LAVA_LAKE) == 0
    {
        let x: i32 = mapX + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
        let y: i32 = random.nextIntBounded(random.nextIntBounded(height::SOURCE - 8) + 8);
        let z: i32 = mapZ + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
        if y < height::SEA || random.nextIntBounded(odds::SURFACE_LAVA) == 0
        {
            vanilla::lake(&world, &random, x, y, z, world.id("minecraft:lava"));
        }
    }
    for _ in 0..counts::DUNGEONS
    {
        let x: i32 = mapX + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
        let y: i32 = random.nextIntBounded(height::SOURCE);
        let z: i32 = mapZ + random.nextIntBounded(chunk::SIZE) + chunk::EDGE;
        vanilla::dungeon(&world, &random, x, y, z);
    }
    let decorator: Decorator = Decorator
    {
        m_world: &world,
        m_builder: Builder { m_world: &world, m_notify: false },
        m_random: &random,
        m_biome: biome,
        m_decor: decor,
        m_x: mapX,
        m_z: mapZ,
    };
    decorator.biome(decorated);
    crate::entities::spawns::worldgen(&world, &random, biome, mapX + 8, mapZ + 8);
    freeze(&world, mapX + 8, mapZ + 8);
}

// the snow and ice pass at the end of populate, by each column's biome temperature
fn freeze(world: &GenWorld, x: i32, z: i32)
{
    let seed: i64 = world.m_world.seed();
    let (ice, snow, water): (i32, i32, i32) =
        (world.id("minecraft:ice"), world.id("minecraft:snow_layer"), world.id("minecraft:water"));
    for dx in 0..16
    {
        for dz in 0..16
        {
            let (bx, bz): (i32, i32) = (x + dx, z + dz);
            let cold: bool = biomes::rowOf(biomes::at(seed, bx, bz)).m_temperature < 0.15;
            if !cold
            {
                continue;
            }
            let top: i32 = world.topSolid(bx, bz).max(world.height(bx, bz));
            if world.block(bx, top - 1, bz) == water && world.meta(bx, top - 1, bz) == 0
            {
                world.set(bx, top - 1, bz, ice);
            }
            else if world.isAir(bx, top, bz) && world.isOpaque(bx, top - 1, bz)
            {
                world.set(bx, top, bz, snow);
            }
        }
    }
}
