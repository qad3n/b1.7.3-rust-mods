use super::{BossBar, Home, say};
use crate::entities::crystal::{self, Kind};
use crate::entities::fraction;
use crate::entities::valkyrie::{self, AIR_SINAGE, GROUND_SINAGE, valkyriePose};
use crate::ids::blockId;
use crate::ids::item::VICTORY_MEDAL;
use crate::models::Phases;
use crate::progress::{self, Award};
use b173_api::math::{floor, yawTowards};
use b173_api::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::f32::consts::TAU;

const NAME: &str = super::VALKYRIE_QUEEN;
const TEXTURE: &str = "aether:entity/boss/valkyrie_queen/valkyrie_queen";
const DIALOGUE: &str = "queen_dialogue";
const FIGHT: &str = "queen_fight";
const LOCKED: &str = "locked_angelic_stone";
const LOCKED_LIGHT: &str = "locked_light_angelic_stone";
const HURT: &str = "random.hurt";

mod queen
{
    // EntityValkyrieQueen
    pub const HEALTH: i32 = 500;
    pub const DAMAGE: i32 = 8;
    pub const SPEED: f64 = 1.2;
    pub const MEDALS: i32 = 10;
    pub const CHAT_COOLDOWN: i32 = 60;
    pub const CLIMB_WAIT: i32 = 75;
    pub const TELEPORT_AT: i32 = 450;
    pub const TELEPORT_START: i32 = 250;
    pub const TELEPORT_GRACE: i32 = 446;
    pub const TELEPORT_BLIND: i32 = 100;
    pub const TELEPORT_HURT: i32 = 60;
    pub const TELEPORT_RANGE: i32 = 4;
    pub const TELEPORT_TRIES: i32 = 32;
    pub const TELEPORT_RESET: i32 = 40;
    pub const SHOT_ODDS: i32 = 5;
    pub const HALL: [i32; 3] = [20, 12, 20];
    pub const SAFE_DROP: f64 = 16.0;
    pub const ENTRANCES: [i32; 3] = [2, 9, 16];
    pub const DUNGEON_SHIFT: i32 = 19;
    pub const UNLOCK_LOW: [i32; 3] = [-27, -1, -6];
    pub const UNLOCK_HIGH: [i32; 3] = [30, 22, 26];
    pub const TRAPDOORS: [([i32; 2], i32); 4] = [([16, 9], 3), ([17, 9], 2), ([16, 10], 3), ([17, 10], 2)];
    pub const POOF: i32 = 20;
    pub const KEY_LIFT: f64 = 0.5;
    pub const WATCH: f64 = 8.0;
}

// what clients see: the bar and whether the duel has begun
#[derive(Serialize, Deserialize, Default, Clone, PartialEq)]
struct Sync
{
    m_bar: BossBar,
    m_ready: bool,
}

#[derive(Default, Clone, Copy)]
struct Mind
{
    m_chatTime: i32,
    m_teleportIn: i32,
    m_climbTime: i32,
    m_lastMotionY: f64,
    m_safeY: f64,
    m_started: bool,
}

thread_local!
{
    static MINDS: RefCell<HashMap<i32, Mind>> = RefCell::new(HashMap::new());
    static SINAGE: Phases = Phases::default();
}

pub(crate) fn bar(boss: Entity) -> BossBar { boss.synced::<Sync>().m_bar }

fn mind(entity: Entity) -> Mind { MINDS.with(|minds| minds.borrow().get(&entity.id).copied()).unwrap_or_default() }

fn setMind(entity: Entity, state: Mind) { MINDS.with(|minds| minds.borrow_mut().insert(entity.id, state)); }

fn sync(entity: Entity) -> Sync
{
    let mut state: Sync = entity.synced();
    if state.m_bar.m_name.is_empty() && entity.world.isAuthority()
    {
        let name: String = super::valkyrieName(entity.world);

        state.m_bar = BossBar
        {
            m_name: format!("{name}, {}", translate("aether.title.valkyrie_queen")),
            m_health: queen::HEALTH,
            m_maxHealth: queen::HEALTH,
            m_focus: None,
        };

        let _ = entity.setSynced(&state);
    }

    state
}

// setDungeon keeps the hall corner nineteen blocks back from the spot the silver dungeon gives
fn hall(entity: Entity) -> [i32; 3]
{
    let [x, y, z] = entity.data::<Home>().m_dungeon;
    [x, y, z - queen::DUNGEON_SHIFT]
}

fn chat(entity: Entity, player: Player, key: &str)
{
    let mut state: Mind = mind(entity);
    if state.m_chatTime <= 0
    {
        say(player, key);
        state.m_chatTime = queen::CHAT_COOLDOWN;
        setMind(entity, state);
    }
}

fn setEntrance(world: World, hall: [i32; 3], k: i32, block: i32)
{
    let [x, y, z] = hall;

    for (dy, dz) in [(0, 0), (0, 1), (1, 1), (1, 0)]
    {
        let pos: BlockPos = BlockPos::new(x - 1, y + dy, z + k + dz);
        if world.block(pos) != block
        {
            let _ = world.setBlock(pos, block, 0);
        }
    }
}

// the duel seals all three entrances of the hall, and the last one opens again when it ends
fn sealEntrances(world: World, hall: [i32; 3])
{
    let locked: i32 = blockId(LOCKED);

    for k in queen::ENTRANCES
    {
        setEntrance(world, hall, k, locked);
    }
}

fn unlockDoor(world: World, hall: [i32; 3]) { setEntrance(world, hall, queen::ENTRANCES[2], 0); }

// EntityValkyrieQueen.unlockTreasure: trapdoors over the treasure and every locked angelic stone freed
fn unlockTreasure(world: World, hall: [i32; 3])
{
    let [x, y, z] = hall;
    let trapdoor: i32 = blockId("minecraft:trapdoor");

    for ([dx, dz], meta) in queen::TRAPDOORS
    {
        let _ = world.setBlock(BlockPos::new(x + dx, y + 1, z + dz), trapdoor, meta);
    }

    let swaps: [(i32, i32); 2] =
        [(blockId(LOCKED), blockId("angelic_stone")), (blockId(LOCKED_LIGHT), blockId("light_angelic_stone"))];

    for bx in x + queen::UNLOCK_LOW[0]..x + queen::UNLOCK_HIGH[0]
    {
        for by in y + queen::UNLOCK_LOW[1]..y + queen::UNLOCK_HIGH[1]
        {
            for bz in z + queen::UNLOCK_LOW[2]..z + queen::UNLOCK_HIGH[2]
            {
                let pos: BlockPos = BlockPos::new(bx, by, bz);
                let block: i32 = world.block(pos);
                if let Some((_, plain)) = swaps.iter().find(|(locked, _)| *locked == block)
                {
                    let _ = world.setBlock(pos, *plain, 0);
                }
            }
        }
    }
}

fn poof(world: World, at: [f64; 3])
{
    for _ in 0..queen::POOF
    {
        world.particle(
            "explode",
            [at[0] + fraction(world) - 0.5, at[1] + fraction(world) * 2.0, at[2] + fraction(world) - 0.5],
            [0.0; 3],
        );
    }
}

fn airy(world: World, x: i32, y: i32, z: i32) -> bool
{
    let block: i32 = world.block(BlockPos::new(x, y, z));
    block == 0 || !World::blockInfo(block).solid
}

fn jump(entity: Entity, to: [f64; 3])
{
    poof(entity.world, entity.info().map_or(to, |info| info.pos));

    let _ = entity.setPosition(to);
    let _ = entity.setMotion([0.0; 3]);

    entity.setField(EntityField::Jumping, 0.0);
}

// EntityValkyrieQueen.teleport: a few tries for a standing spot near the target inside the hall
fn teleport(entity: Entity, info: &EntityInfo, target: [f64; 3], state: &mut Mind)
{
    let world: World = entity.world;
    let rad: i32 = queen::TELEPORT_RANGE;
    let mut a: i32 = world.random(rad + 1);
    let mut b: i32 = world.random(rad / 2);
    let mut c: i32 = rad - a;
    let sign = || world.random(2) * 2 - 1;
    a *= sign();
    b *= sign();
    c *= sign();
    let start: [i32; 3] =
        [floor(target[0] + a as f64 - 0.5), floor(target[1] + b as f64 - 0.5), floor(target[2] + c as f64 - 0.5)];

    let [hx, hy, hz] = hall(entity);
    let shift = || world.random(rad / 2) - world.random(rad / 2);

    for _ in 0..queen::TELEPORT_TRIES
    {
        let [i, j, k] = [start[0] + shift(), start[1] + shift(), start[2] + shift()];
        let inside: bool =
            i > hx && i < hx + queen::HALL[0] && j > hy && j < hy + queen::HALL[1] && k > hz && k < hz + queen::HALL[2];

        if inside && airy(world, i, j, k) && airy(world, i, j + 1, k) && !airy(world, i, j - 1, k)
        {
            jump(entity, [i as f64 + 0.5, j as f64 + 0.5, k as f64 + 0.5]);
            state.m_teleportIn = world.random(queen::TELEPORT_RESET);
            return;
        }
    }

    state.m_teleportIn -= world.random(queen::TELEPORT_RESET) + queen::TELEPORT_RESET;

    if info.pos[1] <= 0.0
    {
        state.m_teleportIn = queen::TELEPORT_GRACE;
    }
}

// EntityValkyrieQueen.updateEntityActionState while the duel runs
fn duel(entity: Entity, info: &EntityInfo, target: &EntityInfo, state: &mut Mind)
{
    let world: World = entity.world;

    if target.pos[1] > info.pos[1]
    {
        state.m_climbTime += 1;
        if state.m_climbTime >= queen::CLIMB_WAIT
        {
            jump(entity, [target.pos[0] + 0.5, target.pos[1] + 0.5, target.pos[2] + 0.5]);
            state.m_climbTime = 0;
        }
    }
    else
    {
        state.m_climbTime = 0;
    }
    state.m_teleportIn += 1;
    if state.m_teleportIn > queen::TELEPORT_AT
    {
        if entity.flag(EntityField::OnGround) && world.random(queen::SHOT_ODDS) == 0
        {
            let at: [f64; 3] = [info.pos[0] - info.motion[0] / 2.0, info.pos[1], info.pos[2] - info.motion[2] / 2.0];
            crystal::spawn(world, Kind::Thunder, at, Some(target.id), Some(entity.id));
        }
        else
        {
            teleport(entity, info, target.pos, state);
        }
    }
    else if state.m_teleportIn < queen::TELEPORT_GRACE
        && (info.pos[1] <= 0.0 || info.pos[1] <= state.m_safeY - queen::SAFE_DROP)
    {
        state.m_teleportIn = queen::TELEPORT_GRACE;
    }
    else if state.m_teleportIn % 5 == 0 && !valkyrie::canSee(world, info, target)
    {
        state.m_teleportIn += queen::TELEPORT_BLIND;
    }
}

// the duel lost its player: open up and wait for a new challenger, at full strength or not
fn standDown(entity: Entity, sync: &mut Sync, loser: Option<Player>)
{
    if let Some(player) = loser
    {
        let mut state: Mind = mind(entity);
        state.m_chatTime = 0;
        setMind(entity, state);
        chat(entity, player, "gui.queen.lost");
    }

    entity.setField(EntityField::TargetId, -1.0);
    unlockDoor(entity.world, hall(entity));
    sync.m_ready = false;
    sync.m_bar.m_focus = None;
}

fn watchClosest(entity: Entity, info: &EntityInfo)
{
    if let Some(player) = crate::entities::closestPlayer(entity.world, info, queen::WATCH)
    {
        entity.setField(EntityField::Yaw, f64::from(yawTowards(info.pos, player.pos)));
    }
}

struct ValkyrieQueen;

#[entity_behavior]
impl EntityBehavior for ValkyrieQueen
{
    fn onTick(&self, entity: Entity, info: &EntityInfo)
    {
        let world: World = entity.world;
        if !world.isAuthority()
        {
            return;
        }

        let before: Sync = sync(entity);
        let mut state: Sync = before.clone();
        let mut thinking: Mind = mind(entity);
        if !thinking.m_started
        {
            thinking = Mind
            {
                m_started: true,
                m_teleportIn: world.random(queen::TELEPORT_START),
                m_safeY: info.pos[1],
                ..thinking
            };
        }

        thinking.m_chatTime -= 1;
        entity.setField(EntityField::AttackStrength, queen::DAMAGE as f64);
        valkyrie::glide(entity, info, &mut thinking.m_lastMotionY);
        state.m_bar.m_health = entity.field(EntityField::Health) as i32;

        if state.m_ready
        {
            entity.setField(EntityField::MoveSpeed, queen::SPEED);
            sealEntrances(world, hall(entity));

            let focus: Option<EntityInfo> = state.m_bar.m_focus.and_then(|id| Entity { world, id }.info());
            match focus
            {
                Some(target) if target.alive && target.health.unwrap_or(0) > 0 =>
                {
                    entity.setField(EntityField::TargetId, target.id as f64);
                    duel(entity, info, &target, &mut thinking);
                }

                Some(target) => standDown(entity, &mut state, Some(Player { id: target.id })),
                None =>
                {}
            }
        }
        else
        {
            entity.setField(EntityField::MoveSpeed, 0.0);

            let _ = entity.setMotion([0.0, info.motion[1] * 0.5, 0.0]);

            watchClosest(entity, info);
        }
        setMind(entity, thinking);

        if state != before
        {
            let _ = entity.setSynced(&state);
        }
    }

    // EntityValkyrieQueen.interact: talks once the duel is set, otherwise opens the dialogue
    fn onInteract(&self, entity: Entity, player: Player) -> bool
    {
        if !entity.world.isAuthority()
        {
            return true;
        }

        if sync(entity).m_ready
        {
            chat(entity, player, "gui.queen.ready");
        }
        else
        {
            let _ = player.openScreen(DIALOGUE, None, b173_api::toBytes(&entity.id));
        }
        true
    }

    // EntityValkyrieQueen.attackEntityFrom: only a challenger who brought the medals can hurt her
    fn onHurt(&self, entity: Entity, source: Option<i32>, amount: i32) -> Option<i32>
    {
        let world: World = entity.world;
        let Some(attacker) =
            source.and_then(|id| Entity { world, id }.info()).filter(|attacker| attacker.player.is_some())
        else
        {
            entity.setField(EntityField::FireTicks, 0.0);
            return None;
        };

        if !world.isAuthority()
        {
            return Some(amount);
        }

        let player: Player = Player { id: attacker.id };
        let mut state: Sync = sync(entity);
        if !state.m_ready
        {
            poof(world, entity.info()?.pos);
            chat(entity, player, "gui.queen.nomedals");
            return None;
        }

        if state.m_bar.m_focus.is_none()
        {
            let mut thinking: Mind = mind(entity);
            thinking.m_chatTime = 0;
            setMind(entity, thinking);
            chat(entity, player, "gui.queen.fight");
            state.m_bar.m_focus = Some(attacker.id);
            entity.setField(EntityField::TargetId, attacker.id as f64);
        }
        else
        {
            let mut thinking: Mind = mind(entity);
            thinking.m_teleportIn += queen::TELEPORT_HURT;
            setMind(entity, thinking);
        }
        state.m_bar.m_health = (entity.field(EntityField::Health) as i32 - amount).max(0);

        let _ = entity.setSynced(&state);

        Some(amount)
    }

    fn onDeath(&self, entity: Entity, _source: Option<i32>)
    {
        let world: World = entity.world;
        let Some(info) = entity.info().filter(|_| world.isAuthority())
        else
        {
            return;
        };

        let hall: [i32; 3] = hall(entity);
        unlockDoor(world, hall);
        unlockTreasure(world, hall);

        if let Some(id) = sync(entity).m_bar.m_focus
        {
            let mut thinking: Mind = mind(entity);
            thinking.m_chatTime = 0;
            setMind(entity, thinking);
            chat(entity, Player { id }, "gui.queen.defeated");
            progress::grant(Player { id }, Award::Silver);
        }

        let _ =
            world.spawnItem([info.pos[0], info.pos[1] + queen::KEY_LIFT, info.pos[2]], ItemStack::of("silver_key", 1));
        let _ = world.spawnItem(info.pos, ItemStack::of("minecraft:golden_sword", 1));

        poof(world, info.pos);
        MINDS.with(|minds| minds.borrow_mut().remove(&entity.id));
        entity.kill();
    }

    fn pose(&self, input: &PoseInput) -> Pose
    {
        let step: f32 = if input.on_ground
        {
            GROUND_SINAGE
        }
        else
        {
            AIR_SINAGE
        };
        let sinage: f32 = SINAGE.with(|phases| phases.advance(input.entity.id, input.age as i32, step, TAU));
        Pose { pre: Vec::new(), model: valkyriePose(input, sinage, 0.0).take() }
    }
}

// PacketInitiateValkyrieFight, checked on the server: ten medals in the named slot pay for the duel
#[derive(Serialize, Deserialize)]
struct Challenge
{
    m_queen: i32,
    m_slot: i32,
}

fn challenge(message: Challenge, player: Player)
{
    let Some(info) = player.info()
    else
    {
        return;
    };

    let world: World = World { dimension: info.world };
    let paid: bool =
        player.slot(message.m_slot).is_some_and(|stack| stack.item.is(VICTORY_MEDAL) && stack.count >= queen::MEDALS);
    let boss: Entity = Entity { world, id: message.m_queen };
    let isQueen: bool = boss.info().and_then(|queen| queen.kind).is_some_and(|kind| kind.path() == NAME);
    if !paid || !isQueen
    {
        return;
    }

    let _ = player.setSlot(message.m_slot, None);
    let mut state: Sync = sync(boss);
    state.m_ready = true;
    state.m_bar.m_focus = Some(player.id);

    let _ = boss.setSynced(&state);
}

mod dialog
{
    // GuiDialogue and DialogueOption
    pub const WRAP: i32 = 300;
    pub const LINE: i32 = 12;
    pub const TEXT_LINE: i32 = 10;
    pub const PAD: i32 = 2;
    pub const SHADE: u32 = 0x66000000;
    pub const WHITE: u32 = 0xFFFFFF;
    pub const HOVER: &str = "\u{a7}e";
    pub const NAME_COLOR: &str = "\u{a7}e";
    pub const RESET: &str = "\u{a7}r";
    pub const MAIN_SLOTS: i32 = 36;
}

#[derive(Clone, Copy, PartialEq)]
enum Stage
{
    Greeting,
    Medals,
}

// GuiValkyrieDialogue: three questions, the fight asks for ten medals from one inventory slot
struct Dialogue
{
    m_queen: i32,
    m_stage: Stage,
    m_medalSlot: Option<i32>,
    m_options: Vec<String>,
    m_text: String,
}

fn wrap(text: &str) -> Vec<String>
{
    let mut lines: Vec<String> = Vec::new();
    let mut line: String = String::new();

    for word in text.split(' ')
    {
        let candidate: String = if line.is_empty()
        {
            word.to_owned()
        }
        else
        {
            format!("{line} {word}")
        };

        if textWidth(&candidate) > dialog::WRAP && !line.is_empty()
        {
            lines.push(std::mem::replace(&mut line, word.to_owned()));
        }
        else
        {
            line = candidate;
        }
    }

    lines.push(line);
    lines
}

impl Dialogue
{
    fn title(&self) -> String
    {
        let name: String = Entity
        {
            world: World { dimension: localPlayer().and_then(Player::info).map_or(0, |info| info.world) },
            id: self.m_queen,
        }
        .synced::<Sync>()
        .m_bar
        .m_name;
        format!("[{}{name}{}]", dialog::NAME_COLOR, dialog::RESET)
    }

    fn answer(&self, key: &str)
    {
        if let Some(player) = localPlayer()
        {
            player.chat(&format!("{}: {}", self.title(), translate(&format!("aether.{key}"))));
        }
    }

    fn finish(&self, key: &str)
    {
        self.answer(key);

        if let Some(player) = localPlayer()
        {
            player.closeScreen();
        }
    }

    fn medalOption(&mut self) -> String
    {
        let player: Option<Player> = localPlayer();

        for slot in 0..dialog::MAIN_SLOTS
        {
            let Some(stack) = player.and_then(|player| player.slot(slot)).filter(|stack| stack.item.is(VICTORY_MEDAL))
            else
            {
                continue;
            };

            if stack.count >= queen::MEDALS
            {
                self.m_medalSlot = Some(slot);
                return translate("aether.gui.valkyrie.dialog.player.havemedals");
            }

            return format!(
                "{} ({}/{})",
                translate("aether.gui.valkyrie.dialog.player.lackmedals"),
                stack.count,
                queen::MEDALS
            );
        }

        format!("{} (0/{})", translate("aether.gui.valkyrie.dialog.player.lackmedals"), queen::MEDALS)
    }

    fn optionBox(&self, index: usize) -> [i32; 4]
    {
        let label: String = format!("[{}]", self.m_options[index]);
        let width: i32 = textWidth(&label) + dialog::PAD;
        let top: i32 = wrap(&self.m_text).len() as i32 * dialog::LINE + dialog::LINE * index as i32;
        [-width / 2, top, width, dialog::LINE]
    }

    fn choose(&mut self, index: usize)
    {
        match (self.m_stage, index)
        {
            (Stage::Greeting, 0) => self.finish("gui.queen.answer.0"),

            (Stage::Greeting, 1) =>
            {
                self.m_stage = Stage::Medals;
                self.m_text = format!("{}: {}", self.title(), translate("aether.gui.queen.answer.1"));
                self.m_options = vec![self.medalOption(), translate("aether.gui.valkyrie.dialog.player.denyfight")];
            }

            (Stage::Greeting, _) => self.finish("gui.queen.answer.2"),
            (Stage::Medals, 0) => match self.m_medalSlot
            {
                Some(slot) =>
                {
                    let _ = send(FIGHT, Target::Server, &Challenge { m_queen: self.m_queen, m_slot: slot });

                    self.finish("gui.valkyrie.dialog.ready");
                }

                None => self.finish("gui.valkyrie.dialog.nomedals"),
            },
            (Stage::Medals, _) => self.finish("gui.valkyrie.dialog.nofight"),
        }
    }
}

impl Screen for Dialogue
{
    fn init(&mut self, _width: i32, _height: i32, _pos: Option<BlockPos>, data: &[u8])
    {
        self.m_queen = b173_api::fromBytes(data).unwrap_or(-1);
        self.m_stage = Stage::Greeting;
        self.m_medalSlot = None;
        self.m_text = self.title();
        self.m_options = (0..3).map(|index| translate(&format!("aether.gui.queen.dialog.{index}"))).collect();
    }

    fn draw(&mut self, draw: &mut Draw, mouse: [i32; 2], _partial: f32)
    {
        for (row, line) in wrap(&self.m_text).iter().enumerate()
        {
            let width: i32 = textWidth(line);
            let row: i32 = row as i32;
            draw.gradient(
                -width / 2 - dialog::PAD,
                row * dialog::LINE - dialog::PAD,
                width + dialog::PAD * 2,
                dialog::LINE,
                dialog::SHADE,
                dialog::SHADE,
            );
            draw.text(-width / 2, row * dialog::TEXT_LINE, line, dialog::WHITE);
        }

        for index in 0..self.m_options.len()
        {
            let [x, y, w, h] = self.optionBox(index);
            let over: bool = mouse[0] >= x && mouse[1] >= y && mouse[0] < x + w && mouse[1] < y + h;

            draw.gradient(x, y, w, h, dialog::SHADE, dialog::SHADE);

            let label: String = format!("{}[{}]", if over { dialog::HOVER } else { "" }, self.m_options[index]);
            draw.text(x + dialog::PAD, y + dialog::PAD, &label, dialog::WHITE);
        }
    }

    fn mouse(&mut self, x: i32, y: i32, button: i32)
    {
        if button != 0
        {
            return;
        }

        let picked: Option<usize> = (0..self.m_options.len()).find(|index|
        {
            let [left, top, w, h] = self.optionBox(*index);
            x >= left && y >= top && x < left + w && y < top + h
        });
        if let Some(index) = picked
        {
            if let Some(player) = localPlayer().and_then(Player::info)
            {
                World { dimension: player.world }.playSound(player.pos, "random.click", 1.0, 1.0);
            }

            self.choose(index);
        }
    }
}

pub(crate) fn register(ctx: &mut RegisterCtx) -> Result<()>
{
    ctx.entities().register(
        NAME,
        EntitySpec
        {
            template: Identifier::parse("minecraft:neutral_monster").ok(),
            texture: Some(TEXTURE.to_owned()),
            health: Some(queen::HEALTH),
            speed: Some(0.0),
            persistent: true,
            drops: Some(Vec::new()),
            sounds: crate::entities::sounds("", HURT, HURT),
            model: Some(EntityModelSpec { shadow: 0.3, ..crate::models::valkyrie::model() }),
            ..EntitySpec::default()
        },
        Some(std::rc::Rc::new(ValkyrieQueen)),
    )?;
    ctx.channels().register::<Challenge>(FIGHT, Direction::ToServer, challenge)?;

    let blank: Dialogue = Dialogue
    {
        m_queen: -1,
        m_stage: Stage::Greeting,
        m_medalSlot: None,
        m_options: Vec::new(),
        m_text: String::new(),
    };
    ctx.screens().register(DIALOGUE, ScreenBuilder::new("", 0, 0).transparent(), blank)
}
