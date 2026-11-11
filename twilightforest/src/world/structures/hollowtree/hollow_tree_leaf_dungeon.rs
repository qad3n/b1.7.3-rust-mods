// draft from ComponentTFHollowTreeLeafDungeon.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(index: i32, x: i32, y: i32, z: i32, radius: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::HollowTreeLeafDungeon);
    init(&this, index, x, y, z, radius);
    Rc::new(this)
}

// ComponentTFHollowTreeLeafDungeon constructor
pub(crate) fn init(this: &Piece, index: i32, x: i32, y: i32, z: i32, radius: i32)
{
    this.m_depth.set(index);
    this.setMode(0);
    this.m_box.set(StructureBox::new([x - radius, y - radius, z - radius], [x + radius, y + radius, z + radius]));
    this.m_radius.set(radius);
}

// ComponentTFHollowTreeLeafDungeon.func_74861_a
pub(crate) fn build(this: &Piece, _structurecomponent: &Piece, _list: &mut Pieces, random: &Random)
{
    this.setMode(random.nextIntBounded(4));
}

// ComponentTFHollowTreeLeafDungeon.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    hollow_tree_leaf_dungeon::drawBlockBlob(
        this,
        world,
        sbb,
        this.m_radius.get(),
        this.m_radius.get(),
        this.m_radius.get(),
        4,
        blk("tf:leaves"),
        0,
        true,
    );
    hollow_tree_leaf_dungeon::drawBlockBlob(
        this,
        world,
        sbb,
        this.m_radius.get(),
        this.m_radius.get(),
        this.m_radius.get(),
        3,
        blk("tf:log"),
        12,
        false,
    );
    hollow_tree_leaf_dungeon::drawBlockBlob(
        this,
        world,
        sbb,
        this.m_radius.get(),
        this.m_radius.get(),
        this.m_radius.get(),
        2,
        blk("air"),
        0,
        false,
    );
    this.placeTreasure(
        world,
        random,
        this.m_radius.get() + 2,
        this.m_radius.get() - 1,
        this.m_radius.get(),
        Table::TreeCache,
        sbb,
    );
    this.placeSpawner(
        world,
        random,
        this.m_radius.get(),
        this.m_radius.get(),
        this.m_radius.get(),
        mob("Swarm Spider"),
        sbb,
    );
    true
}

// ComponentTFHollowTreeLeafDungeon.drawBlockBlob
pub(crate) fn drawBlockBlob(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    sx: i32,
    sy: i32,
    sz: i32,
    blobRadius: i32,
    blockID: Blk,
    metadata: i32,
    isLeaves: bool,
)
{
    for dx in 0..=blobRadius
    {
        for dy in 0..=blobRadius
        {
            for dz in 0..=blobRadius
            {
                let dist: i32 = if (dx >= dy) && (dx >= dz)
                {
                    dx + ((((dy.max(dz)) as f64 * 0.5) + ((dy.min(dz)) as f64 * 0.25)) as i32)
                }
                else if (dy >= dx) && (dy >= dz)
                {
                    dy + ((((dx.max(dz)) as f64 * 0.5) + ((dx.min(dz)) as f64 * 0.25)) as i32)
                }
                else
                {
                    dz + ((((dx.max(dy)) as f64 * 0.5) + ((dx.min(dy)) as f64 * 0.25)) as i32)
                };

                if dist <= blobRadius
                {
                    if isLeaves
                    {
                        this.placeLeafBlock(world, blockID, metadata, sx + dx, sy + dy, sz + dz, sbb);
                        this.placeLeafBlock(world, blockID, metadata, sx + dx, sy + dy, sz - dz, sbb);
                        this.placeLeafBlock(world, blockID, metadata, sx - dx, sy + dy, sz + dz, sbb);
                        this.placeLeafBlock(world, blockID, metadata, sx - dx, sy + dy, sz - dz, sbb);
                        this.placeLeafBlock(world, blockID, metadata, sx + dx, sy - dy, sz + dz, sbb);
                        this.placeLeafBlock(world, blockID, metadata, sx + dx, sy - dy, sz - dz, sbb);
                        this.placeLeafBlock(world, blockID, metadata, sx - dx, sy - dy, sz + dz, sbb);
                        this.placeLeafBlock(world, blockID, metadata, sx - dx, sy - dy, sz - dz, sbb);
                    }
                    else
                    {
                        this.placeBlock(world, blockID, metadata, sx + dx, sy + dy, sz + dz, sbb);
                        this.placeBlock(world, blockID, metadata, sx + dx, sy + dy, sz - dz, sbb);
                        this.placeBlock(world, blockID, metadata, sx - dx, sy + dy, sz + dz, sbb);
                        this.placeBlock(world, blockID, metadata, sx - dx, sy + dy, sz - dz, sbb);
                        this.placeBlock(world, blockID, metadata, sx + dx, sy - dy, sz + dz, sbb);
                        this.placeBlock(world, blockID, metadata, sx + dx, sy - dy, sz - dz, sbb);
                        this.placeBlock(world, blockID, metadata, sx - dx, sy - dy, sz + dz, sbb);
                        this.placeBlock(world, blockID, metadata, sx - dx, sy - dy, sz - dz, sbb);
                    }
                }
            }
        }
    }
}

// ComponentTFHollowTreeLeafDungeon.placeLeafBlock
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
