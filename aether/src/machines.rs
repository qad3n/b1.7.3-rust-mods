use crate::entities::{fraction, moa};
use crate::ids::{block, blockId, item};
use b173_api::prelude::*;
use b173_api::screen::Draw;
use std::rc::Rc;

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
const ENCHANTING: [Recipe; 76] = [
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

// Item.getContainerItem: a bucket going in leaves its empty bucket behind
const CONTAINERS: [(&str, &str); 3] = [
    ("minecraft:water_bucket", "minecraft:bucket"),
    ("minecraft:lava_bucket", "minecraft:bucket"),
    ("skyroot_water_bucket", "skyroot_bucket"),
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
    // BlockAetherContainer.setState
    pub const IDLE: u8 = 0;
    pub const ACTIVE: u8 = 1;
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

const ENCHANTER: Machine = Machine {
    m_name: "enchanter",
    m_title: "Enchanter",
    m_faces: ["enchanter_bottom", "enchanter_bottom", "enchanter_side"],
    m_recipes: &ENCHANTING,
    m_fuel: item::AMBROSIUM_SHARD,
    m_booster: Some(block::ENCHANTED_GRAVITITE),
    m_particle: "flame",
};

// TileEntityFreezer checks for icestone at its own position, which never holds, so it has no booster
const FREEZER: Machine = Machine {
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

fn recipeFor(machine: &Machine, stack: &ItemStack) -> Option<u16>
{
    machine.m_recipes.iter().position(|recipe| stack.item.is(recipe.m_input)).map(|index| index as u16)
}

fn output(recipe: &Recipe, input: &ItemStack) -> ItemStack
{
    match recipe.m_output
    {
        Some(name) => ItemStack::of(name, 1),
        None => ItemStack { count: 1, damage: 0, ..input.clone() },
    }
}

// repaired tools never stack, so they only go into an empty output slot
fn fits(recipe: &Recipe, result: &ItemStack, current: &Option<ItemStack>) -> bool
{
    match current
    {
        None => true,
        Some(stack) =>
        {
            recipe.m_output.is_some()
                && stack.item == result.item
                && stack.damage == result.damage
                && stack.count < slot::MAX_STACK
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
        ctx.setMeta(if now { lit::ACTIVE } else { lit::IDLE });
    }
}

fn finish(world: World, pos: BlockPos, recipe: &Recipe, input: &ItemStack)
{
    let result: ItemStack = output(recipe, input);
    let merged: ItemStack = match world.tileSlot(pos, slot::OUTPUT)
    {
        Some(existing) => ItemStack { count: existing.count + result.count, ..existing },
        None => result,
    };
    let _ = world.setTileSlot(pos, slot::OUTPUT, Some(merged));
    let left: Option<ItemStack> = match CONTAINERS.iter().find(|(full, _)| input.item.is(full))
    {
        Some((_, empty)) => Some(ItemStack::of(*empty, 1)),
        None => takeOne(input.clone()),
    };
    let _ = world.setTileSlot(pos, slot::INPUT, left);
}

fn refuel(machine: &Machine, world: World, pos: BlockPos, state: &mut Work)
{
    match world.tileSlot(pos, slot::FUEL).filter(|fuel| fuel.item.is(machine.m_fuel))
    {
        Some(fuel) =>
        {
            state.m_power += machine::FUEL_TICKS;
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
            let boosted: bool = machine.m_booster.is_some_and(|name| world.block(below) == blockId(name));
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
            let recipe: &Recipe = &machine.m_recipes[index as usize];
            if state.m_progress >= state.m_required
                && let Some(stack) = &input
            {
                finish(world, pos, recipe, stack);
                state.m_progress = 0;
            }
            let current: Option<ItemStack> = world.tileSlot(pos, slot::INPUT);
            if current.as_ref().and_then(|stack| recipeFor(machine, stack)) != Some(index)
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
                && let Some(index) = recipeFor(machine, stack)
            {
                let recipe: &Recipe = &machine.m_recipes[index as usize];
                if fits(recipe, &output(recipe, stack), &world.tileSlot(pos, slot::OUTPUT))
                {
                    state.m_recipe = Some(index);
                    state.m_required = recipe.m_ticks;
                }
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
        Some(stack) if state.m_progress >= machine::HATCH_TICKS =>
        {
            hatchEgg(world, pos, stack);
            state.m_progress = 0;
        }
        Some(_) =>
        {}
    }
    if state.m_power <= 0
    {
        let torch: Option<ItemStack> = world.tileSlot(pos, slot::TORCH).filter(|torch| torch.item.is(AMBROSIUM_TORCH));
        let fertile: bool = world.tileSlot(pos, slot::EGG).is_some_and(|stack| moa::isEgg(&stack));
        match torch.filter(|_| fertile)
        {
            Some(torch) =>
            {
                state.m_power += machine::TORCH_TICKS;
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
            let _ = player.openScreen(self.m_screen, Some(ctx.pos()), Vec::new());
        }
        true
    }

    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        if ctx.meta() != lit::ACTIVE
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
    tile: Rc<dyn TileBehavior>,
) -> Result<()>
{
    let spec: TileEntitySpec = TileEntitySpec { inventory: slot::MACHINE, ticks: true, sync: true, behavior: None };
    ctx.tileEntities().register(name, spec, Some(tile))?;
    ctx.blocks().register(
        name,
        BlockSpecBuilder::template("minecraft:block")
            .material("rock")
            .faces(faces(names))
            .hardness(machine::HARDNESS)
            .sound("stone")
            .tool(ToolClass::Pickaxe, machine::HARVEST_LEVEL)
            .tileEntity(name)
            .behavior(ContainerBlock { m_screen: name, m_particle: particle }),
    )?;
    Ok(())
}

fn containerScreen(title: &str) -> ScreenBuilder
{
    ScreenBuilder::new(title, gui::WIDTH, gui::HEIGHT).playerInventory(gui::INVENTORY[0], gui::INVENTORY[1])
}

fn registerMachine(ctx: &mut RegisterCtx, machine: &'static Machine) -> Result<()>
{
    registerBlock(
        ctx,
        machine.m_name,
        machine.m_faces,
        machine.m_particle,
        Rc::new(MachineTile { m_machine: machine }),
    )?;
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

pub fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    registerMachine(ctx, &ENCHANTER)?;
    registerMachine(ctx, &FREEZER)?;
    registerBlock(ctx, INCUBATOR, INCUBATOR_FACES, "flame", Rc::new(IncubatorTile))?;
    ctx.screens().register(
        INCUBATOR,
        containerScreen(INCUBATOR_TITLE)
            .background(INCUBATOR_SHEET)
            .slot(slot::TORCH, gui::TORCH[0], gui::TORCH[1])
            .slot(slot::EGG, gui::EGG[0], gui::EGG[1]),
        IncubatorScreen { m_pos: None },
    )
}
