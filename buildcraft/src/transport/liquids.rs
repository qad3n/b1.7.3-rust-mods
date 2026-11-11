use super::{Kind, joins, kindAt};
use crate::core::dir;
use b173_api::prelude::*;

// TilePipe's liquid flow: each face holds liquid going to and from the center, the center holds liquid in and out
pub(crate) const FLOW_RATE: i32 = 20;
pub(crate) const SIDE_CAPACITY: i32 = 250;
pub(crate) const CENTER_CAPACITY: i32 = 500;
// BuildCraft 2.1.1 pipes carry one liquid at a time, oil unless another mod's fluid comes in first
pub(crate) const OIL: &str = "buildcraft:oil";

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub(crate) struct Liquid
{
    pub(crate) m_fluid: Option<Identifier>,
    pub(crate) m_sideToCenter: [i32; 6],
    pub(crate) m_centerToSide: [i32; 6],
    pub(crate) m_centerIn: i32,
    pub(crate) m_centerOut: i32,
    pub(crate) m_input: [bool; 6],
    pub(crate) m_lastFrom: i32,
    pub(crate) m_lastTo: i32,
}

impl Liquid
{
    pub(crate) fn new() -> Self { Liquid { m_lastFrom: dir::EAST, m_lastTo: dir::EAST, ..Liquid::default() } }

    fn empty(&self) -> bool
    {
        self.m_centerIn + self.m_centerOut == 0
            && self.m_sideToCenter.iter().chain(&self.m_centerToSide).all(|amount| *amount == 0)
    }

    // TilePipe.fill and the iron pipe that takes nothing from its output face
    pub(crate) fn fill(
        &mut self,
        kind: Kind,
        world: World,
        pos: BlockPos,
        face: i32,
        stack: &FluidStack,
        simulate: bool,
    ) -> i32
    {
        if kind == Kind::Iron && face == world.meta(pos)
        {
            return 0;
        }

        if self.m_fluid.as_ref().is_some_and(|fluid| *fluid != stack.fluid) && !self.empty()
        {
            return 0;
        }

        let side: usize = face as usize;
        let space: i32 = SIDE_CAPACITY - self.m_sideToCenter[side] - self.m_centerToSide[side] + FLOW_RATE;
        if simulate
        {
            return space.clamp(0, stack.amount);
        }

        self.m_input[side] = true;
        if space <= 0
        {
            return 0;
        }

        self.m_fluid = Some(stack.fluid.clone());
        let taken: i32 = space.min(stack.amount);
        self.m_sideToCenter[side] += taken;
        taken
    }

    // TilePipe.canReceiveLiquid, the iron pipe only lets liquid out of its output face
    fn receives(&self, kind: Kind, world: World, pos: BlockPos, face: i32) -> bool
    {
        if kind == Kind::Iron && face != world.meta(pos)
        {
            return false;
        }

        if self.m_input[face as usize] || !joins(world, pos, face)
        {
            return false;
        }

        let target: BlockPos = dir::step(pos, face);
        kindAt(world, target).is_some() || world.capabilities(target, dir::reverse(face)) & capability::FLUIDS != 0
    }

    // TilePipe.moveLiquids, one tick of flow from inputs through the center to the outputs
    pub(crate) fn tick(&mut self, kind: Kind, world: World, pos: BlockPos)
    {
        let mut moved: bool = self.intoCenter();

        if self.m_centerIn + self.m_centerOut >= CENTER_CAPACITY
        {
            self.m_centerOut += self.m_centerIn;
            self.m_centerIn = 0;
        }

        for face in dir::all()
        {
            if self.receives(kind, world, pos, face)
            {
                moved |= self.outOf(world, pos, face);
            }
        }

        if moved || dir::all().any(|face| self.receives(kind, world, pos, face))
        {
            return;
        }

        self.m_input = [false; 6];

        if self.empty()
        {
            self.m_fluid = None;
        }
    }

    fn intoCenter(&mut self) -> bool
    {
        let centerSpace: i32 = CENTER_CAPACITY - self.m_centerIn - self.m_centerOut + FLOW_RATE;
        let mut moved: bool = false;

        for side in 0..dir::COUNT as usize
        {
            if !self.m_input[side]
            {
                continue;
            }

            if self.m_centerToSide[side] > 0 && centerSpace >= FLOW_RATE
            {
                self.m_lastFrom = side as i32;
                self.m_centerToSide[side] -= FLOW_RATE;
                self.m_centerIn += FLOW_RATE;
                moved = true;
            }

            if self.m_sideToCenter[side] + self.m_centerToSide[side] >= SIDE_CAPACITY
            {
                self.m_centerToSide[side] += self.m_sideToCenter[side];
                self.m_sideToCenter[side] = 0;
            }
        }

        moved
    }

    fn outOf(&mut self, world: World, pos: BlockPos, face: i32) -> bool
    {
        let side: usize = face as usize;
        let mut moved: bool = false;

        if self.m_sideToCenter[side] > 0
        {
            let fluid: Identifier =
                self.m_fluid.clone().unwrap_or_else(|| Identifier::parse(OIL).expect("oil is an identifier"));
            let offered: FluidStack = FluidStack { fluid, amount: FLOW_RATE };
            self.m_sideToCenter[side] -= world.fillFluid(dir::step(pos, face), dir::reverse(face), &offered, false);
            moved = true;
        }

        if self.m_centerOut > 0 && self.m_sideToCenter[side] + self.m_centerToSide[side] <= SIDE_CAPACITY
        {
            self.m_lastTo = face;
            self.m_centerToSide[side] += FLOW_RATE;
            self.m_centerOut -= FLOW_RATE;
            moved = true;
        }

        if self.m_centerToSide[side] + self.m_sideToCenter[side] >= SIDE_CAPACITY
        {
            self.m_sideToCenter[side] += self.m_centerToSide[side];
            self.m_centerToSide[side] = 0;
        }

        moved
    }

    // TilePipe.neighborChange, a face that can no longer pass liquid loses what it held
    pub(crate) fn neighborChanged(&mut self, kind: Kind, world: World, pos: BlockPos)
    {
        for face in dir::all()
        {
            if !self.receives(kind, world, pos, face)
            {
                self.m_centerToSide[face as usize] = 0;
                self.m_sideToCenter[face as usize] = 0;
            }
        }
    }
}
