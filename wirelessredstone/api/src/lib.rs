// names and message types other mods use to read the wireless ether, answered on the server
#![no_std]

use serde::{Deserialize, Serialize};

pub const SERVICE: &str = "wirelessredstone:ether";
pub const LOOKUP: &str = "wirelessredstone:node";
pub const VERSION: u32 = 1;
pub const FREQUENCIES: i32 = 10000;

pub mod methods
{
    // service, Query in and bool out
    pub const STATE: &str = "state";
    // service, the dimension in and every powered frequency out as Vec<u16>
    pub const ACTIVE: &str = "active";
    // service, Near in and the closest powered transmitter as Option<[i32; 2]> out
    pub const CLOSEST: &str = "closest";
    // lookup at a receiver, transmitter or clocker, the i32 frequency out
    pub const FREQUENCY: &str = "frequency";
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Query
{
    pub world: i32,
    pub freq: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Near
{
    pub world: i32,
    pub x: i32,
    pub z: i32,
    pub freq: i32,
}
