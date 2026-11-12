use crate::entities::{fraction, moa};
use crate::names::{block, item};
use crate::progress::{self, Award};
use b173_api::prelude::*;
use b173_api::screen::Draw;
use b173_api::{JobBuilder, JobMatch, JobOutput, machineFuel, matchJob};

// one AetherEnchantment or AetherFreezable row, a None output gives back the input fully repaired
struct Recipe
{
    m_input: &'static str,
    m_output: Option<&'static str>,
    m_ticks: i32,
}

const fn repair(input: &'static str, ticks: i32) -> Recipe { Recipe { m_input: input, m_output: None, m_ticks: ticks } }

const fn turn(input: &'static str, output: &'static str, ticks: i32) -> Recipe
{
    Recipe { m_input: input, m_output: Some(output), m_ticks: ticks }
}

// AetherRegistries.initializeEnchantments rows whose items exist in b1.7.3 and this port
const ENCHANTING: [Recipe; 77] = [
    repair("skyroot_pickaxe", 225),
    repair("skyroot_axe", 225),
    repair("skyroot_shovel", 225),
    repair("skyroot_sword", 225),
    repair("holystone_pickaxe", 550),
    repair("holystone_axe", 550),
    repair("holystone_shovel", 550),
    repair("holystone_sword", 550),
    repair("zanite_pickaxe", 2250),
    repair("zanite_axe", 2250),
    repair("zanite_shovel", 2250),
    repair("zanite_sword", 2250),
    repair("gravitite_pickaxe", 5500),
    repair("gravitite_axe", 5500),
    repair("gravitite_shovel", 5500),
    repair("gravitite_sword", 5500),
    repair("zanite_helmet", 6000),
    repair("zanite_chestplate", 6000),
    repair("zanite_leggings", 6000),
    repair("zanite_boots", 6000),
    repair("gravitite_helmet", 13000),
    repair("gravitite_chestplate", 13000),
    repair("gravitite_leggings", 13000),
    repair("gravitite_boots", 13000),
    repair("zanite_ring", 2250),
    repair("zanite_pendant", 2250),
    turn("golden_dart", "enchanted_dart", 250),
    turn("golden_dart_shooter", "enchanted_dart_shooter", 500),
    turn(block::HOLYSTONE, "healing_stone", 750),
    turn(block::GRAVITITE_ORE, block::ENCHANTED_GRAVITITE, 1000),
    turn(block::QUICKSOIL, "quicksoil_glass", 250),
    turn("skyroot_poison_bucket", "skyroot_remedy_bucket", 1000),
    turn(item::BLUEBERRY, "enchanted_blueberry", 300),
    repair("minecraft:bow", 4000),
    repair("minecraft:fishing_rod", 600),
    turn("minecraft:record_13", "aether_tune", 2500),
    turn("minecraft:record_cat", "legacy", 2500),
    repair("minecraft:wooden_pickaxe", 225),
    repair("minecraft:wooden_axe", 225),
    repair("minecraft:wooden_shovel", 225),
    repair("minecraft:wooden_sword", 225),
    repair("minecraft:wooden_hoe", 225),
    repair("minecraft:stone_pickaxe", 550),
    repair("minecraft:stone_axe", 550),
    repair("minecraft:stone_shovel", 550),
    repair("minecraft:stone_sword", 550),
    repair("minecraft:stone_hoe", 550),
    repair("minecraft:iron_pickaxe", 2250),
    repair("minecraft:iron_axe", 2250),
    repair("minecraft:iron_shovel", 2250),
    repair("minecraft:iron_hoe", 2250),
    repair("minecraft:iron_sword", 550),
    repair("minecraft:diamond_pickaxe", 5500),
    repair("minecraft:diamond_axe", 5500),
    repair("minecraft:diamond_shovel", 5500),
    repair("minecraft:diamond_sword", 5500),
    repair("minecraft:diamond_hoe", 5500),
    repair("minecraft:leather_helmet", 550),
    repair("minecraft:leather_chestplate", 550),
    repair("minecraft:leather_leggings", 550),
    repair("minecraft:leather_boots", 550),
    repair("minecraft:iron_helmet", 6000),
    repair("minecraft:iron_chestplate", 6000),
    repair("minecraft:iron_leggings", 6000),
    repair("minecraft:iron_boots", 6000),
    repair("minecraft:golden_helmet", 2250),
    repair("minecraft:golden_chestplate", 2250),
    repair("minecraft:golden_leggings", 2250),
    repair("minecraft:golden_boots", 2250),
    repair("minecraft:chainmail_helmet", 2250),
    repair("minecraft:chainmail_chestplate", 2250),
    repair("minecraft:chainmail_leggings", 2250),
    repair("minecraft:chainmail_boots", 2250),
    repair("minecraft:diamond_helmet", 10000),
    repair("minecraft:diamond_chestplate", 10000),
    repair("minecraft:diamond_leggings", 10000),
    repair("minecraft:diamond_boots", 10000),
];

// AetherRegistries.initializeFreezables rows whose items exist in b1.7.3 and this port
const FREEZING: [Recipe; 11] = [
    turn(block::COLD_AERCLOUD, block::BLUE_AERCLOUD, 100),
    turn(block::SKYROOT_LEAVES, block::CRYSTAL_LEAVES, 150),
    turn(block::GOLDEN_OAK_LEAVES, block::CRYSTAL_LEAVES, 150),
    turn("minecraft:water_bucket", "minecraft:ice", 500),
    turn("minecraft:lava_bucket", "minecraft:obsidian", 500),
    turn("skyroot_water_bucket", "minecraft:ice", 500),
    turn("ascending_dawn", "welcoming_skies", 2500),
    turn("iron_ring", "ice_ring", 2500),
    turn("golden_ring", "ice_ring", 2500),
    turn("iron_pendant", "ice_pendant", 2500),
    turn("golden_pendant", "ice_pendant", 2500),
];

const AMBROSIUM_TORCH: &str = "ambrosium_torch";
const ALTAR_SHEET: &str = "aether:gui/altar";
const INCUBATOR_SHEET: &str = "aether:gui/incubator";
const INCUBATOR: &str = "incubator";
const INCUBATOR_TITLE: &str = "Incubator";
const INCUBATOR_FACES: [&str; 3] = ["incubator_top", "enchanter_bottom", "incubator_side"];

mod slot
{
    pub const INPUT: i32 = 0;
    pub const FUEL: i32 = 1;
    pub const OUTPUT: i32 = 2;
    pub const MACHINE: i32 = 3;
    // TileEntityIncubator keeps the torch first and the egg second
    pub const TORCH: i32 = 0;
    pub const EGG: i32 = 1;
    pub const MAX_STACK: i32 = 64;
}

mod lit
{
    // BlockAetherContainer.setState as a named state
    pub const PROPERTY: &str = "lit";
    // randomDisplayTick puffs rise up to six pixels above the top face
    pub const PUFF_SPREAD: f64 = 6.0 / 16.0;
    pub const CENTER: f64 = 0.5;
    pub const TOP: f64 = 1.0;
}

mod machine
{
    pub const HARDNESS: f32 = 2.0;
    pub const HARVEST_LEVEL: i32 = 0;
    pub const STEP: i32 = 1;
    pub const FUEL_TICKS: i32 = 500;
    // (int)(progress + 2.5F) in TileEntityEnchanter adds two
    pub const ENCHANTER_BOOST: i32 = 2;
    pub const HATCH_TICKS: i32 = 5700;
    pub const TORCH_TICKS: i32 = 1000;
    pub const HATCH_LIFT: f64 = 0.5;
}

mod gui
{
    pub const WIDTH: i32 = 176;
    pub const HEIGHT: i32 = 166;
    pub const TITLE_Y: i32 = 6;
    pub const TEXT: u32 = 4210752;
    pub const INVENTORY: [i32; 2] = [8, 84];
    pub const INVENTORY_LABEL_Y: i32 = 72;
    // GuiEnchanter and GuiFreezer on altar.png
    pub const INPUT: [i32; 2] = [56, 17];
    pub const FUEL: [i32; 2] = [56, 53];
    pub const OUTPUT: [i32; 2] = [116, 35];
    pub const FLAME: [i32; 4] = [56, 36, 176, 12];
    pub const FLAME_WIDTH: i32 = 14;
    pub const FLAME_PAD: i32 = 2;
    pub const ARROW: [i32; 4] = [79, 34, 176, 14];
    pub const ARROW_LENGTH: i32 = 24;
    pub const ARROW_HEIGHT: i32 = 16;
    // GuiIncubator on incubator.png
    pub const EGG: [i32; 2] = [73, 17];
    pub const TORCH: [i32; 2] = [73, 53];
    pub const TORCH_FLAME: [i32; 4] = [74, 47, 176, 12];
    pub const BAR: [i32; 4] = [103, 70, 179, 70];
    pub const BAR_WIDTH: i32 = 10;
    pub const BAR_LENGTH: i32 = 54;
}

// the furnace style machines, TileEntityEnchanter and TileEntityFreezer share every line but their tables
struct Machine
{
    m_name: &'static str,
    m_title: &'static str,
    m_faces: [&'static str; 3],
    m_recipes: &'static [Recipe],
    m_fuel: &'static str,
    m_booster: Option<&'static str>,
    m_particle: &'static str,
}

const ENCHANTER: Machine = Machine
{
    m_name: "enchanter",
    m_title: "Enchanter",
    m_faces: ["enchanter_bottom", "enchanter_bottom", "enchanter_side"],
    m_recipes: &ENCHANTING,
    m_fuel: item::AMBROSIUM_SHARD,
    m_booster: Some(block::ENCHANTED_GRAVITITE),
    m_particle: "flame",
};

// TileEntityFreezer checks for icestone at its own position, which never holds, so it has no booster
const FREEZER: Machine = Machine
{
    m_name: "freezer",
    m_title: "Freezer",
    m_faces: ["freezer_top", "enchanter_bottom", "freezer_side"],
    m_recipes: &FREEZING,
    m_fuel: block::ICESTONE,
    m_booster: None,
    m_particle: "snowshovel",
};

#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq)]
struct Work
{
    m_progress: i32,
    m_required: i32,
    m_power: i32,
    m_recipe: Option<u16>,
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq)]
struct Hatch
{
    m_progress: i32,
    m_power: i32,
}

// the machine's job for what sits in one slot, from every mod's jobs in the machine registry
fn jobFor(machine: &str, stack: &ItemStack) -> Option<JobMatch> { matchJob(machine, &[Some(stack.clone())], &[]) }

// a job's one output, or the input itself handed back mended
fn output(job: &JobMatch, input: &ItemStack) -> Option<ItemStack>
{
    let made: &JobOutput = job.outputs.first()?;
    match made.from_input
    {
        Some(_) => Some(ItemStack { count: made.count, damage: made.damage.unwrap_or(input.damage), ..input.clone() }),
        None => made.stack.clone().map(ItemStack::fromRaw),
    }
}

// repaired tools never stack, so they only go into an empty output slot
fn fits(job: &JobMatch, result: &ItemStack, current: &Option<ItemStack>) -> bool
{
    let returned: bool = job.outputs.first().is_some_and(|made| made.from_input.is_some());
    match current
    {
        None => true,

        Some(stack) =>
        {
            !returned && stack.item == result.item && stack.damage == result.damage && stack.count < slot::MAX_STACK
        }
    }
}

fn takeOne(stack: ItemStack) -> Option<ItemStack>
{
    (stack.count > 1).then(|| ItemStack { count: stack.count - 1, ..stack })
}

fn setLit(ctx: &mut BlockCtx, was: bool, now: bool)
{
    if was != now
    {
        crate::names::setProperty(ctx, lit::PROPERTY, now);
    }
}

fn finish(world: World, pos: BlockPos, job: &JobMatch, input: &ItemStack)
{
    let Some(result) = output(job, input)
    else
    {
        return;
    };

    let merged: ItemStack = match world.tileSlot(pos, slot::OUTPUT)
    {
        Some(existing) => ItemStack { count: existing.count + result.count, ..existing },
        None => result,
    };
    let _ = world.setTileSlot(pos, slot::OUTPUT, Some(merged));
    // Item.getContainerItem: a bucket going in leaves its empty bucket behind
    let left: Option<ItemStack> = containerItem(input).or_else(|| takeOne(input.clone()));
    let _ = world.setTileSlot(pos, slot::INPUT, left);
}

fn refuel(machine: &Machine, world: World, pos: BlockPos, state: &mut Work)
{
    let fuel: Option<ItemStack> = world.tileSlot(pos, slot::FUEL);
    let ticks: i32 = fuel.as_ref().map_or(0, |fuel| machineFuel(machine.m_name, fuel, false));
    match fuel.filter(|_| ticks > 0)
    {
        Some(fuel) =>
        {
            state.m_power += ticks;

            let _ = world.setTileSlot(pos, slot::FUEL, takeOne(fuel));
        }

        None =>
        {
            state.m_recipe = None;
            state.m_progress = 0;
        }
    }
}

// TileEntityEnchanter.updateEntity and TileEntityFreezer.updateEntity
fn work(machine: &Machine, ctx: &mut BlockCtx)
{
    if !ctx.isAuthority()
    {
        return;
    }

    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    let before: Work = ctx.tileData();
    let mut state: Work = before;
    if state.m_power > 0
    {
        state.m_power -= machine::STEP;

        if state.m_recipe.is_some()
        {
            let below: BlockPos = BlockPos::new(pos.x, pos.y - 1, pos.z);
            let boosted: bool = machine.m_booster.is_some_and(|name| world.is(below, name));
            state.m_progress += if boosted
            {
                machine::ENCHANTER_BOOST
            }
            else
            {
                machine::STEP
            };
        }
    }

    let input: Option<ItemStack> = world.tileSlot(pos, slot::INPUT);

    match state.m_recipe
    {
        Some(index) =>
        {
            let job: Option<JobMatch> = input.as_ref().and_then(|stack| jobFor(machine.m_name, stack));
            let done: bool = state.m_progress >= state.m_required;
            if done
                && let (Some(stack), Some(job)) = (&input, &job)
                && job.index == u32::from(index)
            {
                finish(world, pos, job, stack);
                state.m_progress = 0;
            }

            let current: Option<ItemStack> = world.tileSlot(pos, slot::INPUT);
            let still: bool = current.as_ref().and_then(|stack| jobFor(machine.m_name, stack)).map(|job| job.index)
                == Some(u32::from(index));
            if !still
            {
                state.m_recipe = None;
                state.m_progress = 0;
            }

            if state.m_power <= 0
            {
                refuel(machine, world, pos, &mut state);
            }
        }

        None =>
        {
            if let Some(stack) = &input
                && let Some(job) = jobFor(machine.m_name, stack)
                && let Some(result) = output(&job, stack)
                && fits(&job, &result, &world.tileSlot(pos, slot::OUTPUT))
            {
                state.m_recipe = u16::try_from(job.index).ok();
                state.m_required = job.duration;
            }
        }
    }

    setLit(ctx, before.m_recipe.is_some(), state.m_recipe.is_some());

    if state != before
    {
        let _ = ctx.setTileData(&state);
    }
}

fn hatchEgg(world: World, pos: BlockPos, egg: ItemStack)
{
    let mut above: i32 = 0;
    while !world.isAir(BlockPos::new(pos.x, pos.y + above, pos.z))
    {
        above += 1;
    }

    let spot: [f64; 3] =
        [pos.x as f64 + lit::CENTER, (pos.y + above) as f64 + machine::HATCH_LIFT, pos.z as f64 + lit::CENTER];
    moa::hatch(world, &egg, spot);

    if let Some(owner) = progress::owner(world, pos)
    {
        progress::grant(owner, Award::Incubator);
    }

    let _ = world.setTileSlot(pos, slot::EGG, takeOne(egg));
}

// TileEntityIncubator.updateEntity
fn incubate(ctx: &mut BlockCtx)
{
    if !ctx.isAuthority()
    {
        return;
    }

    let world: World = ctx.world();
    let pos: BlockPos = ctx.pos();
    let before: Hatch = ctx.tileData();
    let mut state: Hatch = before;
    let egg: Option<ItemStack> = world.tileSlot(pos, slot::EGG);

    if state.m_power > 0
    {
        state.m_power -= machine::STEP;

        if egg.is_some()
        {
            state.m_progress += machine::STEP;
        }
    }

    // onSlotChanged resets the progress when the egg slot changes, an emptied slot is what the loader shows of that
    match egg
    {
        None => state.m_progress = 0,

        Some(stack) if jobFor(INCUBATOR, &stack).is_some_and(|job| state.m_progress >= job.duration) =>
        {
            hatchEgg(world, pos, stack);
            state.m_progress = 0;
        }

        Some(_) =>
        {}
    }

    if state.m_power <= 0
    {
        let torch: Option<ItemStack> = world.tileSlot(pos, slot::TORCH);
        let ticks: i32 = torch.as_ref().map_or(0, |torch| machineFuel(INCUBATOR, torch, false));
        let fertile: bool = world.tileSlot(pos, slot::EGG).is_some_and(|stack| jobFor(INCUBATOR, &stack).is_some());
        match torch.filter(|_| fertile && ticks > 0)
        {
            Some(torch) =>
            {
                state.m_power += ticks;

                let _ = world.setTileSlot(pos, slot::TORCH, takeOne(torch));
            }

            None =>
            {
                state.m_power = 0;
                state.m_progress = 0;
            }
        }
    }

    setLit(ctx, before.m_power > 0, state.m_power > 0);

    if state != before
    {
        let _ = ctx.setTileData(&state);
    }
}

struct MachineTile
{
    m_machine: &'static Machine,
}

#[tile_behavior]
impl TileBehavior for MachineTile
{
    fn onTick(&self, ctx: &mut BlockCtx) { work(self.m_machine, ctx); }
}

struct IncubatorTile;

#[tile_behavior]
impl TileBehavior for IncubatorTile
{
    fn onTick(&self, ctx: &mut BlockCtx) { incubate(ctx); }
}

// BlockAetherContainer subclasses: open the screen, puff smoke and the machine's particle while lit
struct ContainerBlock
{
    m_screen: &'static str,
    m_particle: &'static str,
}

#[block_behavior]
impl BlockBehavior for ContainerBlock
{
    fn onActivated(&self, ctx: &mut BlockCtx, player: Player) -> bool
    {
        if ctx.isAuthority()
        {
            if self.m_screen == INCUBATOR
            {
                progress::setOwner(ctx.world(), ctx.pos(), player);
            }

            let _ = player.openScreen(self.m_screen, Some(ctx.pos()), Vec::new());
        }

        true
    }

    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.stateAt().is_some_and(|state| state.flag(lit::PROPERTY))
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let spot: [f64; 3] = [
            pos.x as f64 + lit::CENTER,
            pos.y as f64 + lit::TOP + fraction(world) * lit::PUFF_SPREAD,
            pos.z as f64 + lit::CENTER,
        ];
        world.particle("smoke", spot, [0.0; 3]);
        world.particle(self.m_particle, spot, [0.0; 3]);
    }
}

fn tileState<T: serde::de::DeserializeOwned + Default>(pos: Option<BlockPos>) -> Option<T>
{
    let pos: BlockPos = pos?;
    let local: EntityInfo = localPlayer()?.info()?;
    Some(World { dimension: local.world }.tileData(pos))
}

fn labels(draw: &mut Draw, title: &str)
{
    draw.centered(gui::WIDTH / 2, gui::TITLE_Y, title, gui::TEXT);
    draw.text(gui::INVENTORY[0], gui::INVENTORY_LABEL_Y, "Inventory", gui::TEXT);
}

// GuiEnchanter and GuiFreezer: fuel flame and progress arrow from the synced tile
struct MachineScreen
{
    m_machine: &'static Machine,
    m_pos: Option<BlockPos>,
}

impl Screen for MachineScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        labels(draw, self.m_machine.m_title);

        let Some(state) = tileState::<Work>(self.m_pos)
        else
        {
            return;
        };

        if state.m_recipe.is_some() && state.m_power > 0
        {
            let [x, y, u, v] = gui::FLAME;
            let flame: i32 = (state.m_power * v / machine::FUEL_TICKS).min(v);
            draw.texture(ALTAR_SHEET, x, y + v - flame, u, v - flame, gui::FLAME_WIDTH, flame + gui::FLAME_PAD);
        }

        if state.m_required > 0
        {
            let [x, y, u, v] = gui::ARROW;
            let arrow: i32 = state.m_progress * gui::ARROW_LENGTH / state.m_required;
            draw.texture(ALTAR_SHEET, x, y, u, v, arrow + machine::STEP, gui::ARROW_HEIGHT);
        }
    }
}

// GuiIncubator: torch flame and the rising hatch bar
struct IncubatorScreen
{
    m_pos: Option<BlockPos>,
}

impl Screen for IncubatorScreen
{
    fn init(&mut self, _width: i32, _height: i32, pos: Option<BlockPos>, _data: &[u8]) { self.m_pos = pos; }

    fn draw(&mut self, draw: &mut Draw, _mouse: [i32; 2], _partial: f32)
    {
        labels(draw, INCUBATOR_TITLE);

        let Some(state) = tileState::<Hatch>(self.m_pos)
        else
        {
            return;
        };

        if state.m_power > 0
        {
            // getPowerTimeRemainingScaled divides by 500 while a torch gives 1000, the flame is capped at full
            let [x, y, u, v] = gui::TORCH_FLAME;
            let flame: i32 = (state.m_power * v / machine::FUEL_TICKS).min(v);
            draw.texture(INCUBATOR_SHEET, x, y - flame, u, v - flame, gui::FLAME_WIDTH, flame + gui::FLAME_PAD);
        }

        let [x, y, u, v] = gui::BAR;
        let bar: i32 = state.m_progress * gui::BAR_LENGTH / machine::HATCH_TICKS;
        draw.texture(INCUBATOR_SHEET, x, y - bar, u, v - bar, gui::BAR_WIDTH, bar);
    }
}

fn faces(names: [&str; 3]) -> FaceTextures
{
    let [top, bottom, side] = names.map(|name| Some(format!("aether:block/{name}")));
    FaceTextures { top, bottom, side, front: None }
}

fn registerBlock(
    ctx: &mut RegisterCtx,
    name: &'static str,
    names: [&str; 3],
    particle: &'static str,
    tile: impl TileBehavior + 'static,
) -> Result<()>
{
    ctx.blocks().register(
        name,
        BlockSpecBuilder::template("minecraft:block")
            .material("rock")
            .faces(faces(names))
            .hardness(machine::HARDNESS)
            .sound("stone")
            .tool(ToolClass::Pickaxe, machine::HARVEST_LEVEL)
            .tile(Tile::new(tile).inventory(slot::MACHINE).ticks().synced())
            .property(Property::bool(lit::PROPERTY))
            .behavior(ContainerBlock { m_screen: name, m_particle: particle }),
    )?;
    Ok(())
}

fn containerScreen(title: &str) -> ScreenBuilder
{
    ScreenBuilder::new(title, gui::WIDTH, gui::HEIGHT).playerInventory(gui::INVENTORY[0], gui::INVENTORY[1])
}

// every row as a job of the machine registry, which runs it and shows it to recipe viewers
fn registerJobs(ctx: &mut RegisterCtx, machine: &Machine) -> Result<()>
{
    ctx.machines().register(machine.m_name, Some(machine.m_title), &[machine.m_name])?;
    ctx.machines().fuel(machine.m_name, machine.m_fuel, machine::FUEL_TICKS)?;

    for recipe in machine.m_recipes
    {
        let mut job: JobBuilder = ctx.machines().job(machine.m_name, recipe.m_ticks).input(recipe.m_input, 1);
        job = match recipe.m_output
        {
            Some(made) => job.output(made, 1),
            None => job.returns(0, Some(0)).note("Repairs it fully"),
        };

        if machine.m_booster.is_some()
        {
            job = job.note("Faster on enchanted gravitite");
        }

        job.build()?;
    }

    Ok(())
}

fn registerMachine(ctx: &mut RegisterCtx, machine: &'static Machine) -> Result<()>
{
    registerJobs(ctx, machine)?;

    registerBlock(ctx, machine.m_name, machine.m_faces, machine.m_particle, MachineTile { m_machine: machine })?;
    ctx.screens().register(
        machine.m_name,
        containerScreen(machine.m_title)
            .background(ALTAR_SHEET)
            .slot(slot::INPUT, gui::INPUT[0], gui::INPUT[1])
            .slot(slot::FUEL, gui::FUEL[0], gui::FUEL[1])
            .outputSlot(slot::OUTPUT, gui::OUTPUT[0], gui::OUTPUT[1]),
        MachineScreen { m_machine: machine, m_pos: None },
    )
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    registerMachine(ctx, &ENCHANTER)?;
    registerMachine(ctx, &FREEZER)?;
    registerBlock(ctx, INCUBATOR, INCUBATOR_FACES, "flame", IncubatorTile)?;
    ctx.machines().register(INCUBATOR, Some(INCUBATOR_TITLE), &[INCUBATOR])?;
    ctx.machines().fuel(INCUBATOR, AMBROSIUM_TORCH, machine::TORCH_TICKS)?;
    for (kind, egg) in moa::eggs()
    {
        ctx.machines()
            .job(INCUBATOR, machine::HATCH_TICKS)
            .input(&egg, 1)
            .note(&format!("Hatches a {kind} moa"))
            .build()?;
    }
    ctx.screens().register(
        INCUBATOR,
        containerScreen(INCUBATOR_TITLE)
            .background(INCUBATOR_SHEET)
            .slot(slot::TORCH, gui::TORCH[0], gui::TORCH[1])
            .slot(slot::EGG, gui::EGG[0], gui::EGG[1]),
        IncubatorScreen { m_pos: None },
    )
}
