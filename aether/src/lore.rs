use b173_api::prelude::*;

const NAME: &str = "lore";
const BOOK: &str = "lore_book";
const PAGE: &str = "aether:gui/lore";
const COVER: &str = "aether:gui/lore_book";
const WIDGETS: &str = "aether:gui/lore_widgets";
const TEXT_COLOR: u32 = 0x404040;

mod layout
{
    // GuiLore xSize and ySize
    pub const WIDTH: i32 = 256;
    pub const HEIGHT: i32 = 195;
    pub const LORE_SLOT: [i32; 2] = [104, -4];
    pub const INVENTORY: [i32; 2] = [48, 113];
    pub const WRAP: i32 = 109;
    pub const LINES_PER_PAGE: i32 = 15;
    pub const LEFT_PAGE_LINES: i32 = 6;
    pub const LEFT_CENTER: i32 = 71;
    pub const RIGHT_CENTER: i32 = 184;
    pub const LEFT_TOP: i32 = 28;
    pub const RIGHT_TOP: i32 = -68;
    pub const NAME_TOP: i32 = 18;
    pub const LINE: i32 = 10;
}

mod button
{
    pub const SIZE: i32 = 20;
    pub const PREVIOUS: [i32; 2] = [-110, 72];
    pub const NEXT: [i32; 2] = [90, 72];
    // GuiButton rows of lore_widgets.png: disabled, normal, hovered
    pub const ROW_V: i32 = 46;
    pub const RIGHT_U: i32 = 200;
    pub const TEXT_Y: i32 = (SIZE - 8) / 2;
    pub const COLOR: u32 = 0xE0E0E0;
    pub const DISABLED_COLOR: u32 = 0xA0A0A0;
    pub const HOVER_COLOR: u32 = 0xFFFFA0;
}

// GuiLore.drawGuiContainerBackgroundLayer: the page stretched from a small sheet, then the cover and its spine
mod art
{
    use super::{COVER, PAGE};

    type Layer = (&'static str, [i32; 4], [f32; 4], [f32; 2]);

    pub const LAYERS: [Layer; 3] = [
        (PAGE, [0, -4, 256, 256], [0.0, 0.0, 120.0, 120.0], [120.0, 120.0]),
        (COVER, [-1, -20, 276, 135], [0.0, 0.0, 276.0, 135.0], [275.0, 315.0]),
        (COVER, [90, -5, 276, 18], [0.0, 225.0, 276.0, 18.0], [500.0, 500.0]),
    ];
}

// the labels GuiLore.drawGuiContainerForegroundLayer prints around the book
const LABELS: [(&str, i32, i32); 5] =
    [("Prev.", 16, 160), ("Next", 218, 160), ("Book", 32, -5), ("of Lore:", 24, 4), ("Item :", 75, 0)];

// AetherLore.getLoreEntryKey reads the 1.7.10 unlocalized names, which differ from this port's registry names
const LORE_KEYS: [(&str, &str); 16] = [
    ("golden_dart", "dart_golden"),
    ("poison_dart", "dart_poison"),
    ("enchanted_dart", "dart_enchanted"),
    ("golden_dart_shooter", "dart_shooter_golden"),
    ("poison_dart_shooter", "dart_shooter_poison"),
    ("enchanted_dart_shooter", "dart_shooter_enchanted"),
    ("blue_gummy_swet", "gummy_swet_blue"),
    ("golden_gummy_swet", "gummy_swet_golden"),
    ("skyroot_bucket", "skyroot_bucket_empty"),
    ("skyroot_water_bucket", "skyroot_bucket_water"),
    ("skyroot_poison_bucket", "skyroot_bucket_poison"),
    ("skyroot_remedy_bucket", "skyroot_bucket_remedy"),
    ("skyroot_milk_bucket", "skyroot_bucket_milk"),
    ("bronze_key", "dungeon_key_bronze"),
    ("silver_key", "dungeon_key_silver"),
    ("golden_key", "dungeon_key_golden"),
];

const LOOT_COLOR: &str = "\u{a7}a";
const COMMON_COLOR: &str = "\u{a7}f";

fn loreKey(stack: &ItemStack) -> String
{
    let Some(id) = stack.item.name()
    else
    {
        return String::new();
    };
    if id.isVanilla()
    {
        return format!("lore.minecraft.{}", id.path());
    }

    let path: &str = id.path();
    let key: &str = LORE_KEYS.iter().find(|(name, _)| *name == path).map_or(path, |(_, key)| key);
    format!("aether.lore.{key}")
}

// FontRenderer.listFormattedStringToWidth: words fill a line up to the width, a word too long for any line is cut
fn wrap(text: &str, width: i32) -> Vec<String>
{
    let mut lines: Vec<String> = Vec::new();
    let mut line: String = String::new();

    for word in text.split(' ')
    {
        let candidate: String = if line.is_empty()
        {
            word.to_owned()
        }
        else
        {
            format!("{line} {word}")
        };

        if textWidth(&candidate) <= width
        {
            line = candidate;
            continue;
        }

        if !line.is_empty()
        {
            lines.push(std::mem::take(&mut line));
        }

        let mut rest: String = word.to_owned();
        while textWidth(&rest) > width && rest.chars().count() > 1
        {
            let fit: usize = (1..rest.chars().count())
                .rev()
                .find(|count| textWidth(&rest.chars().take(*count).collect::<String>()) <= width)
                .unwrap_or(1);
            lines.push(rest.chars().take(fit).collect());
            rest = rest.chars().skip(fit).collect();
        }

        line = rest;
    }

    if !line.is_empty()
    {
        lines.push(line);
    }

    lines
}

struct LoreBook;

#[item_behavior]
impl ItemBehavior for LoreBook
{
    // ItemLoreBook.onItemRightClick opens the book with an empty slot
    fn onRightClick(&self, world: World, player: Player, _stack: ItemStack) -> Use
    {
        if world.isAuthority()
        {
            let _ = player.openScreen(NAME, None, Vec::new());

            crate::progress::read(player);
        }

        Use::Pass
    }
}

#[derive(Default)]
struct LoreScreen
{
    m_previous: [i32; 2],
    m_next: [i32; 2],
    m_page: i32,
    m_current: Option<ItemStack>,
    m_text: String,
    m_hasPrevious: bool,
    m_hasNext: bool,
}

fn inside(corner: [i32; 2], x: i32, y: i32) -> bool
{
    x >= corner[0] && y >= corner[1] && x < corner[0] + button::SIZE && y < corner[1] + button::SIZE
}

// GuiLoreButton: a vanilla button drawn from the lore widget sheet
fn drawButton(draw: &mut Draw, corner: [i32; 2], label: &str, enabled: bool, mouse: [i32; 2])
{
    let hovered: bool = inside(corner, mouse[0], mouse[1]);
    let state: i32 = match (enabled, hovered)
    {
        (false, _) => 0,
        (true, false) => 1,
        (true, true) => 2,
    };

    let [x, y] = corner;
    let half: i32 = button::SIZE / 2;
    let v: i32 = button::ROW_V + state * button::SIZE;
    draw.texture(WIDGETS, x, y, 0, v, half, button::SIZE);
    draw.texture(WIDGETS, x + half, y, button::RIGHT_U - half, v, half, button::SIZE);

    let color: u32 = match state
    {
        0 => button::DISABLED_COLOR,
        2 => button::HOVER_COLOR,
        _ => button::COLOR,
    };
    draw.centered(x + half, y + button::TEXT_Y, label, color);
}

impl LoreScreen
{
    // GuiLore.drawGuiContainerForegroundLayer: a new item starts on the first page
    fn follow(&mut self, stack: Option<ItemStack>)
    {
        match stack
        {
            Some(stack) =>
            {
                let same: bool = self
                    .m_current
                    .as_ref()
                    .is_some_and(|current| current.item == stack.item && current.damage == stack.damage);

                if !same
                {
                    self.m_page = 0;
                    self.m_text = translate(&loreKey(&stack));
                    self.m_current = Some(stack);
                }
            }

            None =>
            {
                self.m_page = 0;
                self.m_text.clear();
                self.m_current = None;
            }
        }
    }

    fn drawLore(&mut self, draw: &mut Draw, stack: &ItemStack)
    {
        let loot: bool = stack.item.tagged(crate::tags::DUNGEON_LOOT);
        let color: &str = if loot { LOOT_COLOR } else { COMMON_COLOR };
        let names: Vec<String> = wrap(&displayName(stack), layout::WRAP);
        for (row, name) in names.iter().enumerate()
        {
            draw.centered(
                layout::LEFT_CENTER,
                layout::NAME_TOP + layout::LINE * row as i32,
                &format!("{color}{name}"),
                TEXT_COLOR,
            );
        }

        let nameLines: i32 = names.len() as i32 - 1;
        let lines: Vec<String> = wrap(&self.m_text, layout::WRAP);
        let perPage: i32 = layout::LINES_PER_PAGE;

        for (index, line) in lines.iter().enumerate()
        {
            let size: i32 = index as i32;
            if size < perPage * self.m_page || size + nameLines > perPage * (self.m_page + 1)
            {
                continue;
            }

            let row: i32 = if self.m_page >= 1
            {
                size - (perPage - nameLines) * self.m_page
            }
            else
            {
                size + nameLines
            };
            let right: bool = row >= layout::LEFT_PAGE_LINES;
            let center: i32 = if right
            {
                layout::RIGHT_CENTER
            }
            else
            {
                layout::LEFT_CENTER
            };

            let top: i32 = if right
            {
                layout::RIGHT_TOP
            }
            else
            {
                layout::LEFT_TOP
            };
            draw.plain(center - textWidth(line) / 2, top + layout::LINE * row, line, TEXT_COLOR);
        }

        self.m_hasPrevious = self.m_page != 0;
        self.m_hasNext = lines.len() as i32 > perPage * (self.m_page + 1);
    }
}

impl Screen for LoreScreen
{
    fn init(&mut self, width: i32, height: i32, _pos: Option<BlockPos>, _data: &[u8])
    {
        let origin: [i32; 2] = [(width - layout::WIDTH) / 2, (height - layout::HEIGHT) / 2];
        let at = |offset: [i32; 2]| [width / 2 + offset[0] - origin[0], height / 2 + offset[1] - origin[1]];
        self.m_previous = at(button::PREVIOUS);
        self.m_next = at(button::NEXT);
        self.m_page = 0;
        self.m_current = None;
    }

    // GuiLore.drawGuiContainerBackgroundLayer, its buttons and its foreground text
    fn draw(&mut self, draw: &mut Draw, mouse: [i32; 2], _partial: f32)
    {
        for (texture, rect, uv, size) in art::LAYERS
        {
            draw.region(texture, rect, uv, size);
        }

        let stack: Option<ItemStack> = localPlayer().and_then(|player| player.openSlot(0));
        self.follow(stack);

        match self.m_current.clone()
        {
            Some(stack) => self.drawLore(draw, &stack),

            None =>
            {
                self.m_hasPrevious = false;
                self.m_hasNext = false;
            }
        }

        drawButton(draw, self.m_previous, "<", self.m_hasPrevious, mouse);
        drawButton(draw, self.m_next, ">", self.m_hasNext, mouse);

        for (label, x, y) in LABELS
        {
            draw.plain(x, y, label, TEXT_COLOR);
        }
    }

    fn mouse(&mut self, x: i32, y: i32, button: i32)
    {
        if button != 0
        {
            return;
        }

        if self.m_hasPrevious && inside(self.m_previous, x, y)
        {
            self.m_page -= 1;
        }
        else if self.m_hasNext && inside(self.m_next, x, y)
        {
            self.m_page += 1;
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    let [slotX, slotY] = layout::LORE_SLOT;
    let [inventoryX, inventoryY] = layout::INVENTORY;
    ctx.screens().register(
        NAME,
        ScreenBuilder::new("", layout::WIDTH, layout::HEIGHT)
            .slot(0, slotX, slotY)
            .playerInventory(inventoryX, inventoryY),
        LoreScreen::default(),
    )?;
    ctx.items()
        .register(BOOK, ItemSpecBuilder::new().texture("aether:item/misc/lore_book").maxStack(1).behavior(LoreBook))?;
    Ok(())
}
