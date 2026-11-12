use crate::core::AIR;
use b173_api::prelude::*;
use b173_api::terrain::Random;

// BuildCraftEnergy's oil: a slow water-like fluid with no infinite sources, its bucket and the deposits OilPopulate
// scatters through the overworld
pub(crate) const OIL: &str = "buildcraft:oil";
// both oil blocks, so pumps and generators take any mod's oil
pub(crate) const OIL_TAG: &str = "c:oil";
// any mod's oil bucket, the combustion engine burns them all
pub(crate) const OIL_BUCKETS: &str = "c:buckets/oil";
const STILL: &str = "buildcraft:oil_still";
const BUCKET: &str = "buildcraft:oil_bucket";
const HARDNESS: f32 = 100.0;
const OPACITY: i32 = 3;
const FEATURE: &str = "oil_deposits";

mod deposit
{
    // OilPopulate's chances and sizes
    pub const DESERT_CHANCE: f32 = 0.97;
    pub const DESERT_SPREAD: i32 = 10;
    pub const DESERT_EDGE: i32 = 2;
    pub const DESERT_FLOOR: i32 = 65;
    pub const DESERT_RADIUS: i32 = 3;
    pub const MEDIUM_CHANCE: f64 = 0.001;
    pub const LARGE_CHANCE: f64 = 5.0E-5;
    pub const DEPTH: i32 = 20;
    pub const DEPTH_SPREAD: i32 = 10;
    pub const LARGE_RADIUS: i32 = 8;
    pub const LARGE_RADIUS_SPREAD: i32 = 9;
    pub const MEDIUM_RADIUS: i32 = 3;
    pub const MEDIUM_RADIUS_SPREAD: i32 = 4;
    pub const LARGE_POOL: i32 = 20;
    pub const MEDIUM_POOL: i32 = 5;
    pub const LARGE_GEYSER: i32 = 30;
    pub const MEDIUM_GEYSER: i32 = 4;
    pub const TOP: i32 = 128;
    pub const POOL_EDGE: i32 = 4;
}

const DESERT: &str = "minecraft:desert";
const SAND: &str = "minecraft:sand";
const WATER: &str = "minecraft:water";
// a deposit's well looks down through trees and grass for the ground, dirt already counts as ground
const LEAVES: &str = "minecraft:leaves";
const LOGS: &str = "minecraft:logs";
const GRASS: &str = "minecraft:grass";

struct Placer<'a>
{
    m_world: World,
    m_random: &'a Random,
}

impl Placer<'_>
{
    fn block(&self, x: i32, y: i32, z: i32) -> i32 { self.m_world.block(BlockPos::new(x, y, z)) }

    fn air(&self, x: i32, y: i32, z: i32) -> bool { blockIs(self.block(x, y, z), AIR) }

    fn set(&self, x: i32, y: i32, z: i32, block: &str)
    {
        if let Err(error) = self.m_world.place(BlockPos::new(x, y, z), block)
        {
            warn(&format!("could not place oil at {x} {y} {z}: {error}"));
        }
    }

    fn isOil(&self, x: i32, y: i32, z: i32) -> bool { blockTagged(self.block(x, y, z), OIL_TAG) }

    fn isWater(&self, x: i32, y: i32, z: i32) -> bool { blockTagged(self.block(x, y, z), WATER) }

    // OilPopulate.setOilWithProba: a pool cell sinks one block into oil next to oil already there
    fn cell(&self, chance: f32, x: i32, y: i32, z: i32, force: bool)
    {
        let rolled: bool = self.m_random.nextFloat() <= chance && !self.air(x, y - 2, z);
        if !rolled && !force
        {
            return;
        }

        let near: bool = (-1..=1).any(|d|
        {
            self.isOil(x + d, y - 1, z)
                || self.isOil(x - d, y - 1, z)
                || self.isOil(x, y - 1, z + d)
                || self.isOil(x, y - 1, z - d)
        });
        if !near && !force
        {
            return;
        }

        let keep: &str = if self.isWater(x, y, z) || self.isOil(x, y, z)
        {
            STILL
        }
        else
        {
            AIR
        };
        self.set(x, y, z, keep);
        self.set(x, y - 1, z, STILL);
    }

    // OilPopulate.generateSurfaceDeposit, a pool whose edges thin out with distance
    fn pool(&self, x: i32, y: i32, z: i32, radius: i32)
    {
        self.cell(1.0, x, y, z, true);

        for ring in 1..=radius
        {
            let chance: f32 = (radius - ring + deposit::POOL_EDGE) as f32 / (radius + deposit::POOL_EDGE) as f32;

            for d in -ring..=ring
            {
                self.cell(chance, x + d, y, z + ring, false);
                self.cell(chance, x + d, y, z - ring, false);
                self.cell(chance, x + ring, y, z + d, false);
                self.cell(chance, x - ring, y, z + d, false);
            }
        }

        for dx in x - radius..=x + radius
        {
            for dz in z - radius..=z + radius
            {
                let enclosed: bool = !blockIs(self.block(dx, y - 1, dz), STILL)
                    && self.isOil(dx + 1, y - 1, dz)
                    && self.isOil(dx - 1, y - 1, dz)
                    && self.isOil(dx, y - 1, dz + 1)
                    && self.isOil(dx, y - 1, dz - 1);
                if enclosed
                {
                    self.cell(1.0, dx, y, dz, true);
                }
            }
        }
    }

    fn desert(&self, x: i32, z: i32)
    {
        if self.m_world.biomeAt(x, z).is_none_or(|biome| biome.to_string() != DESERT)
            || self.m_random.nextFloat() <= deposit::DESERT_CHANCE
        {
            return;
        }

        let column: i32 = x + self.m_random.nextIntBounded(deposit::DESERT_SPREAD) + deposit::DESERT_EDGE;
        let row: i32 = z + self.m_random.nextIntBounded(deposit::DESERT_SPREAD) + deposit::DESERT_EDGE;

        for y in (deposit::DESERT_FLOOR + 1..=deposit::TOP).rev()
        {
            let block: i32 = self.block(column, y, row);
            if blockIs(block, AIR)
            {
                continue;
            }

            if blockTagged(block, SAND)
            {
                self.pool(column, y, row, deposit::DESERT_RADIUS);
            }

            return;
        }
    }

    // the underground sphere, then a well up to a pool or geyser at the surface
    fn deposit(&self, x: i32, z: i32)
    {
        let medium: bool = self.m_random.nextDouble() <= deposit::MEDIUM_CHANCE;
        let large: bool = self.m_random.nextDouble() <= deposit::LARGE_CHANCE;
        if !medium && !large
        {
            return;
        }

        let depth: i32 = deposit::DEPTH + self.m_random.nextIntBounded(deposit::DEPTH_SPREAD);
        let radius: i32 = if large
        {
            deposit::LARGE_RADIUS + self.m_random.nextIntBounded(deposit::LARGE_RADIUS_SPREAD)
        }
        else
        {
            deposit::MEDIUM_RADIUS + self.m_random.nextIntBounded(deposit::MEDIUM_RADIUS_SPREAD)
        };

        for dx in -radius..=radius
        {
            for dy in -radius..=radius
            {
                for dz in -radius..=radius
                {
                    if dx * dx + dy * dy + dz * dz <= radius * radius
                    {
                        self.set(dx + x, dy + depth, dz + z, STILL);
                    }
                }
            }
        }

        self.well(x, z, depth, large);
    }

    fn well(&self, x: i32, z: i32, depth: i32, large: bool)
    {
        let mut started: bool = false;

        for y in (depth..=deposit::TOP).rev()
        {
            if started
            {
                self.set(x, y, z, STILL);
                continue;
            }

            let block: i32 = self.block(x, y, z);
            let open: bool = blockIs(block, GRASS) || blockTagged(block, LEAVES) || blockTagged(block, LOGS);
            if blockIs(block, AIR) || open
            {
                continue;
            }

            started = true;

            let (pool, geyser): (i32, i32) = if large
            {
                (deposit::LARGE_POOL + self.m_random.nextIntBounded(deposit::LARGE_POOL), deposit::LARGE_GEYSER)
            }
            else
            {
                (deposit::MEDIUM_POOL + self.m_random.nextIntBounded(deposit::MEDIUM_POOL), deposit::MEDIUM_GEYSER)
            };
            self.pool(x, y, z, pool);

            // BuildCraft cast the top to a byte so a geyser reaching 128 vanished, here it stops at the last layer
            for height in y + 1..=(y + geyser).min(deposit::TOP - 1)
            {
                self.set(x, height, z, STILL);
            }
        }
    }
}

// OilPopulate, the loader seeds the random from the feature's name so other mods never shift the deposits
fn populate(ctx: &FeatureCtx) -> bool
{
    let placer: Placer = Placer { m_world: ctx.world, m_random: ctx.random };

    placer.desert(ctx.origin.x, ctx.origin.z);
    placer.deposit(ctx.origin.x, ctx.origin.z);
    true
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.fluids().register("oil", Fluid::new(FluidStyle::Water, OIL, STILL).bucket(BUCKET).infinite(false))?;

    for (name, template, texture) in [
        ("oil", "minecraft:flowing_fluid", "buildcraft:block/oil_flow"),
        ("oil_still", "minecraft:still_fluid", "buildcraft:block/oil_still"),
    ]
    {
        ctx.blocks().register(
            name,
            BlockSpecBuilder::template(template)
                .material(OIL)
                .texture(texture)
                .hardness(HARDNESS)
                .opacity(OPACITY)
                .tag(OIL_TAG),
        )?;
    }

    // the fluid tag lets other mods' engines, tanks and machines take BuildCraft oil
    ctx.tags().add(RegistryKind::Fluid, OIL_TAG, &[OIL])?;

    ctx.items().register(
        "oil_bucket",
        ItemSpecBuilder::template("minecraft:water_bucket").texture("buildcraft:item/oil_bucket").tag(OIL_BUCKETS),
    )?;

    // a named feature, so the loader lists it and players can turn it off in config/loader.toml
    ctx.worldgen().custom(FEATURE, FeatureBuilder::custom(FeatureStep::Lakes), populate)
}
