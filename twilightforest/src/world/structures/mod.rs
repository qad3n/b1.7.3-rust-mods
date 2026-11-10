mod base;
mod component;
mod darktower;
mod finalcastle;
mod gens;
mod hollowtree;
mod icetower;
mod legacy;
mod lichtower;
mod minotaurmaze;
mod mushroomtower;
mod piece;
mod prelude;
mod stronghold;
mod trollcave;

use super::biomes::Biome;
use super::features::{self, Feature};
use super::generate::GenWorld;
use b173_api::World;
use b173_api::prelude::{BlockPos, Identifier, RegisterCtx, Result, StructurePiece, Structures, warn};
use b173_api::terrain::Random;
use b173_api::terrain::structure::{self, StructureBox};
use component::{BoxExt, Pieces};
use piece::Piece;
use prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

// MapGenTFMajorFeature and StructureTFMajorFeatureStart: each feature chunk lays out its pieces once, every chunk the
// pieces cross builds its share of them while it populates
struct Start
{
    m_feature: Feature,
    m_chunk: [i32; 2],
    m_box: StructureBox,
    m_pieces: Pieces,
}

// MapGenTFHollowTree's starts, registered beside the major features
const HOLLOW_TREE: &str = "hollow_tree";
const HOLLOW_TREE_LABEL: &str = "Giant Hollow Oak";

mod range
{
    // MapGenBase looks this many chunks around for structure starts
    pub const STARTS: i32 = 8;
    pub const CACHED: usize = 256;
}

thread_local!
{
    static STARTS: RefCell<HashMap<(Family, i64, i32, i32), Option<Rc<Start>>>> = RefCell::new(HashMap::new());
    // starts already handed to the loader's structure registry this session, it drops repeats by origin anyway
    static RECORDED: RefCell<HashSet<(Family, i64, i32, i32)>> = RefCell::new(HashSet::new());
}

// StructureTFMajorFeatureStart.makeFirstComponent
fn first(world: &GenWorld, feature: Feature, random: &Random, [x, y, z]: [i32; 3]) -> Option<Rc<Piece>>
{
    let piece: Rc<Piece> = match feature
    {
        Feature::NagaCourtyard => naga_courtyard::new(world, random, 0, x, y, z),
        Feature::HedgeMaze => hedge_maze::new(world, random, 0, x, y, z),
        Feature::SmallHill => hollow_hill::new(world, random, 0, 1, x, y, z),
        Feature::MediumHill => hollow_hill::new(world, random, 0, 2, x, y, z),
        Feature::LargeHill => hollow_hill::new(world, random, 0, 3, x, y, z),
        Feature::LichTower => tower_main::new(world, random, 0, x, y, z),
        Feature::QuestGrove => quest_grove::new(world, random, 0, x, y, z),
        Feature::HydraLair => hydra_lair::new(world, random, 0, x, y, z),
        Feature::Labyrinth => maze_ruins::new(world, random, 0, x, y, z),
        Feature::DarkTower => dark_tower_main::new(random, 0, x, y - 1, z),
        Feature::Stronghold => stronghold_entrance::new(world, random, 0, x, y, z),
        Feature::IceTower => ice_tower_main::new(world, random, 0, x, y, z),
        Feature::MushroomTower => mushroom_tower_main::new(world, random, 0, x, y, z),
        Feature::YetiCave => yeti_cave::new(world, random, 0, x, y, z),
        Feature::TrollCave => troll_cave_main::new6(world, random, 0, x, y, z),
        Feature::FinalCastle => final_castle_main::new(world, random, 0, x, y, z),
        _ => return None,
    };
    Some(piece)
}

fn start(world: &GenWorld, seed: i64, chunkX: i32, chunkZ: i32, random: &Random) -> Option<Rc<Start>>
{
    let feature: Feature = features::directlyAt(seed, chunkX, chunkZ);
    if !features::row(feature).m_structure
    {
        return None;
    }

    let at: [i32; 3] = [(chunkX << 4) + 8, super::terrain::SEA_LEVEL + 1, (chunkZ << 4) + 8];
    let root: Rc<Piece> = first(world, feature, random, at)?;
    let mut pieces: Pieces = vec![Rc::clone(&root)];
    root.build(&root, &mut pieces, random);
    if matches!(root.m_kind, Kind::TowerMain | Kind::DarkTowerMain)
    {
        lift(&pieces, seed, at);
    }

    let mut bounds: StructureBox = root.m_box.get();
    for piece in &pieces
    {
        bounds.include(&piece.m_box.get());
    }
    Some(Rc::new(Start { m_feature: feature, m_chunk: [chunkX, chunkZ], m_box: bounds, m_pieces: pieces }))
}

// StructureTFMajorFeatureStart.moveToAvgGroundLevel, towers rise with the terrain of their biome
fn lift(pieces: &Pieces, seed: i64, [x, _, z]: [i32; 3])
{
    let rise: i32 = super::biomes::surfaceRise(seed, x, z);
    if rise <= 0
    {
        return;
    }

    for piece in pieces
    {
        piece.boxed(|bounds| bounds.shift([0, rise, 0]));
    }
}

// the two MapGenBase generators, the major features and the giant twilight oaks, each with its own start cache
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Family
{
    Major,
    HollowTree,
}

type Maker = fn(&GenWorld, i64, i32, i32, &Random) -> Option<Rc<Start>>;

fn starts(world: &GenWorld, family: Family, seed: i64, chunkX: i32, chunkZ: i32) -> Vec<Rc<Start>>
{
    let maker: Maker = match family
    {
        Family::Major => start,
        Family::HollowTree => hollowStart,
    };
    let mut found: Vec<Rc<Start>> = Vec::new();
    structure::candidates(seed, chunkX, chunkZ, range::STARTS, |startX, startZ, random|
    {
        let key: (Family, i64, i32, i32) = (family, seed, startX, startZ);
        let cached: Option<Option<Rc<Start>>> = STARTS.with(|starts| starts.borrow().get(&key).cloned());
        let made: Option<Rc<Start>> = match cached
        {
            Some(made) => made,
            None =>
            {
                let made: Option<Rc<Start>> = maker(world, seed, startX, startZ, random);
                STARTS.with(|starts|
                {
                    let mut starts = starts.borrow_mut();
                    if starts.len() > range::CACHED
                    {
                        starts.clear();
                    }
                    starts.insert(key, made.clone());
                });
                made
            }
        };
        found.extend(made);
    });
    found
}

// MapGenTFHollowTree.oakSpawnBiomes
const OAK_BIOMES: [Biome; 10] = [
    Biome::TwilightForest,
    Biome::DenseForest,
    Biome::Mushrooms,
    Biome::Swamp,
    Biome::Clearing,
    Biome::OakSavanna,
    Biome::FireflyForest,
    Biome::DeepMushrooms,
    Biome::EnchantedForest,
    Biome::FireSwamp,
];

// MapGenTFHollowTree.canSpawnStructureAtCoords and StructureTFHollowTreeStart
fn hollowStart(world: &GenWorld, seed: i64, chunkX: i32, chunkZ: i32, random: &Random) -> Option<Rc<Start>>
{
    let (x, z): (i32, i32) = ((chunkX << 4) + 8, (chunkZ << 4) + 8);
    let decorated: bool = features::row(features::nearest(seed, chunkX, chunkZ).0).m_decorated;
    if random.nextIntBounded(crate::config::oakChance()) != 0
        || !decorated
        || !OAK_BIOMES.contains(&super::biomes::at(seed, x, z))
    {
        return None;
    }

    let trunk: Rc<Piece> = hollow_tree_trunk::new(world, random, 0, x, super::terrain::SEA_LEVEL + 1, z);
    let mut pieces: Pieces = vec![Rc::clone(&trunk)];
    trunk.build(&trunk, &mut pieces, random);
    let mut bounds: StructureBox = trunk.m_box.get();
    for piece in &pieces
    {
        bounds.include(&piece.m_box.get());
    }
    Some(Rc::new(Start { m_feature: Feature::Nothing, m_chunk: [chunkX, chunkZ], m_box: bounds, m_pieces: pieces }))
}

// MapGenTFHollowTree.generateStructuresInChunk, the oaks crossing the chunk built into it
pub fn hollowTrees(world: &GenWorld, random: &Random, seed: i64, chunkX: i32, chunkZ: i32)
{
    let (mapX, mapZ): (i32, i32) = ((chunkX << 4) + 8, (chunkZ << 4) + 8);
    let reach: i32 = 15;
    let clip: StructureBox = StructureBox::flat(mapX, mapZ, mapX + reach, mapZ + reach);
    for start in starts(world, Family::HollowTree, seed, chunkX, chunkZ)
    {
        if !start.m_box.intersectsXZ(mapX, mapZ, mapX + reach, mapZ + reach)
        {
            continue;
        }

        record(world, Family::HollowTree, seed, &start);
        for piece in &start.m_pieces
        {
            if piece.m_box.get().intersects(&clip)
            {
                piece.add(world, random, clip);
            }
        }
    }
}

// MapGenStructure.generateStructuresInChunk with the populate random, true when a structure crosses the chunk
pub fn generate(world: &GenWorld, random: &Random, seed: i64, chunkX: i32, chunkZ: i32) -> bool
{
    let clip: StructureBox = StructureBox::populated(chunkX, chunkZ);
    let mut built: bool = false;
    for start in starts(world, Family::Major, seed, chunkX, chunkZ)
    {
        if !start.m_box.intersects(&clip)
        {
            continue;
        }

        built = true;
        record(world, Family::Major, seed, &start);
        for piece in &start.m_pieces
        {
            if piece.m_box.get().intersects(&clip)
            {
                piece.add(world, random, clip);
            }
        }
    }
    built
}

// the structure kinds this mod builds, the major features can be located before any chunk builds them
pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let structures: Structures = ctx.structures();
    for row in features::FEATURES.iter().filter(|row| row.m_structure)
    {
        let feature: Feature = row.m_feature;
        structures.withFinder(row.m_name, row.m_label, super::DIMENSION, move |seed, chunkX, chunkZ|
        {
            features::directlyAt(seed, chunkX, chunkZ) == feature
        })?;
    }
    structures.register(HOLLOW_TREE, HOLLOW_TREE_LABEL, super::DIMENSION)?;
    Ok(())
}

// hands a start to the loader's structure registry the first time a chunk builds part of it
fn record(world: &GenWorld, family: Family, seed: i64, start: &Start)
{
    let [chunkX, chunkZ] = start.m_chunk;
    let key: (Family, i64, i32, i32) = (family, seed, chunkX, chunkZ);
    let fresh: bool = RECORDED.with(|recorded| recorded.borrow_mut().insert(key));
    if !fresh
    {
        return;
    }

    let name: &str = match family
    {
        Family::Major => features::row(start.m_feature).m_name,
        Family::HollowTree => HOLLOW_TREE,
    };
    let Ok(id) = Identifier::parse(&format!("twilightforest:{name}"))
    else
    {
        warn(&format!("structure name {name} is not an identifier"));
        return;
    };

    let origin: BlockPos = BlockPos::new((chunkX << 4) + 8, start.m_box.m_min[1], (chunkZ << 4) + 8);
    let pieces: Vec<StructurePiece> = start.m_pieces.iter().map(|piece| piece.m_box.get().piece("")).collect();
    if let Err(error) = world.m_world.addStructure(structure::start(&id, origin, pieces))
    {
        warn(&format!("could not record {id}: {}", error.0));
    }
}

// MapGenTFMajorFeature.isBlockProtectedAt and getSBBAt: the box of the protected piece around a block, if any
pub fn protectedAt(world: World, x: i32, y: i32, z: i32) -> Option<StructureBox>
{
    let generation: GenWorld = GenWorld::new(world);
    let mut found: Option<(StructureBox, bool)> = None;
    for start in starts(&generation, Family::Major, world.seed(), x >> 4, z >> 4)
    {
        if !start.m_box.intersectsXZ(x, z, x, z)
        {
            continue;
        }

        for piece in &start.m_pieces
        {
            let bounds: StructureBox = piece.m_box.get();
            if bounds.contains(x, y, z)
            {
                found = Some((bounds, piece.isComponentProtected()));
            }
        }
    }
    found.filter(|(_, protected)| *protected).map(|(bounds, _)| bounds)
}

// Facing.offsetsForSide, the step from a block toward the face clicked
pub fn facing(face: i32) -> [i32; 3] { component::FACING.get(face as usize).copied().unwrap_or([0, 1, 0]) }

// MapGenTFMajorFeature.getSpawnListIndexAt, the feature whose footprint holds the spot and the highest spawn list of
// its pieces there, -1 outside them
pub fn spawnListAt(world: World, x: i32, y: i32, z: i32) -> Option<(Feature, i32)>
{
    let generation: GenWorld = GenWorld::new(world);
    let mut found: Option<(Feature, i32)> = None;
    for start in starts(&generation, Family::Major, world.seed(), x >> 4, z >> 4)
    {
        if !start.m_box.intersectsXZ(x, z, x, z)
        {
            continue;
        }

        found = found.or(Some((start.m_feature, -1)));
        for piece in start.m_pieces.iter().filter(|piece| piece.m_box.get().contains(x, y, z))
        {
            let index: i32 = piece.m_spawnListIndex.get();
            if found.is_none_or(|(_, highest)| index > highest)
            {
                found = Some((start.m_feature, index));
            }
        }
    }
    found
}
