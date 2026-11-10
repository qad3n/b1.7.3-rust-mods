// draft from ComponentTFLeafSphere.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(index: i32, x: i32, y: i32, z: i32, radius: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::LeafSphere);
    init(&this, index, x, y, z, radius);
    Rc::new(this)
}

// ComponentTFLeafSphere constructor
pub fn init(this: &Piece, index: i32, x: i32, y: i32, z: i32, radius: i32)
{
    this.m_depth.set(index);
    this.setMode(0);
    this.m_box.set(StructureBox::new([x - radius, y - radius, z - radius], [x + radius, y + radius, z + radius]));
    this.m_radius.set(radius);
}

// ComponentTFLeafSphere.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let sx: i32 = this.m_radius.get();
    let sy: i32 = this.m_radius.get();
    let sz: i32 = this.m_radius.get();
    for dx in 0..=this.m_radius.get()
    {
        for dy in 0..=this.m_radius.get()
        {
            for dz in 0..=this.m_radius.get()
            {
                let mut dist: i32 = 0;
                if (dx >= dy) && (dx >= dz)
                {
                    dist = dx + ((((dy.max(dz)) as f64 * 0.5) + ((dy.min(dz)) as f64 * 0.25)) as i32);
                }
                else if (dy >= dx) && (dy >= dz)
                {
                    dist = dy + ((((dx.max(dz)) as f64 * 0.5) + ((dx.min(dz)) as f64 * 0.25)) as i32);
                }
                else
                {
                    dist = dz + ((((dx.max(dy)) as f64 * 0.5) + ((dx.min(dy)) as f64 * 0.25)) as i32);
                }
                if dist <= this.m_radius.get()
                {
                    leaf_sphere::placeBlockIfEmpty(this, world, blk("tf:leaves"), 0, sx + dx, sy + dy, sz + dz, sbb);
                    leaf_sphere::placeBlockIfEmpty(this, world, blk("tf:leaves"), 0, sx + dx, sy + dy, sz - dz, sbb);
                    leaf_sphere::placeBlockIfEmpty(this, world, blk("tf:leaves"), 0, sx - dx, sy + dy, sz + dz, sbb);
                    leaf_sphere::placeBlockIfEmpty(this, world, blk("tf:leaves"), 0, sx - dx, sy + dy, sz - dz, sbb);
                    leaf_sphere::placeBlockIfEmpty(this, world, blk("tf:leaves"), 0, sx + dx, sy - dy, sz + dz, sbb);
                    leaf_sphere::placeBlockIfEmpty(this, world, blk("tf:leaves"), 0, sx + dx, sy - dy, sz - dz, sbb);
                    leaf_sphere::placeBlockIfEmpty(this, world, blk("tf:leaves"), 0, sx - dx, sy - dy, sz + dz, sbb);
                    leaf_sphere::placeBlockIfEmpty(this, world, blk("tf:leaves"), 0, sx - dx, sy - dy, sz - dz, sbb);
                }
            }
        }
    }
    return true;
}

// ComponentTFLeafSphere.placeBlockIfEmpty
pub fn placeBlockIfEmpty(
    this: &Piece,
    world: &GenWorld,
    blockID: Blk,
    meta: i32,
    x: i32,
    y: i32,
    z: i32,
    sbb: StructureBox,
)
{
    if this.getBlock(world, x, y, z, sbb) == blk("air")
    {
        this.placeBlock(world, blockID, meta, x, y, z, sbb);
    }
}
