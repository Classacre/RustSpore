//! Original deterministic Tribe-stage RTS baseline.
//!
//! Food, construction, tools, diplomacy, combat and opponent AI use prototype
//! parameters. They do not claim to reproduce the retail game's rules.
pub use crate::cell::CellVec2 as TribeVec2;
use std::collections::HashSet;
use std::fmt;

pub const TRIBE_FIXED_DT: f64 = 1.0 / 60.0;
const DT: f32 = TRIBE_FIXED_DT as f32;
const MAGIC: &[u8; 8] = b"RSTRIB01";
const ALLIANCE: i16 = 60;
const MAX_MEMBERS: usize = 256;
const MAX_BUILDINGS: usize = 128;
const MAX_FOOD_SITES: usize = 256;
const CARRY_CAPACITY: f32 = 8.0;
const MEMBER_RADIUS: f32 = 0.55;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TribeState {
    Playing,
    CivilizationReady,
    Defeated,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolKind {
    Gathering,
    Weapon,
    Instrument,
}
impl ToolKind {
    pub fn cost(self) -> f32 {
        3.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingKind {
    GatheringHut,
    WeaponHut,
    InstrumentHut,
    HealingHut,
}
impl BuildingKind {
    pub fn cost(self) -> f32 {
        match self {
            Self::HealingHut => 12.,
            _ => 10.,
        }
    }
    pub fn build_seconds(self) -> f32 {
        match self {
            Self::HealingHut => 8.,
            _ => 6.,
        }
    }
    pub fn tool(self) -> Option<ToolKind> {
        match self {
            Self::GatheringHut => Some(ToolKind::Gathering),
            Self::WeaponHut => Some(ToolKind::Weapon),
            Self::InstrumentHut => Some(ToolKind::Instrument),
            Self::HealingHut => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocialPerformance {
    Sing,
    Dance,
    Play,
}
impl SocialPerformance {
    pub fn next(self) -> Self {
        match self {
            Self::Sing => Self::Dance,
            Self::Dance => Self::Play,
            Self::Play => Self::Sing,
        }
    }
}

/// Discrete player commands apply once, independently of render-frame duration.
#[derive(Debug, Clone, PartialEq)]
pub enum TribeCommand {
    Idle,
    Move(TribeVec2),
    GatherFood(u32),
    Build {
        kind: BuildingKind,
        position: TribeVec2,
    },
    EquipTool(ToolKind),
    PerformSocial {
        tribe: u32,
        performance: SocialPerformance,
    },
    Attack(u32),
    Heal(u32),
}
/// Persistent jobs refer to a single already-paid construction foundation.
#[derive(Debug, Clone, PartialEq)]
pub enum TribeOrder {
    Idle,
    Move(TribeVec2),
    GatherFood(u32),
    Build(u32),
    EquipTool(ToolKind),
    PerformSocial {
        tribe: u32,
        performance: SocialPerformance,
    },
    Attack(u32),
    Heal(u32),
}
#[derive(Debug, Clone, PartialEq)]
pub struct TribeMember {
    pub id: u32,
    pub tribe_id: u32,
    pub position: TribeVec2,
    pub health: f32,
    pub max_health: f32,
    pub tool: Option<ToolKind>,
    pub order: TribeOrder,
    pub carried_food: f32,
    pub work_timer: f32,
    pub action_cooldown: f32,
}
impl TribeMember {
    pub fn alive(&self) -> bool {
        self.health > 0.
    }
    pub fn radius(&self) -> f32 {
        MEMBER_RADIUS
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Tribe {
    /// Player tribe is zero; opponents have positive tribe IDs.
    pub id: u32,
    pub home: TribeVec2,
    pub food: f32,
    pub relationship: i16,
    pub totem_health: f32,
    pub max_totem_health: f32,
    pub conquered: bool,
    pub social_cue: SocialPerformance,
    pub ai_cooldown: f32,
}
impl Tribe {
    pub fn allied(&self) -> bool {
        self.id != 0 && self.relationship >= ALLIANCE && !self.conquered
    }
    pub fn resolved(&self) -> bool {
        self.conquered || self.allied()
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct TribeBuilding {
    pub id: u32,
    pub tribe_id: u32,
    pub kind: BuildingKind,
    pub position: TribeVec2,
    pub progress: f32,
}
impl TribeBuilding {
    pub fn complete(&self) -> bool {
        self.progress >= 1.
    }
    pub fn radius(&self) -> f32 {
        1.35
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct TribeFoodSite {
    pub id: u32,
    pub position: TribeVec2,
    pub amount: f32,
    pub initial_amount: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct TribeConfig {
    pub half_extents: TribeVec2,
    pub opponent_count: usize,
    pub starting_members: usize,
    pub starting_food: f32,
    pub food_sites_per_tribe: usize,
    pub food_per_site: f32,
    pub ai_enabled: bool,
}
impl Default for TribeConfig {
    fn default() -> Self {
        Self {
            half_extents: TribeVec2::new(44., 32.),
            opponent_count: 2,
            starting_members: 4,
            starting_food: 30.,
            food_sites_per_tribe: 2,
            food_per_site: 100.,
            ai_enabled: true,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum TribeEvent {
    FoodGathered {
        member: u32,
        site: u32,
        amount: f32,
    },
    FoodDelivered {
        tribe: u32,
        amount: f32,
    },
    ConstructionStarted {
        building: u32,
        kind: BuildingKind,
        cost: f32,
    },
    BuildingCompleted {
        building: u32,
    },
    ToolEquipped {
        member: u32,
        tool: ToolKind,
        cost: f32,
    },
    SocialResult {
        tribe: u32,
        matched: bool,
        relationship: i16,
    },
    Damaged {
        member: u32,
        amount: f32,
    },
    TotemDamaged {
        tribe: u32,
        amount: f32,
    },
    MemberDied {
        member: u32,
    },
    Healed {
        member: u32,
        amount: f32,
        cost: f32,
    },
    TribeAllied {
        tribe: u32,
    },
    TribeConquered {
        tribe: u32,
    },
    CivilizationReady,
    PlayerDefeated,
}
#[derive(Debug, Clone, PartialEq)]
pub struct TribeStepReport {
    pub steps: u32,
    pub backlog_seconds: f64,
    pub interpolation: f32,
    pub state: TribeState,
    pub events: Vec<TribeEvent>,
    pub time_rejected: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct TribeWorld {
    pub config: TribeConfig,
    pub tribes: Vec<Tribe>,
    pub members: Vec<TribeMember>,
    pub buildings: Vec<TribeBuilding>,
    pub food_sites: Vec<TribeFoodSite>,
    pub selected: Vec<u32>,
    pub state: TribeState,
    pub tick: u64,
    seed: u64,
    rng: u64,
    next_id: u32,
    accumulator: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct TribeSnapshot {
    pub version: u32,
    pub world: TribeWorld,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TribeError {
    InvalidConfig,
    InvalidEntity,
    InvalidReference,
    DuplicateId,
    TooManyEntities,
    NoSelection,
    NotEnoughFood,
    MissingBuilding,
    MissingTool,
    OccupiedLocation,
    NotPlaying,
    InvalidSnapshot,
    UnsupportedVersion(u32),
    TruncatedSnapshot,
    TrailingBytes,
}
impl fmt::Display for TribeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tribe stage: {self:?}")
    }
}
impl std::error::Error for TribeError {}

fn finite(p: TribeVec2) -> bool {
    p.x.is_finite() && p.y.is_finite()
}
impl TribeWorld {
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, TribeConfig::default()).expect("valid default Tribe config")
    }
    pub fn with_config(seed: u64, config: TribeConfig) -> Result<Self, TribeError> {
        validate_config(&config)?;
        let mut world = Self {
            config,
            tribes: vec![],
            members: vec![],
            buildings: vec![],
            food_sites: vec![],
            selected: vec![],
            state: TribeState::Playing,
            tick: 0,
            seed,
            rng: seed ^ 0x9e3779b97f4a7c15,
            next_id: 1,
            accumulator: 0.,
        };
        if world.rng == 0 {
            world.rng = 1;
        }
        for id in 0..=world.config.opponent_count as u32 {
            let home = if id == 0 {
                TribeVec2::new(-world.config.half_extents.x * 0.6, 0.)
            } else {
                let row = ((id as f32 - 0.5) / world.config.opponent_count as f32 * 2.0 - 1.0)
                    * world.config.half_extents.y
                    * 0.6;
                TribeVec2::new(world.config.half_extents.x * 0.45, row)
            };
            let cue = match world.random() % 3 {
                0 => SocialPerformance::Sing,
                1 => SocialPerformance::Dance,
                _ => SocialPerformance::Play,
            };
            world.tribes.push(Tribe {
                id,
                home,
                food: world.config.starting_food,
                relationship: if id == 0 {
                    0
                } else if id % 2 == 0 {
                    -20
                } else {
                    0
                },
                totem_health: 300.,
                max_totem_health: 300.,
                conquered: false,
                social_cue: cue,
                ai_cooldown: 2. + id as f32 * 0.3,
            });
            for m in 0..world.config.starting_members {
                let angle = m as f32 / world.config.starting_members as f32 * std::f32::consts::TAU;
                let member_id = world.allocate()?;
                world.members.push(TribeMember {
                    id: member_id,
                    tribe_id: id,
                    position: home.add(TribeVec2::new(angle.cos() * 3., angle.sin() * 3.)),
                    health: 100.,
                    max_health: 100.,
                    tool: None,
                    order: TribeOrder::Idle,
                    carried_food: 0.,
                    work_timer: 0.,
                    action_cooldown: 0.,
                });
                if id == 0 {
                    world.selected.push(member_id);
                }
            }
            for s in 0..world.config.food_sites_per_tribe {
                let site_id = world.allocate()?;
                let jitter = (world.random() % 100) as f32 * 0.015;
                let position = world.clamp(home.add(TribeVec2::new(
                    if id == 0 { 7. + jitter } else { -7. - jitter },
                    (s as f32 - (world.config.food_sites_per_tribe - 1) as f32 * 0.5) * 5.,
                )));
                world.food_sites.push(TribeFoodSite {
                    id: site_id,
                    position,
                    amount: world.config.food_per_site,
                    initial_amount: world.config.food_per_site,
                });
            }
        }
        world.validate()?;
        Ok(world)
    }
    fn random(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        x.wrapping_mul(0x2545f4914f6cdd1d)
    }
    fn allocate(&mut self) -> Result<u32, TribeError> {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(TribeError::TooManyEntities)?;
        Ok(id)
    }
    fn clamp(&self, p: TribeVec2) -> TribeVec2 {
        TribeVec2::new(
            p.x.clamp(
                -self.config.half_extents.x + MEMBER_RADIUS,
                self.config.half_extents.x - MEMBER_RADIUS,
            ),
            p.y.clamp(
                -self.config.half_extents.y + MEMBER_RADIUS,
                self.config.half_extents.y - MEMBER_RADIUS,
            ),
        )
    }
    fn tribe_index(&self, id: u32) -> Option<usize> {
        self.tribes.iter().position(|t| t.id == id)
    }
    fn member_index(&self, id: u32) -> Option<usize> {
        self.members.iter().position(|m| m.id == id)
    }
    fn inside(&self, p: TribeVec2) -> bool {
        finite(p)
            && p.x.abs() <= self.config.half_extents.x - MEMBER_RADIUS
            && p.y.abs() <= self.config.half_extents.y - MEMBER_RADIUS
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn restart(&mut self) {
        *self =
            Self::with_config(self.seed, self.config.clone()).expect("existing valid Tribe config");
    }
    pub fn civilization_eligible(&self) -> bool {
        self.tribes
            .iter()
            .filter(|t| t.id != 0)
            .all(Tribe::resolved)
            && self.tribes.iter().any(|t| t.id != 0)
            && self
                .tribes
                .iter()
                .find(|t| t.id == 0)
                .is_some_and(|t| t.totem_health > 0.)
            && self.members.iter().any(|m| m.tribe_id == 0 && m.alive())
    }
    pub fn progress(&self) -> f32 {
        let n = self.tribes.iter().filter(|t| t.id != 0).count();
        if n == 0 {
            0.
        } else {
            self.tribes
                .iter()
                .filter(|t| t.id != 0 && t.resolved())
                .count() as f32
                / n as f32
        }
    }
    pub fn select(&mut self, ids: &[u32]) -> Result<(), TribeError> {
        let mut selected = Vec::new();
        for &id in ids {
            if !self
                .members
                .iter()
                .any(|m| m.id == id && m.tribe_id == 0 && m.alive())
            {
                return Err(TribeError::InvalidReference);
            }
            if !selected.contains(&id) {
                selected.push(id);
            }
        }
        selected.sort_unstable();
        self.selected = selected;
        Ok(())
    }
    pub fn issue_selected(&mut self, command: TribeCommand) -> Result<Vec<TribeEvent>, TribeError> {
        let ids = self.selected.clone();
        self.issue(0, &ids, command)
    }
    fn has_building(&self, tribe: u32, kind: BuildingKind) -> bool {
        self.buildings
            .iter()
            .any(|b| b.tribe_id == tribe && b.kind == kind && b.complete())
    }
    fn issue(
        &mut self,
        tribe: u32,
        ids: &[u32],
        command: TribeCommand,
    ) -> Result<Vec<TribeEvent>, TribeError> {
        if self.state != TribeState::Playing {
            return Err(TribeError::NotPlaying);
        }
        if ids.is_empty() {
            return Err(TribeError::NoSelection);
        }
        let ti = self
            .tribe_index(tribe)
            .ok_or(TribeError::InvalidReference)?;
        let indices: Vec<_> = ids
            .iter()
            .map(|id| {
                self.member_index(*id)
                    .filter(|i| self.members[*i].tribe_id == tribe && self.members[*i].alive())
                    .ok_or(TribeError::InvalidReference)
            })
            .collect::<Result<_, _>>()?;
        let mut events = vec![];
        let order = match command {
            TribeCommand::Idle => TribeOrder::Idle,
            TribeCommand::Move(p) => {
                if !self.inside(p) {
                    return Err(TribeError::InvalidEntity);
                }
                TribeOrder::Move(p)
            }
            TribeCommand::GatherFood(id) => {
                if !self.food_sites.iter().any(|s| s.id == id && s.amount > 0.) {
                    return Err(TribeError::InvalidReference);
                }
                TribeOrder::GatherFood(id)
            }
            TribeCommand::Build { kind, position } => {
                if self.buildings.len() >= MAX_BUILDINGS {
                    return Err(TribeError::TooManyEntities);
                }
                if !self.inside(position) {
                    return Err(TribeError::InvalidEntity);
                }
                if self
                    .buildings
                    .iter()
                    .any(|b| b.position.distance(position) < 3.2)
                    || self.tribes.iter().any(|t| t.home.distance(position) < 3.2)
                {
                    return Err(TribeError::OccupiedLocation);
                }
                if self.tribes[ti].food < kind.cost() {
                    return Err(TribeError::NotEnoughFood);
                }
                let id = self.allocate()?;
                self.tribes[ti].food -= kind.cost();
                self.buildings.push(TribeBuilding {
                    id,
                    tribe_id: tribe,
                    kind,
                    position,
                    progress: 0.,
                });
                events.push(TribeEvent::ConstructionStarted {
                    building: id,
                    kind,
                    cost: kind.cost(),
                });
                TribeOrder::Build(id)
            }
            TribeCommand::EquipTool(tool) => {
                if !self
                    .buildings
                    .iter()
                    .any(|b| b.tribe_id == tribe && b.complete() && b.kind.tool() == Some(tool))
                {
                    return Err(TribeError::MissingBuilding);
                }
                TribeOrder::EquipTool(tool)
            }
            TribeCommand::PerformSocial {
                tribe: target,
                performance,
            } => {
                if target == tribe
                    || target == 0
                    || !self.tribes.iter().any(|t| t.id == target && !t.conquered)
                {
                    return Err(TribeError::InvalidReference);
                }
                if indices
                    .iter()
                    .any(|&i| self.members[i].tool != Some(ToolKind::Instrument))
                {
                    return Err(TribeError::MissingTool);
                }
                TribeOrder::PerformSocial {
                    tribe: target,
                    performance,
                }
            }
            TribeCommand::Attack(target) => {
                if target == tribe || !self.tribes.iter().any(|t| t.id == target && !t.conquered) {
                    return Err(TribeError::InvalidReference);
                }
                if tribe == 0 {
                    let target_index = self.tribe_index(target).unwrap();
                    self.tribes[target_index].relationship = -80;
                }
                TribeOrder::Attack(target)
            }
            TribeCommand::Heal(id) => {
                if !self.has_building(tribe, BuildingKind::HealingHut) {
                    return Err(TribeError::MissingBuilding);
                }
                if !self
                    .members
                    .iter()
                    .any(|m| m.id == id && m.tribe_id == tribe && m.alive())
                {
                    return Err(TribeError::InvalidReference);
                }
                TribeOrder::Heal(id)
            }
        };
        for i in indices {
            if self.members[i].order != order {
                self.members[i].order = order.clone();
                self.members[i].work_timer = 0.;
            }
        }
        Ok(events)
    }
    pub fn advance(&mut self, seconds: f64) -> TribeStepReport {
        let rejected = !seconds.is_finite()
            || seconds < 0.
            || seconds > 10.
            || self.accumulator + seconds > 60.;
        let mut events = vec![];
        let mut steps = 0;
        if !rejected {
            self.accumulator += seconds;
            while self.accumulator + 1e-12 >= TRIBE_FIXED_DT && steps < 240 {
                self.accumulator = (self.accumulator - TRIBE_FIXED_DT).max(0.);
                events.extend(self.step());
                steps += 1;
            }
        }
        TribeStepReport {
            steps,
            backlog_seconds: self.accumulator,
            interpolation: (self.accumulator / TRIBE_FIXED_DT).min(1.) as f32,
            state: self.state,
            events,
            time_rejected: rejected,
        }
    }
    pub fn step(&mut self) -> Vec<TribeEvent> {
        let mut events = vec![];
        if self.state != TribeState::Playing {
            return events;
        }
        self.tick = self.tick.saturating_add(1);
        self.update_ai(&mut events);
        for i in 0..self.members.len() {
            self.work_member(i, &mut events);
        }
        self.collisions();
        for t in &mut self.tribes {
            if t.conquered {
                continue;
            }
            let count = self
                .members
                .iter()
                .filter(|m| m.tribe_id == t.id && m.alive())
                .count();
            let upkeep = count as f32 * 0.025 * DT;
            if t.food >= upkeep {
                t.food -= upkeep;
            } else {
                t.food = 0.;
                for m in self
                    .members
                    .iter_mut()
                    .filter(|m| m.tribe_id == t.id && m.alive())
                {
                    m.health = (m.health - 0.5 * DT).max(0.);
                    if !m.alive() {
                        events.push(TribeEvent::MemberDied { member: m.id });
                    }
                }
            }
            let now_alive = self.members.iter().any(|m| m.tribe_id == t.id && m.alive());
            if t.id != 0 && (t.totem_health <= 0. || !now_alive) {
                t.conquered = true;
                events.push(TribeEvent::TribeConquered { tribe: t.id });
                for m in self.members.iter_mut().filter(|m| m.tribe_id == t.id) {
                    m.order = TribeOrder::Idle;
                }
            }
        }
        self.selected.retain(|id| {
            self.members
                .iter()
                .any(|m| m.id == *id && m.tribe_id == 0 && m.alive())
        });
        if self.tribes[0].totem_health <= 0.
            || !self.members.iter().any(|m| m.tribe_id == 0 && m.alive())
        {
            self.state = TribeState::Defeated;
            events.push(TribeEvent::PlayerDefeated);
        } else if self.civilization_eligible() {
            self.state = TribeState::CivilizationReady;
            events.push(TribeEvent::CivilizationReady);
        }
        events
    }
}

fn validate_config(c: &TribeConfig) -> Result<(), TribeError> {
    if !finite(c.half_extents)
        || c.half_extents.x < 20.
        || c.half_extents.y < 20.
        || c.half_extents.x > 1000.
        || c.half_extents.y > 1000.
        || !(1..=5).contains(&c.opponent_count)
        || !(1..=16).contains(&c.starting_members)
        || !(1..=12).contains(&c.food_sites_per_tribe)
        || !c.starting_food.is_finite()
        || !(0.0..=10000.).contains(&c.starting_food)
        || !c.food_per_site.is_finite()
        || !(1.0..=10000.).contains(&c.food_per_site)
    {
        Err(TribeError::InvalidConfig)
    } else {
        Ok(())
    }
}

impl TribeWorld {
    fn approach_slot(&self, center: TribeVec2, member: u32, radius: f32) -> TribeVec2 {
        let angle = member as f32 * 2.3999631;
        self.clamp(center.add(TribeVec2::new(angle.cos() * radius, angle.sin() * radius)))
    }
    fn walk(&self, m: &mut TribeMember, target: TribeVec2, range: f32) -> bool {
        let delta = target.sub(m.position);
        let distance = delta.length();
        if distance <= range {
            return true;
        }
        let mut direction = delta.scale(1. / distance.max(1e-6));
        for b in &self.buildings {
            let from = m.position.sub(b.position);
            let d = from.length();
            if d < b.radius() + MEMBER_RADIUS + 1. && direction.dot(from) < 0. {
                let side = if m.id % 2 == 0 { 1. } else { -1. };
                let tangent = TribeVec2::new(-from.y * side, from.x * side);
                let aim = direction.add(tangent.scale(2. / d.max(0.2)));
                direction = aim.scale(1. / aim.length().max(1e-6));
            }
        }
        m.position = self.clamp(
            m.position
                .add(direction.scale((4. * DT).min((distance - range).max(0.)))),
        );
        m.position.distance(target) <= range + 0.01
    }
    fn work_member(&mut self, i: usize, events: &mut Vec<TribeEvent>) {
        let mut m = self.members[i].clone();
        if !m.alive() {
            return;
        }
        let Some(ti) = self.tribe_index(m.tribe_id) else {
            return;
        };
        if self.tribes[ti].conquered {
            return;
        }
        m.action_cooldown = (m.action_cooldown - DT).max(0.);
        let member_id = m.id;
        match m.order.clone() {
            TribeOrder::Idle => {}
            TribeOrder::Move(target) => {
                if self.walk(&mut m, target, 0.15) {
                    m.order = TribeOrder::Idle;
                }
            }
            TribeOrder::GatherFood(id) => {
                if let Some(si) = self.food_sites.iter().position(|s| s.id == id) {
                    if m.carried_food >= CARRY_CAPACITY - 1e-4
                        || (self.food_sites[si].amount <= 0. && m.carried_food > 0.)
                    {
                        if self.walk(&mut m, self.tribes[ti].home, 3.4) {
                            let amount = m.carried_food;
                            self.tribes[ti].food += amount;
                            m.carried_food = 0.;
                            m.work_timer = 0.;
                            events.push(TribeEvent::FoodDelivered {
                                tribe: m.tribe_id,
                                amount,
                            });
                        }
                    } else if self.food_sites[si].amount <= 0. {
                        m.order = TribeOrder::Idle;
                    } else if self.walk(
                        &mut m,
                        self.approach_slot(self.food_sites[si].position, member_id, 1.2),
                        0.75,
                    ) {
                        m.work_timer += DT;
                        if m.work_timer >= 1. {
                            m.work_timer -= 1.;
                            let amount = (if m.tool == Some(ToolKind::Gathering) {
                                2f32
                            } else {
                                1f32
                            })
                            .min(self.food_sites[si].amount)
                            .min(CARRY_CAPACITY - m.carried_food);
                            self.food_sites[si].amount =
                                (self.food_sites[si].amount - amount).max(0.);
                            m.carried_food += amount;
                            events.push(TribeEvent::FoodGathered {
                                member: m.id,
                                site: id,
                                amount,
                            });
                        }
                    }
                } else {
                    m.order = TribeOrder::Idle;
                }
            }
            TribeOrder::Build(id) => {
                if let Some(bi) = self
                    .buildings
                    .iter()
                    .position(|b| b.id == id && b.tribe_id == m.tribe_id)
                {
                    if self.buildings[bi].complete() {
                        m.order = TribeOrder::Idle;
                    } else if self.walk(
                        &mut m,
                        self.approach_slot(self.buildings[bi].position, member_id, 2.3),
                        0.85,
                    ) {
                        let b = &mut self.buildings[bi];
                        b.progress = (b.progress + DT / b.kind.build_seconds()).min(1.);
                        if b.complete() {
                            events.push(TribeEvent::BuildingCompleted { building: id });
                            m.order = TribeOrder::Idle;
                        }
                    }
                } else {
                    m.order = TribeOrder::Idle;
                }
            }
            TribeOrder::EquipTool(tool) => {
                if m.tool == Some(tool) {
                    m.order = TribeOrder::Idle;
                } else if let Some(p) = self
                    .buildings
                    .iter()
                    .find(|b| {
                        b.tribe_id == m.tribe_id && b.complete() && b.kind.tool() == Some(tool)
                    })
                    .map(|b| b.position)
                {
                    if self.walk(&mut m, self.approach_slot(p, member_id, 2.3), 0.85) {
                        m.work_timer = (m.work_timer + DT).min(1.);
                        if m.work_timer >= 1. && self.tribes[ti].food >= tool.cost() {
                            self.tribes[ti].food -= tool.cost();
                            m.tool = Some(tool);
                            m.order = TribeOrder::Idle;
                            m.work_timer = 0.;
                            events.push(TribeEvent::ToolEquipped {
                                member: m.id,
                                tool,
                                cost: tool.cost(),
                            });
                        }
                    }
                } else {
                    m.order = TribeOrder::Idle;
                }
            }
            TribeOrder::PerformSocial { tribe, performance } => {
                if let Some(target) = self
                    .tribe_index(tribe)
                    .filter(|j| !self.tribes[*j].conquered && !self.tribes[*j].allied())
                {
                    if self.walk(&mut m, self.tribes[target].home, 4.5) {
                        m.work_timer += DT;
                        if m.work_timer >= 2. {
                            m.work_timer -= 2.;
                            let matched = performance == self.tribes[target].social_cue;
                            let opposition = self
                                .members
                                .iter()
                                .filter(|a| a.tribe_id == tribe && a.alive())
                                .count()
                                .min(6) as i16;
                            let old = self.tribes[target].relationship;
                            let delta = if matched { 12 - opposition } else { -4 };
                            self.tribes[target].relationship = (old + delta).clamp(-100, 100);
                            events.push(TribeEvent::SocialResult {
                                tribe,
                                matched,
                                relationship: self.tribes[target].relationship,
                            });
                            if old < ALLIANCE && self.tribes[target].allied() {
                                for unit in self.members.iter_mut().filter(|a| {
                                    a.tribe_id == tribe && matches!(a.order, TribeOrder::Attack(0))
                                }) {
                                    unit.order = TribeOrder::Idle;
                                }
                                events.push(TribeEvent::TribeAllied { tribe });
                            }
                        }
                    }
                } else {
                    m.order = TribeOrder::Idle;
                }
            }
            TribeOrder::Attack(tribe) => {
                if let Some(target) = self
                    .tribe_index(tribe)
                    .filter(|j| !self.tribes[*j].conquered)
                {
                    let home = self.tribes[target].home;
                    let defender = self
                        .members
                        .iter()
                        .enumerate()
                        .filter(|(_, a)| {
                            a.tribe_id == tribe && a.alive() && a.position.distance(home) < 9.
                        })
                        .min_by(|(_, a), (_, b)| {
                            a.position
                                .distance(m.position)
                                .total_cmp(&b.position.distance(m.position))
                        })
                        .map(|(j, _)| j);
                    let damage: f32 = if m.tool == Some(ToolKind::Weapon) {
                        14.
                    } else {
                        7.
                    };
                    if let Some(j) = defender {
                        if self.walk(&mut m, self.members[j].position, 1.4)
                            && m.action_cooldown <= 0.
                        {
                            m.action_cooldown = 0.8;
                            let victim = &mut self.members[j];
                            let amount = damage.min(victim.health);
                            victim.health = (victim.health - damage).max(0.);
                            events.push(TribeEvent::Damaged {
                                member: victim.id,
                                amount,
                            });
                            if !victim.alive() {
                                events.push(TribeEvent::MemberDied { member: victim.id });
                            }
                        }
                    } else if self.walk(&mut m, home, 2.5) && m.action_cooldown <= 0. {
                        m.action_cooldown = 0.8;
                        let amount = damage.min(self.tribes[target].totem_health);
                        self.tribes[target].totem_health =
                            (self.tribes[target].totem_health - damage).max(0.);
                        events.push(TribeEvent::TotemDamaged { tribe, amount });
                    }
                } else {
                    m.order = TribeOrder::Idle;
                }
            }
            TribeOrder::Heal(id) => {
                if let Some(j) = self
                    .member_index(id)
                    .filter(|j| self.members[*j].alive() && self.members[*j].tribe_id == m.tribe_id)
                {
                    let health = if j == i {
                        m.health
                    } else {
                        self.members[j].health
                    };
                    let max = self.members[j].max_health;
                    if health >= max {
                        m.order = TribeOrder::Idle;
                    } else if self.walk(&mut m, self.members[j].position, 1.8) {
                        m.work_timer = (m.work_timer + DT).min(2.);
                        if m.work_timer >= 2. && self.tribes[ti].food >= 2. {
                            m.work_timer -= 2.;
                            self.tribes[ti].food -= 2.;
                            let amount = (max - health).min(20.);
                            if j == i {
                                m.health += amount;
                            } else {
                                self.members[j].health += amount;
                            }
                            events.push(TribeEvent::Healed {
                                member: id,
                                amount,
                                cost: 2.,
                            });
                        }
                    }
                } else {
                    m.order = TribeOrder::Idle;
                }
            }
        }
        self.members[i] = m;
    }
    fn collisions(&mut self) {
        for i in 0..self.members.len() {
            if !self.members[i].alive() {
                continue;
            }
            for j in i + 1..self.members.len() {
                if !self.members[j].alive() {
                    continue;
                }
                let delta = self.members[j].position.sub(self.members[i].position);
                let distance = delta.length();
                if distance < MEMBER_RADIUS * 2. {
                    let normal = if distance > 1e-6 {
                        delta.scale(1. / distance)
                    } else {
                        TribeVec2::new(1., 0.)
                    };
                    let offset = normal.scale((MEMBER_RADIUS * 2. - distance) * 0.5);
                    self.members[i].position = self.clamp(self.members[i].position.sub(offset));
                    self.members[j].position = self.clamp(self.members[j].position.add(offset));
                }
            }
            for b in &self.buildings {
                let delta = self.members[i].position.sub(b.position);
                let distance = delta.length();
                let radius = b.radius() + MEMBER_RADIUS;
                if distance < radius {
                    let normal = if distance > 1e-6 {
                        delta.scale(1. / distance)
                    } else {
                        TribeVec2::new(1., 0.)
                    };
                    self.members[i].position = self.clamp(b.position.add(normal.scale(radius)));
                }
            }
        }
    }
    fn update_ai(&mut self, events: &mut Vec<TribeEvent>) {
        if !self.config.ai_enabled {
            return;
        }
        for ti in 1..self.tribes.len() {
            if self.tribes[ti].conquered {
                continue;
            }
            self.tribes[ti].ai_cooldown -= DT;
            if self.tribes[ti].ai_cooldown > 0. || self.tribes[ti].conquered {
                continue;
            }
            let random = self.random();
            self.tribes[ti].ai_cooldown = 2. + (random % 50) as f32 / 100.;
            if random % 4 == 0 {
                self.tribes[ti].social_cue = self.tribes[ti].social_cue.next();
            }
            let tribe = self.tribes[ti].id;
            let home = self.tribes[ti].home;
            let ids: Vec<_> = self
                .members
                .iter()
                .filter(|m| m.tribe_id == tribe && m.alive())
                .map(|m| m.id)
                .collect();
            if ids.is_empty() {
                continue;
            }
            if self.tribes[ti].allied() {
                for m in self
                    .members
                    .iter_mut()
                    .filter(|m| m.tribe_id == tribe && matches!(m.order, TribeOrder::Attack(0)))
                {
                    m.order = TribeOrder::Idle;
                }
                continue;
            }
            if self.tribes[ti].relationship > 0 {
                self.tribes[ti].relationship -= 1;
            }
            let food = self
                .food_sites
                .iter()
                .filter(|s| s.amount > 0.)
                .min_by(|a, b| {
                    a.position
                        .distance(home)
                        .total_cmp(&b.position.distance(home))
                })
                .map(|s| s.id);
            let unfinished = self
                .buildings
                .iter()
                .find(|b| b.tribe_id == tribe && !b.complete())
                .map(|b| b.id);
            if let Some(id) = unfinished {
                let i = self.member_index(ids[0]).unwrap();
                self.members[i].order = TribeOrder::Build(id);
            } else if !self.has_building(tribe, BuildingKind::WeaponHut)
                && self.tribes[ti].food >= BuildingKind::WeaponHut.cost()
            {
                for offset in [
                    TribeVec2::new(4., 4.),
                    TribeVec2::new(-4., 4.),
                    TribeVec2::new(4., -4.),
                    TribeVec2::new(-4., -4.),
                ] {
                    if let Ok(e) = self.issue(
                        tribe,
                        &ids[..1],
                        TribeCommand::Build {
                            kind: BuildingKind::WeaponHut,
                            position: self.clamp(home.add(offset)),
                        },
                    ) {
                        events.extend(e);
                        break;
                    }
                }
            } else if let Some(site) = food {
                let _ = self.issue(tribe, &ids[..1], TribeCommand::GatherFood(site));
            }
            if ids.len() > 1 {
                if let Some(site) = food {
                    let _ = self.issue(tribe, &ids[1..2], TribeCommand::GatherFood(site));
                }
            }
            for &id in ids.iter().skip(2) {
                let i = self.member_index(id).unwrap();
                if self.members[i].tool != Some(ToolKind::Weapon)
                    && self.has_building(tribe, BuildingKind::WeaponHut)
                    && self.tribes[ti].food >= ToolKind::Weapon.cost()
                {
                    let _ = self.issue(tribe, &[id], TribeCommand::EquipTool(ToolKind::Weapon));
                } else if self.tick > 1800 && self.members[i].tool == Some(ToolKind::Weapon) {
                    let _ = self.issue(tribe, &[id], TribeCommand::Attack(0));
                } else if let Some(site) = food {
                    let _ = self.issue(tribe, &[id], TribeCommand::GatherFood(site));
                }
            }
        }
    }
}

impl TribeWorld {
    pub fn snapshot(&self) -> TribeSnapshot {
        TribeSnapshot {
            version: 1,
            world: self.clone(),
        }
    }
    pub fn from_snapshot(snapshot: TribeSnapshot) -> Result<Self, TribeError> {
        if snapshot.version != 1 {
            return Err(TribeError::UnsupportedVersion(snapshot.version));
        }
        snapshot.world.validate()?;
        Ok(snapshot.world)
    }
    fn validate(&self) -> Result<(), TribeError> {
        validate_config(&self.config)?;
        if self.tribes.len() != self.config.opponent_count + 1
            || self.members.len() > MAX_MEMBERS
            || self.buildings.len() > MAX_BUILDINGS
            || self.food_sites.len() > MAX_FOOD_SITES
            || self.selected.len() > MAX_MEMBERS
        {
            return Err(TribeError::TooManyEntities);
        }
        if self.rng == 0
            || self.next_id == 0
            || !self.accumulator.is_finite()
            || !(0.0..=60.).contains(&self.accumulator)
        {
            return Err(TribeError::InvalidSnapshot);
        }
        for (i, t) in self.tribes.iter().enumerate() {
            if t.id != i as u32
                || !self.inside(t.home)
                || !t.food.is_finite()
                || !(0.0..=1000000.).contains(&t.food)
                || !(-100..=100).contains(&t.relationship)
                || !t.totem_health.is_finite()
                || !t.max_totem_health.is_finite()
                || !(1.0..=10000.).contains(&t.max_totem_health)
                || !(0.0..=t.max_totem_health).contains(&t.totem_health)
                || !t.ai_cooldown.is_finite()
                || !(-1.0..=10.).contains(&t.ai_cooldown)
                || (t.id == 0 && t.conquered)
            {
                return Err(TribeError::InvalidEntity);
            }
        }
        let mut ids = HashSet::new();
        let mut max_id = 0;
        let mut add = |id: u32| -> Result<(), TribeError> {
            if id == 0 {
                return Err(TribeError::InvalidEntity);
            }
            if !ids.insert(id) {
                return Err(TribeError::DuplicateId);
            }
            max_id = max_id.max(id);
            Ok(())
        };
        for m in &self.members {
            add(m.id)?;
            if self.tribe_index(m.tribe_id).is_none()
                || !self.inside(m.position)
                || !m.health.is_finite()
                || !m.max_health.is_finite()
                || !(1.0..=10000.).contains(&m.max_health)
                || !(0.0..=m.max_health).contains(&m.health)
                || !m.carried_food.is_finite()
                || !(0.0..=CARRY_CAPACITY + 0.001).contains(&m.carried_food)
                || !m.work_timer.is_finite()
                || !(0.0..=2.1).contains(&m.work_timer)
                || !m.action_cooldown.is_finite()
                || !(0.0..=1.).contains(&m.action_cooldown)
            {
                return Err(TribeError::InvalidEntity);
            }
        }
        for b in &self.buildings {
            add(b.id)?;
            if self.tribe_index(b.tribe_id).is_none()
                || !self.inside(b.position)
                || !b.progress.is_finite()
                || !(0.0..=1.).contains(&b.progress)
            {
                return Err(TribeError::InvalidEntity);
            }
        }
        for s in &self.food_sites {
            add(s.id)?;
            if !self.inside(s.position)
                || !s.amount.is_finite()
                || !s.initial_amount.is_finite()
                || !(0.0..=10000.).contains(&s.initial_amount)
                || !(0.0..=s.initial_amount).contains(&s.amount)
            {
                return Err(TribeError::InvalidEntity);
            }
        }
        if self.next_id <= max_id {
            return Err(TribeError::InvalidSnapshot);
        }
        let mut selected = HashSet::new();
        for id in &self.selected {
            if !selected.insert(*id) {
                return Err(TribeError::DuplicateId);
            }
            if !self
                .members
                .iter()
                .any(|m| m.id == *id && m.tribe_id == 0 && m.alive())
            {
                return Err(TribeError::InvalidReference);
            }
        }
        for m in &self.members {
            let valid = match m.order {
                TribeOrder::Idle => true,
                TribeOrder::Move(p) => self.inside(p),
                TribeOrder::GatherFood(id) => self.food_sites.iter().any(|s| s.id == id),
                TribeOrder::Build(id) => self
                    .buildings
                    .iter()
                    .any(|b| b.id == id && b.tribe_id == m.tribe_id),
                TribeOrder::EquipTool(tool) => self.buildings.iter().any(|b| {
                    b.tribe_id == m.tribe_id && b.complete() && b.kind.tool() == Some(tool)
                }),
                TribeOrder::PerformSocial { tribe, .. } => {
                    tribe != m.tribe_id
                        && tribe != 0
                        && self.tribe_index(tribe).is_some()
                        && m.tool == Some(ToolKind::Instrument)
                }
                TribeOrder::Attack(tribe) => {
                    tribe != m.tribe_id && self.tribe_index(tribe).is_some()
                }
                TribeOrder::Heal(id) => {
                    self.members
                        .iter()
                        .any(|a| a.id == id && a.tribe_id == m.tribe_id)
                        && self.has_building(m.tribe_id, BuildingKind::HealingHut)
                }
            };
            if !valid {
                return Err(TribeError::InvalidReference);
            }
        }
        for t in &self.tribes {
            if !self.members.iter().any(|m| m.tribe_id == t.id) {
                return Err(TribeError::InvalidReference);
            }
            if t.id != 0
                && t.conquered
                && t.totem_health > 0.
                && self.members.iter().any(|m| m.tribe_id == t.id && m.alive())
            {
                return Err(TribeError::InvalidSnapshot);
            }
        }
        let expected = if self.tribes[0].totem_health <= 0.
            || !self.members.iter().any(|m| m.tribe_id == 0 && m.alive())
        {
            TribeState::Defeated
        } else if self.civilization_eligible() {
            TribeState::CivilizationReady
        } else {
            TribeState::Playing
        };
        if expected != self.state {
            return Err(TribeError::InvalidSnapshot);
        }
        Ok(())
    }
}

struct Writer(Vec<u8>);
impl Writer {
    fn byte(&mut self, v: u8) {
        self.0.push(v);
    }
    fn word(&mut self, v: u32) {
        self.0.extend(v.to_le_bytes());
    }
    fn wide(&mut self, v: u64) {
        self.0.extend(v.to_le_bytes());
    }
    fn float(&mut self, v: f32) {
        self.word(v.to_bits());
    }
    fn double(&mut self, v: f64) {
        self.wide(v.to_bits());
    }
    fn point(&mut self, p: TribeVec2) {
        self.float(p.x);
        self.float(p.y);
    }
    fn tool(&mut self, v: ToolKind) {
        self.byte(match v {
            ToolKind::Gathering => 0,
            ToolKind::Weapon => 1,
            ToolKind::Instrument => 2,
        });
    }
    fn building(&mut self, v: BuildingKind) {
        self.byte(match v {
            BuildingKind::GatheringHut => 0,
            BuildingKind::WeaponHut => 1,
            BuildingKind::InstrumentHut => 2,
            BuildingKind::HealingHut => 3,
        });
    }
    fn performance(&mut self, v: SocialPerformance) {
        self.byte(match v {
            SocialPerformance::Sing => 0,
            SocialPerformance::Dance => 1,
            SocialPerformance::Play => 2,
        });
    }
    fn order(&mut self, v: &TribeOrder) {
        match v {
            TribeOrder::Idle => self.byte(0),
            TribeOrder::Move(p) => {
                self.byte(1);
                self.point(*p);
            }
            TribeOrder::GatherFood(id) => {
                self.byte(2);
                self.word(*id);
            }
            TribeOrder::Build(id) => {
                self.byte(3);
                self.word(*id);
            }
            TribeOrder::EquipTool(tool) => {
                self.byte(4);
                self.tool(*tool);
            }
            TribeOrder::PerformSocial { tribe, performance } => {
                self.byte(5);
                self.word(*tribe);
                self.performance(*performance);
            }
            TribeOrder::Attack(id) => {
                self.byte(6);
                self.word(*id);
            }
            TribeOrder::Heal(id) => {
                self.byte(7);
                self.word(*id);
            }
        }
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], TribeError> {
        let end = self
            .at
            .checked_add(N)
            .ok_or(TribeError::TruncatedSnapshot)?;
        let v = self
            .bytes
            .get(self.at..end)
            .ok_or(TribeError::TruncatedSnapshot)?;
        self.at = end;
        Ok(v.try_into().unwrap())
    }
    fn byte(&mut self) -> Result<u8, TribeError> {
        Ok(self.take::<1>()?[0])
    }
    fn word(&mut self) -> Result<u32, TribeError> {
        Ok(u32::from_le_bytes(self.take()?))
    }
    fn wide(&mut self) -> Result<u64, TribeError> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn float(&mut self) -> Result<f32, TribeError> {
        Ok(f32::from_bits(self.word()?))
    }
    fn double(&mut self) -> Result<f64, TribeError> {
        Ok(f64::from_bits(self.wide()?))
    }
    fn point(&mut self) -> Result<TribeVec2, TribeError> {
        Ok(TribeVec2::new(self.float()?, self.float()?))
    }
    fn count(&mut self, max: usize) -> Result<usize, TribeError> {
        let n = self.word()? as usize;
        if n > max {
            Err(TribeError::TooManyEntities)
        } else {
            Ok(n)
        }
    }
    fn boolean(&mut self) -> Result<bool, TribeError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(TribeError::InvalidSnapshot),
        }
    }
    fn tool(&mut self) -> Result<ToolKind, TribeError> {
        match self.byte()? {
            0 => Ok(ToolKind::Gathering),
            1 => Ok(ToolKind::Weapon),
            2 => Ok(ToolKind::Instrument),
            _ => Err(TribeError::InvalidSnapshot),
        }
    }
    fn building(&mut self) -> Result<BuildingKind, TribeError> {
        match self.byte()? {
            0 => Ok(BuildingKind::GatheringHut),
            1 => Ok(BuildingKind::WeaponHut),
            2 => Ok(BuildingKind::InstrumentHut),
            3 => Ok(BuildingKind::HealingHut),
            _ => Err(TribeError::InvalidSnapshot),
        }
    }
    fn performance(&mut self) -> Result<SocialPerformance, TribeError> {
        match self.byte()? {
            0 => Ok(SocialPerformance::Sing),
            1 => Ok(SocialPerformance::Dance),
            2 => Ok(SocialPerformance::Play),
            _ => Err(TribeError::InvalidSnapshot),
        }
    }
    fn order(&mut self) -> Result<TribeOrder, TribeError> {
        match self.byte()? {
            0 => Ok(TribeOrder::Idle),
            1 => Ok(TribeOrder::Move(self.point()?)),
            2 => Ok(TribeOrder::GatherFood(self.word()?)),
            3 => Ok(TribeOrder::Build(self.word()?)),
            4 => Ok(TribeOrder::EquipTool(self.tool()?)),
            5 => Ok(TribeOrder::PerformSocial {
                tribe: self.word()?,
                performance: self.performance()?,
            }),
            6 => Ok(TribeOrder::Attack(self.word()?)),
            7 => Ok(TribeOrder::Heal(self.word()?)),
            _ => Err(TribeError::InvalidSnapshot),
        }
    }
}
impl TribeSnapshot {
    pub fn to_bytes(&self) -> Result<Vec<u8>, TribeError> {
        if self.version != 1 {
            return Err(TribeError::UnsupportedVersion(self.version));
        }
        self.world.validate()?;
        let w = &self.world;
        let c = &w.config;
        let mut b = Writer(MAGIC.to_vec());
        b.word(1);
        b.point(c.half_extents);
        b.word(c.opponent_count as u32);
        b.word(c.starting_members as u32);
        b.float(c.starting_food);
        b.word(c.food_sites_per_tribe as u32);
        b.float(c.food_per_site);
        b.byte(c.ai_enabled as u8);
        b.wide(w.seed);
        b.wide(w.rng);
        b.word(w.next_id);
        b.double(w.accumulator);
        b.wide(w.tick);
        b.byte(match w.state {
            TribeState::Playing => 0,
            TribeState::CivilizationReady => 1,
            TribeState::Defeated => 2,
        });
        b.word(w.tribes.len() as u32);
        for t in &w.tribes {
            b.word(t.id);
            b.point(t.home);
            b.float(t.food);
            b.0.extend(t.relationship.to_le_bytes());
            b.float(t.totem_health);
            b.float(t.max_totem_health);
            b.byte(t.conquered as u8);
            b.performance(t.social_cue);
            b.float(t.ai_cooldown);
        }
        b.word(w.members.len() as u32);
        for m in &w.members {
            b.word(m.id);
            b.word(m.tribe_id);
            b.point(m.position);
            b.float(m.health);
            b.float(m.max_health);
            match m.tool {
                None => b.byte(0),
                Some(tool) => {
                    b.byte(1);
                    b.tool(tool);
                }
            }
            b.order(&m.order);
            b.float(m.carried_food);
            b.float(m.work_timer);
            b.float(m.action_cooldown);
        }
        b.word(w.buildings.len() as u32);
        for v in &w.buildings {
            b.word(v.id);
            b.word(v.tribe_id);
            b.building(v.kind);
            b.point(v.position);
            b.float(v.progress);
        }
        b.word(w.food_sites.len() as u32);
        for v in &w.food_sites {
            b.word(v.id);
            b.point(v.position);
            b.float(v.amount);
            b.float(v.initial_amount);
        }
        b.word(w.selected.len() as u32);
        for id in &w.selected {
            b.word(*id);
        }
        Ok(b.0)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, TribeError> {
        if bytes.len() > 1024 * 1024 {
            return Err(TribeError::TooManyEntities);
        }
        let mut r = Reader { bytes, at: 0 };
        if &r.take::<8>()? != MAGIC {
            return Err(TribeError::InvalidSnapshot);
        }
        let version = r.word()?;
        if version != 1 {
            return Err(TribeError::UnsupportedVersion(version));
        }
        let config = TribeConfig {
            half_extents: r.point()?,
            opponent_count: r.count(5)?,
            starting_members: r.count(16)?,
            starting_food: r.float()?,
            food_sites_per_tribe: r.count(12)?,
            food_per_site: r.float()?,
            ai_enabled: r.boolean()?,
        };
        validate_config(&config)?;
        let seed = r.wide()?;
        let rng = r.wide()?;
        let next_id = r.word()?;
        let accumulator = r.double()?;
        let tick = r.wide()?;
        let state = match r.byte()? {
            0 => TribeState::Playing,
            1 => TribeState::CivilizationReady,
            2 => TribeState::Defeated,
            _ => return Err(TribeError::InvalidSnapshot),
        };
        let mut tribes = Vec::new();
        for _ in 0..r.count(6)? {
            tribes.push(Tribe {
                id: r.word()?,
                home: r.point()?,
                food: r.float()?,
                relationship: i16::from_le_bytes(r.take()?),
                totem_health: r.float()?,
                max_totem_health: r.float()?,
                conquered: r.boolean()?,
                social_cue: r.performance()?,
                ai_cooldown: r.float()?,
            });
        }
        let mut members = Vec::new();
        for _ in 0..r.count(MAX_MEMBERS)? {
            let id = r.word()?;
            let tribe_id = r.word()?;
            let position = r.point()?;
            let health = r.float()?;
            let max_health = r.float()?;
            let tool = if r.boolean()? { Some(r.tool()?) } else { None };
            members.push(TribeMember {
                id,
                tribe_id,
                position,
                health,
                max_health,
                tool,
                order: r.order()?,
                carried_food: r.float()?,
                work_timer: r.float()?,
                action_cooldown: r.float()?,
            });
        }
        let mut buildings = Vec::new();
        for _ in 0..r.count(MAX_BUILDINGS)? {
            buildings.push(TribeBuilding {
                id: r.word()?,
                tribe_id: r.word()?,
                kind: r.building()?,
                position: r.point()?,
                progress: r.float()?,
            });
        }
        let mut food_sites = Vec::new();
        for _ in 0..r.count(MAX_FOOD_SITES)? {
            food_sites.push(TribeFoodSite {
                id: r.word()?,
                position: r.point()?,
                amount: r.float()?,
                initial_amount: r.float()?,
            });
        }
        let mut selected = Vec::new();
        for _ in 0..r.count(MAX_MEMBERS)? {
            selected.push(r.word()?);
        }
        if r.at != bytes.len() {
            return Err(TribeError::TrailingBytes);
        }
        let world = TribeWorld {
            config,
            tribes,
            members,
            buildings,
            food_sites,
            selected,
            state,
            tick,
            seed,
            rng,
            next_id,
            accumulator,
        };
        world.validate()?;
        Ok(Self { version, world })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn peaceful() -> TribeWorld {
        TribeWorld::with_config(
            123,
            TribeConfig {
                ai_enabled: false,
                ..Default::default()
            },
        )
        .unwrap()
    }
    fn one_each() -> TribeWorld {
        TribeWorld::with_config(
            123,
            TribeConfig {
                ai_enabled: false,
                starting_members: 1,
                opponent_count: 1,
                ..Default::default()
            },
        )
        .unwrap()
    }
    fn ticks(w: &mut TribeWorld, n: usize) -> Vec<TribeEvent> {
        (0..n).flat_map(|_| w.step()).collect()
    }
    fn hut(w: &mut TribeWorld, kind: BuildingKind) -> u32 {
        let id = w.allocate().unwrap();
        let position = w.tribes[0].home.add(TribeVec2::new(4., 4.));
        w.buildings.push(TribeBuilding {
            id,
            tribe_id: 0,
            kind,
            position,
            progress: 1.,
        });
        id
    }
    fn at_tribe(w: &mut TribeWorld, target: u32) {
        let home = w.tribes[target as usize].home;
        let mut j = 0;
        for m in w.members.iter_mut().filter(|m| m.tribe_id == 0) {
            m.position = home.add(TribeVec2::new(-3., j as f32 * 1.2 - 1.8));
            j += 1;
        }
    }
    #[test]
    fn seeded_world_is_valid_distinct_and_restarts() {
        let mut a = TribeWorld::new(42);
        assert_eq!(a, TribeWorld::new(42));
        assert_ne!(a.food_sites, TribeWorld::new(43).food_sites);
        for i in 0..a.tribes.len() {
            for j in i + 1..a.tribes.len() {
                assert!(a.tribes[i].home.distance(a.tribes[j].home) > 6.);
            }
        }
        ticks(&mut a, 1000);
        a.restart();
        assert_eq!(a, TribeWorld::new(42));
    }
    #[test]
    fn selection_is_transactional_and_moves_only_selected() {
        let mut w = peaceful();
        let id = w.selected[0];
        w.select(&[id, id]).unwrap();
        assert_eq!(w.selected, vec![id]);
        let before = w.selected.clone();
        let enemy = w.members.iter().find(|m| m.tribe_id != 0).unwrap().id;
        assert!(w.select(&[enemy]).is_err());
        assert_eq!(w.selected, before);
        let other = w.members[1].position;
        let target = w.members[0].position.add(TribeVec2::new(6., 0.));
        w.issue_selected(TribeCommand::Move(target)).unwrap();
        ticks(&mut w, 160);
        assert!(w.members[0].position.distance(target) < 0.2);
        assert_eq!(w.members[1].position, other);
    }
    #[test]
    fn food_gathering_depletes_site_and_delivers_without_respawn() {
        let mut w = one_each();
        let site = w.food_sites[0].id;
        w.food_sites[0].amount = 3.;
        w.food_sites[0].initial_amount = 3.;
        w.members[0].position = w.food_sites[0].position;
        let initial = w.tribes[0].food;
        w.issue_selected(TribeCommand::GatherFood(site)).unwrap();
        let e = ticks(&mut w, 900);
        let gathered: f32 = e
            .iter()
            .filter_map(|e| {
                if let TribeEvent::FoodGathered { amount, .. } = e {
                    Some(*amount)
                } else {
                    None
                }
            })
            .sum();
        assert_eq!(gathered, 3.);
        assert_eq!(w.food_sites[0].amount, 0.);
        assert_eq!(w.members[0].carried_food, 0.);
        assert!(w.tribes[0].food > initial + 2.);
        ticks(&mut w, 900);
        assert_eq!(w.food_sites[0].amount, 0.);
    }
    #[test]
    fn gathering_tool_changes_harvest_rate() {
        let mut a = one_each();
        let mut b = a.clone();
        for w in [&mut a, &mut b] {
            w.members[0].position = w.food_sites[0].position;
            w.issue_selected(TribeCommand::GatherFood(w.food_sites[0].id))
                .unwrap();
        }
        b.members[0].tool = Some(ToolKind::Gathering);
        ticks(&mut a, 180);
        ticks(&mut b, 180);
        assert!(b.members[0].carried_food > a.members[0].carried_food * 1.5);
    }
    #[test]
    fn group_build_creates_one_paid_foundation_and_finishes_after_work() {
        let mut w = peaceful();
        let before = w.tribes[0].food;
        let position = w.tribes[0].home.add(TribeVec2::new(4., 4.));
        w.issue_selected(TribeCommand::Build {
            kind: BuildingKind::WeaponHut,
            position,
        })
        .unwrap();
        assert_eq!(w.buildings.len(), 1);
        assert_eq!(w.tribes[0].food, before - 10.);
        assert!(!w.buildings[0].complete());
        let e = ticks(&mut w, 600);
        assert!(w.buildings[0].complete());
        assert_eq!(
            e.iter()
                .filter(|e| matches!(e, TribeEvent::BuildingCompleted { .. }))
                .count(),
            1
        );
    }
    #[test]
    fn unpaid_or_overlapping_construction_is_rejected() {
        let mut w = peaceful();
        w.tribes[0].food = 0.;
        let pos = w.tribes[0].home.add(TribeVec2::new(4., 4.));
        assert_eq!(
            w.issue_selected(TribeCommand::Build {
                kind: BuildingKind::WeaponHut,
                position: pos
            }),
            Err(TribeError::NotEnoughFood)
        );
        assert!(w.buildings.is_empty());
        w.tribes[0].food = 30.;
        assert_eq!(
            w.issue_selected(TribeCommand::Build {
                kind: BuildingKind::WeaponHut,
                position: w.tribes[0].home
            }),
            Err(TribeError::OccupiedLocation)
        );
        assert!(w
            .issue_selected(TribeCommand::Move(TribeVec2::new(f32::NAN, 0.)))
            .is_err());
    }
    #[test]
    fn tools_require_completed_huts_and_consume_food_once() {
        let mut w = one_each();
        assert_eq!(
            w.issue_selected(TribeCommand::EquipTool(ToolKind::Weapon)),
            Err(TribeError::MissingBuilding)
        );
        hut(&mut w, BuildingKind::WeaponHut);
        let initial = w.tribes[0].food;
        w.members[0].position = w.buildings[0].position.add(TribeVec2::new(-2., 0.));
        w.issue_selected(TribeCommand::EquipTool(ToolKind::Weapon))
            .unwrap();
        let e = ticks(&mut w, 200);
        assert_eq!(w.members[0].tool, Some(ToolKind::Weapon));
        assert_eq!(
            e.iter()
                .filter(|e| matches!(e, TribeEvent::ToolEquipped { .. }))
                .count(),
            1
        );
        assert!(w.tribes[0].food < initial - 3.);
        assert!(w.tribes[0].food > initial - 4.);
    }
    #[test]
    fn visible_social_cue_opposes_wrong_action_and_allies_correct_performers() {
        let mut w = peaceful();
        for m in w.members.iter_mut().filter(|m| m.tribe_id == 0) {
            m.tool = Some(ToolKind::Instrument);
        }
        at_tribe(&mut w, 1);
        let cue = w.tribes[1].social_cue;
        w.issue_selected(TribeCommand::PerformSocial {
            tribe: 1,
            performance: cue.next(),
        })
        .unwrap();
        ticks(&mut w, 150);
        assert!(w.tribes[1].relationship < 0);
        w.issue_selected(TribeCommand::PerformSocial {
            tribe: 1,
            performance: cue,
        })
        .unwrap();
        let events = ticks(&mut w, 900);
        assert!(w.tribes[1].allied());
        assert!(events
            .iter()
            .any(|e| matches!(e, TribeEvent::TribeAllied { tribe: 1 })));
        assert_eq!(w.state, TribeState::Playing);
    }
    #[test]
    fn social_commands_require_instruments() {
        let mut w = peaceful();
        assert_eq!(
            w.issue_selected(TribeCommand::PerformSocial {
                tribe: 1,
                performance: SocialPerformance::Sing
            }),
            Err(TribeError::MissingTool)
        );
    }
    #[test]
    fn combat_conquers_tribe_and_opens_civilization_gate() {
        let mut w = one_each();
        let enemy = 1;
        w.members[0].tool = Some(ToolKind::Weapon);
        w.members[enemy].health = 14.;
        w.members[0].position = w.members[enemy].position.add(TribeVec2::new(-1., 0.));
        w.issue_selected(TribeCommand::Attack(1)).unwrap();
        let e = ticks(&mut w, 2);
        assert!(!w.members[enemy].alive());
        assert!(w.tribes[1].conquered);
        assert_eq!(w.state, TribeState::CivilizationReady);
        assert!(e.contains(&TribeEvent::CivilizationReady));
    }
    #[test]
    fn unguarded_totem_can_be_conquered_with_surviving_distant_members() {
        let mut w = one_each();
        w.members[1].position = w.tribes[1].home.add(TribeVec2::new(11., 0.));
        w.members[0].position = w.tribes[1].home.add(TribeVec2::new(-2., 0.));
        w.tribes[1].totem_health = 7.;
        w.issue_selected(TribeCommand::Attack(1)).unwrap();
        ticks(&mut w, 2);
        assert!(w.members[1].alive());
        assert!(w.tribes[1].conquered);
        assert!(w.snapshot().to_bytes().is_ok());
    }
    #[test]
    fn heal_requires_hut_spends_food_and_restores_health() {
        let mut w = one_each();
        let id = w.members[0].id;
        w.members[0].health = 30.;
        assert_eq!(
            w.issue_selected(TribeCommand::Heal(id)),
            Err(TribeError::MissingBuilding)
        );
        hut(&mut w, BuildingKind::HealingHut);
        let food = w.tribes[0].food;
        w.issue_selected(TribeCommand::Heal(id)).unwrap();
        let e = ticks(&mut w, 180);
        assert!(w.members[0].health >= 50.);
        assert!(w.tribes[0].food < food - 2.);
        assert!(e.iter().any(|e| matches!(e, TribeEvent::Healed { .. })));
    }
    #[test]
    fn all_allied_or_conquered_required_and_defeat_has_priority() {
        let mut w = peaceful();
        w.tribes[1].relationship = 60;
        w.step();
        assert_eq!(w.state, TribeState::Playing);
        for m in w.members.iter_mut().filter(|m| m.tribe_id == 2) {
            m.health = 0.;
        }
        w.step();
        assert_eq!(w.state, TribeState::CivilizationReady);
        let mut defeated = peaceful();
        for t in &mut defeated.tribes[1..] {
            t.relationship = 60;
        }
        defeated.tribes[0].totem_health = 0.;
        let e = defeated.step();
        assert_eq!(defeated.state, TribeState::Defeated);
        assert!(e.contains(&TribeEvent::PlayerDefeated));
        assert!(!e.contains(&TribeEvent::CivilizationReady));
        defeated.restart();
        assert_eq!(defeated.state, TribeState::Playing);
    }
    #[test]
    fn building_and_body_collisions_keep_positions_finite_and_separated() {
        let mut w = peaceful();
        hut(&mut w, BuildingKind::GatheringHut);
        let p = w.buildings[0].position;
        w.members[0].position = p;
        w.members[1].position = p;
        w.step();
        assert!(w.members[0].position.distance(p) >= 1.89);
        assert!(w.members[1].position.distance(p) >= 1.89);
        assert!(finite(w.members[0].position));
    }
    #[test]
    fn movement_routes_around_completed_buildings() {
        let mut w = peaceful();
        hut(&mut w, BuildingKind::GatheringHut);
        let home = w.tribes[0].home;
        w.members[0].position = home.add(TribeVec2::new(0., 4.));
        let goal = home.add(TribeVec2::new(8., 4.));
        w.select(&[w.members[0].id]).unwrap();
        w.issue_selected(TribeCommand::Move(goal)).unwrap();
        ticks(&mut w, 600);
        assert!(
            w.members[0].position.distance(goal) < 0.3,
            "position {:?}",
            w.members[0].position
        );
    }
    #[test]
    fn newly_allied_tribe_stops_existing_raid_orders_immediately() {
        let mut w = peaceful();
        w.tribes[1].relationship = 59;
        let id = w.selected[0];
        w.select(&[id]).unwrap();
        w.members[0].tool = Some(ToolKind::Instrument);
        at_tribe(&mut w, 1);
        for m in w.members.iter_mut().filter(|m| m.tribe_id == 1) {
            m.order = TribeOrder::Attack(0);
        }
        w.issue_selected(TribeCommand::PerformSocial {
            tribe: 1,
            performance: w.tribes[1].social_cue,
        })
        .unwrap();
        w.members[0].work_timer = 1.999;
        w.step();
        assert!(w.tribes[1].allied());
        assert!(w
            .members
            .iter()
            .filter(|m| m.tribe_id == 1)
            .all(|m| !matches!(m.order, TribeOrder::Attack(0))));
    }
    #[test]
    fn opponent_ai_gathers_builds_equips_and_issues_raids() {
        let mut w = TribeWorld::new(27);
        let e = ticks(&mut w, 2400);
        assert!(w.buildings.iter().any(|b| b.tribe_id != 0 && b.complete()));
        assert!(e
            .iter()
            .any(|e| matches!(e,TribeEvent::FoodDelivered{tribe,..}if *tribe!=0)));
        assert!(w
            .members
            .iter()
            .any(|m| m.tribe_id != 0 && m.tool == Some(ToolKind::Weapon)));
        assert!(w
            .members
            .iter()
            .any(|m| m.tribe_id != 0 && matches!(m.order, TribeOrder::Attack(0))));
        assert!(w.snapshot().to_bytes().is_ok());
    }
    #[test]
    fn save_restores_mid_jobs_rng_tick_and_fractional_time() {
        let mut a = TribeWorld::new(99);
        let workers = a.selected.clone();
        a.select(&workers[..2]).unwrap();
        let p = a.tribes[0].home.add(TribeVec2::new(4., 4.));
        a.issue_selected(TribeCommand::Build {
            kind: BuildingKind::InstrumentHut,
            position: p,
        })
        .unwrap();
        a.select(&workers[2..]).unwrap();
        a.issue_selected(TribeCommand::GatherFood(a.food_sites[0].id))
            .unwrap();
        a.advance(0.071);
        let bytes = a.snapshot().to_bytes().unwrap();
        let mut b = TribeWorld::from_snapshot(TribeSnapshot::from_bytes(&bytes).unwrap()).unwrap();
        assert_eq!(a, b);
        for i in 0..800 {
            let dt = 0.01 + (i % 5) as f64 * 0.005;
            assert_eq!(a.advance(dt), b.advance(dt));
        }
        assert_eq!(a, b);
        assert_eq!(
            a.snapshot().to_bytes().unwrap(),
            b.snapshot().to_bytes().unwrap()
        );
    }
    #[test]
    fn malformed_binary_prefixes_versions_counts_and_trailing_data_rejected() {
        let bytes = peaceful().snapshot().to_bytes().unwrap();
        for n in 0..bytes.len() {
            assert!(
                TribeSnapshot::from_bytes(&bytes[..n]).is_err(),
                "accepted prefix {n}"
            );
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert_eq!(
            TribeSnapshot::from_bytes(&bad),
            Err(TribeError::TrailingBytes)
        );
        let mut bad = bytes.clone();
        bad[8..12].copy_from_slice(&99u32.to_le_bytes());
        assert_eq!(
            TribeSnapshot::from_bytes(&bad),
            Err(TribeError::UnsupportedVersion(99))
        );
        let mut bad = bytes.clone();
        bad[78..82].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            TribeSnapshot::from_bytes(&bad),
            Err(TribeError::TooManyEntities)
        );
        let mut bad = bytes;
        bad[12..16].copy_from_slice(&f32::NAN.to_le_bytes());
        assert_eq!(
            TribeSnapshot::from_bytes(&bad),
            Err(TribeError::InvalidConfig)
        );
    }
    #[test]
    fn invalid_snapshot_entities_references_and_ids_rejected() {
        let mut w = peaceful();
        w.members[0].health = f32::NAN;
        assert_eq!(w.snapshot().to_bytes(), Err(TribeError::InvalidEntity));
        let mut w = peaceful();
        w.members[0].order = TribeOrder::Build(999999);
        assert_eq!(w.snapshot().to_bytes(), Err(TribeError::InvalidReference));
        let mut w = peaceful();
        w.food_sites[0].id = w.members[0].id;
        assert_eq!(w.snapshot().to_bytes(), Err(TribeError::DuplicateId));
        let mut w = peaceful();
        w.selected = vec![99999];
        assert_eq!(w.snapshot().to_bytes(), Err(TribeError::InvalidReference));
    }
    #[test]
    fn elapsed_time_validation_and_backlog_are_explicit() {
        let mut w = peaceful();
        let before = w.clone();
        for time in [f64::NAN, f64::INFINITY, -1., 11.] {
            assert!(w.advance(time).time_rejected);
            assert_eq!(w, before);
        }
        let report = w.advance(10.);
        assert_eq!(report.steps, 240);
        assert!(report.backlog_seconds > 5.9);
        w.advance(0.);
        w.advance(0.);
        assert_eq!(w.tick, 600);
    }
    #[test]
    fn repeated_ai_raid_orders_do_not_erase_earned_diplomacy() {
        let mut w = peaceful();
        w.tribes[1].relationship = 30;
        let id = w.members.iter().find(|m| m.tribe_id == 1).unwrap().id;
        for _ in 0..10 {
            w.issue(1, &[id], TribeCommand::Attack(0)).unwrap();
        }
        assert_eq!(w.tribes[1].relationship, 30);
    }
    #[test]
    fn four_workers_finish_equip_and_deliver_food_without_station_deadlock() {
        let mut w = peaceful();
        let position = w.tribes[0].home.add(TribeVec2::new(4., 4.));
        w.issue_selected(TribeCommand::Build {
            kind: BuildingKind::InstrumentHut,
            position,
        })
        .unwrap();
        ticks(&mut w, 600);
        w.issue_selected(TribeCommand::EquipTool(ToolKind::Instrument))
            .unwrap();
        ticks(&mut w, 600);
        assert!(w
            .members
            .iter()
            .filter(|m| m.tribe_id == 0)
            .all(|m| m.tool == Some(ToolKind::Instrument)));
        w.issue_selected(TribeCommand::GatherFood(w.food_sites[0].id))
            .unwrap();
        let events = ticks(&mut w, 1800);
        let delivered: f32 = events
            .iter()
            .filter_map(|e| {
                if let TribeEvent::FoodDelivered { tribe: 0, amount } = e {
                    Some(*amount)
                } else {
                    None
                }
            })
            .sum();
        assert!(
            delivered >= 32.,
            "only {delivered} delivered; workers {:?}",
            w.members
                .iter()
                .filter(|m| m.tribe_id == 0)
                .map(|m| (&m.order, m.position, m.carried_food))
                .collect::<Vec<_>>()
        );
    }
}
