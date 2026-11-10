use b173_api::prelude::*;

// TwilightForestMod.loadConfiguration, the options this port has features for
const ENFORCED: &str = "general.enforced_progression";
const PORTAL_CREATION: &str = "general.disable_portal_creation";
const ADMIN_PORTALS: &str = "general.admin_only_portals";
const OTHER_DIMENSIONS: &str = "general.portals_in_other_dimensions";
const SILENT_CICADAS: &str = "general.silent_cicadas";
const UNCRAFTING: &str = "general.disable_uncrafting";
const PORTAL_ITEM: &str = "general.portal_creation_item";
const PORTAL_ITEM_DEFAULT: &str = "minecraft:diamond";
const CANOPY: &str = "performance.canopy_coverage";
const OAK_CHANCE: &str = "performance.twilight_oak_chance";
const OAK_CHANCE_DEFAULT: i64 = 48;
const OAK_CHANCE_MOST: i64 = 1000;
const CANOPY_DEFAULT: f64 = 1.7;
const CANOPY_MOST: f64 = 10.0;

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.config().options(vec![
        Setting::bool(ENFORCED, true).label("Enforced Progression").comment(
            "The tfEnforcedProgression game rule, biomes push back on players who have not beaten the boss before them",
        ),
        Setting::bool(PORTAL_CREATION, false)
            .label("Disable Portal Creation")
            .comment("Disable Twilight Forest portal creation entirely"),
        Setting::bool(ADMIN_PORTALS, false).label("Admin Only Portals").comment("Allow portals only for admins (ops)"),
        Setting::bool(OTHER_DIMENSIONS, false)
            .label("Portals In Other Dimensions")
            .comment("Allow portals to the Twilight Forest to be made outside of the overworld"),
        Setting::bool(SILENT_CICADAS, false).label("Silent Cicadas").comment("Make cicadas silent"),
        Setting::bool(UNCRAFTING, false).label("Disable Uncrafting").comment(
            "Disable the uncrafting function of the uncrafting table, for when other mods' recipes make it exploitable",
        ),
        Setting::text(PORTAL_ITEM, PORTAL_ITEM_DEFAULT)
            .label("Portal Creation Item")
            .comment("Item to create the Twilight Forest portal, defaults to a diamond"),
        Setting::float(CANOPY, CANOPY_DEFAULT, 0.0..=CANOPY_MOST).label("Canopy Coverage").comment(
            "Amount of canopy coverage, lower numbers improve chunk generation speed at the cost of a thinner forest",
        ),
        Setting::int(OAK_CHANCE, OAK_CHANCE_DEFAULT, 1..=OAK_CHANCE_MOST).label("Twilight Oak Chance").comment(
            "Chance that a chunk in the Twilight Forest will contain a twilight oak tree, higher numbers mean fewer trees",
        ),
    ])
}

pub fn enforced() -> bool { Config.getBool(ENFORCED) }

pub fn portals() -> bool { !Config.getBool(PORTAL_CREATION) }

pub fn adminPortals() -> bool { Config.getBool(ADMIN_PORTALS) }

pub fn otherDimensions() -> bool { Config.getBool(OTHER_DIMENSIONS) }

pub fn silentCicadas() -> bool { Config.getBool(SILENT_CICADAS) }

pub fn canopy() -> f32 { Config.getFloat(CANOPY) as f32 }

pub fn uncraftingDisabled() -> bool { Config.getBool(UNCRAFTING) }

// the item a tossed stack must be to light a portal, PortalCreationItem with a bare name meaning minecraft's
pub fn portalItem() -> String
{
    let name: String = Config.get(PORTAL_ITEM);
    let name: &str = name.trim();
    match name
    {
        "" => PORTAL_ITEM_DEFAULT.to_owned(),
        name if name.contains(':') => name.to_owned(),
        name => format!("minecraft:{name}"),
    }
}

pub fn oakChance() -> i32 { Config.getInt(OAK_CHANCE).clamp(1, OAK_CHANCE_MOST) as i32 }
