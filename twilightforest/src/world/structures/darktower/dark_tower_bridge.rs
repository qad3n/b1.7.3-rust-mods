// draft from ComponentTFDarkTowerBridge.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::DarkTowerBridge);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFDarkTowerBridge constructor
pub fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    tower_wing::init7(this, i, x, y, z, 5, 5, direction);
    this.m_dSize.set(pSize);
    this.m_dHeight.set(pHeight);
}

// ComponentTFDarkTowerBridge.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    this.makeTowerWing(list, random, this.depth(), 4, 1, 2, this.m_dSize.get(), this.m_dHeight.get(), 0);
}

// ComponentTFDarkTowerBridge.makeTowerWing
pub fn makeTowerWing(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    wingSize: i32,
    wingHeight: i32,
    rotation: i32,
) -> bool
{
    if wingHeight < 6
    {
        return false;
    }
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, wingSize, direction);
    if (dx[1] + wingHeight) > 255
    {
        return false;
    }
    let wing: Rc<Piece> = dark_tower_wing::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    let intersect: Option<Rc<Piece>> = intersecting(list, wing.bounds());
    if (intersect.is_some()) && (!same(&intersect, &this))
    {
        return false;
    }
    list.push(Rc::clone(&wing));
    wing.build(this, list, random);
    this.addOpening(x, y, z, rotation);
    return true;
}

// ComponentTFDarkTowerBridge.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    this.fill(
        world,
        sbb,
        0,
        0,
        0,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockID,
        false,
    );
    for x in 0..this.m_size.get()
    {
        this.placeBlock(world, this.m_deco.get().m_accentID, this.m_deco.get().m_accentMeta, x, 0, 0, sbb);
        this.placeBlock(
            world,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            x,
            this.m_height.get() - 1,
            0,
            sbb,
        );
        this.placeBlock(
            world,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            x,
            0,
            this.m_size.get() - 1,
            sbb,
        );
        this.placeBlock(
            world,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            x,
            this.m_height.get() - 1,
            this.m_size.get() - 1,
            sbb,
        );
    }
    this.darken(world);
    this.fillAir(world, sbb, 0, 1, 1, this.m_size.get() - 1, this.m_height.get() - 2, this.m_size.get() - 2);
    return true;
}

// ComponentTFDarkTowerBridge.getWingBB
pub fn getWingBB(this: &Piece) -> StructureBox
{
    let dest: Vec<i32> = this.offsetTower(4, 1, 2, this.m_dSize.get(), this.mode());
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
