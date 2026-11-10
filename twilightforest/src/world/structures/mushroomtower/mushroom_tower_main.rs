// draft from ComponentTFMushroomTowerMain.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(world: &GenWorld, random: &Random, index: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MushroomTowerMain);
    init(&this, world, random, index, x, y, z);
    Rc::new(this)
}

pub fn new7(world: &GenWorld, random: &Random, index: i32, x: i32, y: i32, z: i32, rotation: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MushroomTowerMain);
    init7(&this, world, random, index, x, y, z, rotation);
    Rc::new(this)
}

pub fn new7b(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MushroomTowerMain);
    init7b(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFMushroomTowerMain constructor
pub fn init(this: &Piece, world: &GenWorld, random: &Random, index: i32, x: i32, y: i32, z: i32)
{
    init7(this, world, random, index, x + 15, y + 4, z + 15, 2);
}

// ComponentTFMushroomTowerMain constructor
pub fn init7(this: &Piece, world: &GenWorld, random: &Random, index: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    mushroom_tower_wing::init(this, index, x, y, z, 15, 8 + (random.nextIntBounded(3) * 4), rotation);
    this.m_deco.set(Deco::mushroom_tower());
}

// ComponentTFMushroomTowerMain constructor
pub fn init7b(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    mushroom_tower_wing::init(this, i, x, y, z, pSize, pHeight, direction);
}

// ComponentTFMushroomTowerMain.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    this.addOpening(0, 1, this.m_size.get() / 2, 2);
    this.m_hasBase.set(true);
    let mut mainDir: i32 = -1;
    if this.depth() < 3
    {
        for i in 0..6
        {
            mainDir = mushroom_tower_main::makeAscenderTower(this, list, random);
            if mainDir != (-1)
            {
                break;
            }
        }
        for i in 0..4
        {
            if i != mainDir
            {
                let dest: Vec<i32> = this.getValidOpening(random, i);
                let childHeight: i32 = (((random.nextIntBounded(2) + random.nextIntBounded(2)) + 2) * 4) + 1;
                this.makeBridge(
                    list,
                    random,
                    this.depth() + 1,
                    dest[0],
                    dest[1],
                    dest[2],
                    this.m_size.get() - 4,
                    childHeight,
                    i,
                );
            }
        }
    }
    else
    {
        this.makeARoof(&parent, list, random);
    }
}

// ComponentTFMushroomTowerMain.makeAscenderTower
pub fn makeAscenderTower(this: &Piece, list: &mut Pieces, random: &Random) -> i32
{
    let mainDir: i32 = random.nextIntBounded(4);
    let dest: Vec<i32> = this.getValidOpening(random, mainDir);
    let childHeight: i32 =
        ((this.m_height.get() - dest[1]) + (((random.nextIntBounded(2) + random.nextIntBounded(2)) + 3) * 4)) + 1;
    let madeIt: bool = mushroom_tower_wing::makeBridge10(
        this,
        list,
        random,
        this.depth() + 1,
        dest[0],
        dest[1],
        dest[2],
        this.m_size.get() - 4,
        childHeight,
        mainDir,
        true,
    );
    if madeIt
    {
        return mainDir;
    }
    else
    {
        return -1;
    }
}

// ComponentTFMushroomTowerMain.makeARoof
pub fn makeARoof(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    let roof: Rc<Piece> = tower_roof_mushroom::new(this.depth() + 1, this, 1.6);
    list.push(Rc::clone(&roof));
    roof.build(this, list, random);
}

// ComponentTFMushroomTowerMain.makeDoorOpening
pub fn makeDoorOpening(this: &Piece, world: &GenWorld, dx: i32, dy: i32, dz: i32, sbb: StructureBox)
{
    mushroom_tower_wing::makeDoorOpening(this, world, dx, dy, dz, sbb);
    if dx == 0
    {
        this.placeBlock(world, blk("air"), 0, dx + 1, dy + 0, dz, sbb);
        this.placeBlock(world, blk("air"), 0, dx + 1, dy + 1, dz, sbb);
    }
    if dx == (this.m_size.get() - 1)
    {
        this.placeBlock(world, blk("air"), 0, dx - 1, dy + 0, dz, sbb);
        this.placeBlock(world, blk("air"), 0, dx - 1, dy + 1, dz, sbb);
    }
    if dz == 0
    {
        this.placeBlock(world, blk("air"), 0, dx, dy + 0, dz + 1, sbb);
        this.placeBlock(world, blk("air"), 0, dx, dy + 1, dz + 1, sbb);
    }
    if dz == (this.m_size.get() - 1)
    {
        this.placeBlock(world, blk("air"), 0, dx, dy + 0, dz - 1, sbb);
        this.placeBlock(world, blk("air"), 0, dx, dy + 1, dz - 1, sbb);
    }
}
