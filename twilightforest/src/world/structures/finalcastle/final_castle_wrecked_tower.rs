// draft from ComponentTFFinalCastleWreckedTower.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleWreckedTower);
    init(&this, random, i, x, y, z, direction);
    Rc::new(this)
}

// ComponentTFFinalCastleWreckedTower constructor
pub(crate) fn init(this: &Piece, random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32)
{
    final_castle_damaged_tower::init(this, random, i, x, y, z, direction);
}

// ComponentTFFinalCastleWreckedTower.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());

    let thorns: Rc<Piece> = final_castle_foundation13_thorns::new(random, 0, this);
    list.push(Rc::clone(&thorns));
    thorns.build(this, list, random);
}

// ComponentTFFinalCastleWreckedTower.getGlyphMeta
pub(crate) fn getGlyphMeta(_this: &Piece) -> i32 { 1 }

// ComponentTFFinalCastleWreckedTower.determineBlockDestroyed
pub(crate) fn determineBlockDestroyed(
    _this: &Piece,
    world: &GenWorld,
    areas: &[Option<DestroyArea>],
    y: i32,
    x: i32,
    z: i32,
)
{
    let mut isInside: bool = false;

    for dArea in areas.iter().flatten()
    {
        if dArea.vecInside(x, y, z)
        {
            isInside = true;
        }
    }

    if !isInside
    {
        world.clear(x, y, z);
    }
}

// ComponentTFFinalCastleWreckedTower.makeInitialDestroyList
pub(crate) fn makeInitialDestroyList(this: &Piece, random: &Random) -> Vec<Option<DestroyArea>>
{
    let mut areas: Vec<Option<DestroyArea>> = Vec::new();
    let area: Option<DestroyArea> =
        destroy_area::createNonIntersecting(this.bounds(), random, this.bounds().m_max[1] - 1, &areas);
    areas.push(area);

    let area: Option<DestroyArea> =
        destroy_area::createNonIntersecting(this.bounds(), random, this.bounds().m_max[1] - 1, &areas);
    areas.push(area);

    let area: Option<DestroyArea> =
        destroy_area::createNonIntersecting(this.bounds(), random, this.bounds().m_max[1] - 1, &areas);
    areas.push(area);

    let area: Option<DestroyArea> =
        destroy_area::createNonIntersecting(this.bounds(), random, this.bounds().m_max[1] - 1, &areas);
    areas.push(area);
    areas
}
