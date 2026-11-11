// draft from ComponentTFIceTowerEntrance.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::IceTowerEntrance);
    init(&this, i, x, y, z, pSize, pHeight, direction);
    Rc::new(this)
}

// ComponentTFIceTowerEntrance constructor
pub(crate) fn init(this: &Piece, i: i32, x: i32, y: i32, z: i32, pSize: i32, pHeight: i32, direction: i32)
{
    ice_tower_wing::init(this, i, x, y, z, pSize, pHeight, direction);
}

// ComponentTFIceTowerEntrance.shouldHaveBase
pub(crate) fn shouldHaveBase(_this: &Piece, _random: &Random) -> bool { true }

// ComponentTFIceTowerEntrance.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    this.addOpening(0, 1, this.m_size.get() / 2, 2);
    this.addStairs(list, random, this.depth() + 1, this.m_size.get() - 1, 1, this.m_size.get() / 2, 0);
    this.addStairs(list, random, this.depth() + 1, this.m_size.get() / 2, 1, 0, 3);
    this.addStairs(list, random, this.depth() + 1, this.m_size.get() / 2, 1, this.m_size.get() - 1, 1);
    this.m_hasBase.set(this.shouldHaveBase(random));
    this.makeARoof(parent, list, random);
}

// ComponentTFIceTowerEntrance.addStairs
pub(crate) fn addStairs(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    rotation: i32,
) -> bool
{
    this.addOpening(x, y, z, rotation);

    let direction: i32 = (this.mode() + rotation) % 4;
    let dx: [i32; 3] = this.offsetTowerAt(x, y, z, this.m_size.get(), direction);
    let entrance: Rc<Piece> =
        ice_tower_stairs::new(index, dx[0], dx[1], dx[2], this.m_size.get(), this.m_height.get(), direction);
    list.push(Rc::clone(&entrance));
    entrance.build(&Rc::clone(&list[0usize]), list, random);
    true
}

// ComponentTFIceTowerEntrance.makeTowerWing
pub(crate) fn makeTowerWing(
    _this: &Piece,
    _list: &mut Pieces,
    _random: &Random,
    _index: i32,
    _x: i32,
    _y: i32,
    _z: i32,
    _wingSize: i32,
    _wingHeight: i32,
    _rotation: i32,
) -> bool
{
    false
}

// ComponentTFIceTowerEntrance.makeFloorsForTower
pub(crate) fn makeFloorsForTower(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    ice_tower_entrance::decoratePillarsCornersHigh(this, world, random, 0, 11, 0, sbb);
}

// ComponentTFIceTowerEntrance.decoratePillarsCornersHigh
pub(crate) fn decoratePillarsCornersHigh(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    bottom: i32,
    top: i32,
    rotation: i32,
    sbb: StructureBox,
)
{
    ice_tower_wing::crossBeams(this, world, bottom, rotation, sbb);
    this.fillAirTurned(world, sbb, 3, bottom + 5, 3, 7, bottom + 5, 7, rotation);
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 5,
        3,
        3,
        top - 1,
        3,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        7,
        bottom + 5,
        3,
        7,
        top - 1,
        3,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        3,
        bottom + 5,
        7,
        3,
        top - 1,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        7,
        bottom + 5,
        7,
        7,
        top - 1,
        7,
        this.m_deco.get().m_pillarID,
        this.m_deco.get().m_pillarMeta,
        rotation,
    );
}
