// draft from ComponentTFHollowTreeMedBranch.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, sx: i32, sy: i32, sz: i32, length: f64, angle: f64, tilt: f64, leafy: bool) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::HollowTreeMedBranch);
    init(&this, i, sx, sy, sz, length, angle, tilt, leafy);
    Rc::new(this)
}

// ComponentTFHollowTreeMedBranch constructor
pub(crate) fn init(this: &Piece, i: i32, sx: i32, sy: i32, sz: i32, length: f64, angle: f64, tilt: f64, leafy: bool)
{
    this.m_depth.set(i);
    this.m_src.set([sx, sy, sz]);
    this.m_dest.set(translateCoords(
        this.m_src.get()[0],
        this.m_src.get()[1],
        this.m_src.get()[2],
        length,
        angle,
        tilt,
    ));
    this.m_length.set(length);
    this.m_angle.set(angle);
    this.m_tilt.set(tilt);
    this.m_leafy.set(leafy);
    this.setMode(0);
    this.m_box.set(StructureBox::new(
        [
            this.m_src.get()[0].min(this.m_dest.get()[0]),
            this.m_src.get()[1].min(this.m_dest.get()[1]),
            this.m_src.get()[2].min(this.m_dest.get()[2]),
        ],
        [
            this.m_src.get()[0].max(this.m_dest.get()[0]),
            this.m_src.get()[1].max(this.m_dest.get()[1]),
            this.m_src.get()[2].max(this.m_dest.get()[2]),
        ],
    ));
    this.boxed(|bounds| bounds.expand(&hollow_tree_med_branch::makeExpandedBB(this, 0.5, length, angle, tilt)));
    this.boxed(|bounds| bounds.expand(&hollow_tree_med_branch::makeExpandedBB(this, 0.1f64, length, 0.225, tilt)));
    this.boxed(|bounds| bounds.expand(&hollow_tree_med_branch::makeExpandedBB(this, 0.1f64, length, -0.225, tilt)));
}

// ComponentTFHollowTreeMedBranch.makeExpandedBB
pub(crate) fn makeExpandedBB(
    this: &Piece,
    outVar: f64,
    branchLength: f64,
    branchAngle: f64,
    branchTilt: f64,
) -> StructureBox
{
    let branchSrc: [i32; 3] = translateCoords(
        this.m_src.get()[0],
        this.m_src.get()[1],
        this.m_src.get()[2],
        this.m_length.get() * outVar,
        this.m_angle.get(),
        this.m_tilt.get(),
    );

    let branchDest: [i32; 3] =
        translateCoords(branchSrc[0], branchSrc[1], branchSrc[2], branchLength, branchAngle, branchTilt);
    StructureBox::new(
        [branchSrc[0].min(branchDest[0]), branchSrc[1].min(branchDest[1]), branchSrc[2].min(branchDest[2])],
        [branchSrc[0].max(branchDest[0]), branchSrc[1].max(branchDest[1]), branchSrc[2].max(branchDest[2])],
    )
}

// ComponentTFHollowTreeMedBranch.func_74861_a
pub(crate) fn build(this: &Piece, _structurecomponent: &Piece, _list: &mut Pieces, _random: &Random)
{
    let _index: i32 = this.depth();
}

// ComponentTFHollowTreeMedBranch.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    let rSrc: [i32; 3] = [
        this.m_src.get()[0] - this.m_box.get().m_min[0],
        this.m_src.get()[1] - this.m_box.get().m_min[1],
        this.m_src.get()[2] - this.m_box.get().m_min[2],
    ];

    let rDest: [i32; 3] = [
        this.m_dest.get()[0] - this.m_box.get().m_min[0],
        this.m_dest.get()[1] - this.m_box.get().m_min[1],
        this.m_dest.get()[2] - this.m_box.get().m_min[2],
    ];
    hollow_tree_med_branch::drawBresehnam(
        this,
        world,
        sbb,
        rSrc[0],
        rSrc[1],
        rSrc[2],
        rDest[0],
        rDest[1],
        rDest[2],
        blk("tf:log"),
        12,
    );
    hollow_tree_med_branch::drawBresehnam(
        this,
        world,
        sbb,
        rSrc[0],
        rSrc[1] + 1,
        rSrc[2],
        rDest[0],
        rDest[1],
        rDest[2],
        blk("tf:log"),
        12,
    );

    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );

    let numShoots: i32 = (decoRNG.nextIntBounded(3) + 1).min((this.m_length.get() / 5.0) as i32);
    let angleInc: f64 = 0.8 / (numShoots) as f64;

    for i in 0..numShoots
    {
        let angleVar: f64 = (angleInc * (i) as f64) - 0.4;
        let outVar: f64 = (decoRNG.nextDouble() * 0.8) + 0.2;
        let _tiltVar: f64 = (decoRNG.nextDouble() * 0.75) + 0.15;
        let bSrc: [i32; 3] = translateCoords(
            rSrc[0],
            rSrc[1],
            rSrc[2],
            this.m_length.get() * outVar,
            this.m_angle.get(),
            this.m_tilt.get(),
        );

        let _slength: f64 = this.m_length.get() * 0.4;

        hollow_tree_med_branch::drawSmallBranch(
            this,
            world,
            sbb,
            bSrc[0],
            bSrc[1],
            bSrc[2],
            (this.m_length.get() * 0.3f64).max(2.0),
            this.m_angle.get() + angleVar,
            this.m_tilt.get(),
            this.m_leafy.get(),
        );
    }

    if this.m_leafy.get()
    {
        let numLeafBalls: i32 = (decoRNG.nextIntBounded(3) + 1).min((this.m_length.get() / 5.0) as i32);
        for _i in 0..numLeafBalls
        {
            let slength: f64 = (((decoRNG.nextFloat() * 0.6) + 0.2) as f64) * this.m_length.get();
            let bdst: [i32; 3] =
                translateCoords(rSrc[0], rSrc[1], rSrc[2], slength, this.m_angle.get(), this.m_tilt.get());
            hollow_tree_med_branch::makeLeafBlob(
                this,
                world,
                sbb,
                bdst[0],
                bdst[1],
                bdst[2],
                if decoRNG.nextBoolean() { 2 } else { 3 },
            );
        }

        hollow_tree_med_branch::makeLeafBlob(this, world, sbb, rDest[0], rDest[1], rDest[2], 3);
    }

    true
}

// ComponentTFHollowTreeMedBranch.drawBresehnam
pub(crate) fn drawBresehnam(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    x1: i32,
    y1: i32,
    z1: i32,
    x2: i32,
    y2: i32,
    z2: i32,
    blockValue: Blk,
    metaValue: i32,
)
{
    let lineCoords: Vec<[i32; 3]> = bresenham(x1, y1, z1, x2, y2, z2);
    for coords in lineCoords
    {
        this.placeBlock(world, blockValue, metaValue, coords[0], coords[1], coords[2], sbb);
    }
}

// ComponentTFHollowTreeMedBranch.makeLeafBlob
pub(crate) fn makeLeafBlob(this: &Piece, world: &GenWorld, sbb: StructureBox, sx: i32, sy: i32, sz: i32, radius: i32)
{
    for dx in 0..=radius
    {
        for dy in 0..=radius
        {
            for dz in 0..=radius
            {
                let dist: i32 = if (dx >= dy) && (dx >= dz)
                {
                    ((dx) as f32 + (((dy.max(dz)) as f32 * 0.5) + ((dy.min(dz)) as f32 * 0.25))) as i32
                }
                else if (dy >= dx) && (dy >= dz)
                {
                    ((dy) as f32 + (((dx.max(dz)) as f32 * 0.5) + ((dx.min(dz)) as f32 * 0.25))) as i32
                }
                else
                {
                    ((dz) as f32 + (((dx.max(dy)) as f32 * 0.5) + ((dx.min(dy)) as f32 * 0.25))) as i32
                };

                if dist <= radius
                {
                    this.placeLeafBlock(world, blk("tf:leaves"), 0, sx + dx, sy + dy, sz + dz, sbb);
                    this.placeLeafBlock(world, blk("tf:leaves"), 0, sx + dx, sy + dy, sz - dz, sbb);
                    this.placeLeafBlock(world, blk("tf:leaves"), 0, sx - dx, sy + dy, sz + dz, sbb);
                    this.placeLeafBlock(world, blk("tf:leaves"), 0, sx - dx, sy + dy, sz - dz, sbb);
                    this.placeLeafBlock(world, blk("tf:leaves"), 0, sx + dx, sy - dy, sz + dz, sbb);
                    this.placeLeafBlock(world, blk("tf:leaves"), 0, sx + dx, sy - dy, sz - dz, sbb);
                    this.placeLeafBlock(world, blk("tf:leaves"), 0, sx - dx, sy - dy, sz + dz, sbb);
                    this.placeLeafBlock(world, blk("tf:leaves"), 0, sx - dx, sy - dy, sz - dz, sbb);
                }
            }
        }
    }
}

// ComponentTFHollowTreeMedBranch.placeLeafBlock
pub(crate) fn placeLeafBlock(
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
    let offX: i32 = this.xAt(x, z);
    let offY: i32 = this.yAt(y);
    let offZ: i32 = this.zAt(x, z);
    if sbb.contains(offX, offY, offZ)
    {
        let whatsThere: Blk = world.getBlock(offX, offY, offZ);
        if (false) || whatsThere.canBeReplacedByLeaves(world, offX, offY, offZ)
        {
            world.setBlock(offX, offY, offZ, blockID, meta, 2);
        }
    }
}

// ComponentTFHollowTreeMedBranch.drawSmallBranch
pub(crate) fn drawSmallBranch(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    sx: i32,
    sy: i32,
    sz: i32,
    branchLength: f64,
    branchAngle: f64,
    branchTilt: f64,
    _leafy: bool,
)
{
    let branchDest: [i32; 3] = translateCoords(sx, sy, sz, branchLength, branchAngle, branchTilt);
    hollow_tree_med_branch::drawBresehnam(
        this,
        world,
        sbb,
        sx,
        sy,
        sz,
        branchDest[0],
        branchDest[1],
        branchDest[2],
        blk("tf:log"),
        12,
    );
    hollow_tree_med_branch::makeLeafBlob(this, world, sbb, branchDest[0], branchDest[1], branchDest[2], 2);
}
