use crate::math::wrapAngle;
use crate::{items, net};
use b173_api::prelude::*;
use std::cell::{Cell, RefCell};

// main inventory slots searched after the held stack
const INVENTORY: i32 = 36;
const SPRITE: &str = "wirelessredstone:item/triangulator";

// TextureTriangulatorFX, a compass needle on a 16 pixel grid that swings toward the closest active transmitter
mod needle
{
    pub const GRID: i32 = 16;
    pub const CENTER: [f64; 2] = [8.5, 7.5];
    pub const STEP: f64 = 0.3;
    // the needle lies on the icon seen at a slant, half as tall as wide
    pub const SLANT: f64 = 0.5;
    pub const CROSS: i32 = 4;
    pub const BACK: i32 = 8;
    pub const TIP: i32 = 16;
    pub const SWING: f64 = 1.0;
    pub const PULL: f64 = 0.1;
    pub const DAMPING: f64 = 0.8;
    pub const QUARTER: f32 = 90.0;
    pub const CROSS_COLOR: u32 = 0xFF969696;
    pub const BACK_COLOR: u32 = 0xFF646464;
    pub const TIP_COLOR: u32 = 0xFF1414FF;
}

#[derive(Default)]
struct Swing
{
    m_angle: f64,
    m_speed: f64,
}

thread_local!
{
    static CLOSEST: Cell<Option<[i32; 2]>> = const { Cell::new(None) };
    static SWING: RefCell<Swing> = RefCell::new(Swing::default());
}

pub(crate) fn located(pos: Option<[i32; 2]>) { CLOSEST.with(|closest| closest.set(pos)); }

// the held triangulator, else the first one carried, sets the frequency the needle follows
fn carried(player: Player) -> Option<i32>
{
    if player.held().is_some_and(|stack| stack.item.is(items::TRIANGULATOR))
    {
        return Some(stackComponent::<i32>(StackRef::Held { player: player.id }, items::FREQ));
    }

    let slot: i32 =
        (0..INVENTORY).find(|&slot| player.slot(slot).is_some_and(|stack| stack.item.is(items::TRIANGULATOR)))?;
    Some(stackComponent::<i32>(StackRef::Inventory { player: player.id, slot }, items::FREQ))
}

// one needle pixel on the 16 grid, as a block of pixels on a bigger texture pack sprite
fn plot(pixels: &mut [u32], size: u32, at: [f64; 2], color: u32)
{
    let scale: u32 = (size / needle::GRID as u32).max(1);
    let [x, y] = at.map(|value| value as i32);
    let inside: bool = (0..needle::GRID).contains(&x) && (0..needle::GRID).contains(&y);

    if !inside
    {
        return;
    }

    for row in 0..scale
    {
        for column in 0..scale
        {
            let index: u32 = (y as u32 * scale + row) * size + x as u32 * scale + column;
            pixels[index as usize] = color;
        }
    }
}

// the swing toward a target angle, kept between frames as the compass keeps it
fn swing(target: f64) -> f64
{
    SWING.with(|swing|
    {
        let mut swing = swing.borrow_mut();
        let pull: f64 = wrapAngle(target - swing.m_angle).clamp(-needle::SWING, needle::SWING);
        swing.m_speed += pull * needle::PULL;
        swing.m_speed *= needle::DAMPING;
        swing.m_angle += swing.m_speed;
        swing.m_angle
    })
}

fn draw(info: &EntityInfo, target: [i32; 2], size: u32, pixels: &mut [u32])
{
    let [x, _, z] = info.pos;
    let yaw: f64 = f64::from(info.yaw - needle::QUARTER).to_radians();
    let angle: f64 = swing(yaw - (target[1] as f64 - z).atan2(target[0] as f64 - x));
    let (sin, cos): (f64, f64) = angle.sin_cos();
    let [centerX, centerY] = needle::CENTER;

    for i in -needle::CROSS..=needle::CROSS
    {
        let step: f64 = i as f64 * needle::STEP;
        plot(pixels, size, [centerX + cos * step, centerY - sin * step * needle::SLANT], needle::CROSS_COLOR);
    }

    for i in -needle::BACK..=needle::TIP
    {
        let step: f64 = i as f64 * needle::STEP;
        let color: u32 = if i < 0
        {
            needle::BACK_COLOR
        }
        else
        {
            needle::TIP_COLOR
        };
        plot(pixels, size, [centerX + sin * step, centerY + cos * step * needle::SLANT], color);
    }
}

// the source repainted the plain icon and left the needle out while no transmitter on the frequency was on
fn paint(player: Player)
{
    // read every time so a texture pack change shows at once
    let Some((size, mut pixels)) = spritePixels(SPRITE)
    else
    {
        return;
    };

    let target: Option<[i32; 2]> = CLOSEST.with(Cell::get);

    if let (Some(target), Some(info)) = (target, player.info())
    {
        draw(&info, target, size, &mut pixels);
    }

    if let Err(error) = paintSprite(SPRITE, size, pixels)
    {
        warn(&format!("triangulator sprite: {error:?}"));
    }
}

pub(crate) fn tick()
{
    let Some(player) = localPlayer()
    else
    {
        return;
    };

    let Some(freq) = carried(player)
    else
    {
        return;
    };

    if let Err(error) = send(net::LOCATE, Target::Server, &net::Locate { m_freq: freq })
    {
        warn(&format!("triangulator: {error:?}"));
    }

    paint(player);
}
