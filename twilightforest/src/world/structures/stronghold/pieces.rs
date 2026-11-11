use crate::world::structures::prelude::*;
use std::cell::RefMut;

// TFStrongholdPieceWeight, how often a lower stronghold piece is picked and how many it may make
#[derive(Clone, Copy)]
struct Weight
{
    m_kind: Kind,
    m_weight: i32,
    m_limit: i32,
    m_depth: i32,
    m_spawned: i32,
}

const fn weight(kind: Kind, weight: i32, limit: i32, depth: i32) -> Weight
{
    Weight { m_kind: kind, m_weight: weight, m_limit: limit, m_depth: depth, m_spawned: 0 }
}

const WEIGHTS: [Weight; 13] = [
    weight(Kind::StrongholdSmallHallway, 40, 0, 0),
    weight(Kind::StrongholdLeftTurn, 20, 0, 0),
    weight(Kind::StrongholdCrossing, 10, 4, 0),
    weight(Kind::StrongholdRightTurn, 20, 0, 0),
    weight(Kind::StrongholdDeadEnd, 5, 0, 0),
    weight(Kind::StrongholdBalconyRoom, 10, 3, 2),
    weight(Kind::StrongholdTrainingRoom, 10, 2, 0),
    weight(Kind::StrongholdSmallStairs, 10, 0, 0),
    weight(Kind::StrongholdTreasureCorridor, 5, 0, 0),
    weight(Kind::StrongholdAtrium, 5, 2, 3),
    weight(Kind::StrongholdFoundry, 5, 1, 4),
    weight(Kind::StrongholdTreasureRoom, 5, 1, 4),
    weight(Kind::StrongholdBossRoom, 10, 1, 4),
];

// TFStrongholdPieces.getNextComponent tries this many weighted picks before a dead end
const TRIES: i32 = 5;

impl Weight
{
    fn more(&self) -> bool { self.m_limit == 0 || self.m_spawned < self.m_limit }

    fn deepEnough(&self, depth: i32) -> bool { self.more() && depth >= self.m_depth }
}

thread_local!
{
    // TFStrongholdPieces.lastPieceMade is static in the source, shared by every stronghold
    static LAST: Cell<Option<Kind>> = const { Cell::new(None) };
}

// TFStrongholdPieces, the lower pieces the stronghold entrance still has to hand out
#[derive(Default)]
pub(crate) struct StrongholdPieces
{
    m_list: RefCell<Vec<Weight>>,
}

impl StrongholdPieces
{
    pub(crate) fn prepareStructurePieces(&self) { *self.m_list.borrow_mut() = WEIGHTS.to_vec(); }

    pub(crate) fn markBossRoomUsed(&self) { self.m_list.borrow_mut().pop(); }

    pub(crate) fn getNextComponent(
        &self,
        list: &Pieces,
        random: &Random,
        index: i32,
        facing: i32,
        x: i32,
        y: i32,
        z: i32,
    ) -> Option<Rc<Piece>>
    {
        let limited: bool =
            self.m_list.borrow().iter().any(|piece| piece.m_limit > 0 && piece.m_spawned < piece.m_limit);

        if !limited
        {
            return None;
        }

        let total: i32 = self.m_list.borrow().iter().map(|piece| piece.m_weight).sum();

        for _ in 0..TRIES
        {
            let mut counter: i32 = random.nextIntBounded(total);
            let pieces: Vec<Weight> = self.m_list.borrow().clone();
            for (slot, piece) in pieces.iter().enumerate()
            {
                counter -= piece.m_weight;
                if counter >= 0
                {
                    continue;
                }

                if !piece.deepEnough(index) || LAST.with(Cell::get) == Some(piece.m_kind)
                {
                    break;
                }

                let component: Rc<Piece> = make(piece.m_kind, index, facing, x, y, z);
                if intersecting(list, component.bounds()).is_none()
                {
                    let mut chosen: RefMut<Vec<Weight>> = self.m_list.borrow_mut();
                    chosen[slot].m_spawned += 1;

                    if !chosen[slot].more()
                    {
                        chosen.remove(slot);
                    }

                    LAST.with(|last| last.set(Some(piece.m_kind)));
                    return Some(component);
                }
            }
        }

        let deadEnd: Rc<Piece> = stronghold_dead_end::new(index, facing, x, y, z);
        if intersecting(list, deadEnd.bounds()).is_none()
        {
            Some(deadEnd)
        }
        else
        {
            None
        }
    }
}

fn make(kind: Kind, index: i32, facing: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let new: fn(i32, i32, i32, i32, i32) -> Rc<Piece> = match kind
    {
        Kind::StrongholdSmallHallway => stronghold_small_hallway::new,
        Kind::StrongholdLeftTurn => stronghold_left_turn::new,
        Kind::StrongholdCrossing => stronghold_crossing::new,
        Kind::StrongholdRightTurn => stronghold_right_turn::new,
        Kind::StrongholdBalconyRoom => stronghold_balcony_room::new,
        Kind::StrongholdTrainingRoom => stronghold_training_room::new,
        Kind::StrongholdSmallStairs => stronghold_small_stairs::new,
        Kind::StrongholdTreasureCorridor => stronghold_treasure_corridor::new,
        Kind::StrongholdAtrium => stronghold_atrium::new,
        Kind::StrongholdFoundry => stronghold_foundry::new,
        Kind::StrongholdTreasureRoom => stronghold_treasure_room::new,
        Kind::StrongholdBossRoom => stronghold_boss_room::new,
        _ => stronghold_dead_end::new,
    };
    new(index, facing, x, y, z)
}
