// draft from ComponentTFTrollCaveConnect.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(index: i32, x: i32, y: i32, z: i32, caveSize: i32, caveHeight: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::TrollCaveConnect);
    init(&this, index, x, y, z, caveSize, caveHeight, direction);
    Rc::new(this)
}

// ComponentTFTrollCaveConnect constructor
pub(crate) fn init(this: &Piece, index: i32, x: i32, y: i32, z: i32, caveSize: i32, caveHeight: i32, direction: i32)
{
    troll_cave_main::init(this, index);
    *this.m_openingTowards.borrow_mut() = vec![false, false, true, false];
    this.m_size.set(caveSize);
    this.m_height.set(caveHeight);
    this.setMode(direction);
    this.m_box.set(boxFor(
        x,
        y,
        z,
        0,
        0,
        0,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
        direction,
    ));
}

// ComponentTFTrollCaveConnect.func_74861_a
pub(crate) fn build(this: &Piece, _parent: &Piece, list: &mut Pieces, random: &Random)
{
    if this.depth() < 3
    {
        for i in 0..4
        {
            let dest: Option<[i32; 3]> = troll_cave_main::validOpening3(this, random, 2, i);
            if random.nextBoolean()
                || (!troll_cave_connect::makeGardenCave(
                    this,
                    list,
                    random,
                    this.depth() + 1,
                    dest.piece()[0],
                    dest.piece()[1],
                    dest.piece()[2],
                    30,
                    15,
                    i,
                ))
            {
                this.makeSmallerCave(
                    list,
                    random,
                    this.depth() + 1,
                    dest.piece()[0],
                    dest.piece()[1],
                    dest.piece()[2],
                    20,
                    15,
                    i,
                );
            }
        }
    }
}

// ComponentTFTrollCaveConnect.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    if troll_cave_main::boundingBoxOutOfHighlands(this, world, sbb)
    {
        return false;
    }

    troll_cave_main::hollowCaveMiddle(
        this,
        world,
        sbb,
        random,
        0,
        0,
        0,
        this.m_size.get() - 1,
        this.m_height.get() - 1,
        this.m_size.get() - 1,
    );

    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );

    for i in 0..4
    {
        if !this.m_openingTowards.borrow()[(i) as usize]
        {
            troll_cave_connect::decorateWall(this, world, sbb, &decoRNG, i);
        }
    }

    decoRNG.setSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );

    for _i in 0..32
    {
        let dest: [i32; 3] = troll_cave_main::coordsInCave(this, &decoRNG);
        troll_cave_main::generateBlockStalactite9(
            this,
            world,
            &decoRNG,
            blk("stone"),
            0.5,
            true,
            dest[0],
            3,
            dest[2],
            sbb,
        );
    }

    for _i in 0..8
    {
        let dest: [i32; 3] = troll_cave_main::coordsInCave(this, &decoRNG);
        troll_cave_main::generateBlockStalactite9(
            this,
            world,
            &decoRNG,
            blk("stone"),
            0.5,
            false,
            dest[0],
            3,
            dest[2],
            sbb,
        );
    }

    decoRNG.setSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );

    if (troll_cave_connect::countExits(this) == 1) && (decoRNG.nextIntBounded(3) == 0)
    {
        troll_cave_main::makeTreasureCrate(this, world, &decoRNG, sbb);
    }
    else if decoRNG.nextIntBounded(3) == 0
    {
        troll_cave_connect::makeMonolith(this, world, &decoRNG, sbb);
    }
    true
}

// ComponentTFTrollCaveConnect.makeMonolith
pub(crate) fn makeMonolith(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    let mid: i32 = this.m_size.get() / 2;
    let height: i32 = 7 + random.nextIntBounded(8);
    let rotation: i32 = random.nextIntBounded(4);
    this.fillTurned(world, sbb, mid - 1, 0, mid - 1, mid - 1, height, mid - 1, blk("obsidian"), 0, rotation);
    this.fillTurned(world, sbb, mid + 0, 0, mid - 1, mid + 0, height - 2, mid - 1, blk("obsidian"), 0, rotation);
    this.fillTurned(world, sbb, mid - 1, 0, mid + 0, mid - 1, height - 2, mid + 0, blk("obsidian"), 0, rotation);
    this.fillTurned(world, sbb, mid + 0, 0, mid + 0, mid + 0, height - 4, mid + 0, blk("obsidian"), 0, rotation);
}

// ComponentTFTrollCaveConnect.countExits
pub(crate) fn countExits(this: &Piece) -> i32
{
    let mut count: i32 = 0;

    for openingToward in this.m_openingTowards.borrow().clone()
    {
        if openingToward
        {
            count += 1;
        }
    }

    count
}

// ComponentTFTrollCaveConnect.decorateWall
pub(crate) fn decorateWall(this: &Piece, world: &GenWorld, sbb: StructureBox, decoRNG: &Random, rotation: i32)
{
    if decoRNG.nextBoolean()
    {
        troll_cave_connect::decorateBracketMushrooms(this, world, sbb, decoRNG, rotation);
    }
    else if decoRNG.nextBoolean()
    {
        troll_cave_connect::decorateStoneFormation(this, world, sbb, decoRNG, rotation);
        troll_cave_connect::decorateStoneFormation(this, world, sbb, decoRNG, rotation);
    }
    else
    {
        troll_cave_connect::decorateStoneProjection(this, world, sbb, decoRNG, rotation);
    }
}

// ComponentTFTrollCaveConnect.decorateStoneFormation
pub(crate) fn decorateStoneFormation(this: &Piece, world: &GenWorld, sbb: StructureBox, decoRNG: &Random, rotation: i32)
{
    let mut z: i32 = 5 + decoRNG.nextIntBounded(7);
    let startY: i32 = 1 + decoRNG.nextIntBounded(2);
    for y in (startY..this.m_height.get()).step_by(2)
    {
        let width: i32 = 1;
        let depth: i32 = 1 + (if decoRNG.nextIntBounded(3) == 0 { 1 } else { 0 });
        troll_cave_connect::makeSingleStoneFormation(this, world, sbb, decoRNG, rotation, z, y, width, depth);
        z += decoRNG.nextIntBounded(4) - decoRNG.nextIntBounded(4);
        if (z < 5) || (z > (this.m_size.get() - 5))
        {
            z = 5 + decoRNG.nextIntBounded(7);
        }
    }
}

// ComponentTFTrollCaveConnect.makeSingleStoneFormation
pub(crate) fn makeSingleStoneFormation(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    decoRNG: &Random,
    rotation: i32,
    z: i32,
    y: i32,
    width: i32,
    depth: i32,
)
{
    if decoRNG.nextIntBounded(8) == 0
    {
        this.fillTurned(
            world,
            sbb,
            this.m_size.get() - (depth + 1),
            y - width,
            z - width,
            this.m_size.get() - 1,
            y + width,
            z + width,
            blk("obsidian"),
            0,
            rotation,
        );
    }
    else if decoRNG.nextIntBounded(4) == 0
    {
        this.fillTurned(
            world,
            sbb,
            this.m_size.get() - (depth + 1),
            y - width,
            z - width,
            this.m_size.get() - 1,
            y + width,
            z + width,
            blk("tf:trollSteinn"),
            0,
            rotation,
        );
    }
    else
    {
        this.fillTurned(
            world,
            sbb,
            this.m_size.get() - (depth + 1),
            y - width,
            z - width,
            this.m_size.get() - 1,
            y + width,
            z + width,
            blk("stone"),
            0,
            rotation,
        );
    }
}

// ComponentTFTrollCaveConnect.decorateStoneProjection
pub(crate) fn decorateStoneProjection(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    decoRNG: &Random,
    rotation: i32,
)
{
    let z: i32 = (7 + decoRNG.nextIntBounded(3)) - decoRNG.nextIntBounded(3);
    let y: i32 = (7 + decoRNG.nextIntBounded(3)) - decoRNG.nextIntBounded(3);
    this.fillRandomlyTurned(
        world,
        sbb,
        decoRNG,
        0.25,
        this.m_size.get() - 9,
        y,
        z,
        this.m_size.get() - 2,
        y + 3,
        z + 3,
        blk("tf:trollSteinn"),
        0,
        blk("stone"),
        0,
        rotation,
    );

    if decoRNG.nextBoolean()
    {
        this.fillRandomlyTurned(
            world,
            sbb,
            decoRNG,
            0.25,
            this.m_size.get() - 9,
            1,
            z,
            this.m_size.get() - 6,
            y - 1,
            z + 3,
            blk("tf:trollSteinn"),
            0,
            blk("stone"),
            0,
            rotation,
        );
    }
    else
    {
        this.fillRandomlyTurned(
            world,
            sbb,
            decoRNG,
            0.25,
            this.m_size.get() - 9,
            y + 4,
            z,
            this.m_size.get() - 6,
            this.m_height.get() - 2,
            z + 3,
            blk("tf:trollSteinn"),
            0,
            blk("stone"),
            0,
            rotation,
        );
    }
}

// ComponentTFTrollCaveConnect.decorateBracketMushrooms
pub(crate) fn decorateBracketMushrooms(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    decoRNG: &Random,
    rotation: i32,
)
{
    let mut z: i32 = 5 + decoRNG.nextIntBounded(7);
    let startY: i32 = 1 + decoRNG.nextIntBounded(4);
    for y in (startY..this.m_height.get()).step_by(2)
    {
        let width: i32 = (1 + decoRNG.nextIntBounded(2)) + decoRNG.nextIntBounded(2);
        let depth: i32 = (1 + decoRNG.nextIntBounded(2)) + decoRNG.nextIntBounded(2);

        let mushBlock: Blk = if decoRNG.nextIntBounded(3) == 0
        {
            blk("tf:hugeGloomBlock")
        }
        else
        {
            if decoRNG.nextBoolean()
            {
                blk("brown_mushroom_block")
            }
            else
            {
                blk("red_mushroom_block")
            }
        };
        troll_cave_connect::makeSingleBracketMushroom(this, world, sbb, rotation, z, y, width, depth, mushBlock);
        z += decoRNG.nextIntBounded(4) - decoRNG.nextIntBounded(4);
        if (z < 5) || (z > (this.m_size.get() - 5))
        {
            z = 5 + decoRNG.nextIntBounded(7);
        }
    }
}

// ComponentTFTrollCaveConnect.makeSingleBracketMushroom
pub(crate) fn makeSingleBracketMushroom(
    this: &Piece,
    world: &GenWorld,
    sbb: StructureBox,
    rotation: i32,
    z: i32,
    y: i32,
    width: i32,
    depth: i32,
    mushBlock: Blk,
)
{
    this.fillTurned(
        world,
        sbb,
        this.m_size.get() - depth,
        y,
        z - (width - 1),
        this.m_size.get() - 2,
        y,
        z + (width - 1),
        mushBlock,
        5,
        rotation,
    );
    this.fillTurned(
        world,
        sbb,
        this.m_size.get() - (depth + 1),
        y,
        z - (width - 1),
        this.m_size.get() - (depth + 1),
        y,
        z + (width - 1),
        mushBlock,
        troll_cave_connect::mushroomMetaFor(this, 4, rotation),
        rotation,
    );

    for d in 0..(depth - 1)
    {
        this.placeTurned(
            world,
            mushBlock,
            troll_cave_connect::mushroomMetaFor(this, 2, rotation),
            this.m_size.get() - (2 + d),
            y,
            z - width,
            rotation,
            sbb,
        );
    }

    this.placeTurned(
        world,
        mushBlock,
        troll_cave_connect::mushroomMetaFor(this, 1, rotation),
        this.m_size.get() - (depth + 1),
        y,
        z - width,
        rotation,
        sbb,
    );

    for d in 0..(depth - 1)
    {
        this.placeTurned(
            world,
            mushBlock,
            troll_cave_connect::mushroomMetaFor(this, 8, rotation),
            this.m_size.get() - (2 + d),
            y,
            z + width,
            rotation,
            sbb,
        );
    }

    this.placeTurned(
        world,
        mushBlock,
        troll_cave_connect::mushroomMetaFor(this, 7, rotation),
        this.m_size.get() - (depth + 1),
        y,
        z + width,
        rotation,
        sbb,
    );
}

// ComponentTFTrollCaveConnect.getMushroomMetaFor, huge mushroom sides turned with the piece
pub(crate) fn mushroomMetaFor(this: &Piece, meta: i32, rotation: i32) -> i32
{
    if meta <= 0 || meta >= 10
    {
        return meta;
    }

    // the java switch falls through from a quarter turn to the half turn and from three quarters to the stem
    let quarter: Option<i32> = match meta
    {
        1 => Some(3),
        2 => Some(6),
        4 => Some(2),
        7 => Some(1),
        8 => Some(4),
        _ => None,
    };

    let threeQuarters: Option<i32> = match meta
    {
        1 => Some(7),
        2 => Some(4),
        4 => Some(8),
        7 => Some(9),
        8 => Some(6),
        _ => None,
    };

    match (this.mode() + rotation) % 4
    {
        0 => meta,
        1 => quarter.unwrap_or(10 - meta % 10),
        2 => 10 - meta % 10,
        3 => threeQuarters.unwrap_or(15),
        _ => 15,
    }
}

// ComponentTFTrollCaveConnect.makeGardenCave
pub(crate) fn makeGardenCave(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    caveSize: i32,
    caveHeight: i32,
    rotation: i32,
) -> bool
{
    let direction: i32 = (this.mode() + rotation) % 4;
    let dest: [i32; 3] = this.offsetTowerAt(x, y, z, caveSize, direction);
    let cave: Rc<Piece> = troll_cave_garden::new(index, dest[0], dest[1], dest[2], caveSize, caveHeight, direction);
    let intersect: Option<Rc<Piece>> = intersecting(list, cave.bounds());
    let otherGarden: Option<Rc<Piece>> = troll_cave_connect::findNearbyGarden(this, list, cave.bounds());
    if ((intersect.is_none()) || (same(&intersect, &this))) && (otherGarden.is_none())
    {
        list.push(Rc::clone(&cave));
        cave.build(&Rc::clone(&list[0usize]), list, random);
        this.m_openingTowards.borrow_mut()[(rotation) as usize] = true;
        true
    }
    else
    {
        false
    }
}

// ComponentTFTrollCaveConnect.findNearbyGarden
pub(crate) fn findNearbyGarden(_this: &Piece, list: &mut Pieces, boundingBox: StructureBox) -> Option<Rc<Piece>>
{
    let mut largeBox: StructureBox = boundingBox;
    largeBox.m_min[0] -= 30;
    largeBox.m_min[1] -= 30;
    largeBox.m_min[2] -= 30;
    largeBox.m_max[0] += 30;
    largeBox.m_max[1] += 30;
    largeBox.m_max[2] += 30;

    for component in list.clone()
    {
        if (matches!(component.m_kind, Kind::TrollCaveGarden)) && component.bounds().intersects(&largeBox)
        {
            return Some(Rc::clone(&component));
        }
    }

    None
}

// ComponentTFTrollCaveConnect.makeSmallerCave
pub(crate) fn makeSmallerCave(
    this: &Piece,
    list: &mut Pieces,
    random: &Random,
    index: i32,
    x: i32,
    y: i32,
    z: i32,
    caveSize: i32,
    caveHeight: i32,
    rotation: i32,
) -> bool
{
    if troll_cave_main::makeSmallerCave(this, list, random, index, x, y, z, caveSize, caveHeight, rotation)
    {
        this.m_openingTowards.borrow_mut()[(rotation) as usize] = true;
        true
    }
    else
    {
        false
    }
}
