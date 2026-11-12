use crate::effects::Effect;
use b173_api::prelude::*;
use std::cell::Cell;

const COOLDOWN_BAR: &str = "aether:gui/cooldown_bar";
const JUMPS: &str = "aether:gui/jumps";
const POISON_VIGNETTE: &str = "aether:blur/poison_vignette";
const CURE_VIGNETTE: &str = "aether:blur/cure_vignette";
const HAMMER: &str = "notch_hammer";
const WHITE: u32 = 0xFFFFFFFF;

// AetherOverlay.renderCooldown
mod cooldown
{
    pub const WIDTH: i32 = 128;
    pub const HEIGHT: i32 = 8;
    pub const TEXT_Y: i32 = 32;
    pub const BAR_Y: i32 = 42;
    // the empty bar sits under the full one on the sheet
    pub const EMPTY_V: i32 = 8;
}

// AetherOverlay.renderJumps
mod jump
{
    pub const Y: i32 = 18;
    pub const STEP: i32 = 8;
    pub const SIZE: [i32; 2] = [9, 11];
    pub const SPENT_U: i32 = 10;
}

// AetherOverlay.renderPoison and renderCure: grey vignettes, the poison one pulsing with each fifty tick beat
mod vignette
{
    pub const POISON_TICKS: i32 = 500;
    pub const BEAT: i32 = 50;
    pub const PULSE: f32 = 5.0;
    pub const FLOOR: f32 = 0.4;
    pub const CURE_ALPHA: f32 = 0.5;
    pub const GREY: u32 = 0x808080;
    pub const OPAQUE: f32 = 255.0;
}

thread_local!
{
    // PlayerAether.poisonTime and cureTime of the local player
    static POISON: Cell<i32> = const { Cell::new(0) };
    static CURE: Cell<i32> = const { Cell::new(0) };
}

// PlayerAether.setCured: the remedy bucket and the white apple tint the view for a while
pub(crate) fn cured(player: Player, ticks: i32)
{
    if localPlayer().is_some_and(|local| local.id == player.id)
    {
        CURE.set(ticks);
    }
}

fn localEntity() -> Option<Entity>
{
    let info: EntityInfo = localPlayer()?.info()?;
    Some(Entity { world: World { dimension: info.world }, id: info.id })
}

fn tick()
{
    CURE.set((CURE.get() - 1).max(0));

    let poisoned: bool = localEntity().is_some_and(|entity| crate::effects::has(entity, Effect::Inebriation));
    let left: i32 = POISON.get();
    let next: i32 = match (poisoned, left)
    {
        (false, _) => 0,
        (true, 0) => vignette::POISON_TICKS,
        (true, left) => (left - 1).max(1),
    };
    POISON.set(next);
}

fn tint(alpha: f32) -> u32 { ((alpha * vignette::OPAQUE) as u32) << 24 | vignette::GREY }

fn drawVignettes(draw: &mut Draw, width: i32, height: i32)
{
    if CURE.get() > 0
    {
        draw.tinted(CURE_VIGNETTE, 0, 0, width, height, tint(vignette::CURE_ALPHA));
    }

    let poison: i32 = POISON.get();
    if poison > 0
    {
        let beat: f32 = (poison % vignette::BEAT) as f32 / vignette::BEAT as f32;
        draw.tinted(POISON_VIGNETTE, 0, 0, width, height, tint(beat * beat / vignette::PULSE + vignette::FLOOR));
    }
}

fn drawCooldown(draw: &mut Draw, entity: Entity, width: i32)
{
    let left: f32 = crate::weapons::hammerCooldown(entity);
    if left <= 0.0
    {
        return;
    }

    let label: String = format!(
        "{} {}",
        translate(&format!("item.aether.{HAMMER}.name")),
        translate(&format!("item.aether.{HAMMER}.cooldown"))
    );
    let x: i32 = width / 2 - cooldown::WIDTH / 2;
    draw.centered(width / 2, cooldown::TEXT_Y, &label, WHITE);
    draw.texture(COOLDOWN_BAR, x, cooldown::BAR_Y, 0, cooldown::EMPTY_V, cooldown::WIDTH, cooldown::HEIGHT);
    draw.texture(COOLDOWN_BAR, x, cooldown::BAR_Y, 0, 0, (left * cooldown::WIDTH as f32) as i32, cooldown::HEIGHT);
}

fn drawJumps(draw: &mut Draw, entity: Entity, width: i32)
{
    let Some((most, left)) =
        entity.riding().and_then(|id| crate::entities::moa::jumps(Entity { world: entity.world, id }))
    else
    {
        return;
    };

    for index in 0..most
    {
        let x: i32 = width / 2 + index * jump::STEP - most * jump::STEP / 2;
        let u: i32 = if index < left { 0 } else { jump::SPENT_U };
        draw.texture(JUMPS, x, jump::Y, u, 0, jump::SIZE[0], jump::SIZE[1]);
    }
}

// GuiAetherInGame.onRenderGui
fn draw(draw: &mut Draw, _partial: f32, width: i32, height: i32)
{
    let Some(entity) = localEntity()
    else
    {
        return;
    };

    drawVignettes(draw, width, height);
    drawCooldown(draw, entity, width);
    drawJumps(draw, entity, width);
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<ClientTick>(Priority::Normal, |_| tick())?;
    ctx.screens().hud(draw)
}
