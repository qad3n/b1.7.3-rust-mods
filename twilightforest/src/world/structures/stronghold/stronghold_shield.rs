// draft from StructureTFStrongholdShield.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::StrongholdShield);
    init(&this, minX, minY, minZ, maxX, maxY, maxZ);
    Rc::new(this)
}

// StructureTFStrongholdShield constructor
pub fn init(this: &Piece, minX: i32, minY: i32, minZ: i32, maxX: i32, maxY: i32, maxZ: i32)
{
    stronghold_component::init(this, 0, 0, minX, minY, minZ);
    this.m_box.set(StructureBox::new([minX, minY, minZ], [maxX, maxY, maxZ]));
    this.m_spawnListIndex.set(-1);
}

// StructureTFStrongholdShield.generateBoundingBox
pub fn generateBoundingBox(this: &Piece, facing: i32, x: i32, y: i32, z: i32) -> Option<StructureBox> { return None; }

// StructureTFStrongholdShield.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let shieldBlock: Blk = blk("tf:shield");
    this.fillMeta(
        world,
        sbb,
        this.m_box.get().xSize(),
        0,
        0,
        this.m_box.get().xSize(),
        this.m_box.get().ySize(),
        this.m_box.get().zSize(),
        shieldBlock,
        4,
        shieldBlock,
        4,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        0,
        0,
        0,
        0,
        this.m_box.get().ySize(),
        this.m_box.get().zSize(),
        shieldBlock,
        5,
        shieldBlock,
        5,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        0,
        0,
        this.m_box.get().zSize(),
        this.m_box.get().xSize(),
        this.m_box.get().ySize(),
        this.m_box.get().zSize(),
        shieldBlock,
        2,
        shieldBlock,
        2,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        0,
        0,
        0,
        this.m_box.get().xSize(),
        this.m_box.get().ySize(),
        0,
        shieldBlock,
        3,
        shieldBlock,
        3,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        0,
        0,
        0,
        this.m_box.get().xSize(),
        0,
        this.m_box.get().zSize(),
        shieldBlock,
        1,
        shieldBlock,
        1,
        false,
    );
    this.fillMeta(
        world,
        sbb,
        0,
        this.m_box.get().ySize(),
        0,
        this.m_box.get().xSize(),
        this.m_box.get().ySize(),
        this.m_box.get().zSize(),
        shieldBlock,
        0,
        shieldBlock,
        0,
        false,
    );
    return true;
}
