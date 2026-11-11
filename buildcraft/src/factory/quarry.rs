use super::well::MachineBlock;
use super::{FRAME, Faces, soft, unbreakable};
use crate::core::area::Area;
use crate::core::items::{gear, material};
use crate::core::power::{Provider, Setup};
use crate::core::render::{self, Laser, Scene};
use crate::core::{dir, stacks, state};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// TileQuarry: builds a frame around its area, then a mechanical arm digs it out layer by layer
const NAME: &str = "quarry";
const MAX_ENERGY: i32 = 7000;
const FRAME_POWER: Setup = Setup
{
    m_latency: 20,
    m_minReceived: 25,
    m_maxReceived: 25,
    m_minActivation: 25,
    m_maxStored: MAX_ENERGY,
    m_loss: None,
};
const FRAME_COST: i32 = 25;
const DIG_POWER: [i32; 5] = [20, 30, 200, 50, MAX_ENERGY];
const HARDNESS: f32 = 1.5;
const RESISTANCE: f32 = 10.0;
const VIEW_CHANNEL: &str = "quarry";

mod bounds
{
    // the default area in front of a lone quarry, eleven wide and five tall
    pub const WIDTH: i32 = 10;
    pub const DEPTH: i32 = 4;
    pub const HALF: i32 = 5;
    pub const MIN_SIDE: i32 = 3;
    pub const MIN_HEIGHT: i32 = 5;
}

mod arm
{
    // EntityMechanicalArm: its corner sits three quarters into the frame, its head hangs two blocks below the top
    pub const INSET: f64 = 0.75;
    pub const TOP: f64 = 0.25;
    pub const DROP: f64 = 2.0;
    pub const SPAN: f64 = 0.5;
    pub const START_SPEED: f64 = 0.03;
    pub const BASE_SPEED: f64 = 0.015;
    pub const SPEED_DIVISOR: f32 = 200.0;
    pub const ENERGY_PER_STEP: i32 = 1000;
    pub const MIN_STEP: i32 = 2;
    pub const HOME_LIFT: i32 = 2;
    pub const SEARCH_ABOVE: i32 = 3;
    pub const BEAM: f64 = 0.5;
    pub const BEAM_OFFSET: f64 = 0.25;
    pub const HEAD: [f64; 3] = [0.2, 1.0, 0.2];
    pub const HEAD_OFFSET: f64 = 0.4;
}

const FACES: Faces = Faces
{
    m_front: "buildcraft:block/quarry_front",
    m_top: "buildcraft:block/quarry_top",
    m_side: "buildcraft:block/quarry_side",
    m_bottom: None,
    m_back: None,
};

const DRILL: &str = "buildcraft:block/drill";
const DRILL_HEAD: &str = "buildcraft:block/drill_head";

// EntityMechanicalArm's movement, kept in the quarry's data rather than as entities
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct Arm
{
    m_origin: [f64; 3],
    m_sizeX: f64,
    m_sizeZ: f64,
    m_baseY: f64,
    m_head: [f64; 3],
    m_target: [f64; 3],
    m_angle: f64,
    m_speed: f64,
    m_movingXZ: bool,
    m_movingY: bool,
}

impl Arm
{
    fn new(origin: [f64; 3], width: f64, height: f64) -> Self
    {
        let head: [f64; 3] = [origin[0], origin[1] - arm::DROP, origin[2]];
        let mut built: Arm = Arm
        {
            m_origin: origin,
            m_sizeX: height,
            m_sizeZ: width,
            m_baseY: origin[1],
            m_head: head,
            m_speed: arm::START_SPEED,
            ..Arm::default()
        };
        built.setTarget(head);
        built
    }

    fn setTarget(&mut self, target: [f64; 3])
    {
        self.m_target = target;
        self.m_angle = (target[2] - self.m_head[2]).atan2(target[0] - self.m_head[0]);
        self.m_movingXZ = true;
        self.m_movingY = true;
    }

    // EntityMechanicalArm.doMove, true when the head reached its target this step
    fn step(&mut self, speed: f64) -> bool
    {
        let mut reached: bool = false;

        if self.m_movingXZ
        {
            let close: bool = (self.m_target[0] - self.m_head[0]).abs() < speed * 2.0
                && (self.m_target[2] - self.m_head[2]).abs() < speed * 2.0;
            if close
            {
                self.m_head[0] = self.m_target[0];
                self.m_head[2] = self.m_target[2];
                self.m_movingXZ = false;
                reached |= !self.m_movingY;
            }
            else
            {
                self.m_head[0] += self.m_angle.cos() * speed;
                self.m_head[2] += self.m_angle.sin() * speed;
            }
        }

        if self.m_movingY
        {
            if (self.m_target[1] - self.m_head[1]).abs() < speed * 2.0
            {
                self.m_head[1] = self.m_target[1];
                self.m_movingY = false;
                reached |= !self.m_movingXZ;
            }
            else if self.m_target[1] > self.m_head[1]
            {
                self.m_head[1] += speed / 2.0;
            }
            else
            {
                self.m_head[1] -= speed / 2.0;
            }
        }

        reached
    }

    // the beams and the drill head as RenderEntityBlock draws the arm's four entities
    fn draw(&self, draw: &mut Scene, pos: BlockPos, light: f32)
    {
        let [x, y, z] = self.m_origin;
        let [hx, hy, hz] = self.m_head;
        let drill: [f32; 4] = render::uv(DRILL);
        let beam = |world: [f64; 3]| render::relative(pos, world);

        draw.stretched(beam([x, y, hz + arm::BEAM_OFFSET]), [self.m_sizeZ, arm::BEAM, arm::BEAM], drill, light);
        draw.stretched(
            beam([hx + arm::BEAM_OFFSET, hy + 1.0, hz + arm::BEAM_OFFSET]),
            [arm::BEAM, self.m_baseY - hy - 1.0, arm::BEAM],
            drill,
            light,
        );
        draw.stretched(beam([hx + arm::BEAM_OFFSET, y, z]), [arm::BEAM, arm::BEAM, self.m_sizeX], drill, light);
        draw.stretched(
            beam([hx + arm::HEAD_OFFSET, hy, hz + arm::HEAD_OFFSET]),
            arm::HEAD,
            render::uv(DRILL_HEAD),
            light,
        );
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct QuarryState
{
    m_started: bool,
    m_area: Option<Area>,
    m_framed: bool,
    m_lasers: bool,
    m_digging: bool,
    m_working: bool,
    m_arm: Option<Arm>,
    m_target: [i32; 3],
    m_provider: Provider,
}

// what a client draws of a quarry, sent when it changes
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
struct View
{
    m_area: Option<Area>,
    m_lasers: bool,
    m_arm: Option<Arm>,
    m_working: bool,
    m_speed: f64,
    m_digging: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ViewPacket
{
    m_world: i32,
    m_pos: BlockPos,
    m_view: View,
}

thread_local!
{
    static VIEWS: RefCell<HashMap<(i32, BlockPos), View>> = RefCell::new(HashMap::new());
}

fn frameAt(area: &Area, x: i32, y: i32, z: i32) -> bool
{
    let edgeX: bool = x == area.m_min[0] || x == area.m_max[0];
    let edgeY: bool = y == area.m_min[1] || y == area.m_max[1];
    let edgeZ: bool = z == area.m_min[2] || z == area.m_max[2];
    (edgeX && edgeZ) || (edgeY && (edgeX || edgeZ))
}

// TileQuarry.initializeBluePrintBuilder anchors the frame at the quarry's own height, whatever the area's
fn frameArea(pos: BlockPos, area: &Area) -> Area
{
    Area::new([area.m_min[0], pos.y, area.m_min[2]], [area.m_max[0], pos.y + area.size(1) - 1, area.m_max[2]])
}

// BluePrintBuilder.findNextBlock for the frame blueprint: a block in the way, or a missing frame piece
fn nextFrameBlock(world: World, area: &Area) -> Option<(BlockPos, bool)>
{
    for y in area.m_min[1]..=area.m_max[1]
    {
        for x in area.m_min[0]..=area.m_max[0]
        {
            for z in area.m_min[2]..=area.m_max[2]
            {
                let pos: BlockPos = BlockPos::new(x, y, z);
                if y <= 0 || unbreakable(world, pos)
                {
                    continue;
                }

                let wanted: bool = frameAt(area, x, y, z);
                let blocked: bool = if wanted
                {
                    !world.is(pos, FRAME)
                }
                else
                {
                    !soft(world, pos)
                };
                if blocked
                {
                    return Some((pos, wanted));
                }
            }
        }
    }

    None
}

fn canDig(world: World, pos: BlockPos) -> bool
{
    !unbreakable(world, pos) && !soft(world, pos) && !world.is(pos, "minecraft:snow_layer")
}

// TileQuarry.findTarget, sweeping each layer back and forth from three above the quarry down to bedrock
fn findTarget(world: World, origin: BlockPos, area: &Area) -> Option<[i32; 3]>
{
    let sizeX: i32 = area.size(0) - 2;
    let sizeZ: i32 = area.size(2) - 2;
    let mut blocked: Vec<bool> = vec![false; (sizeX.max(0) * sizeZ.max(0)) as usize];

    for y in (0..=origin.y + arm::SEARCH_ABOVE).rev()
    {
        let columns: Vec<i32> = if y % 2 == 0
        {
            (0..sizeX).collect()
        }
        else
        {
            (0..sizeX).rev().collect()
        };

        for column in columns
        {
            let rows: Vec<i32> = if column % 2 == y % 2
            {
                (0..sizeZ).collect()
            }
            else
            {
                (0..sizeZ).rev().collect()
            };

            for row in rows
            {
                let index: usize = (column * sizeZ + row) as usize;
                if blocked[index]
                {
                    continue;
                }

                let pos: BlockPos = BlockPos::new(area.m_min[0] + column + 1, y, area.m_min[2] + row + 1);
                if unbreakable(world, pos)
                {
                    blocked[index] = true;
                }
                else if canDig(world, pos)
                {
                    return Some([pos.x, y + 1, pos.z]);
                }
            }
        }
    }

    None
}

// TileQuarry.setBoundaries: a landmark area next to it, or the default area in front of it
fn boundaries(world: World, pos: BlockPos, defaults: bool) -> Area
{
    let marked: Option<Area> = if defaults
    {
        None
    }
    else
    {
        crate::builders::takeArea(world, pos)
    };
    let usable: Option<Area> =
        marked.filter(|area| area.size(0) >= bounds::MIN_SIDE && area.size(2) >= bounds::MIN_SIDE);

    let Some(mut area) = usable
    else
    {
        let facing: i32 = dir::reverse(state::faceAt(world, pos, state::FACING).unwrap_or(dir::DOWN));
        let (x, z): (i32, i32) = match facing
        {
            dir::EAST => (pos.x + 1, pos.z - bounds::HALF),
            dir::WEST => (pos.x - bounds::WIDTH - 1, pos.z - bounds::HALF),
            dir::SOUTH => (pos.x - bounds::HALF, pos.z + 1),
            _ => (pos.x - bounds::HALF, pos.z - bounds::WIDTH - 1),
        };
        return Area::new([x, pos.y, z], [x + bounds::WIDTH, pos.y + bounds::DEPTH, z + bounds::WIDTH]);
    };

    if area.size(1) < bounds::MIN_HEIGHT
    {
        area.m_max[1] = area.m_min[1] + bounds::MIN_HEIGHT - 1;
    }

    area
}

fn makeArm(pos: BlockPos, area: &Area) -> Arm
{
    let origin: [f64; 3] = [
        f64::from(area.m_min[0]) + arm::INSET,
        f64::from(pos.y + area.size(1) - 1) + arm::TOP,
        f64::from(area.m_min[2]) + arm::INSET,
    ];
    Arm::new(origin, f64::from(area.size(0) - 2) + arm::SPAN, f64::from(area.size(2) - 2) + arm::SPAN)
}

fn view(state: &QuarryState, speed: f64) -> View
{
    View
    {
        m_area: state.m_area,
        m_lasers: state.m_lasers,
        m_arm: state.m_arm.clone(),
        m_working: state.m_working,
        m_speed: speed,
        m_digging: state.m_digging,
    }
}

fn publish(world: World, pos: BlockPos, view: View)
{
    let packet: ViewPacket = ViewPacket { m_world: world.dimension, m_pos: pos, m_view: view };
    if let Err(error) = send(&format!("buildcraft:{VIEW_CHANNEL}"), Target::Tracking(world.dimension, pos), &packet)
    {
        warn(&format!("could not send a quarry update: {error}"));
    }
}

// the frame's area first, then the arm once the frame stands
fn prepare(world: World, pos: BlockPos, state: &mut QuarryState)
{
    let area: Area = match state.m_area
    {
        Some(area) => area,

        None =>
        {
            let area: Area = boundaries(world, pos, false);
            state.m_area = Some(area);
            area
        }
    };

    if nextFrameBlock(world, &frameArea(pos, &area)).is_some()
    {
        state.m_framed = false;
        state.m_lasers = true;
        state.m_digging = true;
        return;
    }

    state.m_framed = true;
    state.m_lasers = false;

    if state.m_arm.is_none()
    {
        // createUtilsIfNeeded only ever sets isDigging, doWork clears it once no target is left
        state.m_arm = Some(makeArm(pos, &area));
    }
}

fn buildFrame(world: World, pos: BlockPos, state: &mut QuarryState, area: &Area)
{
    state.m_provider.configure(
        FRAME_POWER.m_latency,
        FRAME_POWER.m_minReceived,
        FRAME_POWER.m_maxReceived,
        FRAME_POWER.m_minActivation,
        FRAME_POWER.m_maxStored,
    );
    if state.m_provider.useEnergy(FRAME_COST, FRAME_COST, true) != FRAME_COST
    {
        return;
    }

    state.m_provider.m_work.mark(world.time());

    let Some((target, frame)) = nextFrameBlock(world, &frameArea(pos, area))
    else
    {
        return;
    };

    if !soft(world, target)
    {
        stacks::breakBlock(world, target);
    }
    else if frame && let Err(error) = world.place(target, FRAME)
    {
        warn(&format!("the quarry at {pos:?} could not place a frame: {error}"));
    }
}

// TileQuarry.doWork
fn work(world: World, pos: BlockPos, state: &mut QuarryState)
{
    if state.m_working || !state.m_digging
    {
        return;
    }

    prepare(world, pos, state);

    let Some(area) = state.m_area
    else
    {
        return;
    };

    if !state.m_framed
    {
        buildFrame(world, pos, state, &area);
        return;
    }

    let [latency, min, max, activation, stored] = DIG_POWER;
    state.m_provider.configure(latency, min, max, activation, stored);

    let Some(arm) = state.m_arm.as_mut()
    else
    {
        return;
    };

    match findTarget(world, pos, &area)
    {
        Some(target) =>
        {
            arm.setTarget(target.map(f64::from));
            state.m_target = target;
        }

        None =>
        {
            let middle: f64 = arm.m_sizeX / 2.0;
            arm.setTarget([
                f64::from(area.m_min[0]) + middle,
                f64::from(pos.y + arm::HOME_LIFT),
                f64::from(area.m_min[2]) + middle,
            ]);
            state.m_digging = false;
        }
    }

    state.m_working = true;
}

// TileQuarry.positionReached: the block under the head goes to the output and turns to air
fn reached(world: World, pos: BlockPos, state: &mut QuarryState)
{
    state.m_working = false;

    let [x, y, z] = state.m_target;
    let target: BlockPos = BlockPos::new(x, y - 1, z);
    if !canDig(world, target)
    {
        return;
    }

    state.m_provider.m_work.mark(world.time());

    if let Some(stack) = world.blockDropStack(target)
    {
        stacks::output(world, pos, stack);
    }

    if let Err(error) = world.clear(target)
    {
        warn(&format!("the quarry at {pos:?} could not clear {target:?}: {error}"));
    }
}

fn tick(world: World, pos: BlockPos, state: &mut QuarryState)
{
    if !state.m_started
    {
        state.m_started = true;
        state.m_provider = FRAME_POWER.build();
        state.m_digging = true;
    }

    let before: View = view(state, 0.0);

    if state.m_provider.update(world, pos)
    {
        work(world, pos, state);
    }

    state.m_provider.decay(world);

    let mut speed: f64 = 0.0;

    if state.m_working
        && let Some(mut current) = state.m_arm.take()
    {
        current.m_speed = 0.0;
        let wanted: i32 = arm::MIN_STEP + state.m_provider.m_stored / arm::ENERGY_PER_STEP;
        let used: i32 = state.m_provider.useEnergy(wanted, wanted, true);
        let done: bool = used > 0 &&
        {
            // the energy share is a float division as TileQuarry wrote it
            speed = arm::BASE_SPEED + f64::from(used as f32 / arm::SPEED_DIVISOR);
            current.step(speed)
        };
        state.m_arm = Some(current);

        if done
        {
            reached(world, pos, state);
        }
    }
    else if let Some(current) = state.m_arm.as_mut()
        && current.m_speed > 0.0
    {
        speed = current.m_speed;
        current.step(speed);
    }

    let after: View = view(state, speed);
    let changed: bool = after.m_area != before.m_area
        || after.m_lasers != before.m_lasers
        || after.m_working != before.m_working
        || after.m_digging != before.m_digging
        || after.m_arm.as_ref().map(|arm| arm.m_target) != before.m_arm.as_ref().map(|arm| arm.m_target);
    if changed
    {
        publish(world, pos, after);
    }
}

// a client moves the arm at the server's last speed between updates
fn animate(world: World, pos: BlockPos)
{
    VIEWS.with(|views|
    {
        if let Some(shown) = views.borrow_mut().get_mut(&(world.dimension, pos))
            && let Some(current) = shown.m_arm.as_mut()
            && shown.m_speed > 0.0
        {
            current.step(shown.m_speed);
        }
    });
}

// IMachine.isActive, a quarry is active while it digs
pub(crate) fn active(world: World, pos: BlockPos) -> bool { shown(world, pos).is_some_and(|view| view.m_digging) }

fn shown(world: World, pos: BlockPos) -> Option<View>
{
    if world.isAuthority()
    {
        let state: QuarryState = world.tileData(pos);
        return Some(view(&state, 0.0));
    }

    VIEWS.with(|views| views.borrow().get(&(world.dimension, pos)).cloned())
}

struct QuarryTile;

#[tile_behavior]
impl TileBehavior for QuarryTile
{
    fn onTick(&self, ctx: &mut BlockCtx)
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        if !ctx.isAuthority()
        {
            animate(world, pos);
            return;
        }

        let mut state: QuarryState = ctx.tileData();
        tick(world, pos, &mut state);
        let _ = ctx.setTileData(&state);
    }

    fn onRemoved(&self, ctx: &mut BlockCtx)
    {
        VIEWS.with(|views| views.borrow_mut().remove(&(ctx.world().dimension, ctx.pos())));
    }

    fn pose(&self, ctx: &mut BlockCtx, _partial: f32, _ticks: i32) -> Pose
    {
        let world: World = ctx.world();
        let pos: BlockPos = ctx.pos();
        let Some(shown) = shown(world, pos)
        else
        {
            return Pose::default();
        };
        let light: f32 = render::lightAt(world, pos);
        let mut draw: Scene = Scene::new();

        if let Some(area) = shown.m_area.filter(|_| shown.m_lasers)
        {
            draw.laserBox(render::relative(pos, area.min()), render::relative(pos, area.max()), Laser::Stripes, light);
        }

        if let Some(current) = &shown.m_arm
        {
            current.draw(&mut draw, pos, light);
        }

        draw.finish()
    }

    fn energyDemand(&self, ctx: &mut BlockCtx, _face: i32) -> Option<EnergyDemand>
    {
        let state: QuarryState = ctx.tileData();
        if !state.m_started
        {
            return FRAME_POWER.build().demand();
        }

        state.m_provider.demand()
    }

    fn receiveEnergy(&self, ctx: &mut BlockCtx, _face: i32, amount: i64, simulate: bool) -> i64
    {
        let mut state: QuarryState = ctx.tileData();
        if !state.m_started
        {
            state.m_started = true;
            state.m_provider = FRAME_POWER.build();
            state.m_digging = true;
        }

        let taken: i64 = state.m_provider.take(amount, simulate);
        if !simulate
        {
            let _ = ctx.setTileData(&state);
        }

        taken
    }
}

fn onView(packet: ViewPacket)
{
    let world: World = World { dimension: packet.m_world };
    if world.isAuthority()
    {
        return;
    }

    VIEWS.with(|views| views.borrow_mut().insert((packet.m_world, packet.m_pos), packet.m_view));
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    FACES.reserve();
    render::slot(DRILL);
    render::slot(DRILL_HEAD);

    ctx.tileEntities().register(
        NAME,
        TileEntitySpec { inventory: 0, ticks: true, sync: false, behavior: None },
        Some(Rc::new(QuarryTile)),
    )?;
    ctx.tileEntities().model(NAME, EntityModelSpec::default())?;
    ctx.blocks().register(
        NAME,
        BlockSpecBuilder::template("minecraft:block")
            .material("iron")
            .sound("metal")
            .hardness(HARDNESS)
            .resistance(RESISTANCE)
            .texture(FACES.m_front)
            .tileEntity(NAME)
            .tag(crate::machines::MACHINES)
            .tag(crate::machines::CONNECTORS)
            .behavior(MachineBlock { m_faces: &FACES, m_block: StateHandle::named(NAME)? }),
    )?;
    super::registerFacing(ctx, NAME)?;
    ctx.channels().register(VIEW_CHANNEL, Direction::ToClient, |packet: ViewPacket, _player| onView(packet))?;

    if !super::mining()
    {
        return Ok(());
    }

    ctx.recipes()
        .shaped(NAME, ItemStack::of("buildcraft:quarry", 1))
        .pattern(["ipi", "gig", "dDd"])
        .keyTag('i', gear::IRON)
        .keyTag('p', material::REDSTONE)
        .keyTag('g', gear::GOLD)
        .keyTag('d', gear::DIAMOND)
        .key('D', "minecraft:diamond_pickaxe")
        .build()
}
