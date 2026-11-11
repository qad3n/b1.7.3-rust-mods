mod castle;
mod critters;
mod giant;
mod nagastone;
mod plants;
mod slider;
mod stone;
pub(crate) mod tower;
pub(crate) mod wood;

use b173_api::prelude::*;
pub(crate) use nagastone::{bodyMeta, headMeta};
pub(crate) use plants::{AXES, PAD_PIECES, QUARTERS, axisOf, padTurn, plantStays};
pub(crate) use stone::{LATENT_META, SHIELD_LATENT};

pub(crate) enum Look
{
    All(&'static str),
    Column
    {
        top: &'static str,
        side: &'static str,
    },
}

// what a family of blocks is made of, the material, its step sound and the tool that mines it
#[derive(Clone, Copy)]
pub(crate) enum Kind
{
    Rock,
    Wood,
    Metal,
    Ground,
    Cloud,
    Plant,
}

struct KindRow
{
    m_kind: Kind,
    m_material: &'static str,
    m_sound: &'static str,
    m_tool: Option<ToolClass>,
}

const KINDS: [KindRow; 6] = [
    KindRow { m_kind: Kind::Rock, m_material: "rock", m_sound: "stone", m_tool: Some(ToolClass::Pickaxe) },
    KindRow { m_kind: Kind::Wood, m_material: "wood", m_sound: "wood", m_tool: Some(ToolClass::Axe) },
    KindRow { m_kind: Kind::Metal, m_material: "iron", m_sound: "metal", m_tool: Some(ToolClass::Pickaxe) },
    KindRow { m_kind: Kind::Ground, m_material: "ground", m_sound: "gravel", m_tool: Some(ToolClass::Shovel) },
    KindRow { m_kind: Kind::Cloud, m_material: "rock", m_sound: "cloth", m_tool: None },
    KindRow { m_kind: Kind::Plant, m_material: "plants", m_sound: "grass", m_tool: None },
];

fn kindRow(kind: Kind) -> &'static KindRow
{
    KINDS.iter().find(|row| row.m_kind as u8 == kind as u8).expect("every kind has a row")
}

// a plain full cube, values from the source block constructors
pub(crate) struct Cube
{
    pub m_name: &'static str,
    pub m_label: &'static str,
    pub m_kind: Kind,
    pub m_look: Look,
    pub m_hardness: f32,
    pub m_resistance: Option<f32>,
}

pub(crate) const fn cube(name: &'static str, label: &'static str, kind: Kind, look: Look, hardness: f32) -> Cube
{
    Cube { m_name: name, m_label: label, m_kind: kind, m_look: look, m_hardness: hardness, m_resistance: None }
}

pub(crate) const fn tough(
    name: &'static str,
    label: &'static str,
    kind: Kind,
    look: Look,
    hardness: f32,
    resistance: f32,
) -> Cube
{
    Cube
    {
        m_name: name,
        m_label: label,
        m_kind: kind,
        m_look: look,
        m_hardness: hardness,
        m_resistance: Some(resistance),
    }
}

// model coordinates run in sixteenths of a block
pub(crate) const PIXELS: f32 = 16.0;

pub(crate) fn texture(name: &str) -> String { format!("twilightforest:block/{name}") }

pub(crate) fn vanillaTexture(sprite: &str) -> String { format!("minecraft:block/{sprite}") }

fn lookTexture(name: &str) -> String
{
    match name.strip_prefix("minecraft:")
    {
        Some(sprite) => vanillaTexture(sprite),
        None => texture(name),
    }
}

pub(crate) fn faces(look: &Look) -> FaceTextures
{
    match look
    {
        Look::All(name) => FaceTextures
        {
            top: Some(lookTexture(name)),
            bottom: Some(lookTexture(name)),
            side: Some(lookTexture(name)),
            front: None,
        },

        Look::Column { top, side } => FaceTextures
        {
            top: Some(lookTexture(top)),
            bottom: Some(lookTexture(top)),
            side: Some(lookTexture(side)),
            front: None,
        },
    }
}

pub(crate) fn drop(name: &str, count: Vec<i32>, chance: f32) -> Result<DropSpec>
{
    let item: Identifier = Identifier::parseIn(name, "twilightforest").map_err(|error| Error(error.to_string()))?;
    Ok(DropSpec { item, count, meta: 0, chance })
}

// the template every family starts from, before the family adds its own behavior
pub(crate) fn base(template: &str, label: &str, kind: Kind, hardness: f32) -> BlockSpecBuilder
{
    let row: &KindRow = kindRow(kind);
    let spec: BlockSpecBuilder =
        BlockSpecBuilder::template(template).name(label).material(row.m_material).sound(row.m_sound).hardness(hardness);

    match row.m_tool
    {
        Some(tool) => spec.tool(tool, 0),
        None => spec,
    }
}

pub(crate) fn build(row: &Cube) -> BlockSpecBuilder
{
    let spec: BlockSpecBuilder =
        base("minecraft:block", row.m_label, row.m_kind, row.m_hardness).faces(faces(&row.m_look));

    match row.m_resistance
    {
        Some(resistance) => spec.resistance(resistance),
        None => spec,
    }
}

pub(crate) fn put(ctx: &mut RegisterCtx, name: &str, spec: BlockSpecBuilder) -> Result<()>
{
    ctx.blocks().register(name, spec)?;
    Ok(())
}

pub(crate) fn registerCubes(ctx: &mut RegisterCtx, rows: &[Cube]) -> Result<()>
{
    for row in rows
    {
        put(ctx, row.m_name, build(row))?;
    }

    Ok(())
}

// a property listed in metadata order, so meta reads and writes keep their meaning
pub(crate) fn property(name: &str, values: &[&str]) -> PropertySpec
{
    PropertySpec { name: name.to_owned(), values: values.iter().map(|value| (*value).to_owned()).collect() }
}

pub(crate) fn when(name: &str, value: &str) -> std::collections::BTreeMap<String, String>
{
    std::collections::BTreeMap::from([(name.to_owned(), value.to_owned())])
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    wood::register(ctx)?;
    plants::register(ctx)?;
    stone::register(ctx)?;
    tower::register(ctx)?;
    castle::register(ctx)?;
    critters::register(ctx)?;
    nagastone::register(ctx)?;
    slider::register(ctx)?;
    giant::register(ctx)
}
