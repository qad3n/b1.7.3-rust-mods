// draft from ComponentTFCloudCastle.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(index: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::CloudCastle);
    init(&this, index, x, y, z);
    Rc::new(this)
}

// ComponentTFCloudCastle constructor
pub fn init(this: &Piece, index: i32, mut x: i32, mut y: i32, mut z: i32)
{
    this.m_depth.set(index);
    this.m_minerPlaced.set(false);
    this.m_warriorPlaced.set(false);
    this.setMode(0);
    x = (x >> 2) << 2;
    y = (y >> 2) << 2;
    z = (z >> 2) << 2;
    this.m_spawnListIndex.set(1);
    this.m_box.set(boxFor(x, y, z, -8, 0, -8, 16, 16, 16, 0));
}

// ComponentTFCloudCastle.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    let mut plus: bool = random.nextBoolean();
    let mut offset: i32 = random.nextIntBounded(5) - random.nextIntBounded(5);
    let treeX: Rc<Piece> = cloud_tree::new(
        this.depth() + 1,
        this.m_box.get().m_min[0] + (if plus { 16 } else { -16 }),
        168,
        (this.m_box.get().m_min[2] - 8) + (offset * 4),
    );
    list.push(Rc::clone(&treeX));
    treeX.build(this, list, random);
    plus = random.nextBoolean();
    offset = random.nextIntBounded(5) - random.nextIntBounded(5);
    let treeZ: Rc<Piece> = cloud_tree::new(
        this.depth() + 1,
        (this.m_box.get().m_min[0] - 8) + (offset * 4),
        168,
        this.m_box.get().m_min[2] + (if plus { 16 } else { -16 }),
    );
    list.push(Rc::clone(&treeZ));
    treeZ.build(this, list, random);
}

// ComponentTFCloudCastle.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    this.fillMeta(world, sbb, 0, -4, 0, 15, -1, 15, blk("tf:fluffyCloud"), 0, blk("tf:fluffyCloud"), 0, false);
    this.fillMeta(world, sbb, 0, 0, 0, 15, 11, 15, blk("tf:giantCobble"), 0, blk("tf:giantCobble"), 0, false);
    this.fillMeta(world, sbb, 0, 12, 0, 15, 15, 15, blk("tf:giantLog"), 0, blk("tf:giantLog"), 0, false);
    this.fillAir(world, sbb, 4, 0, 4, 11, 11, 11);
    this.fillAir(world, sbb, 0, 0, 4, 4, 7, 7);
    if !this.m_minerPlaced.get()
    {
        let bx: i32 = this.xAt(6, 6);
        let by: i32 = this.yAt(0);
        let bz: i32 = this.zAt(6, 6);
        if sbb.contains(bx, by, bz)
        {
            this.m_minerPlaced.set(true);
            spawnHome(world, "twilightforest:giant_miner", bx, by, bz, false);
        }
    }
    if !this.m_warriorPlaced.get()
    {
        let bx: i32 = this.xAt(9, 9);
        let by: i32 = this.yAt(0);
        let bz: i32 = this.zAt(9, 9);
        if sbb.contains(bx, by, bz)
        {
            this.m_warriorPlaced.set(true);
            spawnHome(world, "twilightforest:armored_giant", bx, by, bz, false);
        }
    }
    return true;
}
