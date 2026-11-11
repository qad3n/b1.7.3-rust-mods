use super::{Kind, PIXELS, base, texture, when};
use crate::names::{self, block};
use b173_api::prelude::*;
use std::rc::Rc;

mod rune
{
    pub const HARDNESS: f32 = 100.0;
    pub const RESISTANCE: f32 = 15.0;
    // BlockTFCastleMagic.magicColors, the inverted bit flips each to its complement
    pub const COLORS: [u32; 4] = [0x00FFFF, 0xFFFF00, 0xFF00FF, 0x4B0082];
    pub const WHITE: u32 = 0xFFFFFF;
}

mod door
{
    // a door waits two to six ticks before it vanishes and eighty before it comes back
    pub const VANISH_BASE: i32 = 2;
    pub const VANISH_SPREAD: i32 = 5;
    pub const RETURN_DELAY: i32 = 80;
}

const FORCE_FIELD_LIGHT: f32 = 2.0 / 15.0;

// BlockTFCastleMagic: castle brick with a glowing rune floating a pixel off every face open to the air
struct Runes;

const RUNE_SPRITES: i32 = 8;
const RUNE_LIFT: f32 = 1.0;

fn runeTexture(index: i32) -> String { texture(&format!("castleblock_magic_{index}")) }

// BlockTFCastleMagic.getMagicIconFor, the rune each spot shows
fn runeAt(pos: BlockPos) -> i32
{
    let seed: i64 =
        i64::from(pos.x).wrapping_mul(3129871) ^ i64::from(pos.y).wrapping_mul(116129781) ^ i64::from(pos.z);
    let mixed: i64 = seed.wrapping_mul(seed).wrapping_mul(42317861).wrapping_add(seed.wrapping_mul(11));
    (mixed >> 12 & i64::from(RUNE_SPRITES - 1)) as i32
}

#[block_behavior]
impl BlockBehavior for Runes
{
    // RenderBlockTFCastleMagic, the rune shows only where the neighbor is air
    fn overlay(&self, ctx: &mut BlockCtx, face: i32) -> Option<FaceOverlay>
    {
        let [dx, dy, dz] = *crate::math::NEIGHBORS.get(usize::try_from(face).ok()?)?;
        let pos: BlockPos = ctx.pos();
        if !ctx.world().isAir(pos.offset(dx, dy, dz))
        {
            return None;
        }

        let sprite: i32 = b173_api::world::terrainSlot(&runeTexture(runeAt(pos)))?;
        let state: Option<Rc<StateData>> = ctx.stateAt();
        let color: u32 = runeColor(names::number(state.clone(), "color"), names::flag(state, "inverted"));
        Some(FaceOverlay { sprite, color, glow: true, lift: RUNE_LIFT })
    }
}

pub(crate) fn runeColor(index: i32, inverted: bool) -> u32
{
    let color: u32 = rune::COLORS[index.rem_euclid(rune::COLORS.len() as i32) as usize];

    if inverted { color ^ rune::WHITE } else { color }
}

// BlockTFCastleDoor: a door that is not locked vanishes when used and pulls its neighbours along
struct CastleDoor
{
    m_vanished: bool,
}

fn fizz(world: World, pos: BlockPos, base: f32)
{
    let volume: f32 = 0.125;
    let spread: f32 = 0.25;
    world.playSound(
        crate::math::center(pos),
        "random.fizz",
        volume,
        world.random(1000) as f32 / 1000.0 * spread + base,
    );
}

fn activateDoor(world: World, pos: BlockPos)
{
    if let Err(error) = world.setProperty(pos, "active", true)
    {
        warn(&format!("castle door at {pos:?}: {}", error.0));
    }

    fizz(world, pos, 1.75);

    let _ = world.scheduleTick(pos, world.block(pos), door::VANISH_BASE + world.random(door::VANISH_SPREAD));
}

const NEIGHBORS: [[i32; 3]; 6] = [[-1, 0, 0], [1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];

#[block_behavior]
impl BlockBehavior for CastleDoor
{
    fn onActivated(&self, ctx: &mut BlockCtx, _player: Player) -> bool
    {
        if names::flag(ctx.stateAt(), "active") || self.m_vanished
        {
            return false;
        }
        if !ctx.isAuthority()
        {
            return true;
        }

        if crate::progress::doorLocked(ctx.world(), ctx.pos())
        {
            ctx.world().playSound(crate::math::center(ctx.pos()), "random.click", 1.0, 0.3);
        }
        else
        {
            activateDoor(ctx.world(), ctx.pos());
        }
        true
    }

    fn onUpdateTick(&self, ctx: &mut BlockCtx)
    {
        if !ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let state: Option<Rc<StateData>> = ctx.stateAt();
        let color: i32 = names::number(state.clone(), "color");
        let active: bool = names::flag(state, "active");

        if self.m_vanished
        {
            if active
            {
                let _ = world.setBlockState(pos, names::state(block::CASTLE_DOOR).with("color", color));

                fizz(world, pos, 1.25);
            }
            else
            {
                activateDoor(world, pos);
            }
            return;
        }

        if !active
        {
            return;
        }

        let _ = world.setBlockState(pos, names::state(block::CASTLE_DOOR_VANISHED).with("color", color));
        let _ = world.scheduleTick(pos, world.block(pos), door::RETURN_DELAY);

        fizz(world, pos, 1.25);

        for [dx, dy, dz] in NEIGHBORS
        {
            let at: BlockPos = pos.offset(dx, dy, dz);
            let ready: bool = world.is(at, block::CASTLE_DOOR)
                && !names::flag(world.stateAt(at), "active")
                && !crate::progress::doorLocked(world, at);
            if ready
            {
                activateDoor(world, at);
            }
        }
    }

    fn colorMultiplier(&self, ctx: &mut BlockCtx) -> u32 { runeColor(names::number(ctx.stateAt(), "color"), false) }
}

fn colorValues(count: u8) -> Vec<String> { (0..count).map(|value| value.to_string()).collect() }

fn registerRunes(ctx: &mut RegisterCtx) -> Result<()>
{
    super::put(
        ctx,
        block::CASTLE_RUNE_BRICK,
        base("minecraft:block", "Castle Rune Brick", Kind::Rock, rune::HARDNESS)
            .resistance(rune::RESISTANCE)
            .texture(&texture("castleblock_brick"))
            .behavior(Runes),
    )?;

    // no block spec names the rune sprites, asking for their slots now gives them room in the atlas
    for index in 0..RUNE_SPRITES
    {
        let _ = b173_api::world::terrainSlot(&runeTexture(index));
    }

    let colors: Vec<String> = colorValues(4);
    let values: Vec<&str> = colors.iter().map(String::as_str).collect();
    ctx.blocks().states(
        block::CASTLE_RUNE_BRICK,
        BlockStatesSpec
        {
            properties: vec![
                PropertySpec::new("inverted", &["false", "true"]),
                PropertySpec::new("spare", &["0", "1"]),
                PropertySpec::new("color", &values),
            ],
            ..BlockStatesSpec::default()
        },
    )?;

    let doors: [(&str, &str, bool); 2] =
        [(block::CASTLE_DOOR, "Castle Door", false), (block::CASTLE_DOOR_VANISHED, "Vanished Castle Door", true)];
    let unbreakable: f32 = -1.0;

    for (name, label, vanished) in doors
    {
        let look: &str = if vanished
        {
            "castle_door_vanished"
        }
        else
        {
            "castle_door"
        };
        let mut spec: BlockSpecBuilder = base("minecraft:block", label, Kind::Rock, unbreakable)
            .resistance(rune::RESISTANCE)
            .texture(&texture(look))
            .drops(Vec::new())
            .behavior(CastleDoor { m_vanished: vanished });

        if vanished
        {
            spec = spec
                .material("glass")
                .opacity(0)
                .collision(Vec::new())
                .model(BlockModelSpec { translucent: true, ..BlockModelSpec::default() });
        }

        super::put(ctx, name, spec)?;

        let shades: Vec<String> = colorValues(8);
        let shadeValues: Vec<&str> = shades.iter().map(String::as_str).collect();
        let active: String = texture(&format!("{look}_active"));
        ctx.blocks().states(
            name,
            BlockStatesSpec
            {
                properties: vec![
                    PropertySpec::new("active", &["false", "true"]),
                    PropertySpec::new("color", &shadeValues),
                ],
                variants: vec![StateVariantSpec
                {
                    when: when("active", "true"),
                    texture: Some(active),
                    textures: None,
                }],
                ..BlockStatesSpec::default()
            },
        )?;
    }

    Ok(())
}

// BlockTFForceField is a glass pane, a post with arms reaching to whatever it joins
const FIELDS: [(&str, &str); 5] =
    [("purple", "Violet"), ("pink", "Pink"), ("orange", "Orange"), ("green", "Green"), ("blue", "Blue")];

fn paneModel() -> BlockModelSpec
{
    let low: f32 = 7.0;
    let high: f32 = 9.0;
    let full: f32 = PIXELS;
    let arm = |from: [f32; 3], to: [f32; 3], face: i32| Cuboid { from, to, connect: Some(face), ..Cuboid::default() };

    BlockModelSpec
    {
        cuboids: vec![
            Cuboid { from: [low, 0.0, low], to: [high, full, high], ..Cuboid::default() },
            arm([low, 0.0, 0.0], [high, full, low], 2),
            arm([low, 0.0, high], [high, full, full], 3),
            arm([0.0, 0.0, low], [low, full, high], 4),
            arm([high, 0.0, low], [full, full, high], 5),
        ],
        translucent: true,
        connects_to: Vec::new(),
        ..BlockModelSpec::default()
    }
}

fn registerFields(ctx: &mut RegisterCtx) -> Result<()>
{
    let unbreakable: f32 = -1.0;
    let resistance: f32 = 6000000.0;

    for (color, label) in FIELDS
    {
        let name: String = format!("{color}_force_field");
        super::put(
            ctx,
            &name,
            BlockSpecBuilder::template("minecraft:block")
                .name(&format!("{label} Force Field"))
                .material("glass")
                .texture(&texture(&format!("forcefield_{color}")))
                .hardness(unbreakable)
                .resistance(resistance)
                .light(FORCE_FIELD_LIGHT)
                .opacity(0)
                .sound("glass")
                .drops(Vec::new())
                .model(paneModel()),
        )?;
    }

    Ok(())
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    registerRunes(ctx)?;
    registerFields(ctx)?;

    let cinderHardness: f32 = 1.0;
    super::put(
        ctx,
        block::CINDER_LOG,
        base("minecraft:block", "Cinder Log [NYI]", Kind::Wood, cinderHardness)
            .faces(super::faces(&super::Look::Column { top: "cinder_top", side: "cinder_side" }))
            .sustainsLeaves(),
    )?;

    let axis: PropertySpec = PropertySpec::new("axis", &super::wood::LOG_AXES);
    ctx.blocks().states(block::CINDER_LOG, BlockStatesSpec { properties: vec![axis], ..BlockStatesSpec::default() })
}
