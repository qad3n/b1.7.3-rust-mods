use crate::effects::Effect;
use b173_api::math::floor;
use b173_api::prelude::*;
use b173_api::{StackData, WornStack};
use std::cell::Cell;

// AccessoryType, the slot an accessory goes in
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Slot
{
    Gloves,
    Ring,
    Pendant,
    Cape,
    Misc,
    Shield,
}

// one ItemAccessory: its slot, icon, tint and uses when it wears out
pub(crate) struct Accessory
{
    pub m_name: &'static str,
    pub m_slot: Slot,
    m_icon: &'static str,
    pub m_color: u32,
    m_uses: Option<i32>,
}

// ItemAccessory.colorHex default
pub(crate) const PLAIN: u32 = 14540253;

const fn accessory(name: &'static str, slot: Slot, icon: &'static str, color: u32, uses: Option<i32>) -> Accessory
{
    Accessory { m_name: name, m_slot: slot, m_icon: icon, m_color: color, m_uses: uses }
}

// ItemsAether accessories in registration order
pub(crate) const ACCESSORIES: [Accessory; 31] = [
    accessory("leather_gloves", Slot::Gloves, "leather_gloves", PLAIN, None),
    accessory("iron_gloves", Slot::Gloves, "solid_gloves", PLAIN, None),
    accessory("golden_gloves", Slot::Gloves, "solid_gloves", 16512036, None),
    accessory("chain_gloves", Slot::Gloves, "chain_gloves", PLAIN, None),
    accessory("diamond_gloves", Slot::Gloves, "solid_gloves", 3402699, None),
    accessory("zanite_gloves", Slot::Gloves, "solid_gloves", 7412456, None),
    accessory("gravitite_gloves", Slot::Gloves, "solid_gloves", 15160027, None),
    accessory("neptune_gloves", Slot::Gloves, "chain_gloves", 2512127, None),
    accessory("phoenix_gloves", Slot::Gloves, "phoenix_gloves", PLAIN, Some(152)),
    accessory("obsidian_gloves", Slot::Gloves, "solid_gloves", 1774663, None),
    accessory("valkyrie_gloves", Slot::Gloves, "valkyrie_gloves", PLAIN, None),
    accessory("iron_ring", Slot::Ring, "ring_base", PLAIN, None),
    accessory("golden_ring", Slot::Ring, "ring_base", 15396439, None),
    accessory("zanite_ring", Slot::Ring, "ring_base", 7412456, Some(49)),
    accessory("ice_ring", Slot::Ring, "ring_base", 9823975, Some(125)),
    accessory("iron_pendant", Slot::Pendant, "pendant_base", PLAIN, None),
    accessory("golden_pendant", Slot::Pendant, "pendant_base", 15396439, None),
    accessory("zanite_pendant", Slot::Pendant, "pendant_base", 7412456, Some(98)),
    accessory("ice_pendant", Slot::Pendant, "pendant_base", 9823975, Some(250)),
    accessory("red_cape", Slot::Cape, "cape_color_base", 15208721, None),
    accessory("blue_cape", Slot::Cape, "cape_color_base", 1277879, None),
    accessory("yellow_cape", Slot::Cape, "cape_color_base", 13486862, None),
    accessory("white_cape", Slot::Cape, "cape_color_base", PLAIN, None),
    accessory("swet_cape", Slot::Cape, "swet_cape", PLAIN, None),
    accessory("invisibility_cape", Slot::Cape, "invisibility_cape", PLAIN, None),
    accessory("agility_cape", Slot::Cape, "agility_cape", PLAIN, None),
    accessory("valkyrie_cape", Slot::Cape, "valkyrie_cape", PLAIN, None),
    accessory("golden_feather", Slot::Misc, "golden_feather", PLAIN, None),
    accessory("regeneration_stone", Slot::Misc, "regeneration_stone", PLAIN, None),
    accessory("iron_bubble", Slot::Misc, "iron_bubble", PLAIN, None),
    accessory("repulsion_shield", Slot::Shield, "repulsion_shield", PLAIN, Some(512)),
];

// the common item tag of each accessory kind, so other mods' rings and capes fit these slots and Aether's fit theirs
const KINDS: [(Slot, &str); 6] = [
    (Slot::Gloves, "c:accessories/gloves"),
    (Slot::Ring, "c:accessories/ring"),
    (Slot::Pendant, "c:accessories/pendant"),
    (Slot::Cape, "c:accessories/cape"),
    (Slot::Misc, "c:accessories/misc"),
    (Slot::Shield, "c:accessories/shield"),
];

// one accessory slot of the loader registry: its name, kind and empty slot icon
struct WornSlot
{
    m_name: &'static str,
    m_slot: Slot,
    m_icon: &'static str,
}

// AccessoryType.createCompleteList, the eight worn slots in order
const WORN: [WornSlot; 8] = [
    WornSlot { m_name: "pendant", m_slot: Slot::Pendant, m_icon: "pendant" },
    WornSlot { m_name: "cape", m_slot: Slot::Cape, m_icon: "cape" },
    WornSlot { m_name: "shield", m_slot: Slot::Shield, m_icon: "shield" },
    WornSlot { m_name: "misc", m_slot: Slot::Misc, m_icon: "misc" },
    WornSlot { m_name: "ring", m_slot: Slot::Ring, m_icon: "ring" },
    WornSlot { m_name: "second_ring", m_slot: Slot::Ring, m_icon: "ring" },
    WornSlot { m_name: "gloves", m_slot: Slot::Gloves, m_icon: "gloves" },
    WornSlot { m_name: "second_misc", m_slot: Slot::Misc, m_icon: "misc" },
];

fn kindTag(slot: Slot) -> &'static str
{
    KINDS.iter().find(|(kind, _)| *kind == slot).map_or(KINDS[0].1, |(_, tag)| tag)
}
pub(crate) const SLOTS: usize = 8;
const CLICK: &str = "accessory_click";

// what a player wears, held in the loader's accessory slots, with the life shards it used and the valkyrie set flag
// kept in the player's synced data so every client can draw them
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
pub(crate) struct Worn
{
    // the slots as worlds saved them before the loader kept accessories, moved into its slots on first read
    pub m_slots: [Option<StackData>; SLOTS],
    pub m_lifeShards: i32,
    // InventoryAccessories.isWearingValkyrieSet, worked out where the armor is known so every client can draw the wings
    pub m_valkyrieSet: bool,
    // the loader's slots as read, so a save writes only the ones that changed
    #[serde(skip)]
    m_held: [Option<StackData>; SLOTS],
}

impl Worn
{
    pub(crate) fn of(player: Player) -> Self
    {
        player.info().map_or_else(Self::default, |info|
        {
            Self::ofEntity(Entity { world: World { dimension: info.world }, id: player.id })
        })
    }

    pub(crate) fn ofEntity(entity: Entity) -> Self
    {
        let mut worn: Worn = entity.synced();
        let held: Vec<WornStack> = entity.accessories();
        for (index, slot) in WORN.iter().enumerate()
        {
            let stack: Option<StackData> = held
                .iter()
                .find(|worn| worn.slot.path() == slot.m_name && worn.slot.namespace() == "aether")
                .and_then(|worn| worn.stack.as_ref())
                .and_then(|stack| stack.toRaw().ok());
            worn.m_held[index] = stack;
        }

        let legacy: bool = worn.m_slots.iter().any(Option::is_some);
        if !legacy
        {
            worn.m_slots = worn.m_held.clone();
            return worn;
        }

        if entity.world.isAuthority()
        {
            worn.store(entity);
        }

        worn
    }

    // the slots into the loader, the rest into this mod's synced data without the old slot copy
    fn store(&mut self, entity: Entity)
    {
        for (index, slot) in WORN.iter().enumerate()
        {
            if self.m_slots[index] == self.m_held[index]
            {
                continue;
            }

            let stack: Option<ItemStack> = self.m_slots[index].clone().map(ItemStack::fromRaw);
            if let Err(error) = entity.setAccessory(slot.m_name, stack)
            {
                warn(&format!("accessory slot {} not saved: {}", slot.m_name, error.0));
                continue;
            }

            self.m_held[index] = self.m_slots[index].clone();
        }

        let synced: Worn = Worn { m_slots: Default::default(), ..self.clone() };
        let _ = entity.setSynced(&synced);
    }

    fn save(&mut self, player: Player)
    {
        if let Some(info) = player.info()
        {
            self.store(Entity { world: World { dimension: info.world }, id: player.id });
        }
    }

    pub(crate) fn stack(&self, index: usize) -> Option<ItemStack>
    {
        self.m_slots[index].clone().map(ItemStack::fromRaw)
    }

    fn shardHealth(&self) -> f64 { f64::from(self.m_lifeShards * crate::player::SHARD_HEALTH) }

    // InventoryAccessories.wearingAccessory
    pub(crate) fn wearing(&self, name: &str) -> bool
    {
        (0..SLOTS).any(|index| self.stack(index).is_some_and(|stack| stack.item.is(name)))
    }

    // InventoryAccessories.damageWornStack: each worn copy loses a use and breaks at the end
    pub(crate) fn wear(&mut self, name: &str, amount: i32)
    {
        for index in 0..SLOTS
        {
            let Some(stack) = self.stack(index).filter(|stack| stack.item.is(name))
            else
            {
                continue;
            };

            let limit: i32 =
                ACCESSORIES.iter().find(|row| row.m_name == name).and_then(|row| row.m_uses).unwrap_or(i32::MAX);
            let damage: i32 = stack.damage + amount;
            self.m_slots[index] = if damage > limit
            {
                None
            }
            else
            {
                stack.withDamage(damage).toRaw().ok()
            };
        }
    }
}

fn accessoryOf(stack: &ItemStack) -> Option<&'static Accessory>
{
    ACCESSORIES.iter().find(|row| stack.item.is(row.m_name))
}

// ItemAccessory.onItemRightClick: used out of hand it goes into the first free slot of its kind
struct Wearable;

#[item_behavior]
impl ItemBehavior for Wearable
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let Some(row) = accessoryOf(&stack)
        else
        {
            return Use::Pass;
        };

        let mut worn: Worn = Worn::of(player);
        let Some(free) = (0..SLOTS).find(|index| WORN[*index].m_slot == row.m_slot && worn.m_slots[*index].is_none())
        else
        {
            return Use::Pass;
        };

        if !world.isAuthority()
        {
            return Use::Done(Some(stack));
        }

        worn.m_slots[free] = ItemStack { count: 1, ..stack.clone() }.toRaw().ok();
        worn.save(player);

        Use::Done((stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack }))
    }
}

// PlayerAetherEvents.onPlayerAetherDrops: a player who dies drops what it wore unless the world keeps inventories
fn spill(event: &mut Event)
{
    let EventData::Death { entity: dead, .. } = event.data().clone()
    else
    {
        return;
    };
    let world: World = World { dimension: dead.world };
    let keeps: bool = matches!(gameRule("minecraft:keep_inventory"), Ok(GameRuleValue::Bool(true)));
    if dead.player.is_none() || !world.isAuthority() || keeps
    {
        return;
    }

    let player: Player = Player { id: dead.id };
    let mut worn: Worn = Worn::of(player);
    for index in 0..SLOTS
    {
        if let Some(stack) = worn.stack(index)
        {
            let _ = world.spawnItem(dead.pos, stack);
        }

        worn.m_slots[index] = None;
    }

    worn.save(player);
}

// SlotAccessory: a click swaps the cursor stack with the slot when the cursor holds an accessory of its kind
#[derive(Serialize, Deserialize)]
struct Click
{
    m_slot: u8,
}

// SlotAccessory: the loader swaps the cursor stack and the slot when the slot's tag takes the cursor stack
fn click(message: Click, player: Player)
{
    if let Some(slot) = WORN.get(message.m_slot as usize)
    {
        player.swapAccessory(slot.m_name);
    }
}

mod panel
{
    // the equipment panel beside the vanilla inventory, the toggle sits where AetherClientEvents puts it on GuiInventory
    pub const TOGGLE: [i32; 4] = [26, 65, 12, 12];
    // GuiAccessoryButton draws the cloud at 14 one pixel left of its 12 pixel hitbox
    pub const TOGGLE_ART: [i32; 4] = [25, 65, 14, 14];
    // same gap as the rei panel on the other side and rows level with the vanilla armor slots
    pub const GAP: i32 = 6;
    pub const LEFT: i32 = -(WIDTH + GAP);
    pub const TOP: i32 = 0;
    pub const WIDTH: i32 = 50;
    pub const HEIGHT: i32 = 86;
    // slot items sit eight in from the edge like vanilla so the slot frames leave seven on every side
    pub const INSET: i32 = 8;
    pub const SLOT: i32 = 18;
    pub const ROWS: usize = 4;
    pub const SHEET: &str = "aether:gui/inventory/accessories";
    pub const SLOT_UV: [i32; 2] = [76, 7];
    pub const TOGGLE_ICON: &str = "aether:gui/inventory/button/cloud";
    pub const TOGGLE_HOVER: &str = "aether:gui/inventory/button/cloud_hover";
    pub const HOVER: u32 = 0x80FFFFFF;
    pub const ICON: i32 = 16;
}

thread_local!
{
    static SHOWN: Cell<bool> = const { Cell::new(false) };
}

fn slotAt(index: usize) -> [i32; 2]
{
    let column: i32 = (index / panel::ROWS) as i32;
    let row: i32 = (index % panel::ROWS) as i32;
    [panel::LEFT + panel::INSET + column * panel::SLOT, panel::TOP + panel::INSET + row * panel::SLOT]
}

fn inside(point: [i32; 2], rect: [i32; 4]) -> bool
{
    point[0] >= rect[0] && point[1] >= rect[1] && point[0] < rect[0] + rect[2] && point[1] < rect[1] + rect[3]
}

// the toggle on the vanilla inventory and, while shown, the eight accessory slots in a beta style panel
struct Equipment;

impl Overlay for Equipment
{
    fn draw(&mut self, draw: &mut Draw, _size: [i32; 2], mouse: [i32; 2])
    {
        let [x, y, w, h] = panel::TOGGLE_ART;

        let texture: &str = if inside(mouse, panel::TOGGLE)
        {
            panel::TOGGLE_HOVER
        }
        else
        {
            panel::TOGGLE_ICON
        };
        draw.image(texture, x, y, w, h);

        if !SHOWN.with(Cell::get)
        {
            return;
        }

        draw.panel(panel::LEFT, panel::TOP, panel::WIDTH, panel::HEIGHT);

        let worn: Worn = localPlayer().map_or_else(Worn::default, Worn::of);

        for (index, slot) in WORN.iter().enumerate()
        {
            let [sx, sy] = slotAt(index);
            draw.texture(panel::SHEET, sx - 1, sy - 1, panel::SLOT_UV[0], panel::SLOT_UV[1], panel::SLOT, panel::SLOT);

            match worn.stack(index)
            {
                Some(stack) => draw.item(sx, sy, &stack),
                None => draw.image(&format!("aether:item/slots/{}", slot.m_icon), sx, sy, panel::ICON, panel::ICON),
            };

            if inside(mouse, [sx, sy, panel::ICON, panel::ICON])
            {
                draw.rect(sx, sy, panel::ICON, panel::ICON, panel::HOVER);
            }
        }
    }

    fn mouse(&mut self, x: i32, y: i32, button: i32) -> bool
    {
        if inside([x, y], panel::TOGGLE)
        {
            SHOWN.with(|shown| shown.set(!shown.get()));
            return true;
        }

        if !SHOWN.with(Cell::get) || button != 0
        {
            return false;
        }

        match (0..SLOTS).find(|index|
        {
            let [sx, sy] = slotAt(*index);
            inside([x, y], [sx, sy, panel::ICON, panel::ICON])
        })
        {
            Some(index) =>
            {
                let _ = send(CLICK, Target::Server, &Click { m_slot: index as u8 });

                true
            }

            None => inside([x, y], [panel::LEFT, panel::TOP, panel::WIDTH, panel::HEIGHT]),
        }
    }
}

mod ability
{
    // AbilityAccessories
    pub const WEAR_EVERY: f64 = 400.0;
    pub const AGILE_STEP: f64 = 1.0;
    pub const NORMAL_STEP: f64 = 0.5;
    pub const FEATHER_FALL: f64 = 0.6;
    // the regeneration stone hands a hurt wearer eighty ticks of regeneration whenever it has none
    pub const REGEN_TICKS: i32 = 80;
    pub const FULL_AIR: f64 = 300.0;
    pub const BURN_EVERY: f64 = 5.0;
    pub const FREEZE_REACH: i32 = 1;
}

// players whose step height the agility cape raised, and players whose worn list reached their own client
static STEPPING: EntityState<bool> = EntityState::new("stepping");
static ANNOUNCED: EntityState<bool> = EntityState::new("announced");
// the life shard count each player's health modifier was last set for
static SHARDS: EntityState<Option<i32>> = EntityState::new("shards");

// PlayerAether's health modifier, set only when the shards change since every set resends the player's modifiers
fn shardHealth(entity: Entity, worn: &Worn)
{
    let applied: Option<i32> = SHARDS.get(entity);
    if applied == Some(worn.m_lifeShards)
    {
        return;
    }

    let modifier: &str = "aether:life_shards";
    if let Err(error) = entity.setModifier("minecraft:max_health", modifier, worn.shardHealth(), ModifierOperation::Add)
    {
        warn(&format!("life shard health: {error}"));
        return;
    }

    crate::entities::store(&SHARDS, entity, &Some(worn.m_lifeShards));
}

// the ice ring and pendant freeze water to ice and lava to obsidian around the wearer's feet
fn freeze(world: World, entity: Entity, worn: &mut Worn) -> bool
{
    let Some(info) = entity.info()
    else
    {
        return false;
    };

    let feet: [i32; 3] = [floor(info.pos[0]), floor(entity.field(EntityField::Bottom)), floor(info.pos[2])];
    let mut changed: bool = false;
    let reach: i32 = ability::FREEZE_REACH;
    for x in feet[0] - reach..=feet[0] + reach
    {
        for y in feet[1] - reach..=feet[1] + reach
        {
            for z in feet[2] - reach..=feet[2] + reach
            {
                let pos: BlockPos = BlockPos::new(x, y, z);
                let into: Option<&str> = if world.tagged(pos, "minecraft:water")
                {
                    Some("minecraft:ice")
                }
                else if world.tagged(pos, "minecraft:lava")
                {
                    Some("minecraft:obsidian")
                }
                else
                {
                    None
                };

                if let Some(name) = into
                {
                    let _ = world.place(pos, name);

                    worn.wear("ice_ring", 1);
                    worn.wear("ice_pendant", 1);
                    changed = true;
                }
            }
        }
    }

    changed
}

// AbilityAccessories.onUpdate on the server for each player
fn serverTick(player: Player)
{
    let Some(info) = player.info()
    else
    {
        return;
    };

    let world: World = World { dimension: info.world };
    let entity: Entity = Entity { world, id: player.id };
    let before: Worn = Worn::of(player);
    let mut worn: Worn = before.clone();
    let age: f64 = entity.field(EntityField::TicksExisted);

    shardHealth(entity, &worn);
    crate::tools::valkyrieReach(player);
    worn.m_valkyrieSet = crate::player::wearingSet(player, "valkyrie");
    crate::abilities::repel(player, &mut worn);

    if !ANNOUNCED.get(entity) && player.canSend(CLICK)
    {
        // the worn list loaded with the player reaches its client once the mod handshake is done
        worn.save(player);
        crate::entities::store(&ANNOUNCED, entity, &true);
    }

    if age % ability::WEAR_EVERY == 0.0
    {
        worn.wear("zanite_ring", 1);
        worn.wear("zanite_pendant", 1);
    }

    if worn.wearing("ice_ring") || worn.wearing("ice_pendant")
    {
        freeze(world, entity, &mut worn);
    }

    if worn.wearing("iron_bubble")
    {
        entity.setField(EntityField::Air, ability::FULL_AIR);
    }

    let health: i32 = info.health.unwrap_or(0);
    let hurt: bool = health > 0 && health < entity.field(EntityField::MaxHealth) as i32;
    if worn.wearing("regeneration_stone") && hurt && !crate::effects::has(entity, Effect::Regeneration)
    {
        crate::effects::give(entity, Effect::Regeneration, ability::REGEN_TICKS);
    }

    if worn.wearing("phoenix_gloves") && entity.flag(EntityField::InWater) && age % ability::BURN_EVERY == 0.0
    {
        let gloves: usize = WORN.iter().position(|slot| slot.m_slot == Slot::Gloves).unwrap_or(SLOTS - 1);

        worn.wear("phoenix_gloves", 1);

        if worn.m_slots[gloves].is_none()
        {
            worn.m_slots[gloves] = ItemStack::of("obsidian_gloves", 1).toRaw().ok();
        }
    }

    if worn != before
    {
        worn.save(player);
    }
}

// ItemLifeShard.onItemRightClick: each shard adds a heart until the configured limit
pub(crate) struct LifeShard;

#[item_behavior]
impl ItemBehavior for LifeShard
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        if !world.isAuthority()
        {
            return Use::Pass;
        }

        let mut worn: Worn = Worn::of(player);
        let limit: i32 = crate::config::maxLifeShards();
        if worn.m_lifeShards >= limit
        {
            player.chat(&translate("aether.gui.item.life_shard.maxshards").replace("%s", &limit.to_string()));
            return Use::Pass;
        }

        worn.m_lifeShards += 1;
        worn.save(player);

        let remaining: i32 = stack.count - 1;
        Use::Done((remaining > 0).then_some(ItemStack { count: remaining, ..stack }))
    }
}

pub(crate) fn lifeShard() -> LifeShard { LifeShard }

// the movement half of AbilityAccessories runs where the player moves, on its own client
fn localTick()
{
    let Some(player) = localPlayer()
    else
    {
        return;
    };
    let Some(info) = player.info()
    else
    {
        return;
    };

    let entity: Entity = Entity { world: World { dimension: info.world }, id: player.id };
    let worn: Worn = Worn::of(player);
    let sneaking: bool = entity.flag(EntityField::Sneaking);
    let agile: bool = worn.wearing("agility_cape") && !sneaking;
    let wasAgile: bool = STEPPING.get(entity);
    if agile || wasAgile
    {
        let value: f64 = if agile
        {
            ability::AGILE_STEP
        }
        else
        {
            ability::NORMAL_STEP
        };
        entity.setField(EntityField::StepHeight, value);
        crate::entities::store(&STEPPING, entity, &agile);
    }

    if worn.wearing("golden_feather") || worn.wearing("valkyrie_cape")
    {
        if !entity.flag(EntityField::OnGround)
            && info.motion[1] < 0.0
            && !entity.flag(EntityField::InWater)
            && !sneaking
        {
            let _ = entity.setMotion([info.motion[0], info.motion[1] * ability::FEATHER_FALL, info.motion[2]]);
        }

        entity.setField(EntityField::FallDistance, -1.0);
    }
}

// the zanite ring and pendant speed up mining as they wear
fn breakSpeed(event: &mut Event)
{
    let Some(player) = event.player()
    else
    {
        return;
    };

    let worn: Worn = Worn::of(player);
    let mut factor: f32 = 1.0;

    for index in 0..SLOTS
    {
        let Some(stack) =
            worn.stack(index).filter(|stack| stack.item.is("zanite_ring") || stack.item.is("zanite_pendant"))
        else
        {
            continue;
        };

        let uses: f32 = accessoryOf(&stack).and_then(|row| row.m_uses).unwrap_or(1) as f32;
        factor *= 1.0 + stack.damage as f32 / (uses * 3.0);
    }

    if let EventData::BreakSpeed { speed, .. } = event.dataMut()
    {
        *speed *= factor;
    }
}

mod look
{
    // PlayerAetherRenderer: modelMisc is a biped grown by one, gloves draw it at 0.9375, the shield a 0.7 biped at 1.1
    pub const MISC: f32 = 1.0;
    pub const GLOVE_SCALE: f32 = 0.9375;
    pub const SHIELD: f32 = 0.7;
    pub const SHIELD_SCALE: f32 = 1.1;
    pub const CAPE: f32 = 0.0;
    pub const WHITE: u32 = 0xFFFFFF;
    pub const BASE: &str = "base";
}

// ItemAccessory.setTexture names, accessories without one draw the tinted base sheet
const SHEETS: [(&str, &str); 9] = [
    ("leather_gloves", "leather"),
    ("chain_gloves", "chain"),
    ("neptune_gloves", "chain"),
    ("phoenix_gloves", "phoenix"),
    ("valkyrie_gloves", "valkyrie"),
    ("swet_cape", "swet_cape"),
    ("agility_cape", "agility_cape"),
    ("valkyrie_cape", "valkyrie_cape"),
    ("repulsion_shield", "repulsion"),
];

fn sheet(row: &Accessory) -> String
{
    let name: &str = SHEETS.iter().find(|(item, _)| *item == row.m_name).map_or(look::BASE, |(_, sheet)| sheet);
    format!("aether:armor/accessory_{name}")
}

fn tint(row: &Accessory) -> u32
{
    if SHEETS.iter().any(|(item, _)| *item == row.m_name)
    {
        look::WHITE
    }
    else
    {
        row.m_color
    }
}

fn layer(row: &Accessory, parts: Vec<BipedPart>, inflate: f32, scale: f32, blend: bool) -> PlayerLayer
{
    PlayerLayer { texture: sheet(row), color: tint(row), parts, inflate, glow: false, scale, blend, ops: Vec::new() }
}

// PlayerAetherRenderer.renderModel: pendant on the body, cape on the cloak, gloves on the arms, the shield all over
fn layers(player: &EntityInfo, _synced: &[u8]) -> Vec<PlayerLayer>
{
    let worn: Worn = Worn::ofEntity(Entity { world: World { dimension: player.world }, id: player.id });
    let mut out: Vec<PlayerLayer> = crate::abilities::wings(player, &worn).into_iter().collect();
    let invisible: bool = worn.wearing("invisibility_cape");

    for index in 0..SLOTS
    {
        let Some(row) = worn.stack(index).as_ref().and_then(accessoryOf)
        else
        {
            continue;
        };

        match row.m_slot
        {
            Slot::Pendant => out.push(layer(row, vec![BipedPart::Body], look::MISC, 1.0, false)),
            Slot::Cape if !invisible => out.push(layer(row, vec![BipedPart::Cloak], look::CAPE, 1.0, false)),
            Slot::Gloves => out.push(layer(
                row,
                vec![BipedPart::LeftArm, BipedPart::RightArm],
                look::MISC,
                look::GLOVE_SCALE,
                false,
            )),

            Slot::Shield =>
            {
                let parts: Vec<BipedPart> = vec![
                    BipedPart::Head,
                    BipedPart::Body,
                    BipedPart::RightArm,
                    BipedPart::LeftArm,
                    BipedPart::RightLeg,
                    BipedPart::LeftLeg,
                ];
                out.push(layer(row, parts, look::SHIELD, look::SHIELD_SCALE, true));
            }

            _ =>
            {}
        }
    }

    out
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.screens().playerLayer(layers)?;

    for row in &ACCESSORIES
    {
        let mut spec: ItemSpecBuilder =
            ItemSpecBuilder::new().texture(&format!("aether:item/accessories/{}", row.m_icon)).maxStack(1);

        if row.m_color != PLAIN
        {
            spec = spec.color(row.m_color);
        }

        if let Some(uses) = row.m_uses
        {
            spec = spec.maxDamage(uses);
        }

        ctx.items().register(row.m_name, spec.behavior(Wearable))?;
    }

    for (kind, tag) in KINDS
    {
        let members: Vec<&str> = ACCESSORIES.iter().filter(|row| row.m_slot == kind).map(|row| row.m_name).collect();
        ctx.tags().add(RegistryKind::Item, tag, &members)?;
    }

    for slot in &WORN
    {
        ctx.accessories().register(
            slot.m_name,
            kindTag(slot.m_slot),
            Some(&format!("aether:item/slots/{}", slot.m_icon)),
        )?;
    }

    ctx.channels().register::<Click>(CLICK, Direction::ToServer, click)?;
    ctx.screens().overlay("GuiInventory", Equipment)?;

    ctx.events().on::<TickPost>(Priority::Normal, |_|
    {
        if isClient()
        {
            localTick();
        }

        for player in allPlayers()
        {
            if player.info().is_some_and(|info| World { dimension: info.world }.isAuthority())
            {
                serverTick(player);
            }
        }
    })?;

    ctx.events().on::<PlayerBreakSpeed>(Priority::Normal, breakSpeed)?;
    ctx.events().on::<LivingDeath>(Priority::Normal, spill)?;
    Ok(())
}
