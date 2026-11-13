use b173_api::prelude::*;

pub(crate) const VALKYRIE_CAPE: &str = "loot.valkyrie_cape";
pub(crate) const GOLDEN_FEATHER: &str = "loot.golden_feather";
pub(crate) const SUN_ALTAR_MULTIPLAYER: &str = "gameplay.sun_altar_multiplayer";
pub(crate) const DISABLE_ETERNAL_DAY: &str = "gameplay.disable_eternal_day";
pub(crate) const SKYROOT_BUCKET_ONLY: &str = "misc.skyroot_bucket_only";
pub(crate) const PORTAL_FRAME_START: &str = "gameplay.portal_frame_start";
pub(crate) const LEGACY_ALTAR_NAME: &str = "misc.legacy_altar_name";
pub(crate) const REPEAT_SPIRIT_DIALOGUE: &str = "misc.repeat_sun_spirit_dialogue";
pub(crate) const GIVE_LORE_BOOK: &str = "misc.give_lore_book";
pub(crate) const GIVE_GOLDEN_PARACHUTE: &str = "misc.give_golden_parachute";
pub(crate) const CHRISTMAS_CONTENT: &str = "world.christmas_content";
pub(crate) const TALL_GRASS: &str = "world.tall_grass";
const MAX_LIFE_SHARDS: &str = "gameplay.max_life_shards";
const LIFE_SHARD_LIMIT: i64 = 100;
const SPAWN_RATE_CATEGORY: &str = "spawn_rates";
const SPAWN_RATE_COMMENT: &str = "1 is always, higher numbers decrease chances";
const MAX_SPAWN_RATE: i64 = 1000;

struct SpawnRate
{
    m_mob: &'static str,
    m_default: i64,
}

// AetherConfig spawnrates in their Java defaults
const SPAWN_RATES: [SpawnRate; 11] = [
    SpawnRate { m_mob: "phyg", m_default: 1 },
    SpawnRate { m_mob: "flying_cow", m_default: 1 },
    SpawnRate { m_mob: "sheepuff", m_default: 1 },
    SpawnRate { m_mob: "aerbunny", m_default: 1 },
    SpawnRate { m_mob: "moa", m_default: 1 },
    SpawnRate { m_mob: "aerwhale", m_default: 1 },
    SpawnRate { m_mob: "zephyr", m_default: 65 },
    SpawnRate { m_mob: "cockatrice", m_default: 45 },
    SpawnRate { m_mob: "swet", m_default: 40 },
    SpawnRate { m_mob: "aechor_plant", m_default: 55 },
    SpawnRate { m_mob: "whirlwind", m_default: 55 },
];

fn spawnKey(mob: &str) -> String { format!("{SPAWN_RATE_CATEGORY}.{mob}") }

// AetherConfig.init, the options this port has features for
pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let mut settings: Vec<Setting> = vec![
        Setting::bool(VALKYRIE_CAPE, true).label("Valkyrie Cape").comment("Enables the Valkyrie Cape in dungeon loot"),
        Setting::bool(GOLDEN_FEATHER, false)
            .label("Golden Feather")
            .comment("Enables the Golden Feather in dungeon loot"),
        Setting::bool(SUN_ALTAR_MULTIPLAYER, false)
            .label("Sun Altar Multiplayer")
            .comment("Removes the requirement for a player to be an operator to use the Sun Altar in multiplayer."),
        Setting::int(MAX_LIFE_SHARDS, 10, 0..=LIFE_SHARD_LIMIT).label("Max Life Shards"),
        Setting::bool(DISABLE_ETERNAL_DAY, false)
            .label("Disable Eternal Day")
            .comment("Disables eternal day making time cycle in the Aether without having to kill the Sun Spirit."),
        Setting::bool(SKYROOT_BUCKET_ONLY, false)
            .label("Skyroot Bucket Portals")
            .comment("Activate portal with only Skyroot bucket"),
        Setting::bool(PORTAL_FRAME_START, false)
            .label("Starting Portal Frame")
            .comment("Spawns Player with Aether Portal Frame"),
        Setting::bool(LEGACY_ALTAR_NAME, false)
            .label("Legacy Altar Name")
            .comment("Changes whether the Altar should be named Enchanter or not."),
        Setting::bool(REPEAT_SPIRIT_DIALOGUE, true)
            .label("Repeat Sun Spirit Dialogue")
            .comment("If disabed, the Sun Spirit's dialog will only show once per world."),
        Setting::bool(GIVE_LORE_BOOK, true)
            .label("First Entry Book of Lore")
            .comment("Gives the Player a Book of Lore upon entering the Aether for the first time"),
        Setting::bool(GIVE_GOLDEN_PARACHUTE, true)
            .label("First Entry Golden Parachute")
            .comment("Gives the Player a Golden Parachute upon entering the Aether for the first time"),
        Setting::bool(CHRISTMAS_CONTENT, false)
            .label("Christmas Content")
            .comment("Holiday trees with presents under them generate in new Aether chunks"),
        Setting::bool(TALL_GRASS, false)
            .label("Enable Tall Grass")
            .comment("Tall grass generates in new Aether chunks"),
    ];
    settings.extend(
        SPAWN_RATES.iter().map(|row|
        {
            Setting::int(&spawnKey(row.m_mob), row.m_default, 1..=MAX_SPAWN_RATE).comment(SPAWN_RATE_COMMENT)
        }),
    );
    ctx.config().options(settings)
}

pub(crate) fn enabled(key: &str) -> bool { Config.getBool(key) }

pub(crate) fn maxLifeShards() -> i32 { Config.getInt(MAX_LIFE_SHARDS).clamp(0, LIFE_SHARD_LIMIT) as i32 }

// EntityPhyg.getCanSpawnHere and the other mobs: one try in the configured rate goes on to the usual checks
pub(crate) fn spawnRoll(entity: Entity, mob: &str) -> bool
{
    let rate: i32 = Config.getInt(&spawnKey(mob)).clamp(1, MAX_SPAWN_RATE) as i32;
    entity.world.random(rate) == 0
}
