use crate::portal;
use b173_api::prelude::*;
use std::cell::Cell;

// PlayerAether.timeInPortal steps and AetherOverlay.renderAetherPortal's fade curve
mod fade
{
    pub const RISE: f32 = 0.0125;
    pub const FALL: f32 = 0.05;
    pub const FLOOR: f32 = 0.2;
    pub const SPAN: f32 = 0.8;
    pub const OPAQUE: f32 = 255.0;
    pub const WHITE: u32 = 0xFFFFFF;
}

const TEXTURE: &str = "aether:block/aether_portal";

thread_local!
{
    // previous and current time in the portal on this client, 0 outside up to 1 after a full wait
    static TIME: Cell<[f32; 2]> = const { Cell::new([0.0; 2]) };
}

fn tick()
{
    let inside: bool = localPlayer().and_then(|player| player.info()).is_some_and(|info| portal::inPortal(&info));
    let [_, time] = TIME.get();

    let next: f32 = if inside
    {
        (time + fade::RISE).min(1.0)
    }
    else
    {
        (time - fade::FALL).max(0.0)
    };
    TIME.set([time, next]);
}

// GuiIngame.renderPortalOverlay with the aether portal texture
fn draw(draw: &mut Draw, partial: f32, width: i32, height: i32)
{
    let [previous, time] = TIME.get();
    let mut strength: f32 = previous + (time - previous) * partial;
    if strength <= 0.0
    {
        return;
    }

    if strength < 1.0
    {
        strength = strength.powi(4) * fade::SPAN + fade::FLOOR;
    }

    let alpha: u32 = (strength * fade::OPAQUE) as u32;
    draw.tinted(TEXTURE, 0, 0, width, height, alpha << 24 | fade::WHITE);
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<ClientTick>(Priority::Normal, |_| tick())?;
    ctx.screens().hud(draw)
}
