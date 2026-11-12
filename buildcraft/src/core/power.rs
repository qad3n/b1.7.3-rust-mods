use b173_api::prelude::*;

// a machine's PowerProvider.configure and configurePowerPerdition numbers, one row per machine
pub(crate) struct Setup
{
    pub(crate) m_latency: i32,
    pub(crate) m_minReceived: i32,
    pub(crate) m_maxReceived: i32,
    pub(crate) m_minActivation: i32,
    pub(crate) m_maxStored: i32,
    pub(crate) m_loss: Option<(i32, i32)>,
}

impl Setup
{
    pub(crate) fn provider(&self) -> Provider
    {
        let provider: Provider = Provider::new(
            self.m_latency,
            self.m_minReceived,
            self.m_maxReceived,
            self.m_minActivation,
            self.m_maxStored,
        );
        match self.m_loss
        {
            Some((loss, regularity)) => provider.perdition(loss, regularity),
            None => provider,
        }
    }
}

// BuildCraft's minecraft joule, worth ten of the loader's shared base units so other mods' energy converts to it
pub(crate) const MJ: &str = "buildcraft:mj";
const MJ_NAME: &str = "mj";
const MJ_WORTH: i64 = 10;

pub(crate) const FRAMEWORK: &str = "power.framework";
pub(crate) const CONTINUOUS: &str = "current.continuous";
pub(crate) const PNEUMATIC: &str = "pneumatic";
pub(crate) const REDSTONE: &str = "redstone";

// PowerProvider's own perdition before a machine sets one
const DEFAULT_LOSS: i32 = 1;
const DEFAULT_LOSS_REGULARITY: i32 = 100;

pub(crate) fn kind() -> Identifier { Identifier::parse(MJ).expect("the mj kind is a valid identifier") }

pub(crate) fn redstone() -> bool { configValue(FRAMEWORK) == REDSTONE }

// SafeTimeTracker, a mark in world time that fires once the delay has passed
#[derive(Serialize, Deserialize, Clone, Copy, Default, PartialEq, Debug)]
pub(crate) struct Tracker
{
    m_last: i64,
}

impl Tracker
{
    pub(crate) fn due(&mut self, now: i64, delay: i64) -> bool
    {
        if now < self.m_last
        {
            self.m_last = now;
            return false;
        }

        if self.m_last + delay > now
        {
            return false;
        }

        self.m_last = now;
        true
    }

    pub(crate) fn mark(&mut self, now: i64) { self.m_last = now; }
}

// buildcraft.api.PowerProvider with its two frameworks: pneumatic takes energy from engines, redstone works on a
// redstone pulse instead
#[derive(Serialize, Deserialize, Clone, Copy, Default, PartialEq, Debug)]
pub(crate) struct Provider
{
    pub(crate) m_latency: i32,
    pub(crate) m_minReceived: i32,
    pub(crate) m_maxReceived: i32,
    pub(crate) m_maxStored: i32,
    pub(crate) m_minActivation: i32,
    pub(crate) m_stored: i32,
    m_loss: i32,
    m_lossRegularity: i32,
    pub(crate) m_work: Tracker,
    m_lossTracker: Tracker,
    m_lastPower: bool,
    m_redstone: bool,
}

impl Provider
{
    // PowerProvider.configure, as the frameworks override it
    pub(crate) fn new(latency: i32, minReceived: i32, maxReceived: i32, minActivation: i32, maxStored: i32) -> Self
    {
        let mut provider: Provider = Provider
        {
            m_loss: DEFAULT_LOSS,
            m_lossRegularity: DEFAULT_LOSS_REGULARITY,
            m_redstone: redstone(),
            ..Provider::default()
        };

        if provider.m_redstone
        {
            provider.m_loss = 0;
            provider.m_lossRegularity = 0;
        }

        provider.configure(latency, minReceived, maxReceived, minActivation, maxStored);
        provider
    }

    pub(crate) fn configure(
        &mut self,
        latency: i32,
        minReceived: i32,
        maxReceived: i32,
        minActivation: i32,
        maxStored: i32,
    )
    {
        self.m_latency = latency;
        self.m_minReceived = minReceived;
        self.m_maxReceived = maxReceived;
        self.m_maxStored = maxStored;
        self.m_minActivation = minActivation;

        if self.m_redstone
        {
            self.m_minActivation = 0;
            self.m_stored = 1;
        }
        else
        {
            self.m_latency = 0;
        }
    }

    // PowerProvider.configurePowerPerdition, which the redstone framework ignores
    pub(crate) fn perdition(mut self, loss: i32, regularity: i32) -> Self
    {
        if !self.m_redstone
        {
            self.m_loss = loss;
            self.m_lossRegularity = regularity;
        }

        self
    }

    // RedstonePowerProvider.preConditions, a rising edge or any power in the continuous model
    fn ready(&mut self, world: World, pos: BlockPos) -> bool
    {
        if !self.m_redstone
        {
            return true;
        }

        let powered: bool = world.isPowered(pos);
        if configValue(CONTINUOUS) == true.to_string()
        {
            return powered;
        }

        if powered == self.m_lastPower
        {
            return false;
        }

        self.m_lastPower = powered;
        powered
    }

    // PowerProvider.update up to doWork, true when the machine does its work this tick
    pub(crate) fn update(&mut self, world: World, pos: BlockPos) -> bool
    {
        if !self.ready(world, pos)
        {
            return false;
        }

        self.m_stored >= self.m_minActivation
            && (self.m_latency == 0 || self.m_work.due(world.time(), i64::from(self.m_latency)))
    }

    // the rest of PowerProvider.update, after doWork spent what it needed
    // only pneumatic providers lose power so failed redstone preconditions never reach this
    pub(crate) fn decay(&mut self, world: World)
    {
        if self.m_loss > 0 && self.m_lossTracker.due(world.time(), i64::from(self.m_lossRegularity))
        {
            self.m_stored = (self.m_stored - self.m_loss).max(0);
        }
    }

    pub(crate) fn useEnergy(&mut self, min: i32, max: i32, take: bool) -> i32
    {
        if self.m_redstone
        {
            return min;
        }

        if self.m_stored < min
        {
            return 0;
        }

        let used: i32 = self.m_stored.min(max);
        if take
        {
            self.m_stored -= used;
        }

        used
    }

    pub(crate) fn receive(&mut self, amount: i32) { self.m_stored = (self.m_stored + amount).min(self.m_maxStored); }

    // what an engine reads before it pushes, only pneumatic machines take engine power
    pub(crate) fn demand(&self) -> Option<EnergyDemand>
    {
        if self.m_redstone
        {
            return None;
        }

        Some(EnergyDemand { kind: kind(), min: i64::from(self.m_minReceived), max: i64::from(self.m_maxReceived) })
    }

    // the loader's receiveEnergy for a provider, engines hand over a packet they already sized to the demand
    pub(crate) fn take(&mut self, amount: i64, simulate: bool) -> i64
    {
        if self.m_redstone || amount <= 0
        {
            return 0;
        }

        let amount: i32 = amount.min(i64::from(i32::MAX)) as i32;
        if !simulate
        {
            self.receive(amount);
        }

        i64::from(amount)
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.energy().register(MJ_NAME, Energy::new("MJ", MJ_WORTH))?;
    Ok(())
}
