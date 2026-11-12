pub(crate) mod cable;
pub(crate) mod net;
pub(crate) mod storage;

use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

pub(crate) const KIND: &str = "ic2:eu";

// what one EU is worth in the loader's shared base unit, 2.5 EU to BuildCraft's MJ of worth 10
const WORTH: i64 = 4;

pub(crate) fn kind() -> Identifier { Identifier::parse(KIND).expect("the energy kind is an identifier") }

// a cable's numbers, TileEntityCable by cable type
pub(crate) struct Conductor
{
    pub(crate) m_loss: f64,
    pub(crate) m_absorption: i32,
    pub(crate) m_breakdown: i32,
}

// IEnergyAcceptor, IEnergyEmitter, IEnergySink, IEnergySource and IEnergyConductor in one, faces are the side of
// this block that looks at the other block
pub(crate) trait EnergyTile
{
    fn accepts(&self, _world: World, _pos: BlockPos, _face: i32) -> bool { false }
    fn emits(&self, _world: World, _pos: BlockPos, _face: i32) -> bool { false }
    fn conductor(&self, _world: World, _pos: BlockPos) -> Option<&'static Conductor> { None }
    fn sink(&self) -> bool { false }
    fn demands(&self, _world: World, _pos: BlockPos) -> bool { false }
    // the amount left over, a packet above the tile's rating blows it up first
    fn inject(&self, _world: World, _pos: BlockPos, _face: i32, amount: i32) -> i32 { amount }
    // whether another block's cable joins it, TileEntityLuminator narrows this
    fn joins(&self, _world: World, _pos: BlockPos, _from: BlockPos) -> bool { true }
}

thread_local!
{
    // by block name without the namespace, the net asks for every target on every tick
    static TILES: RefCell<HashMap<String, &'static dyn EnergyTile>> = RefCell::new(HashMap::new());
}

// the blocks of this mod that take part in the energy net, each with its rules
pub(crate) fn add(block: &str, tile: &'static dyn EnergyTile)
{
    let name: &str =
        block.strip_prefix(crate::core::NAMESPACE).and_then(|rest| rest.strip_prefix(':')).unwrap_or(block);
    TILES.with(|tiles| tiles.borrow_mut().insert(name.to_owned(), tile));
}

// one cached name read and one map lookup, no matter how many IC2 blocks take part
pub(crate) fn tileAt(world: World, pos: BlockPos) -> Option<&'static dyn EnergyTile>
{
    nameWith(RegistryKind::Block, world.block(pos), |name|
    {
        let name: &Identifier = name.filter(|name| name.namespace() == crate::core::NAMESPACE)?;
        TILES.with(|tiles| tiles.borrow().get(name.path()).copied())
    })
}

// a block of another mod that takes EU through the loader's energy registry
pub(crate) fn foreignSink(world: World, pos: BlockPos, face: i32) -> bool
{
    tileAt(world, pos).is_none() && world.energyDemand(pos, face, &kind()).is_some_and(|demand| demand.max > 0)
}

// RenderBlockCable.canConnectCableWith and TileEntityCable.canInteractWith
pub(crate) fn connects(world: World, from: BlockPos, to: BlockPos, face: i32) -> bool
{
    match tileAt(world, to)
    {
        Some(tile) => tile.joins(world, to, from),
        None => world.capabilities(to, crate::core::dir::reverse(face)) & capability::ENERGY != 0,
    }
}

// TileEntityElecMachine and the storage blocks take EU from other mods' engines and cables, never above their rating
pub(crate) fn demand(max: i32) -> Option<EnergyDemand>
{
    Some(EnergyDemand { kind: kind(), min: 1, max: i64::from(max) })
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.energy().register("eu", Energy::new("EU", WORTH))?;
    net::register(ctx)?;
    cable::register(ctx)?;
    storage::register(ctx)
}
