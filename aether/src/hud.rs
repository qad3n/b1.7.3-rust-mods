use b173_api::prelude::*;

const COOLDOWN_BAR: &str = "aether:gui/cooldown_bar";
const JUMPS: &str = "aether:gui/jumps";
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

fn localEntity() -> Option<Entity>
{
    let info: EntityInfo = localPlayer()?.info()?;
    Some(Entity { world: World { dimension: info.world }, id: info.id })
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
fn draw(draw: &mut Draw, _partial: f32, width: i32, _height: i32)
{
    let Some(entity) = localEntity()
    else
    {
        return;
    };

    drawCooldown(draw, entity, width);
    drawJumps(draw, entity, width);
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()> { ctx.screens().hud(draw) }
