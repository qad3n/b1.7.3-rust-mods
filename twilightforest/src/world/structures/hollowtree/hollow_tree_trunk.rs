// draft from ComponentTFHollowTreeTrunk.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(world: &GenWorld, random: &Random, index: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::HollowTreeTrunk);
    init(&this, world, random, index, x, y, z);
    Rc::new(this)
}

// ComponentTFHollowTreeTrunk constructor
pub(crate) fn init(this: &Piece, _world: &GenWorld, random: &Random, index: i32, x: i32, y: i32, z: i32)
{
    this.m_depth.set(index);
    this.m_groundLevel.set(-1);
    this.m_height.set(random.nextIntBounded(64) + 32);
    this.m_radius.set(random.nextIntBounded(4) + 1);
    this.setMode(0);
    this.m_box.set(StructureBox::new(
        [x, y, z],
        [(x + (this.m_radius.get() * 2)) + 2, y + this.m_height.get(), (z + (this.m_radius.get() * 2)) + 2],
    ));
}

// ComponentTFHollowTreeTrunk.func_74861_a
pub(crate) fn build(this: &Piece, _structurecomponent: &Piece, list: &mut Pieces, random: &Random)
{
    let index: i32 = this.depth();
    let numBranches: i32 = random.nextIntBounded(3) + 3;
    for i in 0..=numBranches
    {
        let branchHeight: i32 =
            ((((this.m_height.get()) as f64 * random.nextDouble()) * 0.9) as i32) + (this.m_height.get() / 10);
        let branchRotation: f64 = random.nextDouble();
        hollow_tree_trunk::makeSmallBranch(
            this,
            list,
            random,
            (index + i) + 1,
            branchHeight,
            4,
            branchRotation,
            0.35,
            true,
        );
    }

    hollow_tree_trunk::buildFullCrown(this, list, random, (index + numBranches) + 1);
    hollow_tree_trunk::buildBranchRing(this, list, random, index, 3, 2, 6, 0, 0.75, 0.0, 3, 5, 3, false);
    hollow_tree_trunk::buildBranchRing(this, list, random, index, 1, 2, 8, 0, 0.9, 0.0, 3, 5, 3, false);
}

// ComponentTFHollowTreeTrunk.buildFullCrown
pub(crate) fn buildFullCrown(this: &Piece, list: &mut Pieces, random: &Random, mut index: i32)
{
    let crownRadius: i32 = (this.m_radius.get() * 4) + 4;
    let bvar: i32 = this.m_radius.get() + 2;
    index += hollow_tree_trunk::buildBranchRing(
        this,
        list,
        random,
        index,
        this.m_height.get() - crownRadius,
        0,
        crownRadius,
        0,
        0.35,
        0.0,
        bvar,
        bvar + 2,
        2,
        true,
    );
    index += hollow_tree_trunk::buildBranchRing(
        this,
        list,
        random,
        index,
        this.m_height.get() - (crownRadius / 2),
        0,
        crownRadius,
        0,
        0.28,
        0.0,
        bvar,
        bvar + 2,
        1,
        true,
    );
    index += hollow_tree_trunk::buildBranchRing(
        this,
        list,
        random,
        index,
        this.m_height.get(),
        0,
        crownRadius,
        0,
        0.15,
        0.0,
        2,
        4,
        2,
        true,
    );
    hollow_tree_trunk::buildBranchRing(
        this,
        list,
        random,
        index,
        this.m_height.get(),
        0,
        crownRadius / 2,
        0,
        0.05,
        0.0,
        bvar,
        bvar + 2,
        1,
        true,
    );
}

// ComponentTFHollowTreeTrunk.buildBranchRing
pub(crate) fn buildBranchRing(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    branchHeight: i32,
    heightVar: i32,
    length: i32,
    _lengthVar: i32,
    tilt: f64,
    _tiltVar: f64,
    minBranches: i32,
    maxBranches: i32,
    size: i32,
    leafy: bool,
) -> i32
{
    let numBranches: i32 = random.nextIntBounded((maxBranches - minBranches) + 1) + minBranches;
    let branchRotation: f64 = 1.0 / (numBranches) as f64;
    let branchOffset: f64 = random.nextDouble();

    for i in 0..=numBranches
    {
        let dHeight: i32 = if heightVar > 0
        {
            (branchHeight - heightVar) + random.nextIntBounded(2 * heightVar)
        }
        else
        {
            branchHeight
        };

        if size == 2
        {
            hollow_tree_trunk::makeLargeBranch(
                this,
                list,
                random,
                index,
                dHeight,
                length,
                ((i) as f64 * branchRotation) + branchOffset,
                tilt,
                leafy,
            );
        }
        else if size == 1
        {
            hollow_tree_trunk::makeMedBranch(
                this,
                list,
                random,
                index,
                dHeight,
                length,
                ((i) as f64 * branchRotation) + branchOffset,
                tilt,
                leafy,
            );
        }
        else if size == 3
        {
            hollow_tree_trunk::makeRoot(
                this,
                list,
                random,
                index,
                dHeight,
                length,
                ((i) as f64 * branchRotation) + branchOffset,
                tilt,
            );
        }
        else
        {
            hollow_tree_trunk::makeSmallBranch(
                this,
                list,
                random,
                index,
                dHeight,
                length,
                ((i) as f64 * branchRotation) + branchOffset,
                tilt,
                leafy,
            );
        }
    }

    numBranches
}

// ComponentTFHollowTreeTrunk.makeSmallBranch
pub(crate) fn makeSmallBranch(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    branchHeight: i32,
    branchLength: i32,
    branchRotation: f64,
    branchAngle: f64,
    leafy: bool,
)
{
    let bSrc: [i32; 3] = hollow_tree_trunk::getBranchSrc(this, branchHeight, branchRotation);
    let branch: Rc<Piece> = hollow_tree_small_branch::new(
        index,
        bSrc[0],
        bSrc[1],
        bSrc[2],
        (branchLength) as f64,
        branchRotation,
        branchAngle,
        leafy,
    );
    list.push(Rc::clone(&branch));
    branch.build(this, list, random);
}

// ComponentTFHollowTreeTrunk.makeMedBranch
pub(crate) fn makeMedBranch(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    branchHeight: i32,
    branchLength: i32,
    branchRotation: f64,
    branchAngle: f64,
    leafy: bool,
)
{
    let bSrc: [i32; 3] = hollow_tree_trunk::getBranchSrc(this, branchHeight, branchRotation);
    let branch: Rc<Piece> = hollow_tree_med_branch::new(
        index,
        bSrc[0],
        bSrc[1],
        bSrc[2],
        (branchLength) as f64,
        branchRotation,
        branchAngle,
        leafy,
    );
    list.push(Rc::clone(&branch));
    branch.build(this, list, random);
}

// ComponentTFHollowTreeTrunk.makeLargeBranch
pub(crate) fn makeLargeBranch(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    branchHeight: i32,
    branchLength: i32,
    branchRotation: f64,
    branchAngle: f64,
    leafy: bool,
)
{
    let bSrc: [i32; 3] = hollow_tree_trunk::getBranchSrc(this, branchHeight, branchRotation);
    let branch: Rc<Piece> = hollow_tree_large_branch::new(
        index,
        bSrc[0],
        bSrc[1],
        bSrc[2],
        (branchLength) as f64,
        branchRotation,
        branchAngle,
        leafy,
    );
    list.push(Rc::clone(&branch));
    branch.build(this, list, random);
}

// ComponentTFHollowTreeTrunk.makeRoot
pub(crate) fn makeRoot(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    branchHeight: i32,
    branchLength: i32,
    branchRotation: f64,
    branchAngle: f64,
)
{
    let bSrc: [i32; 3] = hollow_tree_trunk::getBranchSrc(this, branchHeight, branchRotation);
    let branch: Rc<Piece> = hollow_tree_root::new(
        index,
        bSrc[0],
        bSrc[1],
        bSrc[2],
        (branchLength) as f64,
        branchRotation,
        branchAngle,
        false,
    );
    list.push(Rc::clone(&branch));
    branch.build(this, list, random);
}

// ComponentTFHollowTreeTrunk.getBranchSrc
pub(crate) fn getBranchSrc(this: &Piece, branchHeight: i32, branchRotation: f64) -> [i32; 3]
{
    translateCoords(
        (this.m_box.get().m_min[0] + this.m_radius.get()) + 1,
        this.m_box.get().m_min[1] + branchHeight,
        (this.m_box.get().m_min[2] + this.m_radius.get()) + 1,
        this.m_radius.get(),
        branchRotation,
        0.5,
    )
}

// ComponentTFHollowTreeTrunk.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if this.m_groundLevel.get() < 0
    {
        this.m_groundLevel.set(this.groundLevel(world, sbb));

        if this.m_groundLevel.get() < 0
        {
            return true;
        }

        this.m_height.set(this.m_box.get().m_max[1] - this.m_groundLevel.get());
        this.boxed(|bounds| bounds.m_min[1] = this.m_groundLevel.get());
    }

    let hollow: i32 = this.m_radius.get() / 2;

    for dx in 0..=(2 * this.m_radius.get())
    {
        for dz in 0..=(2 * this.m_radius.get())
        {
            let ax: i32 = (dx - this.m_radius.get()).abs();
            let az: i32 = (dz - this.m_radius.get()).abs();
            let dist: i32 = ((ax.max(az)) as f64 + ((ax.min(az)) as f64 * 0.5)) as i32;

            for dy in 0..=this.m_height.get()
            {
                if (dist <= this.m_radius.get()) && (dist > hollow)
                {
                    this.placeBlock(world, blk("tf:log"), 0, dx + 1, dy, dz + 1, sbb);
                }
            }

            if dist <= this.m_radius.get()
            {
                this.fillDown(world, blk("tf:log"), 0, dx + 1, -1, dz + 1, sbb);
            }

            if (dist == hollow) && (dx == (hollow + this.m_radius.get()))
            {
                this.fillDown(world, blk("vine"), 8, dx + 1, this.m_height.get(), dz + 1, sbb);
            }
        }
    }

    let numInsects: i32 =
        (random.nextIntBounded(3 * this.m_radius.get()) + random.nextIntBounded(3 * this.m_radius.get())) + 10;

    for _i in 0..=numInsects
    {
        let fHeight: i32 =
            ((((this.m_height.get()) as f64 * random.nextDouble()) * 0.9) as i32) + (this.m_height.get() / 10);
        let fAngle: f64 = random.nextDouble();
        hollow_tree_trunk::addInsect(this, world, fHeight, fAngle, random, sbb);
    }

    true
}

// ComponentTFHollowTreeTrunk.addInsect
pub(crate) fn addInsect(
    this: &Piece,
    world: &GenWorld,
    fHeight: i32,
    mut fAngle: f64,
    random: &Random,
    sbb: StructureBox,
)
{
    let bugSpot: [i32; 3] = translateCoords(
        this.m_radius.get() + 1,
        fHeight,
        this.m_radius.get() + 1,
        this.m_radius.get() + 1,
        fAngle,
        0.5,
    );
    fAngle %= 1.0;

    let mut insectMeta: i32 = 0;

    if (fAngle > 0.875) || (fAngle <= 0.125)
    {
        insectMeta = 3;
    }
    else if (fAngle > 0.125) && (fAngle <= 0.375)
    {
        insectMeta = 1;
    }
    else if (fAngle > 0.375) && (fAngle <= 0.625)
    {
        insectMeta = 4;
    }
    else if (fAngle > 0.625) && (fAngle <= 0.875)
    {
        insectMeta = 2;
    }

    let blockID: Blk = if random.nextBoolean()
    {
        blk("tf:firefly")
    }
    else
    {
        blk("tf:cicada")
    };
    hollow_tree_trunk::addInsect7(this, world, blockID, insectMeta, bugSpot[0], bugSpot[1], bugSpot[2], sbb);
}

// ComponentTFHollowTreeTrunk.addInsect
pub(crate) fn addInsect7(
    this: &Piece,
    world: &GenWorld,
    blockID: Blk,
    insectMeta: i32,
    posX: i32,
    posY: i32,
    posZ: i32,
    sbb: StructureBox,
)
{
    let ox: i32 = this.xAt(posX, posZ);
    let oy: i32 = this.yAt(posY);
    let oz: i32 = this.zAt(posX, posZ);
    if (sbb.contains(ox, oy, oz) && (true)) && blockID.canPlaceAt(world, ox, oy, oz)
    {
        world.setBlock(ox, oy, oz, blockID, insectMeta, 2);
    }
}
