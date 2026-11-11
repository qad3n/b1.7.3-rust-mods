// draft from ComponentTFFinalCastleBossGazebo.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(random: &Random, i: i32, keep: &Piece) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleBossGazebo);
    init(&this, random, i, keep);
    Rc::new(this)
}

// ComponentTFFinalCastleBossGazebo constructor
pub(crate) fn init(this: &Piece, _random: &Random, i: i32, keep: &Piece)
{
    this.m_depth.set(i);
    this.m_spawnListIndex.set(-1);
    this.setMode(keep.mode());
    this.m_box.set(StructureBox::new(
        [keep.bounds().m_min[0] + 14, keep.bounds().m_max[1] + 2, keep.bounds().m_min[2] + 14],
        [keep.bounds().m_max[0] - 14, keep.bounds().m_max[1] + 13, keep.bounds().m_max[2] - 14],
    ));
}

// ComponentTFFinalCastleBossGazebo.func_74861_a
pub(crate) fn build(this: &Piece, _parent: &Piece, _list: &mut Pieces, _random: &Random)
{
    this.m_deco.set(Deco::decorator_castle());
    this.m_deco.get().m_blockID = blk("tf:castleMagic");
    this.m_deco.get().m_blockMeta = 1;
    this.m_deco.get().m_fenceID = blk("tf:forceField");
    this.m_deco.get().m_fenceMeta = 10;
}

// ComponentTFFinalCastleBossGazebo.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    for rotation in 0..4
    {
        this.fillTurned(
            world,
            sbb,
            0,
            0,
            0,
            0,
            10,
            20,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            rotation,
        );
    }

    this.fillMeta(
        world,
        sbb,
        0,
        11,
        0,
        20,
        11,
        20,
        this.m_deco.get().m_fenceID,
        this.m_deco.get().m_fenceMeta,
        this.m_deco.get().m_fenceID,
        this.m_deco.get().m_fenceMeta,
        false,
    );
    this.placeSign(world, 10, 0, 10, "Final Boss Here", "You win!", sbb);
    true
}
