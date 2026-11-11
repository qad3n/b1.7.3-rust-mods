use super::{PIXELS, texture};
use crate::ids::{self, block};
use crate::models::{Program, tf_cicada, tf_firefly, tf_moonworm};
use b173_api::prelude::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

// BlockTFCritter: tiny bugs that cling to a wall floor or ceiling like a torch
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Critter
{
    Firefly,
    Cicada,
    Moonworm,
}

struct CritterRow
{
    m_critter: Critter,
    m_name: &'static str,
    m_label: &'static str,
    m_texture: &'static str,
    m_light: f32,
    m_drops: bool,
    // the source's TileEntitySpecialRenderer model and its skin
    m_model: fn() -> EntityModelSpec,
    m_skin: &'static str,
}

const CRITTERS: [CritterRow; 3] = [
    CritterRow
    {
        m_critter: Critter::Firefly,
        m_name: block::FIREFLY,
        m_label: "Firefly",
        m_texture: "tffirefly",
        m_light: 0.9375,
        m_drops: true,
        m_model: tf_firefly::model,
        m_skin: "twilightforest:model/firefly-tiny",
    },
    CritterRow
    {
        m_critter: Critter::Cicada,
        m_name: block::CICADA,
        m_label: "Cicada",
        m_texture: "tfcicada",
        m_light: 0.0,
        m_drops: true,
        m_model: tf_cicada::model,
        m_skin: "twilightforest:model/cicada-model",
    },
    CritterRow
    {
        m_critter: Critter::Moonworm,
        m_name: block::MOONWORM,
        m_label: "Moonworm",
        m_texture: "tfmoonworm",
        m_light: 14.0 / 15.0,
        m_drops: false,
        m_model: tf_moonworm::model,
        m_skin: "twilightforest:model/moonworm",
    },
];

mod critter
{
    // TileEntityTFFirefly, TileEntityTFCicada and TileEntityTFMoonworm turn a little every ten to twenty seconds
    pub const TURN_WAIT: i32 = 200;
    pub const TURN_SPREAD: i32 = 200;
    pub const TURN_RANGE: i32 = 15;
    pub const QUARTERS: i32 = 4;
    pub const QUARTER: i32 = 90;
    pub const FULL_TURN: i32 = 360;
    // the firefly's glow climbs and fades a twentieth a tick and rests up to fifty ticks dark
    pub const GLOW_STEP: f32 = 0.05;
    pub const GLOW_REST: i32 = 50;
    // a tiny firefly drifts off one tick in twenty while a player is within sixteen blocks
    pub const FIREFLY_ODDS: i32 = 20;
    pub const FIREFLY_RANGE: f64 = 16.0;
    // the cicada sings for five seconds after a rest of five to ten, a note one tick in five
    pub const SONG_TICKS: i32 = 100;
    pub const SONG_REST: i32 = 100;
    pub const SONG_REST_SPREAD: i32 = 100;
    pub const NOTE_ODDS: i32 = 5;
    pub const SOUND: &str = "twilightforest.mob.cicada";
    pub const PITCH_SPREAD: f32 = 0.2;
}

// one critter's turn, glow and song as its tile entity keeps them on the client
#[derive(Clone, Copy, Default)]
struct Anim
{
    m_yawDelay: i32,
    m_yaw: i32,
    m_desiredYaw: i32,
    m_placed: bool,
    m_glow: f32,
    m_glowing: bool,
    m_glowDelay: i32,
    m_singing: bool,
    m_singTicks: i32,
    m_singDelay: i32,
}

thread_local!
{
    static ANIMS: RefCell<HashMap<(i32, BlockPos), Anim>> = RefCell::new(HashMap::new());
}

impl Anim
{
    // TileEntityTFFirefly and TileEntityTFCicada.updateEntity, a small turn eased in one degree a tick
    fn twitch(&mut self, world: World)
    {
        if self.m_yawDelay > 0
        {
            self.m_yawDelay -= 1;
            return;
        }

        if self.m_yaw == 0 && self.m_desiredYaw == 0
        {
            self.m_yawDelay = critter::TURN_WAIT + world.random(critter::TURN_SPREAD);
            self.m_desiredYaw = world.random(critter::TURN_RANGE) - world.random(critter::TURN_RANGE);
        }

        self.m_yaw += (self.m_desiredYaw - self.m_yaw).signum();
        if self.m_yaw == self.m_desiredYaw
        {
            self.m_desiredYaw = 0;
        }
    }

    // TileEntityTFMoonworm.updateEntity, a quarter turn picked now and then and crawled to a degree a tick
    fn crawl(&mut self, world: World)
    {
        if !self.m_placed
        {
            self.m_placed = true;
            self.m_yaw = world.random(critter::QUARTERS) * critter::QUARTER;
        }

        if self.m_yawDelay > 0
        {
            self.m_yawDelay -= 1;
            return;
        }

        if self.m_desiredYaw == 0
        {
            self.m_yawDelay = critter::TURN_WAIT + world.random(critter::TURN_SPREAD);
            self.m_desiredYaw = world.random(critter::QUARTERS) * critter::QUARTER;
        }

        self.m_yaw += 1;
        if self.m_yaw > critter::FULL_TURN
        {
            self.m_yaw = 0;
        }

        if self.m_yaw == self.m_desiredYaw
        {
            self.m_desiredYaw = 0;
        }
    }

    // TileEntityTFFirefly.updateEntity, the glow swells to full, fades out and rests before the next
    fn pulse(&mut self, world: World)
    {
        if self.m_glowDelay > 0
        {
            self.m_glowDelay -= 1;
            return;
        }

        if self.m_glowing && self.m_glow >= 1.0
        {
            self.m_glowing = false;
        }

        if self.m_glowing && self.m_glow < 1.0
        {
            self.m_glow += critter::GLOW_STEP;
        }

        if !self.m_glowing && self.m_glow > 0.0
        {
            self.m_glow -= critter::GLOW_STEP;
        }

        if !self.m_glowing && self.m_glow <= 0.0
        {
            self.m_glowing = true;
            self.m_glowDelay = world.random(critter::GLOW_REST);
        }
    }

    // TileEntityTFCicada.updateEntity, a song with notes after each rest
    fn sing(&mut self, world: World, pos: BlockPos)
    {
        if self.m_singDelay > 0
        {
            self.m_singDelay -= 1;
            return;
        }

        if self.m_singing && self.m_singTicks == 0
        {
            let pitch: f32 = (ids::fraction(world) - ids::fraction(world)) * critter::PITCH_SPREAD + 1.0;
            world.playSound(ids::center(pos), critter::SOUND, 1.0, pitch);
        }

        if self.m_singing && self.m_singTicks >= critter::SONG_TICKS
        {
            self.m_singing = false;
            self.m_singTicks = 0;
        }

        if self.m_singing && self.m_singTicks < critter::SONG_TICKS
        {
            self.m_singTicks += 1;

            if world.random(critter::NOTE_ODDS) == 0
            {
                let at: [f64; 3] = [pos.x, pos.y, pos.z].map(|axis| f64::from(axis) + f64::from(ids::fraction(world)));
                world.particle("note", at, [0.0; 3]);
            }
        }

        if !self.m_singing && self.m_singTicks <= 0
        {
            self.m_singing = true;
            self.m_singDelay = critter::SONG_REST + world.random(critter::SONG_REST_SPREAD);
        }
    }
}

// TileEntityTFFirefly.anyPlayerInRange for the player of this client
fn playerNear(pos: BlockPos) -> bool
{
    let center: [f64; 3] = ids::center(pos);
    localPlayer()
        .and_then(Player::info)
        .is_some_and(|info| crate::entities::ai::distance(info.pos, center) <= critter::FIREFLY_RANGE)
}

// TileEntityTFCritterRenderer's turn for the face the critter clings to, metadata as BlockTFCritter places it
fn facing(meta: i32) -> (f32, f32)
{
    match meta
    {
        1 => (90.0, -90.0),
        2 => (90.0, 90.0),
        4 => (90.0, 180.0),
        5 => (0.0, 0.0),
        6 => (180.0, 0.0),
        _ => (90.0, 0.0),
    }
}

struct CritterTile
{
    m_critter: Critter,
}

#[tile_behavior]
impl TileBehavior for CritterTile
{
    // the turns, glow and song are only seen, so the client keeps them
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        if ctx.isAuthority()
        {
            return;
        }

        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();

        ANIMS.with(|anims|
        {
            let mut anims: RefMut<HashMap<(i32, b173_api::BlockPos), Anim>> = anims.borrow_mut();
            let anim: &mut Anim = anims.entry((world.dimension, pos)).or_default();

            match self.m_critter
            {
                Critter::Firefly =>
                {
                    if world.random(critter::FIREFLY_ODDS) == 0 && playerNear(pos)
                    {
                        let at: [f64; 3] =
                            [pos.x, pos.y, pos.z].map(|axis| f64::from(axis) + f64::from(ids::fraction(world)));
                        crate::particles::tinyFirefly(world, at);
                    }

                    anim.twitch(world);
                    anim.pulse(world);
                }

                Critter::Cicada =>
                {
                    anim.twitch(world);
                    anim.sing(world, pos);
                }

                Critter::Moonworm => anim.crawl(world),
            }
        });
    }

    fn onRemoved(&self, ctx: &mut BlockCtx)
    {
        let key: (i32, BlockPos) = (ctx.world().dimension, ctx.pos());
        ANIMS.with(|anims| anims.borrow_mut().remove(&key));
    }

    // TileEntityTFFireflyRenderer, TileEntityTFCicadaRenderer and TileEntityTFMoonwormRenderer
    fn pose(&self, ctx: &mut BlockCtx, partial: f32, _ticks: i32) -> Pose
    {
        let anim: Anim =
            ANIMS.with(|anims| anims.borrow().get(&(ctx.world().dimension, ctx.pos())).copied()).unwrap_or_default();
        let row: &CritterRow = row(self.m_critter);
        let (tilt, roll): (f32, f32) = facing(i32::from(ctx.meta()));
        let mut body: Program = Program::new();
        body.translate(0.5, 0.5, 0.5)
            .rotate(tilt, 1.0, 0.0, 0.0)
            .rotate(roll, 0.0, 0.0, 1.0)
            .rotate(anim.m_yaw as f32, 0.0, 1.0, 0.0)
            .texture(row.m_skin)
            .scale(1.0, -1.0, -1.0);

        match self.m_critter
        {
            Critter::Firefly =>
            {
                body.rest(tf_firefly::LEGS).rest(tf_firefly::FATBODY).rest(tf_firefly::SKINNYBODY);
                body.blend(true).lighting(false).color(1.0, 1.0, 1.0, anim.m_glow).rest(tf_firefly::GLOW);
                body.color(1.0, 1.0, 1.0, 1.0).lighting(true).blend(false);
            }

            Critter::Cicada =>
            {
                for part in [
                    tf_cicada::LEGS,
                    tf_cicada::FATBODY,
                    tf_cicada::SKINNYBODY,
                    tf_cicada::EYE1,
                    tf_cicada::EYE2,
                    tf_cicada::WINGS,
                ]
                {
                    body.rest(part);
                }
            }

            Critter::Moonworm =>
            {
                // ModelTFMoonworm.setLivingAnimations, each piece dips in turn while the worm crawls
                let crawling: bool = anim.m_yawDelay == 0;
                let time: f32 = (anim.m_desiredYaw - anim.m_yaw) as f32 - partial;

                for (offset, part) in [tf_moonworm::HEAD, tf_moonworm::SHAPE1, tf_moonworm::SHAPE2, tf_moonworm::SHAPE3]
                    .into_iter()
                    .enumerate()
                {
                    let mut pivot: [f32; 3] = tf_moonworm::PIVOTS[usize::from(part)];

                    if crawling
                    {
                        pivot[1] += (time / 2.0 + offset as f32).sin().min(0.0);
                    }

                    body.part(part, tf_moonworm::ROTATIONS[usize::from(part)], pivot);
                }
            }
        }

        Pose { pre: Vec::new(), model: body.take() }
    }
}

fn row(critter: Critter) -> &'static CritterRow
{
    CRITTERS.iter().find(|row| row.m_critter == critter).expect("every critter has a row")
}

// BlockTFFireflyJar lets tiny fireflies drift around it
struct Jar;

#[block_behavior]
impl BlockBehavior for Jar
{
    fn onRandomDisplayTick(&self, ctx: &mut BlockCtx)
    {
        let pos: [f64; 3] = ids::center(ctx.pos());

        for _ in 0..2
        {
            crate::particles::tinyFirefly(ctx.world(), [pos[0], pos[1] - 0.6, pos[2]]);
        }
    }
}

fn tileName(row: &CritterRow) -> String { format!("{}_tile", row.m_name) }

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    for row in &CRITTERS
    {
        let tile: String = tileName(row);
        let spec: TileEntitySpec = TileEntitySpec { inventory: 0, ticks: true, sync: false, behavior: None };
        ctx.tileEntities().register(&tile, spec, Some(std::rc::Rc::new(CritterTile { m_critter: row.m_critter })))?;
        ctx.tileEntities().model(&tile, (row.m_model)())?;
        // BlockTFCritter draws nothing itself, its tile entity draws the model
        let hidden: Cuboid = Cuboid { texture: Some("twilightforest:block/blank".to_owned()), ..Cuboid::default() };
        let mut spec: BlockSpecBuilder = BlockSpecBuilder::template("minecraft:torch")
            .name(row.m_label)
            .texture(&texture(row.m_texture))
            .hardness(0.0)
            .light(row.m_light)
            .sound("cloth")
            .model(BlockModelSpec { cuboids: vec![hidden], translucent: true, ..BlockModelSpec::default() })
            .tileEntity(&tile);

        if !row.m_drops
        {
            spec = spec.drops(Vec::new());
        }

        super::put(ctx, row.m_name, spec)?;
    }

    let inset: f32 = 3.0;
    let hardness: f32 = 0.3;
    let jar: Cuboid =
        Cuboid { from: [inset, 0.0, inset], to: [PIXELS - inset, PIXELS, PIXELS - inset], ..Cuboid::default() };
    super::put(
        ctx,
        block::FIREFLY_JAR,
        BlockSpecBuilder::template("minecraft:block")
            .name("Firefly Jar")
            .material("glass")
            .faces(FaceTextures
            {
                top: Some(texture("fireflyjar_top")),
                bottom: Some(texture("fireflyjar_top")),
                side: Some(texture("fireflyjar_side")),
                front: None,
            })
            .hardness(hardness)
            .light(1.0)
            .sound("wood")
            .model(BlockModelSpec { cuboids: vec![jar.clone()], translucent: true, ..BlockModelSpec::default() })
            .collision(vec![jar])
            .behavior(Jar),
    )
}
