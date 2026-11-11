// draft from ComponentTFMushroomTowerBridge.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MushroomTowerBridge);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFMushroomTowerBridge constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    mushroom_tower_wing::init(this, i, x, y, z, pSize, pHeight, direction);
    this.m_box.set(boxFor(x, y, z, 0, 0, 0, this.m_size.get() - 1, this.m_height.get() - 1, 3, direction));
    this.m_dSize.set(pSize);
    this.m_dHeight.set(pHeight);
}

// ComponentTFMushroomTowerBridge.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    let dest: Vec<i32> = vec![this.m_dSize.get() - 1, 1, 1];
    let madeWing: bool = this.makeTowerWing(
        list,
        random,
        this.depth(),
        dest[0],
        dest[1],
        dest[2],
        this.m_dSize.get(),
        this.m_dHeight.get(),
        0,
    );
    if !madeWing
    {
        let _dx: Vec<i32> = this.offsetTower(dest[0], dest[1], dest[2], this.m_dSize.get(), 0);
    }
}

// ComponentTFMushroomTowerBridge.getWingBB
pub fn getWingBB(this: &Piece) -> StructureBox
{
    let dest: Vec<i32> = this.offsetTower(this.m_dSize.get() - 1, 1, 1, this.m_dSize.get(), this.mode());
    return boxFor(
        dest[0],
        dest[1],
        dest[2],
        0,
        0,
        0,
        this.m_dSize.get() - 1,
        this.m_dHeight.get() - 1,
        this.m_dSize.get() - 1,
        this.mode(),
    );
}

// ComponentTFMushroomTowerBridge.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    for x in 0..this.m_dSize.get()
    {
        this.placeBlock(world, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceMeta, x, 1, 0, sbb);
        this.placeBlock(world, this.m_deco.get().m_fenceID, this.m_deco.get().m_fenceMeta, x, 1, 2, sbb);
        this.placeBlock(
            world,
            this.m_deco.get().m_floorID,
            if this.m_isAscender.get()
            {
                3
            }
            else
            {
                this.m_deco.get().m_floorMeta
            },
            x,
            0,
            1,
            sbb,
        );
    }
    this.fillAir(world, sbb, 0, 1, 1, 2, 2, 1);
    return true;
}
