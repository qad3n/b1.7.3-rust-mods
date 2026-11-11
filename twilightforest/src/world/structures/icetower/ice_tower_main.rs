// draft from ComponentTFIceTowerMain.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(world: &GenWorld, random: &Random, index: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::IceTowerMain);
    init(&this, world, random, index, x, y, z);
    Rc::new(this)
}

// ComponentTFIceTowerMain constructor
pub(crate) fn init(this: &Piece, world: &GenWorld, random: &Random, index: i32, x: i32, y: i32, z: i32)
{
    init7(this, world, random, index, x + 11, y + 40, z + 11, 2);
    this.m_hasBossWing.set(false);
}

// ComponentTFIceTowerMain constructor
pub(crate) fn init7(this: &Piece, _world: &GenWorld, random: &Random, index: i32, x: i32, y: i32, z: i32, rotation: i32)
{
    ice_tower_wing::init(this, index, x, y, z, 11, 31 + (random.nextIntBounded(3) * 10), rotation);
    this.m_hasBossWing.set(false);
    this.m_deco.set(Deco::ice_tower());
}

// ComponentTFIceTowerMain.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    ice_tower_wing::build(this, parent, list, random);

    let mut towerBB: StructureBox = StructureBox::empty();

    for structurecomponent in list.clone()
    {
        towerBB.expand(&structurecomponent.bounds());
    }

    let myDoor: [i32; 3] = this.m_openings.borrow()[0usize];
    let mut entranceDoor: [i32; 3] = myDoor;

    if myDoor[0] == 0
    {
        let length: i32 = this.bounds().m_min[0] - towerBB.m_min[0];
        if length >= 0
        {
            entranceDoor[0] -= length;
            ice_tower_main::makeEntranceBridge(
                this,
                list,
                random,
                this.depth() + 1,
                myDoor[0],
                myDoor[1],
                myDoor[2],
                length,
                2,
            );
        }
    }

    if myDoor[0] == (this.m_size.get() - 1)
    {
        entranceDoor[0] = entranceDoor[0] + (towerBB.m_max[0] - this.bounds().m_max[0]);
    }

    if myDoor[2] == 0
    {
        entranceDoor[2] = entranceDoor[2] + (towerBB.m_min[2] - this.bounds().m_min[2]);
    }

    if myDoor[0] == (this.m_size.get() - 1)
    {
        entranceDoor[2] = entranceDoor[2] + (towerBB.m_max[2] - this.bounds().m_max[2]);
    }

    this.makeEntranceTower(
        list,
        random,
        this.depth() + 1,
        entranceDoor[0],
        entranceDoor[1],
        entranceDoor[2],
        11,
        11,
        this.mode(),
    );
}

// ComponentTFIceTowerMain.makeEntranceBridge
pub(crate) fn makeEntranceBridge(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    length: i32,
    rotation: i32,
)
{
    let direction: i32 = (this.mode() + rotation) % 4;
    let dest: [i32; 3] = this.offsetTowerAt(x, y, z, 5, direction);
    let bridge: Rc<Piece> = ice_tower_bridge::new(index, dest[0], dest[1], dest[2], length, direction);
    list.push(Rc::clone(&bridge));
    bridge.build(&Rc::clone(&list[0usize]), list, random);
}

// ComponentTFIceTowerMain.makeEntranceTower
pub(crate) fn makeEntranceTower(
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
    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: Vec<i32> = this.offsetTower(x, y, z, wingSize, direction);
    let entrance: Rc<Piece> = ice_tower_entrance::new(index, dx[0], dx[1], dx[2], wingSize, wingHeight, direction);
    list.push(Rc::clone(&entrance));
    entrance.build(&Rc::clone(&list[0usize]), list, random);
    this.addOpening(x, y, z, rotation);
    true
}
