// draft from ComponentTFTowerRoofMushroom.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, wing: &Piece, pHang: f32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TowerRoofMushroom);
    init(&this, i, wing, pHang);
    Rc::new(this)
}

// ComponentTFTowerRoofMushroom constructor
pub fn init(this: &Piece, i: i32, wing: &Piece, pHang: f32)
{
    tower_roof::init(this, i, &wing);
    this.m_height.set(wing.m_size.get());
    let overhang: i32 = ((wing.m_size.get()) as f32 * pHang) as i32;
    this.m_size.set(wing.m_size.get() + (overhang * 2));
    this.setMode(0);
    this.m_box.set(StructureBox::new(
        [wing.bounds().m_min[0] - overhang, wing.bounds().m_max[1] + 2, wing.bounds().m_min[2] - overhang],
        [
            wing.bounds().m_max[0] + overhang,
            (wing.bounds().m_max[1] + this.m_height.get()) + 1,
            wing.bounds().m_max[2] + overhang,
        ],
    ));
}

// ComponentTFTowerRoofMushroom.func_74861_a
pub fn build(this: &Piece, parent: &Piece, _list: &mut Pieces, _random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFTowerRoofMushroom.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    for y in 0..=this.m_height.get()
    {
        let radius: i32 =
            ((sin((((y) as f32 + ((this.m_height.get()) as f32 / 1.2)) / ((this.m_height.get()) as f32 * 2.05)) * 3.14)
                * (this.m_size.get()) as f32)
                / 2.0) as i32;
        let mut hollow: i32 = floorF((radius) as f32 * 0.9);
        if (this.m_height.get() - y) < 3
        {
            hollow = -1;
        }
        tower_roof_mushroom::makeCircle(this, world, y, radius, hollow, sbb);
    }
    return true;
}

// ComponentTFTowerRoofMushroom.makeCircle
pub fn makeCircle(this: &Piece, world: &GenWorld, y: i32, radius: i32, hollow: i32, sbb: StructureBox)
{
    let cx: i32 = this.m_size.get() / 2;
    let cz: i32 = this.m_size.get() / 2;
    for dx in -radius..=radius
    {
        for dz in -radius..=radius
        {
            let dist: f32 = sqrtF(((dx * dx) + (dz * dz)) as f32);
            if dist <= ((radius) as f32 + 0.5)
            {
                if dist > (hollow) as f32
                {
                    this.placeBlock(
                        world,
                        this.m_deco.get().m_accentID,
                        this.m_deco.get().m_accentMeta,
                        dx + cx,
                        y,
                        dz + cz,
                        sbb,
                    );
                }
                else
                {
                    this.placeBlock(world, this.m_deco.get().m_accentID, 0, dx + cx, y, dz + cz, sbb);
                }
            }
        }
    }
}
