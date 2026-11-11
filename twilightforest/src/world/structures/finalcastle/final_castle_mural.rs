// draft from ComponentTFFinalCastleMural.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, width: i32, height: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleMural);
    init(&this, random, i, x, y, z, width, height, direction);
    Rc::new(this)
}

// ComponentTFFinalCastleMural constructor
pub fn init(this: &Piece, _random: &Random, i: i32, x: i32, y: i32, z: i32, width: i32, height: i32, direction: i32)
{
    this.m_depth.set(i);
    this.setMode(direction);
    this.m_box.set(boxFor2(x, y, z, 0, (-height) / 2, (-width) / 2, 1, height - 1, width - 1, direction));
}

// ComponentTFFinalCastleMural.func_74875_a
pub fn add(this: &Piece, world: &GenWorld, _random: &Random, sbb: StructureBox) -> bool
{
    this.m_height.set(this.m_box.get().ySize());
    this.m_width.set(
        if (this.m_mode.get() != 0) && (this.m_mode.get() != 2)
        {
            this.m_box.get().xSize()
        }
        else
        {
            this.m_box.get().zSize()
        },
    );
    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    if this.m_mural.borrow().is_empty()
    {
        let mut mural: Vec<Vec<i32>> = vec![vec![0; (this.m_height.get()) as usize]; (this.m_width.get()) as usize];
        let startX: i32 = (this.m_width.get() / 2) - 1;
        let startY: i32 = 2;
        for x in -1..2
        {
            for y in -1..2
            {
                mural[(startX + x) as usize][(startY + y) as usize] = 1;
            }
        }
        final_castle_mural::makeHorizontalTree(
            this,
            &decoRNG,
            &mut mural,
            startX + 1,
            startY,
            decoRNG.nextIntBounded(this.m_width.get() / 6) + (this.m_width.get() / 6),
            true,
        );
        final_castle_mural::makeHorizontalTree(
            this,
            &decoRNG,
            &mut mural,
            startX - 1,
            startY,
            decoRNG.nextIntBounded(this.m_width.get() / 6) + (this.m_width.get() / 6),
            false,
        );
        final_castle_mural::makeVerticalTree(
            this,
            &decoRNG,
            &mut mural,
            startX,
            startY + 1,
            decoRNG.nextIntBounded(this.m_height.get() / 6) + (this.m_height.get() / 6),
            true,
        );
        final_castle_mural::makeStripes(this, &decoRNG, &mut mural);
        *this.m_mural.borrow_mut() = mural;
    }
    for x in 0..this.m_width.get()
    {
        for y in 0..this.m_height.get()
        {
            if this.m_mural.borrow()[(x) as usize][(y) as usize] > 0
            {
                this.placeBlock(world, blk("tf:castleMagic"), 1, 0, y, x, sbb);
            }
        }
    }
    return true;
}

// ComponentTFFinalCastleMural.makeHorizontalTree
pub fn makeHorizontalTree(
    this: &Piece,
    decoRNG: &Random,
    mural: &mut [Vec<i32>],
    centerX: i32,
    centerY: i32,
    branchLength: i32,
    positive: bool,
)
{
    final_castle_mural::fillHorizontalLine(this, mural, centerX, centerY, branchLength, positive);
    final_castle_mural::makeHorizontalBranch(this, mural, decoRNG, centerX, centerY, branchLength, positive, true);
    final_castle_mural::makeHorizontalBranch(this, mural, decoRNG, centerX, centerY, branchLength, positive, false);
}

// ComponentTFFinalCastleMural.makeVerticalTree
pub fn makeVerticalTree(
    this: &Piece,
    decoRNG: &Random,
    mural: &mut [Vec<i32>],
    centerX: i32,
    centerY: i32,
    branchLength: i32,
    positive: bool,
)
{
    final_castle_mural::fillVerticalLine(this, mural, centerX, centerY, branchLength, positive);
    final_castle_mural::makeVerticalBranch(this, mural, decoRNG, centerX, centerY, branchLength, positive, true);
    final_castle_mural::makeVerticalBranch(this, mural, decoRNG, centerX, centerY, branchLength, positive, false);
}

// ComponentTFFinalCastleMural.makeHorizontalBranch
pub fn makeHorizontalBranch(
    this: &Piece,
    mural: &mut [Vec<i32>],
    random: &Random,
    sx: i32,
    sy: i32,
    length: i32,
    plusX: bool,
    plusY: bool,
) -> bool
{
    let downLine: i32 = ((length / 2) + 1) + random.nextIntBounded((length / 2).max(2));
    let branchLength: i32 = random.nextIntBounded(this.m_width.get() / 8) + (this.m_width.get() / 8);
    let mut clear: bool = true;
    for i in 0..=branchLength
    {
        let cx: i32 = sx
            + (if plusX
            {
                (downLine - 1) + i
            }
            else
            {
                -((downLine - 1) + i)
            });
        let cy: i32 = sy + (if plusY { 2 } else { -2 });
        if ((((cx < 0) || (cx >= this.m_width.get())) || (cy < 0)) || (cy >= this.m_height.get()))
            || (mural[(cx) as usize][(cy) as usize] > 0)
        {
            clear = false;
            break;
        }
    }
    if clear
    {
        let bx: i32 = sx + (if plusX { downLine } else { -downLine });
        let mut by: i32 = sy;
        final_castle_mural::fillVerticalLine(this, mural, bx, by, 1, plusY);
        by += if plusY { 2 } else { -2 };
        final_castle_mural::fillHorizontalLine(this, mural, bx, by, branchLength, plusX);
        if (((bx > 0) && (bx < this.m_width.get())) && (by > 0)) && (by < this.m_height.get())
        {
            if ((!final_castle_mural::makeHorizontalBranch(this, mural, random, bx, by, branchLength, plusX, true))
                && (!final_castle_mural::makeHorizontalBranch(this, mural, random, bx, by, branchLength, plusX, true)))
                && (!final_castle_mural::makeHorizontalBranch(this, mural, random, bx, by, branchLength, plusX, true))
            {
                final_castle_mural::makeHorizontalBranch(this, mural, random, bx, by, branchLength, plusX, true);
            }
            if ((!final_castle_mural::makeHorizontalBranch(this, mural, random, bx, by, branchLength, plusX, false))
                && (!final_castle_mural::makeHorizontalBranch(this, mural, random, bx, by, branchLength, plusX, false)))
                && (!final_castle_mural::makeHorizontalBranch(this, mural, random, bx, by, branchLength, plusX, false))
            {
                final_castle_mural::makeHorizontalBranch(this, mural, random, bx, by, branchLength, plusX, false);
            }
        }
        return true;
    }
    else
    {
        return false;
    }
}

// ComponentTFFinalCastleMural.makeVerticalBranch
pub fn makeVerticalBranch(
    this: &Piece,
    mural: &mut [Vec<i32>],
    random: &Random,
    sx: i32,
    sy: i32,
    length: i32,
    plusY: bool,
    plusX: bool,
) -> bool
{
    let downLine: i32 = ((length / 2) + 1) + random.nextIntBounded((length / 2).max(2));
    let branchLength: i32 = random.nextIntBounded(this.m_height.get() / 8) + (this.m_height.get() / 8);
    let mut clear: bool = true;
    for i in 0..=branchLength
    {
        let cx: i32 = sx + (if plusX { 2 } else { -2 });
        let cy: i32 = sy
            + (if plusY
            {
                (downLine - 1) + i
            }
            else
            {
                -((downLine - 1) + i)
            });
        if ((((cx < 0) || (cx >= this.m_width.get())) || (cy < 0)) || (cy >= this.m_height.get()))
            || (mural[(cx) as usize][(cy) as usize] > 0)
        {
            clear = false;
            break;
        }
    }
    if clear
    {
        let mut bx: i32 = sx;
        let by: i32 = sy + (if plusY { downLine } else { -downLine });
        final_castle_mural::fillHorizontalLine(this, mural, bx, by, 1, plusX);
        bx += if plusX { 2 } else { -2 };
        final_castle_mural::fillVerticalLine(this, mural, bx, by, branchLength, plusY);
        if (((bx > 0) && (bx < this.m_width.get())) && (by > 0)) && (by < this.m_height.get())
        {
            if ((!final_castle_mural::makeVerticalBranch(this, mural, random, bx, by, branchLength, plusY, true))
                && (!final_castle_mural::makeVerticalBranch(this, mural, random, bx, by, branchLength, plusY, true)))
                && (!final_castle_mural::makeVerticalBranch(this, mural, random, bx, by, branchLength, plusY, true))
            {
                final_castle_mural::makeVerticalBranch(this, mural, random, bx, by, branchLength, plusY, true);
            }
            if ((!final_castle_mural::makeVerticalBranch(this, mural, random, bx, by, branchLength, plusY, false))
                && (!final_castle_mural::makeVerticalBranch(this, mural, random, bx, by, branchLength, plusY, false)))
                && (!final_castle_mural::makeVerticalBranch(this, mural, random, bx, by, branchLength, plusY, false))
            {
                final_castle_mural::makeVerticalBranch(this, mural, random, bx, by, branchLength, plusY, false);
            }
        }
        return true;
    }
    else
    {
        return false;
    }
}

// ComponentTFFinalCastleMural.fillHorizontalLine
pub fn fillHorizontalLine(this: &Piece, mural: &mut [Vec<i32>], sx: i32, sy: i32, length: i32, positive: bool)
{
    let mut x: i32 = sx;
    let y: i32 = sy;
    for _i in 0..=length
    {
        if (((x >= 0) && (x < this.m_width.get())) && (y >= 0)) && (y < this.m_height.get())
        {
            mural[(x) as usize][(y) as usize] = if positive { 1 } else { 4 };
            x += if positive { 1 } else { -1 };
        }
    }
}

// ComponentTFFinalCastleMural.fillVerticalLine
pub fn fillVerticalLine(this: &Piece, mural: &mut [Vec<i32>], sx: i32, sy: i32, length: i32, positive: bool)
{
    let x: i32 = sx;
    let mut y: i32 = sy;
    for _i in 0..=length
    {
        if (((x >= 0) && (x < this.m_width.get())) && (y >= 0)) && (y < this.m_height.get())
        {
            mural[(x) as usize][(y) as usize] = if positive { 2 } else { 3 };
            y += if positive { 1 } else { -1 };
        }
    }
}

// ComponentTFFinalCastleMural.makeStripes
pub fn makeStripes(this: &Piece, decoRNG: &Random, mural2: &mut [Vec<i32>])
{
    let mut y: i32 = this.m_height.get() - 2;
    while y > (this.m_height.get() / 3)
    {
        final_castle_mural::makeSingleStripe(this, mural2, y);
        y -= 2 + decoRNG.nextIntBounded(2);
    }
}

// ComponentTFFinalCastleMural.makeSingleStripe
pub fn makeSingleStripe(this: &Piece, mural2: &mut [Vec<i32>], y: i32)
{
    {
        let mut x: i32 = 0;
        while ((x < (this.m_width.get() - 2)) && (mural2[(x + 1) as usize][(y) as usize] == 0))
            && (mural2[(x + 1) as usize][(y + 1) as usize] == 0)
        {
            mural2[(x) as usize][(y) as usize] = 1;
            x += 1;
        }
    }
    {
        let mut x: i32 = this.m_width.get() - 1;
        while ((x > 2) && (mural2[(x - 1) as usize][(y) as usize] == 0))
            && (mural2[(x - 1) as usize][(y + 1) as usize] == 0)
        {
            mural2[(x) as usize][(y) as usize] = 1;
            x -= 1;
        }
    }
}
