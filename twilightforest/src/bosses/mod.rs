mod hydra;
mod knights;
mod lich;
mod minoshroom;
mod naga;
mod snow_queen;
mod trophy;
pub(crate) mod ur_ghast;
pub(crate) mod yeti;

use b173_api::math::{distanceSquared, floor};
use b173_api::prelude::*;
use b173_api::screen::Draw;

// BlockTFBossSpawner and the TileEntityTF*Spawner family: an invisible spawner that calls up its boss once a player
// comes near enough
pub(crate) const SPAWNER: &str = "boss_spawner";

struct Boss
{
    m_entity: &'static str,
    m_range: f64,
    // the player must stand above the spawner minus this many blocks, None for any height
    m_below: Option<f64>,
}

// in BlockTFBossSpawner metadata order
const BOSSES: [Boss; 6] = [
    Boss { m_entity: "twilightforest:naga", m_range: 50.0, m_below: None },
    Boss { m_entity: "twilightforest:twilight_lich", m_range: 9.0, m_below: Some(4.0) },
    Boss { m_entity: "twilightforest:hydra", m_range: 50.0, m_below: None },
    Boss { m_entity: "twilightforest:ur_ghast", m_range: 9.0, m_below: Some(4.0) },
    Boss { m_entity: "twilightforest:knight_phantom", m_range: 9.0, m_below: Some(2.0) },
    Boss { m_entity: "twilightforest:snow_queen", m_range: 9.0, m_below: Some(4.0) },
];

const KINDS: [&str; 6] = ["naga", "lich", "hydra", "ur_ghast", "knights", "snow_queen"];

pub(crate) mod kind
{
    pub(crate) const KNIGHTS: i32 = 4;
}

fn ready(world: World, pos: BlockPos, boss: &Boss) -> bool
{
    let center: [f64; 3] = crate::math::center(pos);

    world.players().into_iter().filter_map(Player::info).any(|player|
    {
        let near: bool = distanceSquared(player.pos, center) < boss.m_range * boss.m_range;
        near && boss.m_below.is_none_or(|below| player.pos[1] > f64::from(pos.y) - below)
    })
}

struct SpawnerTile;

#[tile_behavior]
impl TileBehavior for SpawnerTile
{
    // TileEntityTFBossSpawner.updateEntity
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        let kind: Option<usize> = crate::names::pick(ctx.stateAt(), "boss", &KINDS);
        let Some(boss) = kind.and_then(|kind| BOSSES.get(kind))
        else
        {
            return;
        };
        let (world, pos): (World, BlockPos) = (ctx.world(), ctx.pos());
        if !ready(world, pos, boss)
        {
            return;
        }

        if !ctx.isAuthority()
        {
            let at: [f64; 3] =
                [pos.x, pos.y, pos.z].map(|axis| f64::from(axis) + f64::from(crate::math::fraction(world)));
            world.particle("smoke", at, [0.0; 3]);
            world.particle("flame", at, [0.0; 3]);
            return;
        }

        if kind == Some(kind::KNIGHTS as usize)
        {
            knights::spawnGroup(world, pos);

            let _ = world.clear(pos);

            return;
        }

        let yaw: f32 = crate::math::fraction(world) * 360.0;
        if let Some(spawned) = crate::entities::spawnAt(world, boss.m_entity, crate::math::center(pos), yaw)
        {
            let _ = spawned.setData(&Home { m_home: Some([pos.x, pos.y, pos.z]) });
        }

        let _ = world.clear(pos);
    }
}

// EntityCreature.setHomeArea, the spawner a boss came from and stays near
#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub(crate) struct Home
{
    pub m_home: Option<[i32; 3]>,
}

// BossStatus, what the boss bar shows, synced from the server
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub(crate) struct BossBar
{
    pub m_name: String,
    pub m_health: i32,
    pub m_maxHealth: i32,
}

mod bar
{
    // GuiIngame.renderBossHealth, beta has no boss bar sheet so the bar is drawn in plain colors
    pub const WIDTH: i32 = 182;
    pub const HEIGHT: i32 = 5;
    pub const TOP: i32 = 12;
    pub const TITLE_Y: i32 = 2;
    pub const BACK: u32 = 0xFF3C1E3C;
    pub const FILL: u32 = 0xFFE04CC4;
    pub const TITLE: u32 = 0xFFFFFF;
    pub const SEARCH: f64 = 64.0;
}

pub(crate) fn showBar(entity: Entity, name: &str, info: &EntityInfo)
{
    let shown: BossBar = BossBar
    {
        m_name: name.to_owned(),
        m_health: info.health.unwrap_or(0),
        m_maxHealth: entity.field(EntityField::MaxHealth) as i32,
    };
    if entity.synced::<BossBar>() != shown
    {
        let _ = entity.setSynced(&shown);
    }
}

fn drawBar(draw: &mut Draw, width: i32)
{
    let Some(local) = localPlayer().and_then(Player::info)
    else
    {
        return;
    };

    let world: World = World { dimension: local.world };
    let found: Option<BossBar> = world.entitiesNear(local.pos, bar::SEARCH).into_iter().find_map(|other|
    {
        other.kind.as_ref().filter(|kind| kind.namespace() == "twilightforest")?;

        let shown: BossBar = Entity { world, id: other.id }.synced();
        (shown.m_maxHealth > 0 && shown.m_health > 0).then_some(shown)
    });
    let Some(shown) = found
    else
    {
        return;
    };

    let left: i32 = width / 2 - bar::WIDTH / 2;

    draw.centered(width / 2, bar::TITLE_Y, &shown.m_name, bar::TITLE);
    draw.rect(left, bar::TOP, bar::WIDTH, bar::HEIGHT, bar::BACK);
    draw.rect(left, bar::TOP, shown.m_health * bar::WIDTH / shown.m_maxHealth.max(1), bar::HEIGHT, bar::FILL);
}

// drops a stack where a boss died, EntityLiving.entityDropItem
pub(crate) fn dropAt(world: World, at: [f64; 3], item: &str, count: i32)
{
    if let Err(error) = world.spawnItem(at, ItemStack::of(item, count))
    {
        warn(&format!("boss drop {item}: {}", error.0));
    }
}

// ChunkProviderTwilightForest.setStructureConquered, kept as the feature centers whose boss is beaten
#[derive(Serialize, Deserialize, Default)]
struct Conquered
{
    m_centers: Vec<[i32; 2]>,
}

const CONQUERED: &str = "conquered";

fn featureCenter(world: World, at: [f64; 3]) -> [i32; 2]
{
    let (x, z): (i32, i32) = (floor(at[0]), floor(at[2]));
    let (_, [dx, dz]) = crate::world::features::nearest(world.seed(), x >> 4, z >> 4);
    [(x >> 4 << 4) + dx, (z >> 4 << 4) + dz]
}

pub(crate) fn conquer(world: World, at: [f64; 3])
{
    let center: [i32; 2] = featureCenter(world, at);
    let mut conquered: Conquered = worldData(CONQUERED);
    if !conquered.m_centers.contains(&center)
    {
        conquered.m_centers.push(center);
        setWorldData(CONQUERED, &conquered);
    }
}

pub(crate) fn conquered(world: World, at: [f64; 3]) -> bool
{
    worldData::<Conquered>(CONQUERED).m_centers.contains(&featureCenter(world, at))
}

pub(crate) fn home(entity: Entity) -> Option<[i32; 3]> { entity.data::<Home>().m_home }

// the killer when it is a player, for the achievements a boss grants
pub(crate) fn killer(world: World, source: Option<i32>) -> Option<Player>
{
    let id: i32 = source?;
    Entity { world, id }.info().filter(|info| info.player.is_some()).map(|_| Player { id })
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    naga::register(ctx)?;
    lich::register(ctx)?;
    minoshroom::register(ctx)?;
    yeti::register(ctx)?;
    hydra::register(ctx)?;
    knights::register(ctx)?;
    snow_queen::register(ctx)?;
    ur_ghast::register(ctx)?;
    trophy::register(ctx)?;
    ctx.screens().hud(|draw, _partial, width, _height| drawBar(draw, width))?;

    let spec: TileEntitySpec = TileEntitySpec { inventory: 0, ticks: true, sync: false, behavior: None };
    ctx.tileEntities().register(SPAWNER, spec, Some(std::rc::Rc::new(SpawnerTile)))?;

    let unbreakable: f32 = -1.0;
    let resistance: f32 = 6000000.0;
    ctx.blocks().register(
        SPAWNER,
        BlockSpecBuilder::template("minecraft:block")
            .name("Boss Spawner")
            .material("rock")
            .hardness(unbreakable)
            .resistance(resistance)
            .texture(&crate::blocks::vanillaTexture("spawner"))
            .drops(Vec::new())
            .tileEntity(SPAWNER),
    )?;
    ctx.blocks().states(
        SPAWNER,
        BlockStatesSpec { properties: vec![PropertySpec::new("boss", &KINDS)], ..BlockStatesSpec::default() },
    )?;
    Ok(())
}
