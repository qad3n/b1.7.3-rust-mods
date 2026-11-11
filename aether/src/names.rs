use b173_api::{BlockCtx, BlockPos, BlockStatesSpec, Identifier, PropertySpec, StateData, StateHandle, World};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// registry names, shared by registration, generation and behaviors
pub(crate) mod block
{
    pub(crate) const AETHER_GRASS: &str = "aether_grass";
    pub(crate) const ENCHANTED_AETHER_GRASS: &str = "enchanted_aether_grass";
    pub(crate) const AETHER_DIRT: &str = "aether_dirt";
    pub(crate) const HOLYSTONE: &str = "holystone";
    pub(crate) const QUICKSOIL: &str = "quicksoil";
    pub(crate) const ICESTONE: &str = "icestone";
    pub(crate) const AMBROSIUM_ORE: &str = "ambrosium_ore";
    pub(crate) const ZANITE_ORE: &str = "zanite_ore";
    pub(crate) const GRAVITITE_ORE: &str = "gravitite_ore";
    pub(crate) const ENCHANTED_GRAVITITE: &str = "enchanted_gravitite";
    pub(crate) const COLD_AERCLOUD: &str = "cold_aercloud";
    pub(crate) const BLUE_AERCLOUD: &str = "blue_aercloud";
    pub(crate) const GOLDEN_AERCLOUD: &str = "golden_aercloud";
    pub(crate) const SKYROOT_LOG: &str = "skyroot_log";
    pub(crate) const GOLDEN_OAK_LOG: &str = "golden_oak_log";
    pub(crate) const SKYROOT_LEAVES: &str = "skyroot_leaves";
    pub(crate) const GOLDEN_OAK_LEAVES: &str = "golden_oak_leaves";
    pub(crate) const CRYSTAL_LEAVES: &str = "crystal_leaves";
    pub(crate) const CRYSTAL_FRUIT_LEAVES: &str = "crystal_fruit_leaves";
    pub(crate) const SKYROOT_SAPLING: &str = "skyroot_sapling";
    pub(crate) const GOLDEN_OAK_SAPLING: &str = "golden_oak_sapling";
    pub(crate) const WHITE_FLOWER: &str = "white_flower";
    pub(crate) const PURPLE_FLOWER: &str = "purple_flower";
    pub(crate) const BERRY_BUSH: &str = "berry_bush";
    pub(crate) const AETHER_PORTAL: &str = "aether_portal";
}

pub(crate) mod item
{
    pub(crate) const AMBROSIUM_SHARD: &str = "ambrosium_shard";
    pub(crate) const ZANITE_GEMSTONE: &str = "zanite_gemstone";
    pub(crate) const GOLDEN_AMBER: &str = "golden_amber";
    pub(crate) const SKYROOT_STICK: &str = "skyroot_stick";
    pub(crate) const BLUEBERRY: &str = "blueberry";
    pub(crate) const WHITE_APPLE: &str = "white_apple";
    pub(crate) const SWET_BALL: &str = "swet_ball";
    pub(crate) const AECHOR_PETAL: &str = "aechor_petal";
    pub(crate) const VICTORY_MEDAL: &str = "victory_medal";
}

// a block by name with chosen named properties, its default state when none are listed
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Pick
{
    pub m_block: &'static str,
    pub m_state: &'static [(&'static str, &'static str)],
}

impl Pick
{
    pub(crate) const fn of(block: &'static str) -> Self { Self { m_block: block, m_state: &[] } }

    pub(crate) const fn with(block: &'static str, state: &'static [(&'static str, &'static str)]) -> Self
    {
        Self { m_block: block, m_state: state }
    }

    fn build(self) -> StateData
    {
        let block: Identifier =
            Identifier::parseIn(self.m_block, "aether").expect("block names in this mod are written valid");
        self.m_state.iter().fold(StateData::of(block), |state, (property, value)| state.with(property, value))
    }
}

pub(crate) const AIR: Pick = Pick::of("minecraft:air");

thread_local!
{
    // a handle resolves again after every remap so one per pick serves every call
    static HANDLES: RefCell<HashMap<Pick, Rc<StateHandle>>> = RefCell::new(HashMap::new());
    // states read from the world, few distinct ones so a scan beats hashing their strings
    static READ: RefCell<Vec<Rc<StateHandle>>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn handle(pick: Pick) -> Rc<StateHandle>
{
    HANDLES.with(|cache|
    {
        Rc::clone(cache.borrow_mut().entry(pick).or_insert_with(|| Rc::new(StateHandle::new(pick.build()))))
    })
}

// the id a block's default state is stored as, for engine fields that hold a numeric block
pub(crate) fn numeric(name: &'static str) -> i32 { handle(Pick::of(name)).block() }

// a true or false property, false first so metadata 0 stays the plain block
pub(crate) fn flag(name: &str) -> PropertySpec { PropertySpec::new(name, &["false", "true"]) }

// properties in metadata order, the last one changing fastest
pub(crate) fn states(properties: Vec<PropertySpec>) -> BlockStatesSpec
{
    BlockStatesSpec { properties, ..BlockStatesSpec::default() }
}

// the block's state with one property changed, written where it stands
pub(crate) fn setProperty(ctx: &BlockCtx, property: &str, value: impl ToString)
{
    let Some(state) = ctx.stateAt()
    else
    {
        return;
    };

    if let Err(error) = ctx.setState(StateData::clone(&state).with(property, value))
    {
        b173_api::warn(&format!("could not set {property} on {}: {}", state.block, error.0));
    }
}

// the spot holds exactly this state
pub(crate) fn holds(world: World, pos: BlockPos, pick: Pick) -> bool
{
    handle(pick).matches(world.block(pos), world.meta(pos))
}

// a handle for a state read from the world or saved data, shared by every caller holding an equal state
pub(crate) fn handleOf(state: &StateData) -> Rc<StateHandle>
{
    READ.with(|read|
    {
        let mut read = read.borrow_mut();
        if let Some(found) = read.iter().find(|handle| handle.state() == state)
        {
            return Rc::clone(found);
        }

        let made: Rc<StateHandle> = Rc::new(StateHandle::new(state.clone()));
        read.push(Rc::clone(&made));
        made
    })
}
