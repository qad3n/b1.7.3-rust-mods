// converter output keeps the Java float literals, 3.141593F and friends
pub(crate) mod animated;
pub(crate) mod armor;
#[allow(dead_code, clippy::approx_constant, clippy::excessive_precision)]
mod generated;
mod vanilla;

use b173_api::{EntityModelSpec, ModelBox, ModelOp, ModelPart, PoseInput, math};
pub(crate) use generated::*;
use std::vec::IntoIter;
pub(crate) use vanilla::{ghastModel, ghastPose, spiderPose, wolfPose};

// builders the generated models call, kept short so each generated part stays on one line
pub(crate) fn part(
    name: &str,
    parent: Option<&str>,
    uv: [i32; 2],
    mirror: bool,
    pivot: [f32; 3],
    rotation: [f32; 3],
    boxes: Vec<ModelBox>,
    textureSize: Option<[i32; 2]>,
) -> ModelPart
{
    ModelPart
    {
        name: name.to_owned(),
        parent: parent.map(str::to_owned),
        uv,
        mirror,
        pivot,
        rotation,
        boxes,
        texture_size: textureSize,
    }
}

pub(crate) fn cuboid(origin: [f32; 3], size: [i32; 3], inflate: f32, uv: Option<[i32; 2]>) -> ModelBox
{
    ModelBox { origin, size, inflate, uv }
}

// a pose program under construction, one call per ModelRenderer.render in the Java model
pub(crate) struct Program
{
    pub m_ops: Vec<ModelOp>,
}

impl Program
{
    pub(crate) fn new() -> Self { Self { m_ops: Vec::new() } }

    pub(crate) fn part(&mut self, part: u16, rotation: [f32; 3], pivot: [f32; 3]) -> &mut Self
    {
        self.m_ops.push(ModelOp::Part { part, rotation, pivot });
        self
    }

    pub(crate) fn rest(&mut self, part: u16) -> &mut Self
    {
        self.m_ops.push(ModelOp::Rest(part));
        self
    }

    pub(crate) fn push(&mut self) -> &mut Self
    {
        self.m_ops.push(ModelOp::Push);
        self
    }

    pub(crate) fn pop(&mut self) -> &mut Self
    {
        self.m_ops.push(ModelOp::Pop);
        self
    }

    pub(crate) fn translate(&mut self, x: f32, y: f32, z: f32) -> &mut Self
    {
        self.m_ops.push(ModelOp::Translate([x, y, z]));
        self
    }

    pub(crate) fn rotate(&mut self, degrees: f32, x: f32, y: f32, z: f32) -> &mut Self
    {
        self.m_ops.push(ModelOp::Rotate(degrees, [x, y, z]));
        self
    }

    pub(crate) fn scale(&mut self, x: f32, y: f32, z: f32) -> &mut Self
    {
        self.m_ops.push(ModelOp::Scale([x, y, z]));
        self
    }

    pub(crate) fn color(&mut self, red: f32, green: f32, blue: f32, alpha: f32) -> &mut Self
    {
        self.m_ops.push(ModelOp::Color([red, green, blue, alpha]));
        self
    }

    pub(crate) fn texture(&mut self, texture: &str) -> &mut Self
    {
        self.m_ops.push(ModelOp::Texture(texture.to_owned()));
        self
    }

    pub(crate) fn blend(&mut self, on: bool) -> &mut Self
    {
        self.m_ops.push(ModelOp::Blend(on));
        self
    }

    pub(crate) fn faceCamera(&mut self) -> &mut Self
    {
        self.m_ops.push(ModelOp::FaceCamera);
        self
    }

    pub(crate) fn normal(&mut self, x: f32, y: f32, z: f32) -> &mut Self
    {
        self.m_ops.push(ModelOp::Normal([x, y, z]));
        self
    }

    // one quad of four x, y, z, u, v vertices
    pub(crate) fn quad(&mut self, vertices: [[f32; 5]; 4]) -> &mut Self
    {
        self.m_ops.push(ModelOp::Quads(vertices.to_vec()));
        self
    }

    pub(crate) fn lighting(&mut self, on: bool) -> &mut Self
    {
        self.m_ops.push(ModelOp::Lighting(on));
        self
    }

    pub(crate) fn take(&mut self) -> Vec<ModelOp> { std::mem::take(&mut self.m_ops) }

    // RenderBiped's held item, drawn in the hand at the end of the part
    pub(crate) fn held(&mut self, part: u16, item: &str) -> &mut Self
    {
        match b173_api::ItemStack::of(item, 1).toRaw()
        {
            Ok(stack) => self.m_ops.push(ModelOp::HeldItem { part, stack }),
            Err(error) => b173_api::warn(&format!("held item {item}: {}", error.0)),
        }

        self
    }

    pub(crate) fn extend(&mut self, ops: Vec<ModelOp>) -> &mut Self
    {
        self.m_ops.extend(ops);
        self
    }
}

// ModelBase angle conversions the Java models write inline
pub(crate) const DEGREES_PER_RADIAN: f32 = 180.0 / std::f32::consts::PI;
pub(crate) const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;

// several Java models drawn as one spec, a later model's part index is shifted by the parts before it
pub(crate) fn combine(models: Vec<EntityModelSpec>) -> EntityModelSpec
{
    let mut models: IntoIter<EntityModelSpec> = models.into_iter();
    let mut combined: EntityModelSpec = models.next().unwrap_or_default();

    for model in models
    {
        let size: [i32; 2] = model.texture_size;
        combined.parts.extend(
            model.parts.into_iter().map(|part| ModelPart { texture_size: part.texture_size.or(Some(size)), ..part }),
        );
    }

    combined
}

// ModelQuadruped part order: head, body, then four legs
pub(crate) mod quadruped
{
    pub(crate) const HEAD: u16 = 0;
    pub(crate) const BODY: u16 = 1;
    pub(crate) const LEGS: [u16; 4] = [2, 3, 4, 5];
    pub(crate) const PARTS: u16 = 6;
    pub(crate) const LEG_SWING: f32 = 1.4;
}

pub(crate) const WALK_RATE: f32 = 0.6662;

// ModelQuadruped.setRotationAngles and render for the six parts starting at base
pub(crate) fn quadrupedPose(program: &mut Program, base: u16, pivots: &[[f32; 3]], input: &PoseInput, bodyPitch: f32)
{
    let pivot = |part: u16| pivots[part as usize];
    let pitch: f32 = input.head_pitch / DEGREES_PER_RADIAN;
    program.part(base + quadruped::HEAD, [pitch, input.head_yaw / DEGREES_PER_RADIAN, 0.0], pivot(quadruped::HEAD));
    program.part(base + quadruped::BODY, [bodyPitch, 0.0, 0.0], pivot(quadruped::BODY));

    let swing: f32 = math::cos(input.walk * WALK_RATE) * quadruped::LEG_SWING * input.walk_speed;
    let back: f32 = math::cos(input.walk * WALK_RATE + std::f32::consts::PI) * quadruped::LEG_SWING * input.walk_speed;
    for (leg, angle) in quadruped::LEGS.iter().zip([swing, back, back, swing])
    {
        program.part(base + leg, [angle, 0.0, 0.0], pivot(*leg));
    }
}

// ModelBiped part order: head, headwear, body, arms then legs, right before left
pub(crate) mod biped
{
    pub(crate) const HEAD: u16 = 0;
    pub(crate) const HEADWEAR: u16 = 1;
    pub(crate) const BODY: u16 = 2;
    pub(crate) const RIGHT_ARM: u16 = 3;
    pub(crate) const LEFT_ARM: u16 = 4;
    pub(crate) const RIGHT_LEG: u16 = 5;
    pub(crate) const LEFT_LEG: u16 = 6;
    pub(crate) const PARTS: u16 = 7;
    pub(crate) const ARM_SWING: f32 = 2.0 * 0.5;
    pub(crate) const LEG_SWING: f32 = 1.4;
    pub(crate) const SWAY: f32 = 0.05;
    pub(crate) const SWAY_ROLL: f32 = 0.09;
    pub(crate) const SWAY_PITCH: f32 = 0.067;
}

// ModelBiped.setRotationAngles for the seven biped parts, the rest of the model's parts drawn as they stand
pub(crate) fn bipedPose(program: &mut Program, pivots: &[[f32; 3]], rotations: &[[f32; 3]], input: &PoseInput)
{
    limbPose(program, pivots, rotations, input, false);
}

// ModelZombie.setRotationAngles, arms held straight out in front
pub(crate) fn zombiePose(program: &mut Program, pivots: &[[f32; 3]], rotations: &[[f32; 3]], input: &PoseInput)
{
    limbPose(program, pivots, rotations, input, true);
}

// ModelTFLichMinion and ModelTFLoyalZombie, a zombie washed in a tint
pub(crate) fn minionPose(input: &PoseInput, tint: [f32; 3]) -> b173_api::Pose
{
    let mut body: Program = Program::new();
    body.color(tint[0], tint[1], tint[2], 1.0);
    zombiePose(&mut body, &tf_lich_minion::PIVOTS, &tf_lich_minion::ROTATIONS, input);
    body.color(1.0, 1.0, 1.0, 1.0);
    b173_api::Pose { pre: Vec::new(), model: body.take() }
}

// ModelZombie lifts the arms level, a tenth of a radian apart
const ZOMBIE_SPREAD: f32 = 0.1;

fn limbPose(program: &mut Program, pivots: &[[f32; 3]], rotations: &[[f32; 3]], input: &PoseInput, zombie: bool)
{
    for (part, rotation) in limbRotations(input, zombie)
    {
        program.part(part, rotation, pivots[part as usize]);
    }

    for part in biped::PARTS..pivots.len() as u16
    {
        program.part(part, rotations[part as usize], pivots[part as usize]);
    }
}

// ModelBiped.setRotationAngles for its seven parts
pub(crate) fn limbRotations(input: &PoseInput, zombie: bool) -> [(u16, [f32; 3]); 7]
{
    use std::f32::consts::PI;

    let head: [f32; 3] = [input.head_pitch / DEGREES_PER_RADIAN, input.head_yaw / DEGREES_PER_RADIAN, 0.0];
    let swing: f32 = input.walk * WALK_RATE;
    let mut right: [f32; 3] = [math::cos(swing + PI) * biped::ARM_SWING * input.walk_speed, 0.0, 0.0];
    let mut left: [f32; 3] = [math::cos(swing) * biped::ARM_SWING * input.walk_speed, 0.0, 0.0];
    let mut rightLeg: [f32; 3] = [math::cos(swing) * biped::LEG_SWING * input.walk_speed, 0.0, 0.0];
    let mut leftLeg: [f32; 3] = [math::cos(swing + PI) * biped::LEG_SWING * input.walk_speed, 0.0, 0.0];

    if input.riding
    {
        right[0] -= PI / 5.0;
        left[0] -= PI / 5.0;
        rightLeg = [-PI * 2.0 / 5.0, PI / 10.0, 0.0];
        leftLeg = [-PI * 2.0 / 5.0, -PI / 10.0, 0.0];
    }

    if zombie
    {
        right = [-HALF_PI, -ZOMBIE_SPREAD, 0.0];
        left = [-HALF_PI, ZOMBIE_SPREAD, 0.0];
    }

    let roll: f32 = math::cos(input.age * biped::SWAY_ROLL) * biped::SWAY + biped::SWAY;
    let pitch: f32 = math::sin(input.age * biped::SWAY_PITCH) * biped::SWAY;

    right[2] += roll;
    left[2] -= roll;
    right[0] += pitch;
    left[0] -= pitch;
    [
        (biped::HEAD, head),
        (biped::HEADWEAR, head),
        (biped::BODY, [0.0; 3]),
        (biped::RIGHT_ARM, right),
        (biped::LEFT_ARM, left),
        (biped::RIGHT_LEG, rightLeg),
        (biped::LEFT_LEG, leftLeg),
    ]
}
