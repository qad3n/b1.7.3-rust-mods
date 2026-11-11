// draft from StructureTFStrongholdComponent.java by tools/structgen.py
use crate::world::structures::prelude::*;

// StructureTFStrongholdComponent constructor
pub(crate) fn init(this: &Piece, i: i32, facing: i32, x: i32, y: i32, z: i32)
{
    this.m_depth.set(i);
    *this.m_doors.borrow_mut() = Vec::new();

    if let Some(bounds) = this.generateBoundingBox(facing, x, y, z)
    {
        this.m_box.set(bounds);
    }

    this.setMode(facing);
}

// StructureTFStrongholdComponent.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, _list: &mut Pieces, _random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
}

// StructureTFStrongholdComponent.addNewComponent
pub(crate) fn addNewComponent(
    this: &Piece,
    entrance: &Piece,
    list: &mut Pieces,
    random: &Random,
    facing: i32,
    x: i32,
    y: i32,
    z: i32,
)
{
    let index: i32 = this.m_depth.get() + 1;
    let nFacing: i32 = (this.m_mode.get() + facing) % 4;
    let nx: i32 = this.xAt(x, z);
    let ny: i32 = this.yAt(y);
    let nz: i32 = this.zAt(x, z);
    if (index <= 50) && (!this.isOutOfRange(entrance, nx, ny, nz, 112))
    {
        let breakIn: Option<Rc<Piece>> = stronghold_component::findBreakInComponent(this, list, nx, ny, nz);
        if (breakIn.is_some()) && stronghold_component::attemptToBreakIn(breakIn.piece(), nx, ny, nz)
        {
            stronghold_component::addDoorwayTo(this, x, y, z, facing);
        }
        else
        {
            let nextComponent: Option<Rc<Piece>> =
                entrance.m_lowerPieces.getNextComponent(list, random, index, nFacing, nx, ny, nz);

            if nextComponent.is_some()
            {
                list.push(Rc::clone(nextComponent.piece()));
                nextComponent.piece().build(entrance, list, random);
                stronghold_component::addDoorwayTo(this, x, y, z, facing);
            }
        }
    }
}

// StructureTFStrongholdComponent.findBreakInComponent
pub(crate) fn findBreakInComponent(_this: &Piece, list: &mut Pieces, x: i32, y: i32, z: i32) -> Option<Rc<Piece>>
{
    for component in list.clone()
    {
        if (true) && component.m_box.get().contains(x, y, z)
        {
            return Some(Rc::clone(&component));
        }
    }

    None
}

// StructureTFStrongholdComponent.addNewUpperComponent
pub(crate) fn addNewUpperComponent(
    this: &Piece,
    parent: &Piece,
    list: &mut Pieces,
    random: &Random,
    facing: i32,
    x: i32,
    y: i32,
    z: i32,
)
{
    let index: i32 = this.m_depth.get() + 1;
    let nFacing: i32 = (this.m_mode.get() + facing) % 4;
    let nx: i32 = this.xAt(x, z);
    let ny: i32 = this.yAt(y);
    let nz: i32 = this.zAt(x, z);
    if (index <= 100) && (!this.isOutOfRange(parent, nx, ny, nz, 48))
    {
        let var10000: Rc<Piece> = match random.nextIntBounded(5)
        {
            1 => stronghold_upper_left_turn::new(index, nFacing, nx, ny, nz),
            2 => stronghold_upper_right_turn::new(index, nFacing, nx, ny, nz),
            3 => stronghold_upper_corridor::new(index, nFacing, nx, ny, nz),
            4 => stronghold_upper_ascender::new(index, nFacing, nx, ny, nz),
            _ => stronghold_upper_tintersection::new(index, nFacing, nx, ny, nz),
        };
        let attempted: Option<Rc<Piece>> = Some(Rc::clone(&var10000));
        if attempted.is_some() && intersecting(list, attempted.piece().bounds()).is_none()
        {
            list.push(Rc::clone(attempted.piece()));
            attempted.piece().build(parent, list, random);
        }
    }
}

// StructureTFStrongholdComponent.isOutOfRange
pub(crate) fn isOutOfRange(_this: &Piece, parent: &Piece, nx: i32, _ny: i32, nz: i32, range: i32) -> bool
{
    ((nx - parent.bounds().m_min[0]).abs() > range) || ((nz - parent.bounds().m_min[2]).abs() > range)
}

// StructureTFStrongholdComponent.placeDoorwayAt
pub(crate) fn placeDoorwayAt(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    x: i32,
    y: i32,
    z: i32,
    sbb: StructureBox,
)
{
    if (x != 0) && (x != stronghold_component::getXSize(this))
    {
        this.fillMeta(
            world,
            sbb,
            x - 2,
            y,
            z,
            x + 2,
            y + 3,
            z,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            blk("air"),
            0,
            false,
        );
        this.fillAir(world, sbb, x - 1, y, z, x + 1, y + 3, z);
    }
    else
    {
        this.fillMeta(
            world,
            sbb,
            x,
            y,
            z - 2,
            x,
            y + 3,
            z + 2,
            this.m_deco.get().m_fenceID,
            this.m_deco.get().m_fenceMeta,
            blk("air"),
            0,
            false,
        );
        this.fillAir(world, sbb, x, y, z - 1, x, y + 3, z + 1);
    }
}

// StructureTFStrongholdComponent.getXSize
pub(crate) fn getXSize(this: &Piece) -> i32
{
    let var10000: i32 = match this.mode()
    {
        1 | 3 => this.m_box.get().zSize() - 1,
        _ => this.m_box.get().xSize() - 1,
    };
    var10000
}

// StructureTFStrongholdComponent.placeSmallDoorwayAt
pub(crate) fn placeSmallDoorwayAt(
    this: &Piece,
    world: &GenWorld,
    _random: &Random,
    facing: i32,
    x: i32,
    y: i32,
    z: i32,
    sbb: StructureBox,
)
{
    if (facing != 0) && (facing != 2)
    {
        this.fillMeta(world, sbb, x, y, z - 1, x, y + 1, z + 1, blk("cobblestone_wall"), 0, blk("air"), 0, true);
        this.fillAir(world, sbb, x, y, z, x, y + 1, z);
    }
    else
    {
        this.fillMeta(world, sbb, x - 1, y, z, x + 1, y + 1, z, blk("cobblestone_wall"), 0, blk("air"), 0, true);
        this.fillAir(world, sbb, x, y, z, x, y + 1, z);
    }
}

// StructureTFStrongholdComponent.placeCornerStatue
pub(crate) fn placeCornerStatue(this: &Piece, world: &GenWorld, x: i32, y: i32, z: i32, facing: i32, sbb: StructureBox)
{
    let mut ox: i32 = 1;
    let mut oz: i32 = 1;
    let mut smx: i32 = 2;
    let mut smz: i32 = 3;

    match facing
    {
        0 =>
        {}
        1 =>
        {
            oz = -1;
            smz = 1;
        }

        2 =>
        {
            ox = -1;
            smx = 0;
        }

        3 =>
        {
            ox = -1;
            oz = -1;
            smx = 0;
            smz = 1;
        }

        _ =>
        {}
    }

    for sy in 0..5
    {
        this.placeBlock(world, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, x, y + sy, z, sbb);
    }

    this.placeBlock(world, blk("fence"), 0, x + 0, y + 4, z + oz, sbb);
    this.placeBlock(world, blk("fence"), 0, x + ox, y + 4, z + 0, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(smz), x + 0, y + 3, z + oz, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(smx), x + ox, y + 3, z + 0, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(smz) + 4, x + 0, y + 2, z + oz, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(smx) + 4, x + ox, y + 2, z + 0, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(smx) + 4, x + ox, y + 2, z + oz, sbb);
    this.placeBlock(world, blk("cobblestone_wall"), 0, x + ox, y + 0, z + oz, sbb);
    this.placeBlock(world, blk("cobblestone_wall"), 0, x + ox, y + 1, z + oz, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(smz), x + 0, y + 0, z + oz, sbb);
    this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(smx), x + ox, y + 0, z + 0, sbb);
}

// StructureTFStrongholdComponent.placeWallStatue
pub(crate) fn placeWallStatue(this: &Piece, world: &GenWorld, x: i32, y: i32, z: i32, facing: i32, sbb: StructureBox)
{
    let mut ox: i32 = 1;
    let mut oz: i32 = 1;

    for sy in 0..5
    {
        this.placeBlock(world, this.m_deco.get().m_pillarID, this.m_deco.get().m_pillarMeta, x, y + sy, z, sbb);
    }

    if (facing != 0) && (facing != 2)
    {
        if facing == 3
        {
            oz = -oz;
            ox = -ox;
        }

        this.placeBlock(world, blk("fence"), 0, x, y + 4, z - oz, sbb);
        this.placeBlock(world, blk("fence"), 0, x, y + 4, z + oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(0 + facing), x, y + 3, z - oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(2 + facing), x, y + 3, z + oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1 + facing), x + ox, y + 3, z - oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1 + facing), x + ox, y + 3, z + oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(0 + facing) + 4, x, y + 2, z - oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(2 + facing) + 4, x, y + 2, z + oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1 + facing) + 4, x + oz, y + 2, z + 0, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1 + facing) + 4, x + ox, y + 2, z - oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1 + facing) + 4, x + ox, y + 2, z + oz, sbb);
        this.placeBlock(world, blk("cobblestone_wall"), 0, x + ox, y + 0, z, sbb);
        this.placeBlock(world, blk("cobblestone_wall"), 0, x + ox, y + 1, z, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(0 + facing), x, y + 0, z - ox, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(2 + facing), x, y + 0, z + ox, sbb);
    }
    else
    {
        if facing == 2
        {
            ox = -ox;
            oz = -oz;
        }

        this.placeBlock(world, blk("fence"), 0, x - ox, y + 4, z, sbb);
        this.placeBlock(world, blk("fence"), 0, x + ox, y + 4, z, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(0 + facing), x - ox, y + 3, z, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(2 + facing), x + ox, y + 3, z, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1 + facing), x - ox, y + 3, z - oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1 + facing), x + ox, y + 3, z - oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(0 + facing) + 4, x - ox, y + 2, z, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(2 + facing) + 4, x + ox, y + 2, z, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1 + facing) + 4, x + 0, y + 2, z - oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1 + facing) + 4, x - ox, y + 2, z - oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(1 + facing) + 4, x + ox, y + 2, z - oz, sbb);
        this.placeBlock(world, blk("cobblestone_wall"), 0, x, y + 0, z - oz, sbb);
        this.placeBlock(world, blk("cobblestone_wall"), 0, x, y + 1, z - oz, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(0 + facing), x - ox, y + 0, z + 0, sbb);
        this.placeBlock(world, this.m_deco.get().m_stairID, this.stairMeta(2 + facing), x + ox, y + 0, z + 0, sbb);
    }
}

// StructureTFStrongholdComponent.attemptToBreakIn
pub(crate) fn attemptToBreakIn(this: &Piece, wx: i32, wy: i32, wz: i32) -> bool
{
    if !this.isValidBreakInPoint(wx, wy, wz)
    {
        return false;
    }

    let dx: i32 = stronghold_component::relativeX(this, wx, wz);
    let dy: i32 = stronghold_component::relativeY(this, wy);
    let dz: i32 = stronghold_component::relativeZ(this, wx, wz);
    stronghold_component::addDoor(this, dx, dy, dz);
    true
}

// StructureTFStrongholdComponent.addDoorwayTo
pub(crate) fn addDoorwayTo(this: &Piece, dx: i32, dy: i32, dz: i32, facing: i32)
{
    match facing
    {
        0 =>
        {
            stronghold_component::addDoor(this, dx, dy, dz - 1);
        }

        1 =>
        {
            stronghold_component::addDoor(this, dx + 1, dy, dz);
        }

        2 =>
        {
            stronghold_component::addDoor(this, dx, dy, dz + 1);
        }

        3 =>
        {
            stronghold_component::addDoor(this, dx - 1, dy, dz);
        }

        _ =>
        {}
    }
}

// StructureTFStrongholdComponent.addDoor
pub(crate) fn addDoor(this: &Piece, dx: i32, dy: i32, dz: i32) { this.m_doors.borrow_mut().push([dx, dy, dz]); }

// StructureTFStrongholdComponent.isValidBreakInPoint
pub(crate) fn isValidBreakInPoint(this: &Piece, wx: i32, wy: i32, wz: i32) -> bool
{
    if (wy < this.m_box.get().m_min[1]) || (wy > this.m_box.get().m_max[1])
    {
        false
    }
    else if (wx == this.m_box.get().m_min[0]) || (wx == this.m_box.get().m_max[0])
    {
        (wz > this.m_box.get().m_min[2]) && (wz < this.m_box.get().m_max[2])
    }
    else
    {
        if (wz != this.m_box.get().m_min[2]) && (wz != this.m_box.get().m_max[2])
        {
            false
        }
        else
        {
            (wx > this.m_box.get().m_min[0]) && (wx < this.m_box.get().m_max[0])
        }
    }
}

// StructureTFStrongholdComponent.getRelativeX
pub(crate) fn relativeX(this: &Piece, x: i32, z: i32) -> i32
{
    let var10000: i32 = match this.mode()
    {
        0 => x - this.m_box.get().m_min[0],
        1 => z - this.m_box.get().m_min[2],
        2 => this.m_box.get().m_max[0] - x,
        3 => this.m_box.get().m_max[2] - z,
        _ => x,
    };
    var10000
}

// StructureTFStrongholdComponent.getRelativeY
pub(crate) fn relativeY(this: &Piece, par1: i32) -> i32 { par1 - this.m_box.get().m_min[1] }

// StructureTFStrongholdComponent.getRelativeZ
pub(crate) fn relativeZ(this: &Piece, x: i32, z: i32) -> i32
{
    let var10000: i32 = match this.mode()
    {
        0 => z - this.m_box.get().m_min[2],
        1 => this.m_box.get().m_max[0] - x,
        2 => this.m_box.get().m_max[2] - z,
        3 => x - this.m_box.get().m_min[0],
        _ => z,
    };
    var10000
}

// StructureTFStrongholdComponent.placeDoors
pub(crate) fn placeDoors(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    for doorCoords in this.m_doors.borrow().clone()
    {
        this.placeDoorwayAt(world, random, doorCoords[0], doorCoords[1], doorCoords[2], sbb);
    }
}

// StructureTFStrongholdComponent.placeStrongholdWalls
pub(crate) fn placeStrongholdWalls(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    sx: i32,
    sy: i32,
    sz: i32,
    dx: i32,
    dy: i32,
    dz: i32,
    random: &Random,
    randomBlocks: Stones,
)
{
    for y in sy..=dy
    {
        for x in sx..=dx
        {
            for z in sz..=dz
            {
                let wall: bool = (((((y == sy) || (y == dy)) || (x == sx)) || (x == dx)) || (z == sz)) || (z == dz);
                let blockID: Blk = this.getBlock(world, x, y, z, sbb);
                if blockID == blk("air")
                {
                    if wall
                    {
                        this.placeBlock(world, blk("cobblestone"), 0, x, y, z, sbb);
                    }
                }
                else if (y == sy) || (y == dy)
                {
                    let strongBlocks: Stones = Stones::stronghold();
                    strongBlocks.select(random, x, y, z, wall);
                    this.placeBlock(world, strongBlocks.block(), strongBlocks.meta(), x, y, z, sbb);
                }
                else if (!wall) || (blockID != blk("dirt"))
                {
                    randomBlocks.select(random, x, y, z, wall);
                    this.placeBlock(world, randomBlocks.block(), randomBlocks.meta(), x, y, z, sbb);
                }
            }
        }
    }
}

// StructureTFStrongholdComponent.placeUpperStrongholdWalls
pub(crate) fn placeUpperStrongholdWalls(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    sx: i32,
    sy: i32,
    sz: i32,
    dx: i32,
    dy: i32,
    dz: i32,
    random: &Random,
    randomBlocks: Stones,
)
{
    for y in sy..=dy
    {
        for x in sx..=dx
        {
            for z in sz..=dz
            {
                let wall: bool = (((((y == sy) || (y == dy)) || (x == sx)) || (x == dx)) || (z == sz)) || (z == dz);
                let blockID: Blk = this.getBlock(world, x, y, z, sbb);
                if ((blockID != blk("air"))
                    && (((blockID.material() == Material::Rock) || (blockID.material() == Material::Grass))
                        || (blockID.material() == Material::Ground)))
                    || (((blockID == blk("air")) && (random.nextIntBounded(3) == 0))
                        && (this.getBlock(world, x, y - 1, z, sbb) == blk("stonebrick")))
                {
                    if (y != sy) && (y != dy)
                    {
                        randomBlocks.select(random, x, y, z, wall);
                        this.placeBlock(world, randomBlocks.block(), randomBlocks.meta(), x, y, z, sbb);
                    }
                    else
                    {
                        let strongBlocks: Stones = Stones::stronghold();
                        strongBlocks.select(random, x, y, z, wall);
                        this.placeBlock(world, strongBlocks.block(), strongBlocks.meta(), x, y, z, sbb);
                    }
                }
            }
        }
    }
}
