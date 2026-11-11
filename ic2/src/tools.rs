use crate::core::{dir, ids, itemTexture, sound, sprites, stacks};
use crate::electric;
use crate::energy::{cable, net, storage, tileAt};
use crate::items::wear;
use crate::keys::{self, Key};
use crate::machines::{self, mining, personal, terraformer};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

// the player's held stack changed by a tool, logged when the inventory refuses it
fn hold(player: Player, stack: Option<ItemStack>)
{
    if let Err(error) = player.setHeld(stack)
    {
        warn(&format!("could not update a held IC2 tool: {error}"));
    }
}

fn worldOf(player: Player) -> Option<World> { player.info().map(|info| World { dimension: info.world }) }

mod wrench
{
    pub const TURN: i32 = 1;
    pub const DISMANTLE: i32 = 10;
    pub const DEFUSE: i32 = 15;
    pub const USES: i32 = 160;
    pub const ELECTRIC_USES: i32 = 202;
}

const EXPLOSIVES: [&str; 5] =
    ["minecraft:tnt", "ic2:industrial_tnt", "ic2:nuke", "ic2:dynamite_block", "ic2:remote_dynamite_block"];

// ItemToolWrenchElectric.canTakeDamage: the electric wrench stops short of empty, the plain one wears through
fn canWear(stack: &ItemStack, amount: i32) -> bool
{
    !stack.item.is("ic2:electric_wrench") || amount < maxDamage(&stack.item) - stack.damage
}

fn wearWrench(player: Player, stack: ItemStack, amount: i32)
{
    let worn: Option<ItemStack> = if stack.item.is("ic2:electric_wrench")
    {
        Some(ItemStack { damage: stack.damage + amount, ..stack })
    }
    else
    {
        wear(stack, amount)
    };
    hold(player, worn);
}

// ItemToolWrench.onItemUseFirst: turn a storage block, pull a terraformer's blueprint, dismantle a machine or
// defuse an explosive
fn useWrench(world: World, player: Player, stack: ItemStack, pos: BlockPos, face: i32) -> bool
{
    let center: [f64; 3] = dir::center(pos);

    if storage::isStorage(world, pos) && canWear(&stack, wrench::TURN) && storage::turn(world, pos, face)
    {
        wearWrench(player, stack, wrench::TURN);
        world.playSound(center, &sound("wrench"), 1.0, 1.0);
        return true;
    }

    if terraformer::isTerraformer(world, pos) && canWear(&stack, wrench::TURN) && terraformer::eject(world, pos)
    {
        wearWrench(player, stack, wrench::TURN);
        world.playSound(center, &sound("wrench"), 1.0, 1.0);
        return true;
    }

    let block: Option<Identifier> = nameOf(RegistryKind::Block, world.block(pos));
    let machine: Option<&machines::Machine> = block.as_ref().and_then(|id| machines::find(&id.to_string()));
    let allowed: bool = !personal::isPersonal(world, pos) || personal::canAccess(world, pos, player);

    if let Some(machine) = machine.filter(|machine| machine.m_wrench > 0.0)
        && canWear(&stack, wrench::DISMANTLE)
        && allowed
    {
        if world.isAuthority()
        {
            machines::dismantle(world, pos, machine);
        }

        wearWrench(player, stack, wrench::DISMANTLE);
        world.playSound(center, &sound("wrench"), 1.0, 1.0);
        return true;
    }

    if machines::luminator::isLuminator(world, pos) && canWear(&stack, wrench::DISMANTLE)
    {
        if world.isAuthority()
        {
            stacks::drop(world, pos, &ItemStack::of("ic2:luminator", 1).withDamage(world.meta(pos)));
        }

        let _ = world.setBlock(pos, 0, 0);
        net::changed(world);
        wearWrench(player, stack, wrench::DISMANTLE);
        world.playSound(center, &sound("wrench"), 1.0, 1.0);
        return true;
    }

    let explosive: bool = block.as_ref().is_some_and(|id| EXPLOSIVES.contains(&id.to_string().as_str()));
    if explosive && canWear(&stack, wrench::DEFUSE)
    {
        if world.isAuthority()
            && let Some(id) = &block
        {
            let dropped: &str = if id.to_string().ends_with("dynamite_block")
            {
                "ic2:dynamite"
            }
            else
            {
                ""
            };
            let item: String = if dropped.is_empty()
            {
                id.to_string()
            }
            else
            {
                dropped.to_owned()
            };
            stacks::drop(world, pos, &ItemStack::of(item.as_str(), 1).withDamage(world.meta(pos)));
        }

        let _ = world.setBlock(pos, 0, 0);
        wearWrench(player, stack, wrench::DEFUSE);
        world.playSound(center, &sound("wrench"), 1.0, 1.0);
        return true;
    }

    false
}

thread_local!
{
    // ItemToolMeter.lastMeasuredTileEntity, lastTotalEnergyConducted and lastMeasureTime per player
    static METER: RefCell<HashMap<i32, (BlockPos, i64, i64)>> = RefCell::new(HashMap::new());
}

// ItemToolMeter.onItemUseFirst: two clicks on one energy block give its average flow between them
fn useMeter(world: World, player: Player, pos: BlockPos) -> bool
{
    if tileAt(world, pos).is_none()
    {
        return true;
    }

    let conducted: i64 = net::conducted(world, pos);
    let now: i64 = world.time();
    let last: Option<(BlockPos, i64, i64)> = METER.with(|meter| meter.borrow().get(&player.id).copied());

    match last
    {
        Some((at, total, time)) if at == pos =>
        {
            const HUNDREDTHS: f64 = 100.0;

            let period: i64 = (now - time).max(1);
            let flow: f64 = (conducted - total) as f64 / period as f64;
            let shown: f64 = (flow * HUNDREDTHS).round() / HUNDREDTHS;
            player.chat(&format!("Measured power: {shown} EU/t (avg. over {period} ticks)"));
        }

        _ => player.chat("Starting new measurement"),
    }

    METER.with(|meter| meter.borrow_mut().insert(player.id, (pos, conducted, now)));
    true
}

fn onUseFirst(event: &mut Event)
{
    let EventData::ItemUse { world, player, stack: Some(stack), pos: Some(pos), face } = event.data().clone()
    else
    {
        return;
    };
    let world: World = World { dimension: world };
    let player: Player = Player { id: player };
    let stack: ItemStack = ItemStack::fromRaw(stack);

    let used: bool = match stack.item.name().map(|id| id.to_string()).as_deref()
    {
        Some("ic2:wrench" | "ic2:electric_wrench") => useWrench(world, player, stack, pos, face),
        Some("ic2:eu_reader") => world.isAuthority() && useMeter(world, player, pos),
        _ => false,
    };

    if used
    {
        event.cancel();
    }
}

// ItemToolCutter.onItemUse: one rubber from the inventory wraps one layer of insulation around a cable
struct Cutter;

const CUTTER_USES: i32 = 512;
const INVENTORY: i32 = 36;

fn takeRubber(player: Player) -> bool
{
    let Some(slot) = (0..INVENTORY).find(|slot| player.slot(*slot).is_some_and(|stack| stack.item.is("ic2:rubber")))
    else
    {
        return false;
    };

    let left: Option<ItemStack> = player.slot(slot).and_then(|stack| stacks::less(stack, 1));
    player.setSlot(slot, left).is_ok()
}

#[item_behavior]
impl ItemBehavior for Cutter
{
    fn onUseOnBlock(&self, world: World, player: Player, stack: ItemStack, pos: BlockPos, _face: i32) -> Use
    {
        if !cable::isCable(world, pos) || !cable::canInsulate(world, pos) || !takeRubber(player)
        {
            return Use::Pass;
        }

        cable::insulate(world, pos);
        Use::Done(wear(stack, 1))
    }
}

// ItemElectricToolDrill, DDrill and Chainsaw: fast on their blocks while charged, a point per block, two per hit
struct ElectricTool
{
    m_kind: ToolKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToolKind
{
    Drill,
    DiamondDrill,
    Chainsaw,
}

mod speed
{
    pub const DRILL: f32 = 8.0;
    pub const DIAMOND_DRILL: f32 = 16.0;
    pub const CHAINSAW: f32 = 12.0;
    pub const BLOCK_COST: i32 = 1;
    pub const HIT_COST: i32 = 2;
}

const CHAINSAW_BLOCKS: [&str; 6] = [
    "minecraft:planks",
    "minecraft:bookshelf",
    "minecraft:log",
    "minecraft:chest",
    "minecraft:leaves",
    "ic2:rubber_leaves",
];

fn chainsawCuts(block: i32) -> bool
{
    let wooden: bool = nameOf(RegistryKind::Block, block).is_some_and(|id|
    {
        tagsOf(RegistryKind::Block, &id).iter().any(|tag| tag.to_string() == "minecraft:mineable/axe")
    });
    wooden || CHAINSAW_BLOCKS.iter().any(|name| ids::block(name) == Some(block))
}

impl ElectricTool
{
    fn harvests(&self, block: i32) -> bool
    {
        match self.m_kind
        {
            ToolKind::Drill => mining::drillHarvests(block, false),
            ToolKind::DiamondDrill => mining::drillHarvests(block, true),
            ToolKind::Chainsaw => chainsawCuts(block),
        }
    }

    fn efficiency(&self) -> f32
    {
        match self.m_kind
        {
            ToolKind::Drill => speed::DRILL,
            ToolKind::DiamondDrill => speed::DIAMOND_DRILL,
            ToolKind::Chainsaw => speed::CHAINSAW,
        }
    }
}

// ItemElectricTool.use: wears the charge and tops it up from a worn batpack
fn useCharge(player: Player, mut stack: ItemStack, points: i32)
{
    crate::gear::batpackCharge(player, &mut stack);
    electric::use_(&mut stack, points);
    crate::gear::batpackCharge(player, &mut stack);
    hold(player, Some(stack));
}

#[item_behavior]
impl ItemBehavior for ElectricTool
{
    fn strVsBlock(&self, stack: ItemStack, block: i32) -> f32
    {
        if !electric::charged(&stack) || !self.harvests(block)
        {
            return 1.0;
        }

        self.efficiency()
    }

    fn canHarvest(&self, block: i32) -> bool { self.harvests(block) }

    fn onBlockDestroyed(&self, stack: ItemStack, _block: i32, _pos: BlockPos, player: Player) -> bool
    {
        useCharge(player, stack, speed::BLOCK_COST);
        true
    }

    fn onHitEntity(&self, stack: ItemStack, _target: Entity, attacker: i32) -> bool
    {
        useCharge(Player { id: attacker }, stack, speed::HIT_COST);
        true
    }
}

mod saber
{
    pub const USES: i32 = 5002;
    pub const ACTIVE_DAMAGE: i32 = 16;
    pub const IDLE_DAMAGE: i32 = 3;
    pub const WEB_SPEED: f32 = 4.0;
    pub const HIT: i32 = 5;
    pub const ARMOR_HIT: i32 = 2;
    pub const NANO_WEAR: i32 = 10;
    pub const BLOCK: i32 = 10;
    pub const HOTBAR_EVERY: i64 = 16;
    pub const BACKPACK_EVERY: i64 = 64;
    pub const HOTBAR_DRAIN: i32 = 16;
    pub const BACKPACK_DRAIN: i32 = 64;
    pub const HOTBAR: i32 = 9;
    pub const ARMOR: [i32; 4] = [36, 37, 38, 39];
}

// ItemNanoSaber.drainSaber: an emptied saber switches itself off
fn drainSaber(stack: ItemStack, points: i32) -> ItemStack
{
    let last: i32 = saber::USES - 1;
    if stack.damage + points >= last
    {
        return ItemStack::of("ic2:nano_saber", 1).withDamage(last);
    }

    ItemStack { damage: stack.damage + points, ..stack }
}

fn swingSound(world: World) -> String
{
    const SWINGS: [&str; 3] = ["nanosabreSwing", "nanosabreSwingOne", "nanosabreSwingTwo"];

    sound(SWINGS[world.random(SWINGS.len() as i32) as usize])
}

// ItemNanoSaber: switched on by a right click, cuts anything, eats the charge and the target's nano armor
struct NanoSaber
{
    m_active: bool,
}

#[item_behavior]
impl ItemBehavior for NanoSaber
{
    fn icon(&self, _damage: i32) -> i32
    {
        let shine: bool = self.m_active && localPlayer().and_then(worldOf).is_some_and(|world| world.random(2) == 0);
        let name: &str = match (self.m_active, shine)
        {
            (true, true) => "nano_saber_shine",
            (true, false) => "nano_saber_active",
            _ => "nano_saber",
        };
        sprites::item(&itemTexture(name))
    }

    fn strVsBlock(&self, _stack: ItemStack, _block: i32) -> f32 { if self.m_active { saber::WEB_SPEED } else { 1.0 } }

    fn canHarvest(&self, block: i32) -> bool { ids::block("minecraft:web") == Some(block) }

    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        if self.m_active
        {
            return Use::Done(Some(ItemStack { item: ItemRef::from("ic2:nano_saber"), ..stack }));
        }

        if stack.damage >= saber::USES - 1
        {
            return Use::Done(Some(stack));
        }

        if let Some(info) = player.info()
        {
            world.playSound(info.pos, &sound("nanosabrePower"), 1.0, 1.0);
        }

        Use::Done(Some(ItemStack { item: ItemRef::from("ic2:nano_saber_active"), ..stack }))
    }

    fn onHitEntity(&self, stack: ItemStack, target: Entity, attacker: i32) -> bool
    {
        if !self.m_active
        {
            return true;
        }

        let mut stack: ItemStack = stack;
        let world: World = target.world;
        let victim: Option<String> = target.info().and_then(|info| info.player);

        if victim.is_some()
        {
            let enemy: Player = Player { id: target.id };
            for slot in saber::ARMOR
            {
                let Some(piece) = enemy.slot(slot).filter(crate::gear::isNano)
                else
                {
                    continue;
                };

                let _ = enemy.setSlot(slot, wear(piece, saber::NANO_WEAR));
                stack = drainSaber(stack, saber::ARMOR_HIT);
            }
        }

        if let Some(info) = world.entity(attacker)
        {
            world.playSound(info.pos, &swingSound(world), 1.0, 1.0);
        }

        hold(Player { id: attacker }, Some(drainSaber(stack, saber::HIT)));
        true
    }

    fn onBlockDestroyed(&self, stack: ItemStack, _block: i32, _pos: BlockPos, player: Player) -> bool
    {
        if self.m_active
        {
            hold(player, Some(drainSaber(stack, saber::BLOCK)));
        }

        true
    }
}

// ItemNanoSaber.timedLoss: a lit saber burns its charge, faster in the hotbar
fn saberLoss(ticks: i64)
{
    if ticks % saber::HOTBAR_EVERY != 0
    {
        return;
    }

    for player in allPlayers()
    {
        let backpack: bool = ticks % saber::BACKPACK_EVERY == 0;
        let slots: Vec<(i32, i32)> = (0..INVENTORY)
            .filter_map(|slot| match slot < saber::HOTBAR
            {
                true => Some((slot, saber::HOTBAR_DRAIN)),
                false if backpack => Some((slot, saber::BACKPACK_DRAIN)),
                false => None,
            })
            .collect();

        for (slot, drain) in slots
        {
            if let Some(stack) = player.slot(slot).filter(|stack| stack.item.is("ic2:nano_saber_active"))
            {
                let _ = player.setSlot(slot, Some(drainSaber(stack, drain)));
            }
        }
    }
}

// ItemScanner: the ore density below the player, or with the OV scanner its value
struct Scanner
{
    m_advanced: bool,
}

mod scan
{
    pub const RANGE: i32 = 2;
    pub const ADVANCED_RANGE: i32 = 4;
    pub const COST: i32 = 1;
    pub const ADVANCED_COST: i32 = 5;
    pub const PER_MILLE: f64 = 1000.0;
}

fn scanArea(world: World, at: BlockPos, advanced: bool) -> i32
{
    let range: i32 = if advanced
    {
        scan::ADVANCED_RANGE
    }
    else
    {
        scan::RANGE
    };
    let mut score: i64 = 0;
    let mut scanned: i64 = 0;

    for y in (1..=at.y).rev()
    {
        for x in at.x - range..=at.x + range
        {
            for z in at.z - range..=at.z + range
            {
                let value: i32 = mining::oreValue(world.block(BlockPos::new(x, y, z)));
                score += i64::from(
                    if advanced
                    {
                        value
                    }
                    else
                    {
                        i32::from(value > 0)
                    },
                );
                scanned += 1;
            }
        }
    }

    if scanned == 0
    {
        0
    }
    else
    {
        (scan::PER_MILLE * score as f64 / scanned as f64) as i32
    }
}

#[item_behavior]
impl ItemBehavior for Scanner
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        let mut stack: ItemStack = stack;
        let cost: i32 = if self.m_advanced
        {
            scan::ADVANCED_COST
        }
        else
        {
            scan::COST
        };
        if !electric::use_(&mut stack, cost)
        {
            return Use::Done(Some(stack));
        }

        let Some(info) = player.info()
        else
        {
            return Use::Done(Some(stack));
        };
        world.playSound(info.pos, &sound("scanner"), 1.0, 1.0);

        if world.isAuthority()
        {
            let at: BlockPos = BlockPos::new(info.pos[0] as i32, info.pos[1] as i32, info.pos[2] as i32);
            let value: i32 = scanArea(world, at, self.m_advanced);
            let message: String = if self.m_advanced
            {
                format!("SCAN RESULT: Ore value in this area is {value}")
            }
            else
            {
                format!("SCAN RESULT: Ore density in this area is {value}")
            };
            player.chat(&message);
        }

        Use::Done(Some(stack))
    }
}

// ItemToolMiningLaser: five modes picked with the laser key, each a burst of laser entities
struct Laser;

#[derive(Clone, Copy)]
struct Mode
{
    m_name: &'static str,
    m_cost: i32,
    m_range: f32,
    m_explosive: bool,
    m_scatter: bool,
}

const MODES: [Mode; 5] = [
    Mode { m_name: "Mining", m_cost: 125, m_range: 8.0, m_explosive: false, m_scatter: false },
    Mode { m_name: "Low-Focus", m_cost: 10, m_range: 1.5, m_explosive: false, m_scatter: false },
    Mode { m_name: "Long-Range", m_cost: 1000, m_range: 32.0, m_explosive: false, m_scatter: false },
    Mode { m_name: "Scatter", m_cost: 2000, m_range: 6.0, m_explosive: false, m_scatter: true },
    Mode { m_name: "Explosive", m_cost: 1000, m_range: 12.0, m_explosive: true, m_scatter: false },
];

thread_local!
{
    // ItemToolMiningLaser.setting per player
    static LASER_MODE: RefCell<HashMap<i32, usize>> = RefCell::new(HashMap::new());
}

#[item_behavior]
impl ItemBehavior for Laser
{
    fn onRightClick(&self, world: World, player: Player, stack: ItemStack) -> Use
    {
        const SCATTER_STEP: f32 = 20.0;
        const SCATTER: i32 = 2;

        let mode: usize = LASER_MODE.with(|modes| modes.borrow().get(&player.id).copied().unwrap_or(0));

        if keys::held(player, Key::Laser)
        {
            let next: usize = (mode + 1) % MODES.len();
            LASER_MODE.with(|modes| modes.borrow_mut().insert(player.id, next));
            if world.isAuthority()
            {
                player.chat(&format!("Laser Mode: {}", MODES[next].m_name));
            }

            return Use::Done(Some(stack));
        }

        let chosen: Mode = MODES[mode];
        let mut stack: ItemStack = stack;
        crate::gear::batpackCharge(player, &mut stack);
        if !electric::use_(&mut stack, chosen.m_cost)
        {
            return Use::Done(Some(stack));
        }

        let Some(info) = player.info().filter(|_| world.isAuthority())
        else
        {
            return Use::Done(Some(stack));
        };

        if chosen.m_scatter
        {
            for x in -SCATTER..=SCATTER
            {
                for z in -SCATTER..=SCATTER
                {
                    let aim: [f32; 2] = [info.yaw + SCATTER_STEP * x as f32, info.pitch + SCATTER_STEP * z as f32];
                    crate::explosives::fireLaser(world, &info, aim, chosen.m_range, chosen.m_explosive);
                }
            }
        }
        else
        {
            crate::explosives::fireLaser(world, &info, [info.yaw, info.pitch], chosen.m_range, chosen.m_explosive);
        }

        world.playSound(info.pos, &sound("miningLaser"), 1.0, 1.0);
        Use::Done(Some(stack))
    }
}

fn electricItem(name: &str, uses: i32) -> ItemSpecBuilder
{
    ItemSpecBuilder::new().texture(&itemTexture(name)).maxStack(1).maxDamage(uses)
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    const DRILL_USES: i32 = 202;
    const DIAMOND_USES: i32 = 122;
    const SCANNER_USES: i32 = 202;
    const LASER_USES: i32 = 8002;
    const CHAINSAW_DAMAGE: i32 = 10;
    const IRON: i32 = 2;
    const DIAMOND: i32 = 3;

    ctx.items().register("wrench", electricItem("wrench", wrench::USES))?;
    ctx.items().register("electric_wrench", electricItem("electric_wrench", wrench::ELECTRIC_USES))?;
    ctx.items().register("eu_reader", ItemSpecBuilder::new().texture(&itemTexture("eu_reader")).maxStack(1))?;
    ctx.items().register("insulation_cutter", electricItem("insulation_cutter", CUTTER_USES).behavior(Cutter))?;

    ctx.items().register(
        "mining_drill",
        electricItem("mining_drill", DRILL_USES)
            .tool(ToolClass::Pickaxe, IRON, speed::DRILL, 0, DRILL_USES)
            .behavior(ElectricTool { m_kind: ToolKind::Drill }),
    )?;
    ctx.items().register(
        "diamond_drill",
        electricItem("diamond_drill", DIAMOND_USES)
            .tool(ToolClass::Pickaxe, DIAMOND, speed::DIAMOND_DRILL, 0, DIAMOND_USES)
            .behavior(ElectricTool { m_kind: ToolKind::DiamondDrill }),
    )?;
    ctx.items().register(
        "chainsaw",
        electricItem("chainsaw", DRILL_USES)
            .tool(ToolClass::Axe, IRON, speed::CHAINSAW, CHAINSAW_DAMAGE, DRILL_USES)
            .behavior(ElectricTool { m_kind: ToolKind::Chainsaw }),
    )?;

    for (name, active, damage) in
        [("nano_saber", false, saber::IDLE_DAMAGE), ("nano_saber_active", true, saber::ACTIVE_DAMAGE)]
    {
        ctx.items().register(
            name,
            electricItem(name, saber::USES)
                .tool(ToolClass::Sword, IRON, 1.0, damage, saber::USES)
                .behavior(NanoSaber { m_active: active }),
        )?;
    }

    sprites::item(&itemTexture("nano_saber_shine"));
    ctx.items()
        .register("od_scanner", electricItem("od_scanner", SCANNER_USES).behavior(Scanner { m_advanced: false }))?;
    ctx.items()
        .register("ov_scanner", electricItem("ov_scanner", SCANNER_USES).behavior(Scanner { m_advanced: true }))?;
    ctx.items().register("mining_laser", electricItem("mining_laser", LASER_USES).behavior(Laser))?;

    ctx.events().on::<ItemUseFirst>(Priority::Normal, onUseFirst)?;
    ctx.events().on::<TickPost>(Priority::Normal, |_|
    {
        let world: World = World::OVERWORLD;
        if world.isAuthority()
        {
            saberLoss(world.time());
        }
    })
}
