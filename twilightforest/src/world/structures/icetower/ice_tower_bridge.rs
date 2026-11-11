// draft from ComponentTFIceTowerBridge.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(index: i32, x: i32, y: i32, z: i32, length: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::IceTowerBridge);
    init(&this, index, x, y, z, length, direction);
    Rc::new(this)
}

// ComponentTFIceTowerBridge constructor
pub(crate) fn init(this: &Piece, index: i32, x: i32, y: i32, z: i32, length: i32, direction: i32)
{
    this.m_depth.set(index);
    this.m_lengthIceTowerBridge.set(length);
    this.setMode(direction);
    this.m_box.set(boxFor(x, y, z, 0, 0, 0, length, 6, 5, direction));
}

// ComponentTFIceTowerBridge.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, _list: &mut Pieces, _random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// ComponentTFIceTowerBridge.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    this.fillAir(world, sbb, 0, 1, 0, this.m_lengthIceTowerBridge.get(), 5, 4);
    this.fill(
        world,
        sbb,
        0,
        0,
        0,
        this.m_lengthIceTowerBridge.get(),
        0,
        4,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockID,
        false,
    );
    this.fill(
        world,
        sbb,
        0,
        6,
        0,
        this.m_lengthIceTowerBridge.get(),
        6,
        4,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockID,
        false,
    );

    for x in (2..this.m_lengthIceTowerBridge.get()).step_by(3)
    {
        this.fillMeta(
            world,
            sbb,
            x,
            1,
            0,
            x,
            5,
            0,
            this.m_deco.get().m_pillarID,
            this.m_deco.get().m_pillarMeta,
            this.m_deco.get().m_pillarID,
            this.m_deco.get().m_pillarMeta,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            x,
            1,
            4,
            x,
            5,
            4,
            this.m_deco.get().m_pillarID,
            this.m_deco.get().m_pillarMeta,
            this.m_deco.get().m_pillarID,
            this.m_deco.get().m_pillarMeta,
            false,
        );
    }

    true
}
