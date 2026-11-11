use b173_api::prelude::*;

// buildcraft.core.Box, an inclusive block box; a missing box is None rather than Integer.MAX_VALUE corners
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Area
{
    pub(crate) m_min: [i32; 3],
    pub(crate) m_max: [i32; 3],
}

impl Area
{
    pub(crate) fn new(min: [i32; 3], max: [i32; 3]) -> Self { Area { m_min: min, m_max: max } }

    pub(crate) fn size(&self, axis: usize) -> i32 { self.m_max[axis] - self.m_min[axis] + 1 }

    pub(crate) fn min(&self) -> [f64; 3] { self.m_min.map(f64::from) }

    pub(crate) fn max(&self) -> [f64; 3] { self.m_max.map(f64::from) }
}
