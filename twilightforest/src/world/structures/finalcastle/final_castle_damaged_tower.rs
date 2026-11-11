// draft from ComponentTFFinalCastleDamagedTower.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub(crate) fn new(random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::FinalCastleDamagedTower);
    init(&this, random, i, x, y, z, direction);
    Rc::new(this)
}

// ComponentTFFinalCastleDamagedTower constructor
pub(crate) fn init(this: &Piece, random: &Random, i: i32, x: i32, y: i32, z: i32, direction: i32)
{
    final_castle_maze_tower13::init(this, random, i, x, y, z, 2, direction);
}

// ComponentTFFinalCastleDamagedTower.func_74861_a
pub(crate) fn build(this: &Piece, parent: &Piece, list: &mut Pieces, random: &Random)
{
    this.m_deco.set((parent).m_deco.get());

    let foundation: Rc<Piece> = final_castle_foundation13::new(random, 0, this);
    list.push(Rc::clone(&foundation));
    foundation.build(this, list, random);

    let thorns: Rc<Piece> = final_castle_foundation13_thorns::new(random, 0, this);
    list.push(Rc::clone(&thorns));
    thorns.build(this, list, random);
    final_castle_maze_tower13::buildNonCriticalTowers(this, parent, list, random);
}

// ComponentTFFinalCastleDamagedTower.makeNewDamagedTower
pub(crate) fn makeNewDamagedTower(this: &Piece, random: &Random, direction: i32, tc: [i32; 3]) -> Rc<Piece>
{
    final_castle_wrecked_tower::new(random, this.depth() + 1, tc[0], tc[1], tc[2], direction)
}

// ComponentTFFinalCastleDamagedTower.func_74875_a
pub(crate) fn add(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox) -> bool
{
    final_castle_maze_tower13::add(this, world, random, sbb);

    let decoRNG: Random = Random::withSeed(
        (world.seed() + ((this.m_box.get().m_min[0] * 321534781) as i64))
            ^ ((this.m_box.get().m_min[2] * 756839) as i64),
    );
    final_castle_damaged_tower::destroyTower(this, world, &decoRNG, sbb);
    true
}

// ComponentTFFinalCastleDamagedTower.destroyTower
pub(crate) fn destroyTower(this: &Piece, world: &GenWorld, random: &Random, sbb: StructureBox)
{
    let mut areas: Vec<Option<DestroyArea>> = this.makeInitialDestroyList(random);

    let mut hitDeadRock: bool = false;
    {
        let mut y: i32 = this.m_box.get().m_max[1];
        while (!hitDeadRock) && (y > 64)
        {
            for x in this.m_box.get().m_min[0] - 2..=(this.m_box.get().m_max[0] + 2)
            {
                for z in this.m_box.get().m_min[2] - 2..=(this.m_box.get().m_max[2] + 2)
                {
                    if sbb.contains(x, y, z)
                    {
                        if world.getBlock(x, y, z) == blk("tf:deadrock")
                        {
                            hitDeadRock = true;
                        }

                        this.determineBlockDestroyed(world, &areas, y, x, z);
                    }
                }
            }

            let mut removeArea: Option<DestroyArea> = None;

            for dArea in areas.clone()
            {
                if dArea.is_none_or(|area| area.entirelyAbove(y))
                {
                    removeArea = dArea;
                }
            }

            if let Some(index) = removeArea.and_then(|remove| areas.iter().position(|area| *area == Some(remove)))
            {
                areas.remove(index);

                let replacement: Option<DestroyArea> =
                    destroy_area::createNonIntersecting(this.bounds(), random, y, &areas);
                areas.push(replacement);
            }

            y -= 1;
        }
    }
}

// ComponentTFFinalCastleDamagedTower.makeInitialDestroyList
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
    areas
}

// ComponentTFFinalCastleDamagedTower.determineBlockDestroyed
pub(crate) fn determineBlockDestroyed(
    _this: &Piece,
    world: &GenWorld,
    areas: &[Option<DestroyArea>],
    y: i32,
    x: i32,
    z: i32,
)
{
    for dArea in areas.iter().flatten()
    {
        if dArea.vecInside(x, y, z)
        {
            world.clear(x, y, z);
        }
    }
}
