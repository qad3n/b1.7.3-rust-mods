use crate::entities::floating_block::Floating;
use crate::harvest;
use crate::ids::{Palette, block, item};
use crate::world::GenWorld;
use crate::world::trees::Tree;
use b173_api::prelude::*;
use b173_api::terrain::Random;

enum Look
{
    All(&'static str),
    Column
    {
        top: &'static str,
        side: &'static str,
    },
    Grass
    {
        top: &'static str,
        side: &'static str,
        bottom: &'static str,
    },
}

// a plain cube built on the "block" template, values from the Aether block constructors and BlocksAether
struct Cube
{
    m_name: &'static str,
    m_material: &'static str,
    m_look: Look,
    m_hardness: f32,
    m_resistance: Option<f32>,
    m_light: f32,
    m_sound: &'static str,
    m_tool: (ToolClass, i32),
    // an item dropped in place of the block itself
    m_drop: Option<&'static str>,
}

// BlockDungeonBase lit variants answer light 11 of 15
const DUNGEON_LIGHT: f32 = 11.0 / 15.0;
const ALWAYS: f32 = 1.0;
// BlockLeaves drops its sapling one time in twenty
const SAPLING_CHANCE: f32 = 0.05;
// aether leaves keep their own colors instead of the foliage tint
const UNTINTED: u32 = 0xFFFFFF;
// quicksoil and its glass speed walkers up, BlockQuicksoil.slipperiness
const QUICKSOIL_SLIPPERINESS: f32 = 1.1;
const QUICKSOIL_GLASS_LIGHT: f32 = 0.7375;
const AMBROSIUM_TORCH_LIGHT: f32 = 0.9375;
const AEROGEL_RESISTANCE: f32 = 2000.0;
const AEROGEL_OPACITY: i32 = 3;
const BOOKSHELF_BOOKS: i32 = 3;
const BERRIES: [i32; 2] = [1, 3];
const SKYROOT_HARDNESS: f32 = 2.0;
const SKYROOT_RESISTANCE: f32 = 5.0;
const QUICKSOIL_HARDNESS: f32 = 0.5;
const AEROGEL_HARDNESS: f32 = 1.0;

const CUBES: [Cube; 22] = [
    Cube {
        m_name: block::HOLYSTONE,
        m_material: "rock",
        m_look: Look::All("holystone"),
        m_hardness: 0.5,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: None,
    },
    Cube {
        m_name: "mossy_holystone",
        m_material: "rock",
        m_look: Look::All("mossy_holystone"),
        m_hardness: 0.5,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: None,
    },
    Cube {
        m_name: "holystone_brick",
        m_material: "rock",
        m_look: Look::All("holystone_brick"),
        m_hardness: 0.5,
        m_resistance: Some(10.0),
        m_light: 0.0,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: None,
    },
    Cube {
        m_name: block::ICESTONE,
        m_material: "rock",
        m_look: Look::All("icestone"),
        m_hardness: 3.0,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "glass",
        m_tool: (ToolClass::Pickaxe, 1),
        m_drop: None,
    },
    Cube {
        m_name: block::GRAVITITE_ORE,
        m_material: "rock",
        m_look: Look::All("gravitite_ore"),
        m_hardness: 5.0,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 2),
        m_drop: None,
    },
    Cube {
        m_name: block::ENCHANTED_GRAVITITE,
        m_material: "rock",
        m_look: Look::All("enchanted_gravitite"),
        m_hardness: 5.0,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 2),
        m_drop: None,
    },
    Cube {
        m_name: "zanite_block",
        m_material: "iron",
        m_look: Look::All("zanite_block"),
        m_hardness: 3.0,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "metal",
        m_tool: (ToolClass::Pickaxe, 1),
        m_drop: None,
    },
    Cube {
        m_name: "skyroot_planks",
        m_material: "wood",
        m_look: Look::All("skyroot_planks"),
        m_hardness: 2.0,
        m_resistance: Some(5.0),
        m_light: 0.0,
        m_sound: "wood",
        m_tool: (ToolClass::Axe, 0),
        m_drop: None,
    },
    Cube {
        m_name: block::AETHER_DIRT,
        m_material: "ground",
        m_look: Look::All("aether_dirt"),
        m_hardness: 0.2,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "gravel",
        m_tool: (ToolClass::Shovel, 0),
        m_drop: None,
    },
    Cube {
        m_name: block::ENCHANTED_AETHER_GRASS,
        m_material: "grass",
        m_look: Look::Grass {
            top: "enchanted_aether_grass_top",
            side: "enchanted_aether_grass_side",
            bottom: "aether_dirt",
        },
        m_hardness: 0.2,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "grass",
        m_tool: (ToolClass::Shovel, 0),
        m_drop: None,
    },
    Cube {
        m_name: "carved_stone",
        m_material: "rock",
        m_look: Look::All("carved_stone"),
        m_hardness: 0.5,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: None,
    },
    Cube {
        m_name: "sentry_stone",
        m_material: "rock",
        m_look: Look::All("sentry_stone"),
        m_hardness: 0.5,
        m_resistance: None,
        m_light: DUNGEON_LIGHT,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: None,
    },
    Cube {
        m_name: "angelic_stone",
        m_material: "rock",
        m_look: Look::All("angelic_stone"),
        m_hardness: 0.5,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: None,
    },
    Cube {
        m_name: "light_angelic_stone",
        m_material: "rock",
        m_look: Look::All("light_angelic_stone"),
        m_hardness: 0.5,
        m_resistance: None,
        m_light: DUNGEON_LIGHT,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: None,
    },
    Cube {
        m_name: "hellfire_stone",
        m_material: "rock",
        m_look: Look::All("hellfire_stone"),
        m_hardness: 0.5,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: None,
    },
    Cube {
        m_name: "light_hellfire_stone",
        m_material: "rock",
        m_look: Look::All("light_hellfire_stone"),
        m_hardness: 0.5,
        m_resistance: None,
        m_light: DUNGEON_LIGHT,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: None,
    },
    Cube {
        m_name: "pillar",
        m_material: "rock",
        m_look: Look::Column { top: "pillar_top", side: "pillar_side" },
        m_hardness: 0.5,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "metal",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: None,
    },
    Cube {
        m_name: "pillar_top",
        m_material: "rock",
        m_look: Look::Column { top: "pillar_top", side: "pillar_carved" },
        m_hardness: 0.5,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "metal",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: None,
    },
    Cube {
        m_name: block::AMBROSIUM_ORE,
        m_material: "rock",
        m_look: Look::All("ambrosium_ore"),
        m_hardness: 3.0,
        m_resistance: Some(5.0),
        m_light: 0.0,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 0),
        m_drop: Some(item::AMBROSIUM_SHARD),
    },
    Cube {
        m_name: block::ZANITE_ORE,
        m_material: "rock",
        m_look: Look::All("zanite_ore"),
        m_hardness: 3.0,
        m_resistance: Some(5.0),
        m_light: 0.0,
        m_sound: "stone",
        m_tool: (ToolClass::Pickaxe, 1),
        m_drop: Some(item::ZANITE_GEMSTONE),
    },
    Cube {
        m_name: block::SKYROOT_LOG,
        m_material: "wood",
        m_look: Look::Column { top: "skyroot_log_top", side: "skyroot_log_side" },
        m_hardness: 2.0,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "wood",
        m_tool: (ToolClass::Axe, 0),
        m_drop: None,
    },
    Cube {
        m_name: block::GOLDEN_OAK_LOG,
        m_material: "wood",
        m_look: Look::Column { top: "golden_oak_log_top", side: "golden_oak_log_side" },
        m_hardness: 2.0,
        m_resistance: None,
        m_light: 0.0,
        m_sound: "wood",
        m_tool: (ToolClass::Axe, 0),
        m_drop: None,
    },
];

const AETHER_GRASS: Cube = Cube {
    m_name: block::AETHER_GRASS,
    m_material: "grass",
    m_look: Look::Grass { top: "aether_grass_top", side: "aether_grass_side", bottom: "aether_dirt" },
    m_hardness: 0.2,
    m_resistance: None,
    m_light: 0.0,
    m_sound: "grass",
    m_tool: (ToolClass::Shovel, 0),
    m_drop: Some(block::AETHER_DIRT),
};

fn texture(name: &str) -> String { format!("aether:block/{name}") }

fn faces(look: &Look) -> FaceTextures
{
    match look
    {
        Look::All(name) => FaceTextures {
            top: Some(texture(name)),
            bottom: Some(texture(name)),
            side: Some(texture(name)),
            front: None,
        },
        Look::Column { top, side } =>
        {
            FaceTextures { top: Some(texture(top)), bottom: Some(texture(top)), side: Some(texture(side)), front: None }
        }
        Look::Grass { top, side, bottom } => FaceTextures {
            top: Some(texture(top)),
            bottom: Some(texture(bottom)),
            side: Some(texture(side)),
            front: None,
        },
    }
}

fn drop(name: &str, count: Vec<i32>, chance: f32) -> Result<DropSpec>
{
    let item: Identifier = Identifier::parseIn(name, "aether").map_err(|error| Error(error.to_string()))?;
    Ok(DropSpec { item, count, meta: 0, chance })
}

fn cube(row: &Cube) -> Result<BlockSpecBuilder>
{
    let mut spec: BlockSpecBuilder = BlockSpecBuilder::template("minecraft:block")
        .material(row.m_material)
        .faces(faces(&row.m_look))
        .hardness(row.m_hardness)
        .light(row.m_light)
        .sound(row.m_sound)
        .tool(row.m_tool.0, row.m_tool.1);
    if let Some(resistance) = row.m_resistance
    {
        spec = spec.resistance(resistance);
    }
    if let Some(item) = row.m_drop
    {
        spec = spec.drops(vec![drop(item, vec![1], ALWAYS)?]);
    }
    Ok(spec)
}

// BlockAetherGrass.updateTick, dies under cover and spreads onto lit untouched aether dirt
struct AetherGrass;

mod grass
{
    pub const DIES_BELOW_LIGHT: i32 = 4;
    pub const DIES_ABOVE_OPACITY: i32 = 2;
    pub const SPREADS_FROM_LIGHT: i32 = 9;
    pub const SPREAD_TRIES: i32 = 4;
    pub const SPREAD_HORIZONTAL: i32 = 3;
    pub const SPREAD_VERTICAL: i32 = 5;
    pub const SPREAD_DROP: i32 = 3;
}

fn opacityAbove(world: World, pos: BlockPos) -> i32 { World::blockInfo(world.block(pos.offset(0, 1, 0))).opacity }

#[block_behavior]
impl BlockBehavior for AetherGrass
{
    fn onPlacedBy(&self, ctx: &mut BlockCtx, entity: Entity) { harvest::placed(ctx, entity) }

    fn onHarvest(&self, ctx: &mut BlockCtx, player: Player) { harvest::doubled(ctx, player, "shovel") }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let dirt: i32 = crate::ids::blockId(block::AETHER_DIRT);
        if world.light(pos.offset(0, 1, 0)) < grass::DIES_BELOW_LIGHT
            && opacityAbove(world, pos) > grass::DIES_ABOVE_OPACITY
        {
            let _ = world.setBlock(pos, dirt, 0);
            return;
        }
        if world.light(pos.offset(0, 1, 0)) < grass::SPREADS_FROM_LIGHT
        {
            return;
        }
        for _ in 0..grass::SPREAD_TRIES
        {
            let target: BlockPos = pos.offset(
                world.random(grass::SPREAD_HORIZONTAL) - 1,
                world.random(grass::SPREAD_VERTICAL) - grass::SPREAD_DROP,
                world.random(grass::SPREAD_HORIZONTAL) - 1,
            );
            if world.block(target) == dirt
                && world.meta(target) == 0
                && world.light(target.offset(0, 1, 0)) >= grass::DIES_BELOW_LIGHT
                && opacityAbove(world, target) <= grass::DIES_ABOVE_OPACITY
            {
                let _ = world.setBlock(target, ctx.block(), 0);
            }
        }
    }
}

// BlockAercloud.onEntityCollidedWithBlock, cold and golden clouds slow a fall, blue clouds launch upward
struct Aercloud
{
    m_bounce: bool,
}

mod aercloud
{
    pub const DRAG: f64 = 0.005;
    pub const LAUNCH: f64 = 2.0;
    pub const SPLASHES: i32 = 50;
    pub const HARDNESS: f32 = 0.2;
    // cold and golden clouds hold an entity on a sliver at their base and let it sink through slowly
    pub const SLIVER: f32 = 0.16;
    pub const JITTER_STEPS: i32 = 1000;
}

#[block_behavior]
impl BlockBehavior for Aercloud
{
    fn onEntityCollided(&self, ctx: &mut BlockCtx, entity: Entity)
    {
        entity.setField(EntityField::FallDistance, 0.0);
        let Some(info) = entity.info()
        else
        {
            return;
        };
        let [motionX, motionY, motionZ] = info.motion;
        let slowed: [f64; 3] = [motionX, motionY * aercloud::DRAG, motionZ];
        if !self.m_bounce || info.player.is_some() && entity.flag(EntityField::Sneaking)
        {
            if motionY < 0.0
            {
                let _ = entity.setMotion(slowed);
            }
            return;
        }
        let _ = entity.setMotion([motionX, aercloud::LAUNCH, motionZ]);
        if isClient()
        {
            let world: World = ctx.world();
            let pos: BlockPos = ctx.pos();
            for _ in 0..aercloud::SPLASHES
            {
                let jitter = || world.random(aercloud::JITTER_STEPS) as f64 / aercloud::JITTER_STEPS as f64;
                world.particle(
                    "splash",
                    [pos.x as f64 + jitter(), pos.y as f64 + jitter(), pos.z as f64 + jitter()],
                    [0.0; 3],
                );
            }
        }
    }
}

// BlockAetherFlower.canBlockStay, plants root only in aether soil
struct Plant;

fn rooted(ctx: &BlockCtx) -> bool { Palette::load().isSoil(ctx.world().block(ctx.pos().offset(0, -1, 0))) }

#[block_behavior]
impl BlockBehavior for Plant
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { ctx.world().isAir(ctx.pos()) && rooted(ctx) }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { rooted(ctx) }
}

// BlockBerryBush.harvestBlock: picked berries leave the bare stem behind
struct BerryBush;

#[block_behavior]
impl BlockBehavior for BerryBush
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { ctx.world().isAir(ctx.pos()) && rooted(ctx) }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { rooted(ctx) }

    // BlockBerryBush.harvestBlock reads the spot it just cleared, so the enchanted grass bonus never applies
    fn onHarvest(&self, ctx: &mut BlockCtx, _player: Player)
    {
        if !ctx.isAuthority()
        {
            return;
        }
        let world: World = ctx.world();
        let _ = world.setBlock(ctx.pos(), crate::ids::blockId("berry_bush_stem"), 0);
        let _ = world.dropBlockItems(ctx.pos(), ctx.block(), i32::from(ctx.meta()));
    }
}

// BlockAetherSapling.updateTick, grows its tree with light above
struct Sapling
{
    m_tree: Tree,
}

mod sapling
{
    pub const LIGHT: i32 = 9;
    pub const CHANCE: i32 = 30;
}

#[block_behavior]
impl BlockBehavior for Sapling
{
    fn canPlaceAt(&self, ctx: &mut BlockCtx) -> bool { ctx.world().isAir(ctx.pos()) && rooted(ctx) }

    fn canStay(&self, ctx: &mut BlockCtx) -> bool { rooted(ctx) }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        if !ctx.isAuthority() || world.light(pos.offset(0, 1, 0)) < sapling::LIGHT || world.random(sapling::CHANCE) != 0
        {
            return;
        }
        let sapling: i32 = ctx.block();
        let _ = world.setBlockQuiet(pos, 0, 0);
        let random: Random = Random::withSeed(world.random(i32::MAX) as i64);
        if !self.m_tree.generate(&GenWorld::new(world), &random, pos.x, pos.y, pos.z)
        {
            let _ = world.setBlockQuiet(pos, sapling, 0);
        }
    }
}

fn plant(name: &str, template: &str) -> BlockSpecBuilder
{
    BlockSpecBuilder::template(template).texture(&texture(name)).hardness(0.0).sound("grass")
}

fn leaves(name: &'static str) -> BlockSpecBuilder
{
    BlockSpecBuilder::template("minecraft:leaves")
        .faces(faces(&Look::All(name)))
        .color(UNTINTED)
        .hardness(0.2)
        .sound("grass")
}

fn aercloud(name: &str, bounce: bool) -> BlockSpecBuilder
{
    let sliver: Vec<Cuboid> =
        vec![Cuboid { from: [0.0; 3], to: [16.0, aercloud::SLIVER, 16.0], texture: None, ..Cuboid::default() }];
    BlockSpecBuilder::template("minecraft:block")
        .material("ice")
        .texture(&texture(name))
        .hardness(aercloud::HARDNESS)
        .sound("cloth")
        .tool(ToolClass::Shovel, 0)
        .model(BlockModelSpec { translucent: true, ..BlockModelSpec::default() })
        .collision(if bounce { Vec::new() } else { sliver })
        .behavior(Aercloud { m_bounce: bounce })
}

fn registerSpecial(ctx: &mut RegisterCtx) -> Result<()>
{
    let blocks = ctx.blocks();
    blocks.register(block::AETHER_GRASS, cube(&AETHER_GRASS)?.tickRandomly().behavior(AetherGrass))?;
    blocks.register(
        block::QUICKSOIL,
        BlockSpecBuilder::template("minecraft:block")
            .material("sand")
            .texture(&texture("quicksoil"))
            .hardness(QUICKSOIL_HARDNESS)
            .sound("sand")
            .tool(ToolClass::Shovel, 0)
            .slipperiness(QUICKSOIL_SLIPPERINESS)
            .behavior(harvest::DoubleDrop { m_tool: "shovel" }),
    )?;
    blocks.register(
        "quicksoil_glass",
        BlockSpecBuilder::template("minecraft:glass")
            .texture(&texture("quicksoil_glass"))
            .hardness(0.2)
            .light(QUICKSOIL_GLASS_LIGHT)
            .opacity(0)
            .sound("glass")
            .slipperiness(QUICKSOIL_SLIPPERINESS)
            .model(BlockModelSpec { translucent: true, ..BlockModelSpec::default() }),
    )?;
    blocks.register(
        "aerogel",
        BlockSpecBuilder::template("minecraft:block")
            .material("rock")
            .texture(&texture("aerogel"))
            .hardness(AEROGEL_HARDNESS)
            .resistance(AEROGEL_RESISTANCE)
            .opacity(AEROGEL_OPACITY)
            .sound("metal")
            .tool(ToolClass::Pickaxe, 3)
            .model(BlockModelSpec { translucent: true, ..BlockModelSpec::default() }),
    )?;
    blocks.register(block::COLD_AERCLOUD, aercloud("aercloud", false))?;
    blocks.register(block::BLUE_AERCLOUD, aercloud("blue_aercloud", true))?;
    blocks.register(block::GOLDEN_AERCLOUD, aercloud("golden_aercloud", false))?;
    blocks.register(
        block::SKYROOT_LEAVES,
        leaves("skyroot_leaves").drops(vec![drop(block::SKYROOT_SAPLING, vec![1], SAPLING_CHANCE)?]),
    )?;
    blocks.register(
        block::GOLDEN_OAK_LEAVES,
        leaves("golden_oak_leaves")
            .drops(vec![drop(block::GOLDEN_OAK_SAPLING, vec![1], SAPLING_CHANCE)?])
            .behavior(crate::sparkle::GOLDEN_OAK),
    )?;
    blocks.register(
        block::CRYSTAL_LEAVES,
        leaves("crystal_leaves").drops(Vec::new()).behavior(crate::sparkle::CRYSTAL),
    )?;
    blocks.register(
        block::CRYSTAL_FRUIT_LEAVES,
        leaves("crystal_fruit_leaves")
            .drops(vec![drop(item::WHITE_APPLE, vec![1], ALWAYS)?])
            .behavior(crate::sparkle::CRYSTAL),
    )?;
    blocks.register(
        block::BERRY_BUSH,
        leaves("berry_bush")
            .leaves(false)
            .drops(vec![drop(item::BLUEBERRY, BERRIES.to_vec(), ALWAYS)?])
            .behavior(BerryBush),
    )?;
    blocks.register(
        "ambrosium_torch",
        BlockSpecBuilder::template("minecraft:torch")
            .texture(&texture("ambrosium_torch"))
            .light(AMBROSIUM_TORCH_LIGHT)
            .sound("wood"),
    )?;
    blocks.register(block::WHITE_FLOWER, plant("white_flower", "minecraft:red_flower").behavior(Plant))?;
    blocks.register(block::PURPLE_FLOWER, plant("purple_flower", "minecraft:red_flower").behavior(Plant))?;
    blocks.register(
        block::SKYROOT_SAPLING,
        plant("skyroot_sapling", "minecraft:red_flower").behavior(Sapling { m_tree: Tree::Skyroot }),
    )?;
    blocks.register(
        block::GOLDEN_OAK_SAPLING,
        plant("golden_oak_sapling", "minecraft:red_flower").behavior(Sapling { m_tree: Tree::GoldenOak }),
    )?;
    blocks.register(
        "skyroot_bookshelf",
        BlockSpecBuilder::template("minecraft:bookshelf")
            .faces(FaceTextures {
                top: Some(texture("skyroot_planks")),
                bottom: Some(texture("skyroot_planks")),
                side: Some(texture("skyroot_bookshelf")),
                front: None,
            })
            .hardness(SKYROOT_HARDNESS)
            .resistance(SKYROOT_RESISTANCE)
            .sound("wood")
            .tool(ToolClass::Axe, 0)
            .drops(vec![drop("minecraft:book", vec![BOOKSHELF_BOOKS], ALWAYS)?]),
    )?;
    blocks.register(
        "skyroot_fence",
        BlockSpecBuilder::template("minecraft:fence")
            .texture(&texture("skyroot_planks"))
            .hardness(SKYROOT_HARDNESS)
            .resistance(SKYROOT_RESISTANCE)
            .sound("wood")
            .tool(ToolClass::Axe, 0),
    )?;
    Ok(())
}

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &CUBES
    {
        let mut spec: BlockSpecBuilder = cube(row)?;
        spec = match row.m_name
        {
            block::SKYROOT_LOG | block::GOLDEN_OAK_LOG => spec.sustainsLeaves().behavior(harvest::Log),
            block::GRAVITITE_ORE | block::ENCHANTED_GRAVITITE =>
            {
                spec.behavior(Floating { m_leveled: row.m_name == block::ENCHANTED_GRAVITITE })
            }
            block::HOLYSTONE => spec.behavior(harvest::DoubleDrop { m_tool: "pickaxe" }),
            block::AETHER_DIRT => spec.behavior(harvest::DoubleDrop { m_tool: "shovel" }),
            block::AMBROSIUM_ORE | block::ZANITE_ORE => spec.behavior(harvest::Ore),
            _ => spec,
        };
        ctx.blocks().register(row.m_name, spec)?;
    }
    registerSpecial(ctx)?;
    harvest::registerStates(
        ctx,
        &[
            block::AETHER_GRASS,
            block::AETHER_DIRT,
            block::HOLYSTONE,
            block::QUICKSOIL,
            block::SKYROOT_LOG,
            block::GOLDEN_OAK_LOG,
        ],
    )?;
    crate::portal::registerBlock(ctx)
}
