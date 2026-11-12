use crate::models::{Program, tf_cube_of_annihilation};
use b173_api::math::{flatLength, floor};
use b173_api::prelude::*;

// EntityTFRovingCube with EntityAICubeMoveToRedstoneSymbols and EntityAICubeCenterOnSymbol, it drifts to the middle
// of a ring of eight redstone wires and sits there
const NAME: &str = "roving_cube";
// Entity.getEyeHeight of a 2.1 high mob less the quarter block the sparks start under it
const EYE: f32 = 2.1 * 0.85 - 0.25;

mod tuning
{
    pub const SEARCH_CHANCE: i32 = 20;
    pub const REACH: i32 = 16;
    pub const RISE: i32 = 5;
    pub const SPEED: f64 = 0.1;
    pub const CENTERED: f64 = 0.1;
    pub const GIVE_UP: f64 = 100.0;
    pub const SPARKS: i32 = 3;
    pub const SPREAD: f32 = 0.75;
    pub const SPIN: f32 = 11.0;
    pub const SIZE: f32 = 2.0;
    pub const LIFT: f32 = 0.75;
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
struct Symbol
{
    m_pos: Option<[i32; 3]>,
}

// a free block ringed by redstone wire on all eight sides
fn symbol(world: World, x: i32, y: i32, z: i32) -> bool
{
    let ring: [[i32; 2]; 8] = [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [-1, 1], [1, -1], [-1, -1]];
    world.block(BlockPos::new(x, y, z)) == 0
        && ring.iter().all(|[dx, dz]| world.is(BlockPos::new(x + dx, y, z + dz), "minecraft:redstone_wire"))
}

fn search(world: World, pos: [f64; 3]) -> Option<[i32; 3]>
{
    let [x, y, z]: [i32; 3] = pos.map(floor);
    for sx in x - tuning::REACH..x + tuning::REACH
    {
        for sz in z - tuning::REACH..z + tuning::REACH
        {
            if let Some(sy) = (y - tuning::RISE..y + tuning::RISE).find(|sy| symbol(world, sx, *sy, sz))
            {
                return Some([sx, sy, sz]);
            }
        }
    }

    None
}

fn sparks(world: World, info: &EntityInfo)
{
    for _ in 0..tuning::SPARKS
    {
        let spread = || (crate::math::fraction(world) - crate::math::fraction(world)) * tuning::SPREAD;
        let at: [f64; 3] = [
            info.pos[0] + f64::from(spread()),
            info.pos[1] + f64::from(EYE + spread()),
            info.pos[2] + f64::from(spread()),
        ];
        crate::particles::spawn(world, "annihilate", at, [0.0; 3]);
    }
}

struct Cube;

#[entity_behavior]
impl EntityBehavior for Cube
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        // the sparks are the client's own, as TF draws them on the remote world only
        if !world.isAuthority()
        {
            sparks(world, info);
            return;
        }

        let mut state: Symbol = entity.data();

        if world.random(tuning::SEARCH_CHANCE) == 0
        {
            state.m_pos = search(world, info.pos).or(state.m_pos);

            let _ = entity.setData(&state);
        }

        let Some([x, y, z]) = state.m_pos
        else
        {
            return;
        };

        let goal: [f64; 3] = [f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5];
        let (dx, dz): (f64, f64) = (goal[0] - info.pos[0], goal[2] - info.pos[2]);
        let gap: f64 = flatLength(dx, dz);
        if gap < tuning::CENTERED || gap > tuning::GIVE_UP
        {
            return;
        }

        let step: f64 = tuning::SPEED.min(gap);
        let _ = entity.setMotion([dx / gap * step, info.motion[1], dz / gap * step]);
    }

    // RenderTFRovingCube spins the cube of annihilation model twice its size, see through and unlit
    fn pose(&self, input: &PoseInput) -> Pose
    {
        let mut program: Program = Program::new();
        program
            .scale(tuning::SIZE, tuning::SIZE, tuning::SIZE)
            .rotate((input.age * tuning::SPIN) % 360.0, 0.0, 1.0, 0.0)
            .lighting(false)
            .blend(true)
            .translate(0.0, tuning::LIFT, 0.0);

        for part in 0..tf_cube_of_annihilation::PIVOTS.len() as u16
        {
            program.rest(part);
        }

        program.lighting(true).blend(false);
        Pose { pre: Vec::new(), model: program.take() }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            // the source cube runs no target task so it never fights back
            template: Identifier::parse("minecraft:passive_monster").ok(),
            texture: Some("twilightforest:model/cubeofannihilation".to_owned()),
            size: Some([1.2, 2.1]),
            health: Some(10),
            speed: Some(0.5),
            name: Some("Roving Cube [NYI]".to_owned()),
            model: Some(EntityModelSpec { shadow: 0.0, ..tf_cube_of_annihilation::model() }),
            persistent: true,
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(Cube)),
    )?;
    Ok(())
}
