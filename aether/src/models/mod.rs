#[allow(dead_code)]
// converter output keeps the Java float literals, 3.141593F and friends
#[allow(clippy::approx_constant, clippy::excessive_precision)]
mod generated;

use b173_api::terrain::math;
use b173_api::{EntityModelSpec, ModelBox, ModelOp, ModelPart, PoseInput};
pub(crate) use generated::*;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::vec::IntoIter;

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
pub(crate) fn quadrupedPose(
    program: &mut Program,
    base: u16,
    pivots: &[[f32; 3]],
    input: &PoseInput,
    headPitch: Option<f32>,
)
{
    let pivot = |part: u16| pivots[part as usize];
    let pitch: f32 = headPitch.unwrap_or(input.head_pitch / DEGREES_PER_RADIAN);
    program.part(base + quadruped::HEAD, [pitch, input.head_yaw / DEGREES_PER_RADIAN, 0.0], pivot(quadruped::HEAD));
    program.part(base + quadruped::BODY, [HALF_PI, 0.0, 0.0], pivot(quadruped::BODY));

    let swing: f32 = math::cos(input.walk * WALK_RATE) * quadruped::LEG_SWING * input.walk_speed;
    let back: f32 = math::cos(input.walk * WALK_RATE + std::f32::consts::PI) * quadruped::LEG_SWING * input.walk_speed;
    for (leg, angle) in quadruped::LEGS.iter().zip([swing, back, back, swing])
    {
        program.part(base + leg, [angle, 0.0, 0.0], pivot(*leg));
    }
}

// a per entity angle a Java entity steps once a tick for its renderer, kept on the client by the last tick seen
#[derive(Default)]
pub(crate) struct Phases
{
    m_phases: RefCell<HashMap<i32, (f32, i32)>>,
}

impl Phases
{
    pub(crate) fn advance(&self, id: i32, tick: i32, step: f32, wrap: f32) -> f32
    {
        let mut phases: RefMut<HashMap<i32, (f32, i32)>> = self.m_phases.borrow_mut();
        let entry: &mut (f32, i32) = phases.entry(id).or_insert((0.0, tick));
        while entry.1 < tick
        {
            entry.0 = (entry.0 + step) % wrap;
            entry.1 += 1;
        }

        entry.0
    }
}
