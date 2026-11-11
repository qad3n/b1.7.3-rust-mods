use crate::blocks;
use b173_api::prelude::*;
use b173_api::world::World;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};

// nodes sort by x then y then z as RedstoneEtherNode.compareTo does
type Key = (i32, i32, i32);

#[derive(Default)]
struct Frequency
{
    m_transmitters: BTreeMap<Key, bool>,
    m_receivers: BTreeSet<Key>,
    m_state: bool,
}

impl Frequency
{
    fn count(&self) -> usize { self.m_transmitters.len() + self.m_receivers.len() }

    fn refresh(&mut self) { self.m_state = self.m_transmitters.values().any(|&on| on); }
}

#[derive(Default)]
struct Dimension
{
    m_frequencies: HashMap<i32, Frequency>,
    // the frequency each placed node joined, so a tile tick sees a changed or missing registration
    m_joined: HashMap<Key, i32>,
}

thread_local!
{
    static ETHER: RefCell<HashMap<i32, Dimension>> = RefCell::new(HashMap::new());
}

fn key(pos: BlockPos) -> Key { (pos.x, pos.y, pos.z) }

fn at(key: Key) -> BlockPos { BlockPos { x: key.0, y: key.1, z: key.2 } }

fn with<R>(world: World, action: impl FnOnce(&mut Dimension) -> R) -> R
{
    ETHER.with(|ether| action(ether.borrow_mut().entry(world.dimension).or_default()))
}

fn dropEmpty(dimension: &mut Dimension, freq: i32)
{
    if dimension.m_frequencies.get(&freq).is_some_and(|frequency| frequency.count() == 0)
    {
        dimension.m_frequencies.remove(&freq);
    }
}

// receivers to update after a frequency changed state, collected so no borrow is held while they run
fn receivers(world: World, freq: i32) -> Vec<Key>
{
    with(world, |dimension|
    {
        dimension.m_frequencies.get(&freq).map(|frequency| frequency.m_receivers.iter().copied().collect())
    })
    .unwrap_or_default()
}

fn updateReceivers(world: World, freq: i32)
{
    for receiver in receivers(world, freq)
    {
        blocks::updateReceiver(world, at(receiver));
    }
}

pub(crate) fn joined(world: World, pos: BlockPos) -> Option<i32>
{
    with(world, |dimension| dimension.m_joined.get(&key(pos)).copied())
}

pub(crate) fn addTransmitter(world: World, pos: BlockPos, freq: i32)
{
    with(world, |dimension|
    {
        dimension.m_frequencies.entry(freq).or_default().m_transmitters.insert(key(pos), false);
        dimension.m_joined.insert(key(pos), freq);
    });
}

pub(crate) fn removeTransmitter(world: World, pos: BlockPos, freq: i32)
{
    let removed: bool = with(world, |dimension|
    {
        if dimension.m_joined.get(&key(pos)) == Some(&freq)
        {
            dimension.m_joined.remove(&key(pos));
        }

        let Some(frequency) = dimension.m_frequencies.get_mut(&freq)
        else
        {
            return false;
        };

        let removed: bool = frequency.m_transmitters.remove(&key(pos)).is_some();
        frequency.refresh();
        dropEmpty(dimension, freq);
        removed
    });

    if removed
    {
        updateReceivers(world, freq);
    }
}

pub(crate) fn addReceiver(world: World, pos: BlockPos, freq: i32)
{
    with(world, |dimension|
    {
        dimension.m_frequencies.entry(freq).or_default().m_receivers.insert(key(pos));
        dimension.m_joined.insert(key(pos), freq);
    });
}

pub(crate) fn removeReceiver(world: World, pos: BlockPos, freq: i32)
{
    with(world, |dimension|
    {
        if dimension.m_joined.get(&key(pos)) == Some(&freq)
        {
            dimension.m_joined.remove(&key(pos));
        }

        if let Some(frequency) = dimension.m_frequencies.get_mut(&freq)
        {
            frequency.m_receivers.remove(&key(pos));
        }

        dropEmpty(dimension, freq);
    });
}

pub(crate) fn setTransmitterState(world: World, pos: BlockPos, freq: i32, state: bool)
{
    let changed: bool = with(world, |dimension|
    {
        let Some(frequency) = dimension.m_frequencies.get_mut(&freq)
        else
        {
            return false;
        };

        let Some(node) = frequency.m_transmitters.get_mut(&key(pos))
        else
        {
            return false;
        };

        *node = state;

        if state
        {
            frequency.m_state = true;
        }
        else
        {
            frequency.refresh();
        }

        true
    });

    if changed
    {
        updateReceivers(world, freq);
    }
}

pub(crate) fn state(world: World, freq: i32) -> bool
{
    with(world, |dimension| dimension.m_frequencies.get(&freq).is_some_and(|frequency| frequency.m_state))
}

pub(crate) fn activeFrequencies(world: World) -> Vec<u16>
{
    with(world, |dimension|
    {
        dimension
            .m_frequencies
            .iter()
            .filter(|(_, frequency)| frequency.m_state)
            .filter_map(|(&freq, _)| u16::try_from(freq).ok())
            .collect()
    })
}

// the source kept the first node's distance while replacing the position, so it never found the closest one
pub(crate) fn closestActive(world: World, x: i32, z: i32, freq: i32) -> Option<[i32; 2]>
{
    with(world, |dimension|
    {
        let frequency: &Frequency = dimension.m_frequencies.get(&freq)?;
        let mut closest: Option<([i32; 2], f32)> = None;

        for (&(nodeX, _, nodeZ), _) in frequency.m_transmitters.iter().filter(|(_, on)| **on)
        {
            let distance: f32 = ((nodeX - x) as f64).hypot((nodeZ - z) as f64) as f32;
            let nearer: bool = closest.is_none_or(|(_, best)| distance < best);

            if nearer
            {
                closest = Some(([nodeX, nodeZ], distance));
            }
        }

        closest.map(|(pos, _)| pos)
    })
}

fn inChunk(key: Key, chunkX: i32, chunkZ: i32) -> bool { key.0 >> 4 == chunkX && key.2 >> 4 == chunkZ }

// receivers of an unloaded chunk leave so updates never load it, their tiles join again when it loads
pub(crate) fn unloadChunk(world: World, chunkX: i32, chunkZ: i32)
{
    with(world, |dimension|
    {
        for frequency in dimension.m_frequencies.values_mut()
        {
            let leaving: Vec<Key> =
                frequency.m_receivers.iter().copied().filter(|key| inChunk(*key, chunkX, chunkZ)).collect();

            for receiver in leaving
            {
                frequency.m_receivers.remove(&receiver);
                dimension.m_joined.remove(&receiver);
            }
        }

        dimension.m_frequencies.retain(|_, frequency| frequency.count() > 0);
    });
}

pub(crate) fn clear(world: World) { ETHER.with(|ether| ether.borrow_mut().remove(&world.dimension)); }
