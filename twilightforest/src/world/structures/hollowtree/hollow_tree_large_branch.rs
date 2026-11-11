// draft from ComponentTFHollowTreeLargeBranch.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, sx: i32, sy: i32, sz: i32, length: f64, angle: f64, tilt: f64, leafy: bool) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::HollowTreeLargeBranch);
    init(&this, i, sx, sy, sz, length, angle, tilt, leafy);
    Rc::new(this)
}

// ComponentTFHollowTreeLargeBranch constructor
pub(crate) fn init(this: &Piece, i: i32, sx: i32, sy: i32, sz: i32, length: f64, angle: f64, tilt: f64, leafy: bool)
{
    hollow_tree_med_branch::init(this, i, sx, sy, sz, length, angle, tilt, leafy);
    this.m_hasLeafDungeon.set(false);
}

// ComponentTFHollowTreeLargeBranch.func_74861_a
pub(crate) fn build(this: &Piece, _structurecomponent: &Piece, list: &mut Pieces, random: &Random)
{
    let index: i32 = this.depth();
    let numMedBranches: i32 =
        random.nextIntBounded((this.m_length.get() / 6.0) as i32) + ((this.m_length.get() / 8.0) as i32);

    for i in 0..=numMedBranches
    {
        let outVar: f64 = (random.nextDouble() * 0.3) + 0.3;
        let angleVar: f64 = (random.nextDouble() * 0.225) * (if (i & 1) == 0 { 1.0 } else { -1.0 });
        let bsrc: [i32; 3] = translateCoords(
            this.m_src.get()[0],
            this.m_src.get()[1],
            this.m_src.get()[2],
            this.m_length.get() * outVar,
            this.m_angle.get(),
            this.m_tilt.get(),
        );
        hollow_tree_large_branch::makeMedBranch10(
            this,
            list,
            random,
            (index + 2) + i,
            bsrc[0],
            bsrc[1],
            bsrc[2],
            this.m_length.get() * 0.6,
            this.m_angle.get() + angleVar,
            this.m_tilt.get(),
            this.m_leafy.get(),
        );
    }

    this.m_hasLeafDungeon.set(random.nextIntBounded(8) == 0);

    if this.m_hasLeafDungeon.get()
    {
        hollow_tree_large_branch::makeLeafDungeon(
            this,
            list,
            random,
            index + 1,
            this.m_dest.get()[0],
            this.m_dest.get()[1],
            this.m_dest.get()[2],
        );
    }
}

// ComponentTFHollowTreeLargeBranch.makeLeafDungeon
pub(crate) fn makeLeafDungeon(this: &Piece, list: &mut Pieces, random: &Random, index: i32, x: i32, y: i32, z: i32)
{
    let dungeon: Rc<Piece> = hollow_tree_leaf_dungeon::new(index, x, y, z, 4);
    list.push(Rc::clone(&dungeon));
    dungeon.build(this, list, random);
}

// ComponentTFHollowTreeLargeBranch.makeMedBranch
pub(crate) fn makeMedBranch10(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    branchLength: f64,
    branchRotation: f64,
    branchAngle: f64,
    leafy: bool,
)
{
    let branch: Rc<Piece> =
        hollow_tree_med_branch::new(index, x, y, z, branchLength, branchRotation, branchAngle, leafy);
    list.push(Rc::clone(&branch));
    branch.build(this, list, random);
}

// ComponentTFHollowTreeLargeBranch.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    let rsrc: [i32; 3] = [
        this.m_src.get()[0] - this.m_box.get().m_min[0],
        this.m_src.get()[1] - this.m_box.get().m_min[1],
        this.m_src.get()[2] - this.m_box.get().m_min[2],
    ];

    let rdest: [i32; 3] = [
        this.m_dest.get()[0] - this.m_box.get().m_min[0],
        this.m_dest.get()[1] - this.m_box.get().m_min[1],
        this.m_dest.get()[2] - this.m_box.get().m_min[2],
    ];
    hollow_tree_med_branch::drawBresehnam(
        this,
        world,
        sbb,
        rsrc[0],
        rsrc[1],
        rsrc[2],
        rdest[0],
        rdest[1],
        rdest[2],
        blk("tf:log"),
        12,
    );

    let reinforcements: i32 = 4;
    for i in 0..=reinforcements
    {
        let vx: i32 = if (i & 2) == 0 { 1 } else { 0 };
        let vy: i32 = if (i & 1) == 0 { 1 } else { -1 };
        let vz: i32 = if (i & 2) == 0 { 0 } else { 1 };
        hollow_tree_med_branch::drawBresehnam(
            this,
            world,
            sbb,
            rsrc[0] + vx,
            rsrc[1] + vy,
            rsrc[2] + vz,
            rdest[0],
            rdest[1],
            rdest[2],
            blk("tf:log"),
            12,
        );
    }

    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );

    let numSmallBranches: i32 = decoRNG.nextIntBounded(2) + 1;
    for i in 0..=numSmallBranches
    {
        let outVar: f64 = ((decoRNG.nextFloat() * 0.25) + 0.25) as f64;
        let angleVar: f64 = ((decoRNG.nextFloat() * 0.25) * (if (i & 1) == 0 { 1.0 } else { -1.0 })) as f64;
        let bsrc: [i32; 3] = translateCoords(
            rsrc[0],
            rsrc[1],
            rsrc[2],
            this.m_length.get() * outVar,
            this.m_angle.get(),
            this.m_tilt.get(),
        );
        hollow_tree_med_branch::drawSmallBranch(
            this,
            world,
            sbb,
            bsrc[0],
            bsrc[1],
            bsrc[2],
            (this.m_length.get() * 0.3f64).max(2.0),
            this.m_angle.get() + angleVar,
            this.m_tilt.get(),
            this.m_leafy.get(),
        );
    }

    if this.m_leafy.get() && (!this.m_hasLeafDungeon.get())
    {
        hollow_tree_med_branch::makeLeafBlob(this, world, sbb, rdest[0], rdest[1], rdest[2], 3);
    }

    true
}
