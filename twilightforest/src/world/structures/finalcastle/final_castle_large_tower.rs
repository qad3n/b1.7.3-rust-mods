// draft from ComponentTFFinalCastleLargeTower.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleLargeTower);
    init(&this, random, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFFinalCastleLargeTower constructor
pub fn init(this: &Piece, _random: &Random, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    tower_wing::init(this, i);
    this.setMode(rotation);
    this.m_size.set(13);
    this.m_height.set(61);
    this.m_box.set(boxFor(x, y, z, -6, 0, -6, 12, 60, 12, 0));
}

// ComponentTFFinalCastleLargeTower.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    let roof: Rc<Piece> = final_castle_roof9_crenellated::new(random, 4, this);
    list.push(Rc::clone(&roof));
    roof.build(this, list, random);
}

// ComponentTFFinalCastleLargeTower.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    this.fillSelected(world, sbb, 0, 0, 0, 12, 59, 12, false, random, this.m_deco.get().m_randomBlocks);
    let numBranches: i32 = 6 + decoRNG.nextIntBounded(4);
    for _i in 0..numBranches
    {
        tower_wing::makeGlyphBranches(this, world, &decoRNG, this.getGlyphMeta(), sbb);
    }
    for i in 1..4
    {
        this.fillSelected(
            world,
            sbb,
            i,
            0 - (i * 2),
            i,
            8 - i,
            1 - (i * 2),
            8 - i,
            false,
            random,
            this.m_deco.get().m_randomBlocks,
        );
    }
    this.placeBlock(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, 4, -7, 4, sbb);
    this.fillMeta(world, sbb, 0, 1, 1, 0, 4, 3, blk("tf:castleDoor"), 0, blk("air"), this.getGlyphMeta(), false);
    this.placeSign(world, 6, 1, 6, "Parkour area 1", "Unique monster?", sbb);
    return true;
}

// ComponentTFFinalCastleLargeTower.getGlyphMeta
pub fn getGlyphMeta(_this: &Piece) -> i32 { return 0; }
