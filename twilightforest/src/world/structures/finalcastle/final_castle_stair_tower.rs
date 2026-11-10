// draft from ComponentTFFinalCastleStairTower.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleStairTower);
    init(&this, random, i, x, y, z, rotation);
    Rc::new(this)
}

// ComponentTFFinalCastleStairTower constructor
pub fn init(this: &Piece, random: &Random, i: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    tower_wing::init(this, i);
    this.setMode(rotation);
    this.m_size.set(9);
    this.m_height.set(51);
    this.m_box.set(boxFor(x, y, z, -4, 0, -4, 8, 50, 8, 0));
}

// ComponentTFFinalCastleStairTower.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    let roof: Rc<Piece> = final_castle_roof9_crenellated::new(random, 4, this);
    list.push(Rc::clone(&roof));
    roof.build(this, list, random);
}

// ComponentTFFinalCastleStairTower.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    this.fillSelected(world, sbb, 0, 0, 0, 8, 49, 8, false, random, this.m_deco.get().m_randomBlocks);
    let numBranches: i32 = 6 + decoRNG.nextIntBounded(4);
    for i in 0..numBranches
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
    this.fillMeta(world, sbb, 0, 1, 1, 0, 3, 2, blk("tf:castleDoor"), this.getGlyphMeta(), blk("air"), 0, false);
    for f in 0..5
    {
        let rotation: i32 = (f + 2) % 4;
        let y: i32 = (f * 3) + 1;
        for i in 0..3
        {
            let sx: i32 = 3 + i;
            let sy: i32 = y + i;
            let sz: i32 = 1;
            this.placeTurned(
                world,
                this.m_deco.get().m_stairID,
                this.stairMeta(0 + rotation),
                sx,
                sy,
                sz,
                rotation,
                sbb,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                sx,
                sy - 1,
                sz,
                rotation,
                sbb,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_stairID,
                this.stairMeta(0 + rotation),
                sx,
                sy,
                sz + 1,
                rotation,
                sbb,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                sx,
                sy - 1,
                sz + 1,
                rotation,
                sbb,
            );
        }
        this.fillTurned(
            world,
            sbb,
            6,
            y + 2,
            1,
            7,
            y + 2,
            2,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
    }
    this.fillMeta(world, sbb, 1, 18, 0, 2, 20, 0, blk("tf:castleDoor"), this.getGlyphMeta(), blk("air"), 0, false);
    this.fillMeta(
        world,
        sbb,
        1,
        17,
        1,
        3,
        17,
        3,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        1,
        17,
        4,
        2,
        17,
        4,
        this.m_deco.get().m_stairID,
        this.stairMeta(3),
        this.m_deco.get().m_stairID,
        this.stairMeta(3),
        false,
    );
    this.fillMeta(
        world,
        sbb,
        1,
        16,
        4,
        2,
        16,
        4,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        1,
        16,
        5,
        2,
        16,
        5,
        this.m_deco.get().m_stairID,
        this.stairMeta(3),
        this.m_deco.get().m_stairID,
        this.stairMeta(3),
        false,
    );
    this.fillMeta(
        world,
        sbb,
        1,
        15,
        5,
        2,
        15,
        5,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        false,
    );
    this.fillMeta(world, sbb, 1, 39, 0, 2, 41, 0, blk("tf:castleDoor"), this.getGlyphMeta(), blk("air"), 0, false);
    for f in 0..7
    {
        let rotation: i32 = (f + 0) % 4;
        let y: i32 = (f * 3) + 18;
        for i in 0..3
        {
            let sx: i32 = 3 + i;
            let sy: i32 = y + i;
            let sz: i32 = 1;
            this.placeTurned(
                world,
                this.m_deco.get().m_stairID,
                this.stairMeta(0 + rotation),
                sx,
                sy,
                sz,
                rotation,
                sbb,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                sx,
                sy - 1,
                sz,
                rotation,
                sbb,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_stairID,
                this.stairMeta(0 + rotation),
                sx,
                sy,
                sz + 1,
                rotation,
                sbb,
            );
            this.placeTurned(
                world,
                this.m_deco.get().m_blockID,
                this.m_deco.get().m_blockMeta,
                sx,
                sy - 1,
                sz + 1,
                rotation,
                sbb,
            );
        }
        this.fillTurned(
            world,
            sbb,
            6,
            y + 2,
            1,
            7,
            y + 2,
            2,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            rotation,
        );
    }
    this.fillMeta(
        world,
        sbb,
        1,
        38,
        1,
        3,
        38,
        5,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        3,
        39,
        1,
        3,
        39,
        5,
        this.m_deco.get().m_fenceID,
        this.m_deco.get().m_fenceMeta,
        this.m_deco.get().m_fenceID,
        this.m_deco.get().m_fenceMeta,
        false,
    );
    return true;
}

// ComponentTFFinalCastleStairTower.getGlyphMeta
pub fn getGlyphMeta(this: &Piece) -> i32 { return 1; }
