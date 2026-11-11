use super::{Program, biped, limbRotations};
use b173_api::{EntityModelSpec, ModelBox, ModelPart, PoseInput};

// RenderBiped's armor passes: two ModelBiped layers drawn over a mob with its own limb swing, whatever its model
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Piece
{
    Helmet,
    Chestplate,
    Leggings,
    Boots,
}

// one worn piece and the sheet it draws from, as ItemArmor names armor/<sheet>_1 and _2
pub(crate) struct Worn
{
    pub m_piece: Piece,
    pub m_sheet: &'static str,
}

struct Layer
{
    m_inflate: f32,
    m_headwear: f32,
}

// RenderBiped's field_82423_g and field_82425_h, ModelBiped(1.0) and ModelBiped(0.5)
const LAYERS: [Layer; 2] = [Layer { m_inflate: 1.0, m_headwear: 1.5 }, Layer { m_inflate: 0.5, m_headwear: 1.0 }];
const SHEET: [i32; 2] = [64, 32];

// ModelBiped's boxes in part order: origin, size, uv, rest pivot and mirror
struct Bone
{
    m_origin: [f32; 3],
    m_size: [i32; 3],
    m_uv: [i32; 2],
    m_pivot: [f32; 3],
    m_mirror: bool,
}

const BONES: [Bone; 7] = [
    Bone { m_origin: [-4.0, -8.0, -4.0], m_size: [8, 8, 8], m_uv: [0, 0], m_pivot: [0.0; 3], m_mirror: false },
    Bone { m_origin: [-4.0, -8.0, -4.0], m_size: [8, 8, 8], m_uv: [32, 0], m_pivot: [0.0; 3], m_mirror: false },
    Bone { m_origin: [-4.0, 0.0, -2.0], m_size: [8, 12, 4], m_uv: [16, 16], m_pivot: [0.0; 3], m_mirror: false },
    Bone
    {
        m_origin: [-3.0, -2.0, -2.0],
        m_size: [4, 12, 4],
        m_uv: [40, 16],
        m_pivot: [-5.0, 2.0, 0.0],
        m_mirror: false,
    },
    Bone { m_origin: [-1.0, -2.0, -2.0], m_size: [4, 12, 4], m_uv: [40, 16], m_pivot: [5.0, 2.0, 0.0], m_mirror: true },
    Bone
    {
        m_origin: [-2.0, 0.0, -2.0],
        m_size: [4, 12, 4],
        m_uv: [0, 16],
        m_pivot: [-1.9, 12.0, 0.0],
        m_mirror: false,
    },
    Bone { m_origin: [-2.0, 0.0, -2.0], m_size: [4, 12, 4], m_uv: [0, 16], m_pivot: [1.9, 12.0, 0.0], m_mirror: true },
];

// both armor layers as parts to append after a mob's own, seven per layer
pub(crate) fn model() -> EntityModelSpec
{
    let mut parts: Vec<ModelPart> = Vec::new();

    for (layer, row) in LAYERS.iter().enumerate()
    {
        for (index, bone) in BONES.iter().enumerate()
        {
            let inflate: f32 = if index as u16 == biped::HEADWEAR
            {
                row.m_headwear
            }
            else
            {
                row.m_inflate
            };

            parts.push(ModelPart
            {
                name: format!("armor_{layer}_{index}"),
                uv: bone.m_uv,
                mirror: bone.m_mirror,
                pivot: bone.m_pivot,
                boxes: vec![ModelBox { origin: bone.m_origin, size: bone.m_size, inflate, uv: None }],
                texture_size: Some(SHEET),
                ..ModelPart::default()
            });
        }
    }

    EntityModelSpec { texture_size: SHEET, parts, ..EntityModelSpec::default() }
}

// RenderBiped.shouldRenderPass, which layer a piece draws from and which parts it shows
fn shown(piece: Piece) -> (usize, &'static [u16])
{
    match piece
    {
        Piece::Helmet => (0, &[biped::HEAD, biped::HEADWEAR]),
        Piece::Chestplate => (0, &[biped::BODY, biped::RIGHT_ARM, biped::LEFT_ARM]),
        Piece::Leggings => (1, &[biped::BODY, biped::RIGHT_LEG, biped::LEFT_LEG]),
        Piece::Boots => (0, &[biped::RIGHT_LEG, biped::LEFT_LEG]),
    }
}

// the worn pieces over a mob whose armor parts start at base, swinging with its own walk
pub(crate) fn draw(program: &mut Program, base: u16, worn: &[Worn], input: &PoseInput)
{
    let turns: [(u16, [f32; 3]); 7] = limbRotations(input, false);

    for piece in worn
    {
        let (layer, parts): (usize, &[u16]) = shown(piece.m_piece);
        program.texture(&format!("{}_{}", piece.m_sheet, layer + 1));

        let first: u16 = base + (layer * BONES.len()) as u16;

        for &(part, rotation) in turns.iter().filter(|(part, _)| parts.contains(part))
        {
            program.part(first + part, rotation, BONES[part as usize].m_pivot);
        }
    }
}
