use b173_api::prelude::*;
use std::cell::{Cell, RefCell};

// TextureOilFX and TextureOilFlowFX: the water effects' heat field drawn in near black greys, stepped every client
// tick and painted onto the oil sprites
const SIDE: usize = 16;
const PIXELS: usize = SIDE * SIDE;
const STILL: &str = "buildcraft:block/oil_still";
const FLOW: &str = "buildcraft:block/oil_flow";
const OPAQUE: u32 = 0xFF00_0000;

// one effect's numbers: how its field blurs and cools and how its greys come out
struct Kind
{
    m_divisor: f32,
    m_cooling: f32,
    m_sparkChance: f64,
    m_lightRange: f32,
    m_flowing: bool,
}

const STILL_KIND: Kind =
    Kind { m_divisor: 3.3, m_cooling: 0.1, m_sparkChance: 0.05, m_lightRange: 21.0, m_flowing: false };
const FLOW_KIND: Kind =
    Kind { m_divisor: 3.2, m_cooling: 0.3, m_sparkChance: 0.2, m_lightRange: 22.0, m_flowing: true };

mod shade
{
    pub const KEEP: f32 = 0.8;
    pub const HEAT: f32 = 0.05;
    pub const SPARK: f32 = 0.5;
    pub const BASE: f32 = 10.0;
}

struct Field
{
    m_current: [f32; PIXELS],
    m_next: [f32; PIXELS],
    m_heat: [f32; PIXELS],
    m_spark: [f32; PIXELS],
    m_ticks: usize,
}

impl Field
{
    const fn new() -> Self
    {
        Field
        {
            m_current: [0.0; PIXELS],
            m_next: [0.0; PIXELS],
            m_heat: [0.0; PIXELS],
            m_spark: [0.0; PIXELS],
            m_ticks: 0,
        }
    }

    // onTick: blur along the flow, add heat, cool the sparks, swap, then draw
    fn step(&mut self, kind: &Kind, random: &mut impl FnMut() -> f64) -> Vec<u32>
    {
        self.m_ticks += 1;

        for i in 0..SIDE
        {
            for k in 0..SIDE
            {
                let total: f32 = if kind.m_flowing
                {
                    (k as isize - 2..=k as isize).map(|j| self.m_current[(i & 15) + (j as usize & 15) * SIDE]).sum()
                }
                else
                {
                    (i as isize - 1..=i as isize + 1).map(|j| self.m_current[(j as usize & 15) + (k & 15) * SIDE]).sum()
                };
                self.m_next[i + k * SIDE] = total / kind.m_divisor + self.m_heat[i + k * SIDE] * shade::KEEP;
            }
        }

        for index in 0..PIXELS
        {
            self.m_heat[index] = (self.m_heat[index] + self.m_spark[index] * shade::HEAT).max(0.0);
            self.m_spark[index] -= kind.m_cooling;

            if random() < kind.m_sparkChance
            {
                self.m_spark[index] = shade::SPARK;
            }
        }

        std::mem::swap(&mut self.m_current, &mut self.m_next);

        (0..PIXELS)
            .map(|index|
            {
                let source: usize = if kind.m_flowing
                {
                    index.wrapping_sub(self.m_ticks * SIDE) & 0xFF
                }
                else
                {
                    index
                };
                let value: f32 = self.m_current[source].clamp(0.0, 1.0);
                let grey: u32 = (shade::BASE + value * value * kind.m_lightRange) as u32;
                OPAQUE | grey << 16 | grey << 8 | grey
            })
            .collect()
    }
}

thread_local!
{
    static FIELDS: RefCell<(Field, Field)> = const { RefCell::new((Field::new(), Field::new())) };
    // Math.random's stand in, a xorshift the client alone steps
    static SEED: Cell<u64> = const { Cell::new(0x9E37_79B9_7F4A_7C15) };
}

fn random() -> f64
{
    const BITS: u32 = 53;

    SEED.with(|seed|
    {
        let mut value: u64 = seed.get();
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        seed.set(value);
        (value >> (64 - BITS)) as f64 / (1u64 << BITS) as f64
    })
}

fn tick()
{
    FIELDS.with(|fields|
    {
        let mut fields = fields.borrow_mut();
        let mut draw = random;
        let still: Vec<u32> = fields.0.step(&STILL_KIND, &mut draw);
        let flow: Vec<u32> = fields.1.step(&FLOW_KIND, &mut draw);

        for (sprite, pixels) in [(STILL, still), (FLOW, flow)]
        {
            if let Err(error) = paintSprite(sprite, SIDE as u32, pixels)
            {
                warn(&format!("could not draw {sprite}: {error}"));
            }
        }
    });
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.events().on::<ClientTick>(Priority::Normal, |_| tick())
}
