// draft from ComponentTFFinalCastleMazeTower13.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, r#type: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleMazeTower13);
    init(&this, random, i, x, y, z, r#type, direction);
    Rc::new(this)
}

// ComponentTFFinalCastleMazeTower13 constructor
pub fn init(this: &Piece, random: &Random, i: i32, x: i32, y: i32, z: i32, r#type: i32, direction: i32)
{
    tower_wing::init(this, i);
    this.setMode(direction);
    this.m_type.set(r#type);
    this.m_size.set(13);
    let floors: i32 = random.nextIntBounded(3) + 2;
    this.m_height.set((floors * 8) + 1);
    let mut entranceFloor: i32 = random.nextIntBounded(floors);
    if (y - (entranceFloor * 8)) < 144
    {
        entranceFloor = 0;
    }
    if (y + ((floors - entranceFloor) * 8)) > 222
    {
        entranceFloor = floors - 1;
    }
    this.m_box.set(boxFor(
        x,
        y,
        z,
        -6,
        0 - (entranceFloor * 8),
        -6,
        this.m_size.get() - 1,
        this.m_height.get(),
        this.m_size.get() - 1,
        0,
    ));
    this.addOpening(0, (entranceFloor * 8) + 1, this.m_size.get() / 2, 2);
}

// ComponentTFFinalCastleMazeTower13 constructor
pub fn init9(
    this: &Piece,
    _random: &Random,
    i: i32,
    x: i32,
    y: i32,
    z: i32,
    floors: i32,
    entranceFloor: i32,
    r#type: i32,
    direction: i32,
)
{
    tower_wing::init(this, i);
    this.setMode(direction);
    this.m_type.set(r#type);
    this.m_size.set(13);
    this.m_height.set((floors * 8) + 1);
    this.m_box.set(boxFor(
        x,
        y,
        z,
        -6,
        0 - (entranceFloor * 8),
        -6,
        this.m_size.get() - 1,
        this.m_height.get(),
        this.m_size.get() - 1,
        0,
    ));
    this.addOpening(0, (entranceFloor * 8) + 1, this.m_size.get() / 2, 2);
}

// ComponentTFFinalCastleMazeTower13.func_74861_a
pub fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());
    let foundation: Rc<Piece> = final_castle_foundation13::new(random, 4, this);
    list.push(Rc::clone(&foundation));
    foundation.build(this, list, random);
    let roof: Rc<Piece> = if random.nextBoolean()
    {
        final_castle_roof13_conical::new(random, 4, this)
    }
    else
    {
        final_castle_roof13_crenellated::new(random, 4, this)
    };
    list.push(Rc::clone(&roof));
    roof.build(this, list, random);
}

// ComponentTFFinalCastleMazeTower13.buildTowards
pub fn buildTowards(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random, dest: [i32; 3])
{
    this.build(&parent, list, random);
    if this.depth() < 20
    {
        if final_castle_maze_tower13::isWithinRange(
            this,
            dest[0],
            dest[2],
            this.m_box.get().m_min[0] + 6,
            this.m_box.get().m_min[2] + 6,
            30,
        )
        {
            let howFar: i32 = 20;
            if ((!final_castle_maze_tower13::buildEndTowerTowards(
                this,
                list,
                random,
                dest,
                final_castle_maze_tower13::findBestDirectionTowards(this, dest),
                howFar,
            )) && (!final_castle_maze_tower13::buildEndTowerTowards(
                this,
                list,
                random,
                dest,
                final_castle_maze_tower13::findSecondDirectionTowards(this, dest),
                howFar,
            ))) && (!final_castle_maze_tower13::buildEndTowerTowards(
                this,
                list,
                random,
                dest,
                final_castle_maze_tower13::findThirdDirectionTowards(this, dest),
                howFar,
            ))
            {}
        }
        else
        {
            let howFar: i32 = 14 + random.nextIntBounded(24);
            let mut direction: i32 = final_castle_maze_tower13::findBestDirectionTowards(this, dest);
            if (direction == 0)
                || (!final_castle_maze_tower13::buildContinueTowerTowards(this, list, random, dest, direction, howFar))
            {
                direction = final_castle_maze_tower13::findSecondDirectionTowards(this, dest);
                if (direction == 0)
                    || (!final_castle_maze_tower13::buildContinueTowerTowards(
                        this, list, random, dest, direction, howFar,
                    ))
                {
                    direction = final_castle_maze_tower13::findThirdDirectionTowards(this, dest);
                    if ((direction == 0)
                        || (!final_castle_maze_tower13::buildContinueTowerTowards(
                            this, list, random, dest, direction, howFar,
                        )))
                        && (!final_castle_maze_tower13::buildContinueTowerTowards(this, list, random, dest, 0, howFar))
                    {
                    }
                }
            }
        }
    }
    final_castle_maze_tower13::buildNonCriticalTowers(this, &parent, list, random);
}

// ComponentTFFinalCastleMazeTower13.buildNonCriticalTowers
pub fn buildNonCriticalTowers(this: &Piece, _parent: &Piece, list: &mut Pieces, random: &Random)
{
    let mut dir: i32 = random.nextIntBounded(4);
    if (!this.m_openingTowards.borrow()[(dir) as usize])
        && (!final_castle_maze_tower13::buildDamagedTower(this, list, random, dir))
    {
        dir = (dir + random.nextIntBounded(4)) % 4;
        if !final_castle_maze_tower13::buildDamagedTower(this, list, random, dir)
        {}
    }
}

// ComponentTFFinalCastleMazeTower13.findBestDirectionTowards
pub fn findBestDirectionTowards(this: &Piece, dest: [i32; 3]) -> i32
{
    let cx: i32 = this.m_box.get().m_min[0] + 6;
    let cz: i32 = this.m_box.get().m_min[2] + 6;
    let dx: i32 = cx - dest[0];
    let dz: i32 = cz - dest[2];
    let absoluteDir: i32 = if dx.abs() > dz.abs()
    {
        if dx >= 0 { 2 } else { 0 }
    }
    else
    {
        if dz >= 0 { 3 } else { 1 }
    };
    return ((absoluteDir + 4) - this.m_mode.get()) % 4;
}

// ComponentTFFinalCastleMazeTower13.findSecondDirectionTowards
pub fn findSecondDirectionTowards(this: &Piece, dest: [i32; 3]) -> i32
{
    let cx: i32 = this.m_box.get().m_min[0] + 6;
    let cz: i32 = this.m_box.get().m_min[2] + 6;
    let dx: i32 = cx - dest[0];
    let dz: i32 = cz - dest[2];
    let absoluteDir: i32 = if dx.abs() < dz.abs()
    {
        if dx >= 0 { 2 } else { 0 }
    }
    else
    {
        if dz >= 0 { 3 } else { 1 }
    };
    return ((absoluteDir + 4) - this.m_mode.get()) % 4;
}

// ComponentTFFinalCastleMazeTower13.findThirdDirectionTowards
pub fn findThirdDirectionTowards(this: &Piece, dest: [i32; 3]) -> i32
{
    let first: i32 = final_castle_maze_tower13::findBestDirectionTowards(this, dest);
    let second: i32 = final_castle_maze_tower13::findSecondDirectionTowards(this, dest);
    if (first == 0) && (second == 1)
    {
        return 3;
    }
    else
    {
        return if (first == 1) && (second == 3) { 0 } else { 1 };
    }
}

// ComponentTFFinalCastleMazeTower13.buildContinueTowerTowards
pub fn buildContinueTowerTowards(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    dest: [i32; 3],
    mut direction: i32,
    howFar: i32,
) -> bool
{
    let mut opening: [i32; 3] = this.getValidOpeningCC(random, direction);
    let adjustmentRange: i32 = 60;
    if final_castle_maze_tower13::isWithinRange(
        this,
        dest[0],
        dest[2],
        this.m_box.get().m_min[0] + 6,
        this.m_box.get().m_min[2] + 6,
        adjustmentRange,
    )
    {
        opening[1] = final_castle_maze_tower13::adjustOpening(this, opening[1], dest);
    }
    direction += this.m_mode.get();
    direction %= 4;
    let tc: [i32; 3] = this.offsetTowerAt(opening[0], opening[1], opening[2], howFar, direction);
    let start: Rc<Piece> = Rc::clone(&list[(0) as usize]);
    let centerX: i32 = ((start.bounds().m_min[0] + 128) >> 8) << 8;
    let centerZ: i32 = ((start.bounds().m_min[2] + 128) >> 8) << 8;
    if final_castle_maze_tower13::isWithinRange(this, centerX, centerZ, tc[0], tc[2], 128)
    {
        let sTower: Rc<Piece> =
            final_castle_maze_tower13::new(random, this.depth() + 1, tc[0], tc[1], tc[2], this.m_type.get(), direction);
        let mut largerBB: StructureBox = sTower.bounds();
        largerBB.m_min[0] -= 6;
        largerBB.m_min[2] -= 6;
        largerBB.m_max[0] += 6;
        largerBB.m_max[2] += 6;
        largerBB.m_min[1] = 0;
        largerBB.m_max[1] = 255;
        let intersect: Option<Rc<Piece>> = intersecting(list, largerBB);
        if intersect.is_none()
        {
            list.push(Rc::clone(&sTower));
            final_castle_maze_tower13::buildTowards(&sTower, this, list, random, dest);
            let bc: [i32; 3] = this.offsetTowerAt(opening[0], opening[1], opening[2], 1, direction);
            let bridge: Rc<Piece> =
                final_castle_bridge::new(this.depth() + 1, bc[0], bc[1], bc[2], howFar - 7, direction);
            list.push(Rc::clone(&bridge));
            bridge.build(this, list, random);
            this.addOpening(opening[0], opening[1] + 1, opening[2], direction);
            return true;
        }
        else
        {
            return false;
        }
    }
    else
    {
        return false;
    }
}

// ComponentTFFinalCastleMazeTower13.buildDamagedTower
pub fn buildDamagedTower(this: &Piece, list: &mut Pieces, random: &Random, mut direction: i32) -> bool
{
    let opening: [i32; 3] = this.getValidOpeningCC(random, direction);
    direction += this.m_mode.get();
    direction %= 4;
    let howFar: i32 = 14 + random.nextIntBounded(24);
    let tc: [i32; 3] = this.offsetTowerAt(opening[0], opening[1], opening[2], howFar, direction);
    let eTower: Rc<Piece> = this.makeNewDamagedTower(random, direction, tc);
    let mut largerBB: StructureBox = eTower.bounds();
    largerBB.m_min[0] -= 6;
    largerBB.m_min[2] -= 6;
    largerBB.m_max[0] += 6;
    largerBB.m_max[2] += 6;
    let intersect: Option<Rc<Piece>> = intersecting(list, largerBB);
    if intersect.is_none()
    {
        list.push(Rc::clone(&eTower));
        eTower.build(this, list, random);
        let bc: [i32; 3] = this.offsetTowerAt(opening[0], opening[1], opening[2], 1, direction);
        let bridge: Rc<Piece> = final_castle_bridge::new(this.depth() + 1, bc[0], bc[1], bc[2], howFar - 7, direction);
        list.push(Rc::clone(&bridge));
        bridge.build(this, list, random);
        this.addOpening(opening[0], opening[1] + 1, opening[2], direction);
        return true;
    }
    else
    {
        return false;
    }
}

// ComponentTFFinalCastleMazeTower13.makeNewDamagedTower
pub fn makeNewDamagedTower(this: &Piece, random: &Random, direction: i32, tc: [i32; 3]) -> Rc<Piece>
{
    return final_castle_damaged_tower::new(random, this.depth() + 1, tc[0], tc[1], tc[2], direction);
}

// ComponentTFFinalCastleMazeTower13.adjustOpening
pub fn adjustOpening(this: &Piece, posY: i32, dest: [i32; 3]) -> i32
{
    let mut openY: i32 = posY;
    let realOpeningY: i32 = this.yAt(openY);
    if (realOpeningY - dest[1]) < 12
    {
        openY = this.m_height.get() - 9;
    }
    else if (dest[1] - realOpeningY) < 12
    {
        openY = 0;
    }
    return openY;
}

// ComponentTFFinalCastleMazeTower13.buildEndTowerTowards
pub fn buildEndTowerTowards(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    dest: [i32; 3],
    mut direction: i32,
    howFar: i32,
) -> bool
{
    let mut opening: [i32; 3] = this.getValidOpeningCC(random, direction);
    opening[1] = final_castle_maze_tower13::adjustOpening(this, opening[1], dest);
    direction += this.m_mode.get();
    direction %= 4;
    let tc: [i32; 3] = this.offsetTowerAt(opening[0], opening[1], opening[2], howFar, direction);
    let eTower: Rc<Piece>;
    if this.m_type.get() == 0
    {
        eTower = final_castle_entrance_tower::new(random, this.depth() + 1, tc[0], tc[1], tc[2], direction);
    }
    else
    {
        eTower = final_castle_bell_tower21::new(random, this.depth() + 1, tc[0], tc[1], tc[2], direction);
    }
    let mut largerBB: StructureBox = eTower.bounds();
    largerBB.m_min[0] -= 6;
    largerBB.m_min[2] -= 6;
    largerBB.m_max[0] += 6;
    largerBB.m_max[2] += 6;
    let intersect: Option<Rc<Piece>> = intersecting(list, largerBB);
    if intersect.is_none()
    {
        list.push(Rc::clone(&eTower));
        eTower.build(this, list, random);
        let bc: [i32; 3] = this.offsetTowerAt(opening[0], opening[1], opening[2], 1, direction);
        let bridge: Rc<Piece> = final_castle_bridge::new(this.depth() + 1, bc[0], bc[1], bc[2], howFar - 7, direction);
        list.push(Rc::clone(&bridge));
        bridge.build(this, list, random);
        this.addOpening(opening[0], opening[1] + 1, opening[2], direction);
        return true;
    }
    else
    {
        return false;
    }
}

// ComponentTFFinalCastleMazeTower13.isWithinRange
pub fn isWithinRange(_this: &Piece, centerX: i32, centerZ: i32, posX: i32, posZ: i32, range: i32) -> bool
{
    let inRange: bool = ((centerX - posX).abs() < range) && ((centerZ - posZ).abs() < range);
    if !inRange
    {}
    return inRange;
}

// ComponentTFFinalCastleMazeTower13.getValidOpeningCC
pub fn getValidOpeningCC(this: &Piece, random: &Random, direction: i32) -> [i32; 3]
{
    let floors: i32 = this.m_height.get() / 8;
    if (direction == 0) || (direction == 2)
    {
        let rx: i32 = if direction == 0 { 12 } else { 0 };
        let rz: i32 = 6;
        let ry: i32 = random.nextIntBounded(floors) * 8;
        return [rx, ry, rz];
    }
    if (direction != 1) && (direction != 3)
    {
        return [0, 0, 0];
    }
    let rx: i32 = 6;
    let rz: i32 = if direction == 1 { 12 } else { 0 };
    let ry: i32 = random.nextIntBounded(floors) * 8;
    return [rx, ry, rz];
}

// ComponentTFFinalCastleMazeTower13.offsetTowerCCoords
pub fn offsetTowerCCoords(this: &Piece, x: i32, y: i32, z: i32, howFar: i32, direction: i32) -> [i32; 3]
{
    let mut dx: i32 = this.xAt(x, z);
    let dy: i32 = this.yAt(y);
    let mut dz: i32 = this.zAt(x, z);
    match direction
    {
        0 =>
        {
            dx += howFar;
        }
        1 =>
        {
            dz += howFar;
        }
        2 =>
        {
            dx -= howFar;
        }
        3 =>
        {
            dz -= howFar;
        }
        _ =>
        {}
    }
    return [dx, dy, dz];
}

// ComponentTFFinalCastleMazeTower13.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    this.fillSelected(
        world,
        sbb,
        0,
        0,
        0,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
        false,
        random,
        this.m_deco.get().m_randomBlocks,
    );
    for x in 0..this.m_size.get()
    {
        for z in 0..this.m_size.get()
        {
            this.fillDown(world, this.m_deco.get().m_blockID, this.m_deco.get().m_blockMeta, x, -1, z, sbb);
        }
    }
    let numBranches: i32 = (2 + decoRNG.nextIntBounded(4)) + decoRNG.nextIntBounded(3);
    for _i in 0..numBranches
    {
        tower_wing::makeGlyphBranches(this, world, &decoRNG, this.getGlyphMeta(), sbb);
    }
    final_castle_maze_tower13::addFloors(this, world, &decoRNG, sbb);
    this.makeOpenings(world, sbb);
    return true;
}

// ComponentTFFinalCastleMazeTower13.getGlyphMeta
pub fn getGlyphMeta(this: &Piece) -> i32 { return this.m_type.get(); }

// ComponentTFFinalCastleMazeTower13.addFloors
pub fn addFloors(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox)
{
    let floors: i32 = (this.m_highestOpening.get() / 8) + 1;
    for i in 1..floors
    {
        this.fillMeta(
            world,
            sbb,
            1,
            i * 8,
            1,
            11,
            i * 8,
            11,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            false,
        );
        final_castle_maze_tower13::addStairsDown(this, world, sbb, (i + 2) % 4, i * 8);
    }
    if this.hasAccessibleRoof()
    {
        final_castle_maze_tower13::addStairsDown(this, world, sbb, (floors + 2) % 4, this.m_height.get() - 1);
    }
}

// ComponentTFFinalCastleMazeTower13.hasAccessibleRoof
pub fn hasAccessibleRoof(this: &Piece) -> bool { return (this.m_height.get() - this.m_highestOpening.get()) < 9; }

// ComponentTFFinalCastleMazeTower13.addStairsDown
pub fn addStairsDown(this: &Piece, world: &GenWorld, sbb: StructureBox, rotation: i32, y: i32)
{
    for i in 0..4
    {
        let sx: i32 = 8 - i;
        let sy: i32 = y - i;
        let sz: i32 = 9;
        this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(0 + rotation), sx, sy, sz, rotation, sbb);
        this.placeTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            sx,
            sy - 1,
            sz,
            rotation,
            sbb,
        );
        this.placeTurned(
            world,
            this.m_deco.get().m_stairID,
            this.stairMeta(0 + rotation),
            sx,
            sy,
            sz - 1,
            rotation,
            sbb,
        );
        this.placeTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            sx,
            sy - 1,
            sz - 1,
            rotation,
            sbb,
        );
        this.fillAirTurned(world, sbb, sx, sy + 1, sz - 1, sx, sy + 3, sz, rotation);
    }
    this.fillTurned(
        world,
        sbb,
        3,
        y - 4,
        8,
        4,
        y - 4,
        9,
        this.m_deco.get().m_blockID,
        this.m_deco.get().m_blockMeta,
        rotation,
    );
    for i in 0..4
    {
        let sx: i32 = 4;
        let sy: i32 = (y - i) - 4;
        let sz: i32 = 7 - i;
        this.placeTurned(world, this.m_deco.get().m_stairID, this.stairMeta(1 + rotation), sx, sy, sz, rotation, sbb);
        this.placeTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            sx,
            sy - 1,
            sz,
            rotation,
            sbb,
        );
        this.placeTurned(
            world,
            this.m_deco.get().m_stairID,
            this.stairMeta(1 + rotation),
            sx - 1,
            sy,
            sz,
            rotation,
            sbb,
        );
        this.placeTurned(
            world,
            this.m_deco.get().m_blockID,
            this.m_deco.get().m_blockMeta,
            sx - 1,
            sy - 1,
            sz,
            rotation,
            sbb,
        );
        this.fillAirTurned(world, sbb, sx, sy + 1, sz, sx - 1, sy + 3, sz, rotation);
    }
}

// ComponentTFFinalCastleMazeTower13.makeDoorOpening
pub fn makeDoorOpening(this: &Piece, world: &GenWorld, dx: i32, dy: i32, dz: i32, sbb: StructureBox)
{
    if (dx == 0) || (dx == (this.m_size.get() - 1))
    {
        this.fillMeta(
            world,
            sbb,
            dx,
            dy - 1,
            dz - 2,
            dx,
            dy + 4,
            dz + 2,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            blk("air"),
            0,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            dx,
            dy,
            dz - 1,
            dx,
            dy + 3,
            dz + 1,
            blk("tf:castleDoor"),
            this.getGlyphMeta(),
            blk("air"),
            0,
            false,
        );
    }
    if (dz == 0) || (dz == (this.m_size.get() - 1))
    {
        this.fillMeta(
            world,
            sbb,
            dx - 2,
            dy - 1,
            dz,
            dx + 2,
            dy + 4,
            dz,
            this.m_deco.get().m_accentID,
            this.m_deco.get().m_accentMeta,
            blk("air"),
            0,
            false,
        );
        this.fillMeta(
            world,
            sbb,
            dx - 1,
            dy,
            dz,
            dx + 1,
            dy + 3,
            dz,
            blk("tf:castleDoor"),
            this.getGlyphMeta(),
            blk("air"),
            0,
            false,
        );
    }
}
