// draft from ComponentTFMazeRuins.java by tools/structgen.py
use crate::world::structures::prelude::*;

pub fn new(world: &GenWorld, random: &Random, i: i32, x: i32, y: i32, z: i32) -> Rc<Piece>
{
    let this: Piece = Piece::blank(Kind::MazeRuins);
    init(&this, world, random, i, x, y, z);
    Rc::new(this)
}

// ComponentTFMazeRuins constructor
pub fn init(this: &Piece, _world: &GenWorld, _random: &Random, i: i32, x: i32, y: i32, z: i32)
{
    this.m_depth.set(i);
    this.setMode(0);
    this.m_box.set(boxFor(x, y, z, 0, 0, 0, 0, 0, 0, 0));
}

// ComponentTFMazeRuins.func_74861_a
pub fn build(this: &Piece, _structurecomponent: &Piece, list: &mut Pieces, random: &Random)
{
    let maze: Rc<Piece> =
        minotaur_maze::new(1, this.m_box.get().m_min[0], this.m_box.get().m_min[1] - 14, this.m_box.get().m_min[2], 1);
    list.push(Rc::clone(&maze));
    maze.build(this, list, random);
    let mazeEnter: Rc<Piece> = maze_entrance_shaft::new(
        2,
        random,
        this.m_box.get().m_min[0] + 1,
        this.m_box.get().m_min[1],
        this.m_box.get().m_min[2] + 1,
    );
    list.push(Rc::clone(&mazeEnter));
    mazeEnter.build(this, list, random);
    let mazeAbove: Rc<Piece> = maze_mound::new(
        2,
        random,
        this.m_box.get().m_min[0] - 14,
        this.m_box.get().m_min[1],
        this.m_box.get().m_min[2] - 14,
    );
    list.push(Rc::clone(&mazeAbove));
    mazeAbove.build(this, list, random);
}

// ComponentTFMazeRuins.func_74875_a
pub fn add(_this: &Piece, _world: &GenWorld, _random: &Random, _sbb: StructureBox) -> bool { return true; }
