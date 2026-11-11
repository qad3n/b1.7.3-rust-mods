use super::
{
    DEGREES_PER_RADIAN, HALF_PI, Program, WALK_RATE, part, tf_bunny, tf_death_tome, tf_fire_beetle, tf_helmet_crab,
    tf_ice_crystal, tf_ice_exploder, tf_ice_shooter, tf_mosquito_swarm, tf_penguin, tf_pinch_beetle, tf_raven,
    tf_slime_beetle, tf_squirrel, tf_tiny_bird, tf_tower_golem,
};
use b173_api::{EntityModelSpec, ModelPart, Pose, PoseInput, math};
use std::f32::consts::{FRAC_PI_4, FRAC_PI_8, PI};

// setRotationAngles and setLivingAnimations of the twilight models the converter leaves standing, plus the child
// parts it loses where a Java loop reuses one local for every spike's cube

fn head(input: &PoseInput) -> [f32; 3]
{
    [input.head_pitch / DEGREES_PER_RADIAN, input.head_yaw / DEGREES_PER_RADIAN, 0.0]
}

fn walk(input: &PoseInput, phase: f32, swing: f32) -> f32
{
    math::cos(input.walk * WALK_RATE + phase) * swing * input.walk_speed
}

// poses the listed parts, children before their parents, and draws every other part as it stands
fn finish(body: &mut Program, posed: &[(u16, [f32; 3], [f32; 3])], parts: usize) -> Pose
{
    for (part, rotation, pivot) in posed
    {
        body.part(*part, *rotation, *pivot);
    }

    for part in 0..parts as u16
    {
        if posed.iter().all(|(posedPart, _, _)| *posedPart != part)
        {
            body.rest(part);
        }
    }

    Pose { pre: Vec::new(), model: body.take() }
}

// one leg of a scuttling bug, its resting yaw and roll and the scuttle phase it moves in
struct Leg
{
    m_part: u16,
    m_yaw: f32,
    m_roll: f32,
    m_phase: f32,
    m_sign: f32,
}

const fn leg(part: u16, yaw: f32, roll: f32, phase: f32, sign: f32) -> Leg
{
    Leg { m_part: part, m_yaw: yaw, m_roll: roll, m_phase: phase, m_sign: sign }
}

const SCUTTLE: f32 = 0.4;
// the beetles' legZ, ModelSpider's quarter turn for the crab
const BEETLE_ROLL: f32 = 0.28559935;
const MIDDLE_ROLL: f32 = 0.74;
const THREE_HALVES_PI: f32 = PI * 3.0 / 2.0;

fn scuttle(input: &PoseInput, legs: &[Leg], pivots: &[[f32; 3]]) -> Vec<(u16, [f32; 3], [f32; 3])>
{
    let step: f32 = input.walk * WALK_RATE;
    legs.iter()
        .map(|leg|
        {
            let swing: f32 = -(math::cos(step * 2.0 + leg.m_phase) * SCUTTLE) * input.walk_speed;
            let lift: f32 = (math::sin(step + leg.m_phase) * SCUTTLE).abs() * input.walk_speed;
            let rotation: [f32; 3] = [0.0, leg.m_yaw + swing * leg.m_sign, leg.m_roll + lift * leg.m_sign];
            (leg.m_part, rotation, pivots[leg.m_part as usize])
        })
        .collect()
}

// ModelTFFireBeetle, ModelTFSlimeBeetle and ModelTFPinchBeetle share six legs in three pairs
fn beetleLegs(legs: [u16; 6]) -> [Leg; 6]
{
    let [one, two, three, four, five, six] = legs;
    [
        leg(one, FRAC_PI_8 * 2.0, -BEETLE_ROLL, 0.0, 1.0),
        leg(two, -FRAC_PI_8 * 2.0, BEETLE_ROLL, 0.0, -1.0),
        leg(three, FRAC_PI_8, -BEETLE_ROLL * MIDDLE_ROLL, PI, 1.0),
        leg(four, -FRAC_PI_8, BEETLE_ROLL * MIDDLE_ROLL, PI, -1.0),
        leg(five, -FRAC_PI_8 * 2.0, -BEETLE_ROLL, THREE_HALVES_PI, 1.0),
        leg(six, FRAC_PI_8 * 2.0, BEETLE_ROLL, THREE_HALVES_PI, -1.0),
    ]
}

pub(crate) fn fireBeetlePose(input: &PoseInput) -> Pose
{
    let legs: [Leg; 6] = beetleLegs([
        tf_fire_beetle::LEG1,
        tf_fire_beetle::LEG2,
        tf_fire_beetle::LEG3,
        tf_fire_beetle::LEG4,
        tf_fire_beetle::LEG5,
        tf_fire_beetle::LEG6,
    ]);

    let mut posed: Vec<(u16, [f32; 3], [f32; 3])> = scuttle(input, &legs, &tf_fire_beetle::PIVOTS);
    posed.push((tf_fire_beetle::HEAD, head(input), tf_fire_beetle::PIVOTS[tf_fire_beetle::HEAD as usize]));
    finish(&mut Program::new(), &posed, tf_fire_beetle::PIVOTS.len())
}

pub(crate) fn pinchBeetlePose(input: &PoseInput) -> Pose
{
    let legs: [Leg; 6] = beetleLegs([
        tf_pinch_beetle::LEG1,
        tf_pinch_beetle::LEG2,
        tf_pinch_beetle::LEG3,
        tf_pinch_beetle::LEG4,
        tf_pinch_beetle::LEG5,
        tf_pinch_beetle::LEG6,
    ]);

    let mut posed: Vec<(u16, [f32; 3], [f32; 3])> = scuttle(input, &legs, &tf_pinch_beetle::PIVOTS);
    posed.push((tf_pinch_beetle::HEAD, head(input), tf_pinch_beetle::PIVOTS[tf_pinch_beetle::HEAD as usize]));
    finish(&mut Program::new(), &posed, tf_pinch_beetle::PIVOTS.len())
}

// the slime beetle's tail and slime sway on their own clocks
pub(crate) fn slimeBeetlePose(input: &PoseInput) -> Pose
{
    const TAILS: [(f32, f32, f32); 4] =
        [(0.5555, 0.25, 0.25), (0.5555, 0.0, 0.25), (0.4445, 0.0, 0.2), (0.3335, 0.0, 0.15)];
    let legs: [Leg; 6] = beetleLegs([
        tf_slime_beetle::LEG1,
        tf_slime_beetle::LEG2,
        tf_slime_beetle::LEG3,
        tf_slime_beetle::LEG4,
        tf_slime_beetle::LEG5,
        tf_slime_beetle::LEG6,
    ]);

    let mut posed: Vec<_> = Vec::new();

    for (part, (rate, phase, reach)) in
        [tf_slime_beetle::SLIME_CENTER, tf_slime_beetle::SLIME_CUBE, tf_slime_beetle::TAIL2, tf_slime_beetle::TAIL1]
            .into_iter()
            .zip(TAILS)
    {
        posed.push((
            part,
            [math::cos(input.age * rate + phase) * reach, 0.0, 0.0],
            tf_slime_beetle::PIVOTS[part as usize],
        ));
    }

    posed.extend(scuttle(input, &legs, &tf_slime_beetle::PIVOTS));
    posed.push((tf_slime_beetle::HEAD, head(input), tf_slime_beetle::PIVOTS[tf_slime_beetle::HEAD as usize]));
    finish(&mut Program::new(), &posed, tf_slime_beetle::PIVOTS.len())
}

// ModelTFHelmetCrab, five legs and a claw arm that swings as it walks
pub(crate) fn helmetCrabPose(input: &PoseInput) -> Pose
{
    const ARM_YAW: f32 = -1.319531;

    let legs: [Leg; 5] = [
        leg(tf_helmet_crab::LEG3, FRAC_PI_8, -FRAC_PI_4 * MIDDLE_ROLL, PI, 1.0),
        leg(tf_helmet_crab::LEG4, -FRAC_PI_8, FRAC_PI_4 * MIDDLE_ROLL, PI, -1.0),
        leg(tf_helmet_crab::LEG5, -FRAC_PI_8, -FRAC_PI_4 * MIDDLE_ROLL, HALF_PI, 1.0),
        leg(tf_helmet_crab::LEG6, FRAC_PI_8, FRAC_PI_4 * MIDDLE_ROLL, HALF_PI, -1.0),
        leg(tf_helmet_crab::LEG8, FRAC_PI_8 * 2.0, FRAC_PI_4, THREE_HALVES_PI, -1.0),
    ];

    let mut posed: Vec<(u16, [f32; 3], [f32; 3])> = scuttle(input, &legs, &tf_helmet_crab::PIVOTS);
    let arm: f32 = ARM_YAW + walk(input, PI, 1.0);
    posed.push((
        tf_helmet_crab::RIGHT_ARM,
        [0.0, arm, 0.0],
        tf_helmet_crab::PIVOTS[tf_helmet_crab::RIGHT_ARM as usize],
    ));
    posed.push((tf_helmet_crab::BODY, head(input), tf_helmet_crab::PIVOTS[tf_helmet_crab::BODY as usize]));
    finish(&mut Program::new(), &posed, tf_helmet_crab::PIVOTS.len())
}

const LEG_SWING: f32 = 1.4;

// ModelTFBunny, a quadruped on four short legs
pub(crate) fn bunnyPose(input: &PoseInput) -> Pose
{
    let posed: Vec<(u16, [f32; 3], [f32; 3])> =
        [(tf_bunny::LEG1, 0.0), (tf_bunny::LEG2, PI), (tf_bunny::LEG3, PI), (tf_bunny::LEG4, 0.0)]
            .into_iter()
            .map(|(part, phase)| (part, [walk(input, phase, LEG_SWING), 0.0, 0.0], tf_bunny::PIVOTS[part as usize]))
            .chain([(tf_bunny::HEAD, head(input), tf_bunny::PIVOTS[tf_bunny::HEAD as usize])])
            .collect();
    finish(&mut Program::new(), &posed, tf_bunny::PIVOTS.len())
}

// ModelTFSquirrel, its tail lashes when it runs and curls when it sits
pub(crate) fn squirrelPose(input: &PoseInput) -> Pose
{
    const RUNNING: f32 = 0.2;
    const WIGGLE: f32 = 0.6;

    let tail: [f32; 3] = if input.walk_speed > RUNNING
    {
        let wiggle: f32 = input.walk_speed.min(WIGGLE);
        [
            (math::cos(input.age * WALK_RATE) - PI / 3.0) * wiggle,
            math::cos(input.age * 0.7774) * 1.2 * wiggle,
            math::cos(input.age * 0.8886 + HALF_PI) * LEG_SWING * wiggle,
        ]
    }
    else
    {
        [
            0.2 + math::cos(input.age * 0.3335) * 0.15,
            0.1 + math::cos(input.age * 0.4445) * 0.2,
            0.1 + math::cos(input.age * 0.5555) * 0.25,
        ]
    };
    let mut posed: Vec<(u16, [f32; 3], [f32; 3])> = vec![
        (tf_squirrel::FLUFF3, [tail[2], 0.0, 0.0], tf_squirrel::PIVOTS[tf_squirrel::FLUFF3 as usize]),
        (tf_squirrel::FLUFF2, [tail[1], 0.0, 0.0], tf_squirrel::PIVOTS[tf_squirrel::FLUFF2 as usize]),
        (tf_squirrel::TAIL, [tail[0], 0.0, 0.0], tf_squirrel::PIVOTS[tf_squirrel::TAIL as usize]),
    ];

    for (part, phase) in
        [(tf_squirrel::LEG1, 0.0), (tf_squirrel::LEG2, PI), (tf_squirrel::LEG3, PI), (tf_squirrel::LEG4, 0.0)]
    {
        posed.push((part, [walk(input, phase, LEG_SWING), 0.0, 0.0], tf_squirrel::PIVOTS[part as usize]));
    }

    posed.push((tf_squirrel::HEAD, head(input), tf_squirrel::PIVOTS[tf_squirrel::HEAD as usize]));
    finish(&mut Program::new(), &posed, tf_squirrel::PIVOTS.len())
}

// RenderTFBird.getWingRotation, wings beat while it is off the ground
fn wings(input: &PoseInput) -> f32
{
    const BEAT: f32 = 2.0;

    if input.on_ground
    {
        0.0
    }
    else
    {
        math::sin(input.age * BEAT) + 1.0
    }
}

struct Bird
{
    m_head: u16,
    m_arms: [u16; 2],
    m_legs: [u16; 2],
    m_pivots: &'static [[f32; 3]],
    // leg pivot heights landed and flying
    m_legHeight: [f32; 2],
    m_legSwing: f32,
    m_legRate: f32,
}

// ModelTFTinyBird, ModelTFRaven and ModelTFPenguin
fn birdPose(input: &PoseInput, bird: &Bird, tilt: f32) -> Pose
{
    let flap: f32 = wings(input);

    let height: f32 = if input.on_ground
    {
        bird.m_legHeight[0]
    }
    else
    {
        bird.m_legHeight[1]
    };
    let step: f32 = input.walk * bird.m_legRate;
    let mut posed: Vec<(u16, [f32; 3], [f32; 3])> = Vec::new();

    for (index, part) in bird.m_legs.into_iter().enumerate()
    {
        let phase: f32 = if index == 0 { 0.0 } else { PI };
        let [x, _, z] = bird.m_pivots[part as usize];
        posed.push((part, [math::cos(step + phase) * bird.m_legSwing * input.walk_speed, 0.0, 0.0], [x, height, z]));
    }

    posed.push((bird.m_arms[0], [0.0, 0.0, flap], bird.m_pivots[bird.m_arms[0] as usize]));
    posed.push((bird.m_arms[1], [0.0, 0.0, -flap], bird.m_pivots[bird.m_arms[1] as usize]));

    let [pitch, yaw, _] = head(input);
    posed.push((bird.m_head, [pitch, yaw, tilt], bird.m_pivots[bird.m_head as usize]));
    finish(&mut Program::new(), &posed, bird.m_pivots.len())
}

pub(crate) fn tinyBirdPose(input: &PoseInput) -> Pose
{
    let bird: Bird = Bird
    {
        m_head: tf_tiny_bird::HEAD,
        m_arms: [tf_tiny_bird::RIGHTARM, tf_tiny_bird::LEFTARM],
        m_legs: [tf_tiny_bird::RIGHTLEG, tf_tiny_bird::LEFTLEG],
        m_pivots: &tf_tiny_bird::PIVOTS,
        m_legHeight: [23.0, 22.5],
        m_legSwing: LEG_SWING,
        m_legRate: WALK_RATE,
    };
    birdPose(input, &bird, 0.0)
}

// the raven cocks its head once it looks far enough aside
pub(crate) fn ravenPose(input: &PoseInput) -> Pose
{
    const COCK_FROM: f32 = 5.0;

    let bird: Bird = Bird
    {
        m_head: tf_raven::HEAD,
        m_arms: [tf_raven::RIGHTARM, tf_raven::LEFTARM],
        m_legs: [tf_raven::RIGHTLEG, tf_raven::LEFTLEG],
        m_pivots: &tf_raven::PIVOTS,
        m_legHeight: [21.0, 20.0],
        m_legSwing: LEG_SWING,
        m_legRate: WALK_RATE,
    };

    let tilt: f32 = if input.head_yaw > COCK_FROM
    {
        -PI / 12.0
    }
    else
    {
        0.0
    };
    birdPose(input, &bird, tilt)
}

// the penguin waddles on short strides at the raw walk rate
pub(crate) fn penguinPose(input: &PoseInput) -> Pose
{
    const WADDLE: f32 = 0.7;

    let height: f32 = tf_penguin::PIVOTS[tf_penguin::RIGHTLEG as usize][1];

    let bird: Bird = Bird
    {
        m_head: tf_penguin::HEAD,
        m_arms: [tf_penguin::RIGHTARM, tf_penguin::LEFTARM],
        m_legs: [tf_penguin::RIGHTLEG, tf_penguin::LEFTLEG],
        m_pivots: &tf_penguin::PIVOTS,
        m_legHeight: [height, height],
        m_legSwing: WADDLE,
        m_legRate: 1.0,
    };
    birdPose(input, &bird, 0.0)
}

// ModelTFTowerGolem's triangle wave, func_78172_a
fn triangle(value: f32, period: f32) -> f32
{
    ((value % period - period * 0.5).abs() - period * 0.25) / (period * 0.25)
}

// ModelTFTowerGolem, stiff legs and arms that sway against them
pub(crate) fn towerGolemPose(input: &PoseInput) -> Pose
{
    const STRIDE: f32 = 13.0;
    const ARM_STRIDE: f32 = 25.0;
    const SWING: f32 = 1.5;
    const HANG: f32 = 0.2;
    const SWAY: f32 = 0.05;
    const SWAY_RATE: f32 = 0.09;

    let legs: f32 = SWING * triangle(input.walk, STRIDE) * input.walk_speed;
    let arms: f32 = SWING * triangle(input.walk, ARM_STRIDE);
    let sway: f32 = math::cos(input.age * SWAY_RATE) * SWAY + SWAY;
    let posed: [(u16, [f32; 3], [f32; 3]); 5] = [
        (tf_tower_golem::HEAD, head(input), tf_tower_golem::PIVOTS[tf_tower_golem::HEAD as usize]),
        (tf_tower_golem::LEFTLEG, [-legs, 0.0, 0.0], tf_tower_golem::PIVOTS[tf_tower_golem::LEFTLEG as usize]),
        (tf_tower_golem::RIGHTLEG, [legs, 0.0, 0.0], tf_tower_golem::PIVOTS[tf_tower_golem::RIGHTLEG as usize]),
        (
            tf_tower_golem::RIGHTARM,
            [(-HANG + arms) * input.walk_speed, 0.0, sway],
            tf_tower_golem::PIVOTS[tf_tower_golem::RIGHTARM as usize],
        ),
        (
            tf_tower_golem::LEFTARM,
            [(-HANG - arms) * input.walk_speed, 0.0, -sway],
            tf_tower_golem::PIVOTS[tf_tower_golem::LEFTARM as usize],
        ),
    ];
    finish(&mut Program::new(), &posed, tf_tower_golem::PIVOTS.len())
}

// ModelTFMosquitoSwarm, the six clouds of bugs tumble around the core
pub(crate) fn mosquitoPose(input: &PoseInput) -> Pose
{
    let t: f32 = input.age;
    let nodes: [(u16, [f32; 3]); 6] = [
        (tf_mosquito_swarm::NODE1, [math::sin(t / 6.0) / 2.0, t / 2.0, math::cos(t / 5.0) / 4.0]),
        (tf_mosquito_swarm::NODE2, [t / 5.0, math::sin(t / 2.0) / 3.0, math::cos(t / 5.0) / 4.0]),
        (tf_mosquito_swarm::NODE3, [math::cos(t / 4.0) / 2.0, math::sin(t / 7.0) / 3.0, t / 5.0]),
        (tf_mosquito_swarm::NODE4, [t / 2.0, math::sin(t / 5.0) / 4.0, math::sin(t / 6.0) / 2.0]),
        (tf_mosquito_swarm::NODE5, [math::cos(t / 5.0) / 4.0, math::cos(t / 5.0) / 4.0, math::sin(t / 2.0) / 3.0]),
        (tf_mosquito_swarm::NODE6, [math::cos(t / 4.0) / 2.0, t / 5.0, math::cos(t / 7.0) / 3.0]),
    ];

    let mut posed: Vec<(u16, [f32; 3], [f32; 3])> =
        nodes.iter().map(|(part, rotation)| (*part, *rotation, tf_mosquito_swarm::PIVOTS[*part as usize])).collect();
    posed.push((
        tf_mosquito_swarm::CORE,
        [math::sin(t / 5.0) / 4.0, t / 5.0, math::cos(t / 5.0) / 4.0],
        tf_mosquito_swarm::PIVOTS[tf_mosquito_swarm::CORE as usize],
    ));
    finish(&mut Program::new(), &posed, tf_mosquito_swarm::PIVOTS.len())
}

// ModelTFDeathTome, the book bobs and flaps open while loose pages circle it
pub(crate) fn deathTomePose(input: &PoseInput) -> Pose
{
    const OPEN: f32 = 0.9;
    const FLIP_RIGHT: f32 = 0.4;
    const FLIP_LEFT: f32 = 0.6;
    const TILT: f32 = -0.87266463;

    let t: f32 = input.age;
    let open: f32 = (math::sin(t * 0.4) * 0.3 + 1.25) * OPEN;
    let spread: f32 = math::sin(open);
    let at = |part: u16, x: f32| [x, tf_death_tome::PIVOTS[part as usize][1], tf_death_tome::PIVOTS[part as usize][2]];
    let pages: [(u16, f32, Option<f32>); 6] = [
        (tf_death_tome::FIELD_78102_A, PI + open, None),
        (tf_death_tome::FIELD_78100_B, -open, None),
        (tf_death_tome::FIELD_78101_C, open, Some(spread)),
        (tf_death_tome::FIELD_78098_D, -open, Some(spread)),
        (tf_death_tome::FIELD_78099_E, open - open * 2.0 * FLIP_RIGHT, Some(spread)),
        (tf_death_tome::FIELD_78096_F, open - open * 2.0 * FLIP_LEFT, Some(spread)),
    ];

    let mut posed: Vec<(u16, [f32; 3], [f32; 3])> = pages
        .iter()
        .map(|(part, yaw, x)|
        {
            (*part, [0.0, *yaw, 0.0], x.map_or(tf_death_tome::PIVOTS[*part as usize], |x| at(*part, x)))
        })
        .collect();
    let loose: [(u16, [f32; 3]); 4] = [
        (tf_death_tome::LOOSE_PAGE1, [math::sin(t / 5.0) / 3.0, t / 4.0, math::cos(t / 5.0) / 5.0]),
        (tf_death_tome::LOOSE_PAGE2, [math::sin(t / 5.0) / 3.0, t / 3.0, math::cos(t / 5.0) / 4.0 + 2.0]),
        (tf_death_tome::LOOSE_PAGE3, [-math::sin(t / 5.0) / 3.0, t / 4.0, math::cos(t / 5.0) / 5.0 - 1.0]),
        (tf_death_tome::LOOSE_PAGE4, [-math::sin(t / 2.0) / 4.0, t / 4.0, math::cos(t / 7.0) / 5.0]),
    ];
    posed.extend(loose.iter().map(|(part, rotation)| (*part, *rotation, tf_death_tome::PIVOTS[*part as usize])));

    let bob: [f32; 3] = [0.0, 4.0 + math::sin(t * 0.3) * 2.0, 0.0];
    posed.push((tf_death_tome::BOOK, [0.0, 0.0, TILT], bob));
    posed.push((
        tf_death_tome::EVERYTHING,
        [0.0, input.head_pitch / DEGREES_PER_RADIAN + HALF_PI, 0.0],
        tf_death_tome::PIVOTS[tf_death_tome::EVERYTHING as usize],
    ));
    finish(&mut Program::new(), &posed, tf_death_tome::PIVOTS.len())
}

const SPIKES: usize = 16;

// the loop in ModelTFIceExploder and ModelTFIceCrystal hangs one cube on every spike, the converter kept a single cube
fn spikeCubes(mut model: EntityModelSpec, cube: usize, height: impl Fn(usize) -> f32) -> EntityModelSpec
{
    let mut template: ModelPart = model.parts[cube].clone();
    template.boxes.truncate(1);

    model.parts[cube] = ModelPart
    {
        parent: Some(format!("spikes_{}", SPIKES - 1)),
        pivot: [0.0, height(SPIKES - 1), 0.0],
        ..template.clone()
    };

    for spike in 0..SPIKES - 1
    {
        let pivot: [f32; 3] = [0.0, height(spike), 0.0];
        model.parts.push(part(
            &format!("cube_{spike}"),
            Some(&format!("spikes_{spike}")),
            template.uv,
            false,
            pivot,
            template.rotation,
            template.boxes.clone(),
            None,
        ));
    }

    model
}

// ModelTFIceExploder's cube sits eight down every spike
const CORE_CUBE: f32 = 8.0;

pub(crate) fn iceExploderModel() -> EntityModelSpec
{
    spikeCubes(tf_ice_exploder::model(), tf_ice_exploder::CUBE as usize, |_| CORE_CUBE)
}

pub(crate) fn iceShooterModel() -> EntityModelSpec
{
    spikeCubes(tf_ice_shooter::model(), tf_ice_shooter::CUBE as usize, |_| CORE_CUBE)
}

// ModelTFIceCrystal's cube caps each spike, six or eight long
pub(crate) fn iceCrystalModel() -> EntityModelSpec
{
    spikeCubes(tf_ice_crystal::model(), tf_ice_crystal::CUBE as usize, |spike| if spike % 2 == 0 { 6.0 } else { 8.0 })
}

fn spikeParts(first: u16) -> impl Iterator<Item = (usize, u16)>
{
    (0..SPIKES).map(move |spike| (spike, first + spike as u16))
}

// ModelTFIceExploder.setLivingAnimations, the spikes orbit the core on drifting paths
pub(crate) fn iceExploderPose(input: &PoseInput) -> Pose
{
    let t: f32 = input.age;
    let mut body: Program = Program::new();
    body.color(1.0, 1.0, 1.0, SPIKE_ALPHA).blend(true);

    let base: u16 = tf_ice_exploder::PIVOTS.len() as u16;

    for (spike, part) in spikeParts(tf_ice_exploder::SPIKES_0)
    {
        let i: f32 = spike as f32;
        // spike 0 divides by zero in the source and sits wherever NaN puts it, here it stays at the core
        let orbit: f32 = if spike == 0 { 0.0 } else { t / i };

        let cube: u16 = if spike == SPIKES - 1
        {
            tf_ice_exploder::CUBE
        }
        else
        {
            base + spike as u16
        };
        let cubeDrop: f32 = 10.0
            + if spike == 0
            {
                0.0
            }
            else
            {
                math::sin((i + t) / i) * 3.0
            };
        body.part(cube, tf_ice_exploder::ROTATIONS[tf_ice_exploder::CUBE as usize], [0.0, cubeDrop, 0.0]);

        let rotation: [f32; 3] =
            [math::sin(t / 5.0) / 4.0 + i * 5.0, t / 5.0 + i * 2.5, math::cos(t / 5.0) / 4.0 + i * 3.0];
        let pivot: [f32; 3] = [math::cos(orbit) * 3.0, 5.0 + math::sin(orbit) * 3.0, math::sin(orbit) * 3.0];
        body.part(part, rotation, pivot);
    }

    body.blend(false).color(1.0, 1.0, 1.0, 1.0);
    Pose { pre: Vec::new(), model: body.take() }
}

// ModelTFIceShooter, spikes fanned out and bobbing in place
pub(crate) fn iceShooterPose(input: &PoseInput) -> Pose
{
    const FAN: f32 = 1.570795;

    let t: f32 = input.age;
    let mut body: Program = Program::new();
    body.color(1.0, 1.0, 1.0, SPIKE_ALPHA).blend(true);

    let base: u16 = tf_ice_shooter::PIVOTS.len() as u16;

    for (spike, part) in spikeParts(tf_ice_shooter::SPIKES_0)
    {
        let i: f32 = spike as f32;

        let cube: u16 = if spike == SPIKES - 1
        {
            tf_ice_shooter::CUBE
        }
        else
        {
            base + spike as u16
        };
        body.part(
            cube,
            tf_ice_shooter::ROTATIONS[tf_ice_shooter::CUBE as usize],
            [0.0, 9.5 + math::sin((i + t) / 3.0) * 3.0, 0.0],
        );

        let rotation: [f32; 3] = [t / 5.0 + i * FRAC_PI_8, FAN + math::sin(t / 5.0) * 0.5, math::cos(i / 5.0) / 4.0];
        body.part(part, rotation, tf_ice_shooter::PIVOTS[part as usize]);
    }

    body.blend(false).color(1.0, 1.0, 1.0, 1.0);
    Pose { pre: Vec::new(), model: body.take() }
}

// ModelTFIceCrystal, the star turns with every other spike leaning out
pub(crate) fn iceCrystalPose(input: &PoseInput) -> Pose
{
    let t: f32 = input.age;
    let mut body: Program = Program::new();
    body.color(1.0, 1.0, 1.0, SPIKE_ALPHA).blend(true);

    for (spike, part) in spikeParts(tf_ice_crystal::SPIKES_0)
    {
        let i: f32 = spike as f32;

        let lean: f32 = match spike % 4
        {
            0 => 1.0,
            2 => -1.0,
            _ => 0.0,
        };
        let rotation: [f32; 3] = [math::sin(t / 5.0) / 4.0 + i * FRAC_PI_8, t / 5.0 + lean, math::cos(t / 5.0) / 4.0];
        body.part(part, rotation, tf_ice_crystal::PIVOTS[part as usize]);
    }

    body.blend(false).color(1.0, 1.0, 1.0, 1.0);
    Pose { pre: Vec::new(), model: body.take() }
}

// the ice models draw their spikes see through while alive
const SPIKE_ALPHA: f32 = 0.6;
