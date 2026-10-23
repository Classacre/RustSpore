//! Original deterministic Space-stage exploration and economy baseline.
//! Physical parameters are normalized prototype values, not scientific units or
//! recovered retail-game rules. Planets are generated only when first visited.
pub use crate::cell::CellVec2 as SpaceVec2;
use std::{collections::HashSet, fmt};
pub const SPACE_FIXED_DT: f64 = 0.1;
const DT: f32 = SPACE_FIXED_DT as f32;
const MAGIC: &[u8; 8] = b"RSSPAC01";
const MAX_SYSTEMS: usize = 64;
const MAX_PLANETS: usize = 384;
const MAX_COLONIES: usize = 64;
const CREDIT_LIMIT: u32 = 10_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceState {
    Active,
    Freeplay,
    Disabled,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Commodity {
    Ore,
    Food,
    FuelCell,
    TerraformKit,
}
impl Commodity {
    pub const ALL: [Self; 4] = [Self::Ore, Self::Food, Self::FuelCell, Self::TerraformKit];
    pub fn index(self) -> usize {
        match self {
            Self::Ore => 0,
            Self::Food => 1,
            Self::FuelCell => 2,
            Self::TerraformKit => 3,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerraformParameter {
    Atmosphere,
    Temperature,
    Water,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceLocation {
    Galaxy { system: u32 },
    System(u32),
    Planet { system: u32, planet: u32 },
}
impl SpaceLocation {
    pub fn system(self) -> u32 {
        match self {
            Self::Galaxy { system } | Self::System(system) | Self::Planet { system, .. } => system,
        }
    }
    pub fn planet(self) -> Option<u32> {
        if let Self::Planet { planet, .. } = self {
            Some(planet)
        } else {
            None
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct StarSystem {
    pub id: u32,
    pub position: SpaceVec2,
    pub planet_count: u32,
    pub visited: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceMarket {
    pub inventory: [u32; 4],
    pub prices: [u32; 4],
    pub credits: u32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SpacePlanet {
    pub id: u32,
    pub system_id: u32,
    pub ordinal: u32,
    pub atmosphere: f32,
    pub temperature: f32,
    pub water: f32,
    pub life: bool,
    pub surveyed: bool,
    pub terraform_actions: u32,
    pub ore_remaining: u32,
    pub initial_ore: u32,
    pub market: SpaceMarket,
}
impl SpacePlanet {
    pub fn suitable(&self) -> bool {
        (0.35..=0.75).contains(&self.atmosphere)
            && (0.35..=0.65).contains(&self.temperature)
            && (0.25..=0.75).contains(&self.water)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceShip {
    pub location: SpaceLocation,
    pub fuel: f32,
    pub health: f32,
    pub credits: u32,
    pub cargo: [u32; 4],
}
impl SpaceShip {
    pub fn cargo_used(&self) -> u32 {
        self.cargo.iter().sum()
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceColony {
    pub id: u32,
    pub planet_id: u32,
    pub health: f32,
    pub inventory: [u32; 4],
    pub credits: u32,
    pub production_timer: f32,
    pub maintenance_timer: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub enum MissionKind {
    ReachSystem(u32),
    SurveyPlanet(u32),
    Deliver {
        planet: u32,
        commodity: Commodity,
        amount: u32,
    },
    Terraform {
        planet: u32,
        parameter: TerraformParameter,
        min: f32,
        max: f32,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionStatus {
    Offered,
    Active,
    Claimed,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceMission {
    pub id: u32,
    pub kind: MissionKind,
    pub reward_credits: u32,
    pub status: MissionStatus,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceAiKind {
    Merchant,
    Raider,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceAi {
    pub id: u32,
    pub kind: SpaceAiKind,
    pub system_id: u32,
    pub cargo: [u32; 4],
    pub credits: u32,
    pub timer: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceConfig {
    pub system_count: usize,
    pub galaxy_radius: f32,
    pub jump_range: f32,
    pub max_fuel: f32,
    pub cargo_capacity: u32,
    pub objective_systems: usize,
    pub objective_colonies: usize,
    pub objective_missions: usize,
}
impl Default for SpaceConfig {
    fn default() -> Self {
        Self {
            system_count: 12,
            galaxy_radius: 32.,
            jump_range: 36.,
            max_fuel: 100.,
            cargo_capacity: 40,
            objective_systems: 3,
            objective_colonies: 2,
            objective_missions: 3,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum SpaceCommand {
    ViewGalaxy,
    EnterSystem,
    JumpSystem(u32),
    EnterPlanet(u32),
    LeavePlanet,
    Survey,
    AcceptMission(u32),
    ClaimMission(u32),
    Buy {
        commodity: Commodity,
        amount: u32,
    },
    Sell {
        commodity: Commodity,
        amount: u32,
    },
    Refuel(u32),
    Repair(u32),
    FoundColony,
    Terraform {
        parameter: TerraformParameter,
        delta: f32,
    },
    SeedLife,
    TransferColony {
        commodity: Commodity,
        amount: u32,
        deposit: bool,
    },
    FundColony(u32),
}
#[derive(Debug, Clone, PartialEq)]
pub enum SpaceEvent {
    Jumped {
        system: u32,
        fuel: f32,
    },
    Landed {
        planet: u32,
    },
    Surveyed {
        planet: u32,
    },
    Trade {
        commodity: Commodity,
        amount: u32,
        bought: bool,
        credits: u32,
    },
    Refueled {
        cells: u32,
        gained: f32,
    },
    Repaired {
        gained: f32,
        cost: u32,
    },
    MissionAccepted {
        mission: u32,
    },
    MissionClaimed {
        mission: u32,
        reward: u32,
    },
    ColonyFounded {
        colony: u32,
        planet: u32,
    },
    TerraformChanged {
        planet: u32,
        parameter: TerraformParameter,
        value: f32,
    },
    LifeSeeded {
        planet: u32,
    },
    LifeLost {
        planet: u32,
    },
    ColonyProduced {
        colony: u32,
        amount: u32,
    },
    ColonyMaintenance {
        colony: u32,
        supplied: bool,
    },
    AiMoved {
        actor: u32,
        system: u32,
    },
    AiTraded {
        actor: u32,
        planet: u32,
    },
    Raided {
        actor: u32,
        damage: f32,
    },
    ExpansionEstablished,
    ShipDisabled,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceWorld {
    pub config: SpaceConfig,
    pub systems: Vec<StarSystem>,
    pub visited_planets: Vec<SpacePlanet>,
    pub ship: SpaceShip,
    pub missions: Vec<SpaceMission>,
    pub colonies: Vec<SpaceColony>,
    pub ai: Vec<SpaceAi>,
    pub state: SpaceState,
    pub objective_complete: bool,
    pub tick: u64,
    seed: u64,
    rng: u64,
    next_colony_id: u32,
    accumulator: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceSnapshot {
    pub version: u32,
    pub world: SpaceWorld,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceStepReport {
    pub steps: u32,
    pub backlog_seconds: f64,
    pub interpolation: f32,
    pub state: SpaceState,
    pub events: Vec<SpaceEvent>,
    pub time_rejected: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpaceError {
    InvalidConfig,
    InvalidTarget,
    WrongLocation,
    OutOfRange,
    NotEnoughFuel,
    CargoFull,
    NotEnoughCargo,
    NotEnoughCredits,
    MarketEmpty,
    MarketBroke,
    InvalidAmount,
    UnsuitablePlanet,
    MissingLife,
    NotSurveyed,
    AlreadyColonized,
    MissionNotActive,
    MissionNotReady,
    AlreadyClaimed,
    ShipDisabled,
    InvalidSnapshot,
    InvalidEntity,
    InvalidReference,
    DuplicateId,
    TooManyEntities,
    UnsupportedVersion(u32),
    TruncatedSnapshot,
    TrailingBytes,
}
impl fmt::Display for SpaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Space stage: {self:?}")
    }
}
impl std::error::Error for SpaceError {}
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}
fn unit(x: u64) -> f32 {
    (x >> 40) as f32 / (1u32 << 24) as f32
}
fn planet_id(system: u32, ordinal: u32) -> u32 {
    (system - 1) * 8 + ordinal + 1
}
fn systems(seed: u64, config: &SpaceConfig) -> Vec<StarSystem> {
    let n = config.system_count;
    let radius = config
        .galaxy_radius
        .min(config.jump_range / (2. * (std::f32::consts::PI / n as f32).sin()) * 0.8);
    let angle_offset = unit(mix(seed)) * std::f32::consts::TAU;
    (0..n)
        .map(|i| {
            let h = mix(seed ^ (i as u64).wrapping_mul(0x1729));
            let angle = angle_offset + i as f32 / n as f32 * std::f32::consts::TAU;
            let r = radius * (0.97 + unit(h) * 0.06);
            StarSystem {
                id: i as u32 + 1,
                position: SpaceVec2::new(angle.cos() * r, angle.sin() * r),
                planet_count: 2 + (h % 4) as u32,
                visited: i == 0,
            }
        })
        .collect()
}
fn generate_planet(seed: u64, system: u32, ordinal: u32) -> SpacePlanet {
    let id = planet_id(system, ordinal);
    let h = mix(seed ^ ((id as u64) << 16));
    let (atmosphere, temperature, water) = if ordinal == 0 {
        (0.52, 0.5, 0.5)
    } else {
        (
            0.1 + unit(h) * 0.8,
            0.1 + unit(mix(h)) * 0.8,
            0.1 + unit(mix(h ^ 17)) * 0.8,
        )
    };
    let prices = [
        12 + (h % 4) as u32,
        8 + ((h >> 5) % 3) as u32,
        10 + ((h >> 10) % 4) as u32,
        25 + ((h >> 15) % 6) as u32,
    ];
    let ore = 80 + (h % 121) as u32;
    SpacePlanet {
        id,
        system_id: system,
        ordinal,
        atmosphere,
        temperature,
        water,
        life: system == 1 && ordinal == 0,
        surveyed: false,
        terraform_actions: 0,
        ore_remaining: ore,
        initial_ore: ore,
        market: SpaceMarket {
            inventory: [20, 30, 12, 10],
            prices,
            credits: 1500,
        },
    }
}
fn validate_config(c: &SpaceConfig) -> Result<(), SpaceError> {
    if !(3..=MAX_SYSTEMS).contains(&c.system_count)
        || !c.galaxy_radius.is_finite()
        || !(10.0..=1000.).contains(&c.galaxy_radius)
        || !c.jump_range.is_finite()
        || !(10.0..=2000.).contains(&c.jump_range)
        || !c.max_fuel.is_finite()
        || !(50.0..=1000.).contains(&c.max_fuel)
        || !(30..=256).contains(&c.cargo_capacity)
        || !(2..=c.system_count).contains(&c.objective_systems)
        || !(1..=c.system_count.min(8)).contains(&c.objective_colonies)
        || !(1..=4).contains(&c.objective_missions)
    {
        Err(SpaceError::InvalidConfig)
    } else {
        Ok(())
    }
}
impl SpaceWorld {
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, SpaceConfig::default()).expect("valid default Space config")
    }
    pub fn with_config(seed: u64, config: SpaceConfig) -> Result<Self, SpaceError> {
        validate_config(&config)?;
        let stars = systems(seed, &config);
        if stars.iter().enumerate().any(|(i, s)| {
            2. + s.position.distance(stars[(i + 1) % stars.len()].position) * 1.5 > config.max_fuel
        }) {
            return Err(SpaceError::InvalidConfig);
        }
        let raider_system = (config.system_count as u32).min(6);
        let fuel = config.max_fuel;
        let mut rng = mix(seed);
        if rng == 0 {
            rng = 1;
        }
        Ok(Self {
            config,
            systems: stars,
            visited_planets: vec![],
            ship: SpaceShip {
                location: SpaceLocation::System(1),
                fuel,
                health: 100.,
                credits: 700,
                cargo: [12, 10, 4, 4],
            },
            missions: vec![
                SpaceMission {
                    id: 1,
                    kind: MissionKind::ReachSystem(2),
                    reward_credits: 80,
                    status: MissionStatus::Offered,
                },
                SpaceMission {
                    id: 2,
                    kind: MissionKind::SurveyPlanet(1),
                    reward_credits: 100,
                    status: MissionStatus::Offered,
                },
                SpaceMission {
                    id: 3,
                    kind: MissionKind::Deliver {
                        planet: 9,
                        commodity: Commodity::Food,
                        amount: 3,
                    },
                    reward_credits: 150,
                    status: MissionStatus::Offered,
                },
                SpaceMission {
                    id: 4,
                    kind: MissionKind::Terraform {
                        planet: 1,
                        parameter: TerraformParameter::Atmosphere,
                        min: 0.60,
                        max: 0.70,
                    },
                    reward_credits: 120,
                    status: MissionStatus::Offered,
                },
            ],
            colonies: vec![],
            ai: vec![
                SpaceAi {
                    id: 1,
                    kind: SpaceAiKind::Merchant,
                    system_id: 1,
                    cargo: [4, 10, 8, 2],
                    credits: 200,
                    timer: 20.,
                },
                SpaceAi {
                    id: 2,
                    kind: SpaceAiKind::Raider,
                    system_id: raider_system,
                    cargo: [0, 0, 8, 0],
                    credits: 100,
                    timer: 27.,
                },
            ],
            state: SpaceState::Active,
            objective_complete: false,
            tick: 0,
            seed,
            rng,
            next_colony_id: 1,
            accumulator: 0.,
        })
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn restart(&mut self) {
        *self =
            Self::with_config(self.seed, self.config.clone()).expect("existing valid Space config");
    }
    fn random(&mut self) -> u64 {
        self.rng = mix(self.rng);
        if self.rng == 0 {
            self.rng = 1;
        }
        self.rng
    }
    fn system_index(&self, id: u32) -> Option<usize> {
        self.systems.iter().position(|s| s.id == id)
    }
    fn planet_index(&self, id: u32) -> Option<usize> {
        self.visited_planets.iter().position(|p| p.id == id)
    }
    fn valid_planet(&self, id: u32) -> bool {
        if id == 0 {
            return false;
        }
        let system = (id - 1) / 8 + 1;
        let ordinal = (id - 1) % 8;
        self.system_index(system)
            .is_some_and(|i| ordinal < self.systems[i].planet_count)
    }
    fn current_planet(&self) -> Result<usize, SpaceError> {
        self.ship
            .location
            .planet()
            .and_then(|id| self.planet_index(id))
            .ok_or(SpaceError::WrongLocation)
    }
    pub fn jump_cost(&self, target: u32) -> Result<f32, SpaceError> {
        let from = self
            .system_index(self.ship.location.system())
            .ok_or(SpaceError::InvalidTarget)?;
        let to = self.system_index(target).ok_or(SpaceError::InvalidTarget)?;
        if from == to {
            return Err(SpaceError::InvalidTarget);
        }
        let distance = self.systems[from]
            .position
            .distance(self.systems[to].position);
        if distance > self.config.jump_range {
            return Err(SpaceError::OutOfRange);
        }
        Ok(2. + distance * 1.5)
    }
    pub fn viable_colonies(&self) -> usize {
        self.colonies
            .iter()
            .filter(|c| {
                c.health > 0.
                    && self
                        .visited_planets
                        .iter()
                        .any(|p| p.id == c.planet_id && p.suitable() && p.life)
            })
            .count()
    }
    pub fn expansion_eligible(&self) -> bool {
        self.ship.health > 0.
            && self.systems.iter().filter(|s| s.visited).count() >= self.config.objective_systems
            && self.viable_colonies() >= self.config.objective_colonies
            && self
                .missions
                .iter()
                .filter(|m| m.status == MissionStatus::Claimed)
                .count()
                >= self.config.objective_missions
    }
    pub fn progress(&self) -> f32 {
        let explored = (self.systems.iter().filter(|s| s.visited).count() as f32
            / self.config.objective_systems as f32)
            .min(1.);
        let colonies =
            (self.viable_colonies() as f32 / self.config.objective_colonies as f32).min(1.);
        let missions = (self
            .missions
            .iter()
            .filter(|m| m.status == MissionStatus::Claimed)
            .count() as f32
            / self.config.objective_missions as f32)
            .min(1.);
        (explored + colonies + missions) / 3.
    }
    fn update_state(&mut self, events: &mut Vec<SpaceEvent>) {
        if !self.objective_complete && self.expansion_eligible() {
            self.objective_complete = true;
            events.push(SpaceEvent::ExpansionEstablished);
        }
        let next = if self.ship.health <= 0. {
            SpaceState::Disabled
        } else if self.objective_complete {
            SpaceState::Freeplay
        } else {
            SpaceState::Active
        };
        if next == SpaceState::Disabled && self.state != next {
            events.push(SpaceEvent::ShipDisabled);
        }
        self.state = next;
    }
    pub fn mission_ready(&self, mission: &SpaceMission) -> bool {
        if mission.status != MissionStatus::Active {
            return false;
        }
        match mission.kind {
            MissionKind::ReachSystem(id) => self.systems.iter().any(|s| s.id == id && s.visited),
            MissionKind::SurveyPlanet(id) => self
                .visited_planets
                .iter()
                .any(|p| p.id == id && p.surveyed),
            MissionKind::Deliver {
                planet,
                commodity,
                amount,
            } => {
                self.ship.location.planet() == Some(planet)
                    && self.ship.cargo[commodity.index()] >= amount
            }
            MissionKind::Terraform {
                planet,
                parameter,
                min,
                max,
            } => self.visited_planets.iter().any(|p| {
                p.id == planet
                    && p.terraform_actions > 0
                    && (min..=max).contains(&parameter_value(p, parameter))
            }),
        }
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
    fn point(&mut self, p: SpaceVec2) {
        self.float(p.x);
        self.float(p.y);
    }
    fn stock(&mut self, v: [u32; 4]) {
        for n in v {
            self.word(n);
        }
    }
    fn commodity(&mut self, v: Commodity) {
        self.byte(v.index() as u8);
    }
    fn parameter(&mut self, v: TerraformParameter) {
        self.byte(match v {
            TerraformParameter::Atmosphere => 0,
            TerraformParameter::Temperature => 1,
            TerraformParameter::Water => 2,
        });
    }
    fn location(&mut self, v: SpaceLocation) {
        match v {
            SpaceLocation::Galaxy { system } => {
                self.byte(0);
                self.word(system);
            }
            SpaceLocation::System(id) => {
                self.byte(1);
                self.word(id);
            }
            SpaceLocation::Planet { system, planet } => {
                self.byte(2);
                self.word(system);
                self.word(planet);
            }
        }
    }
    fn mission(&mut self, v: &MissionKind) {
        match *v {
            MissionKind::ReachSystem(id) => {
                self.byte(0);
                self.word(id);
            }
            MissionKind::SurveyPlanet(id) => {
                self.byte(1);
                self.word(id);
            }
            MissionKind::Deliver {
                planet,
                commodity,
                amount,
            } => {
                self.byte(2);
                self.word(planet);
                self.commodity(commodity);
                self.word(amount);
            }
            MissionKind::Terraform {
                planet,
                parameter,
                min,
                max,
            } => {
                self.byte(3);
                self.word(planet);
                self.parameter(parameter);
                self.float(min);
                self.float(max);
            }
        }
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take<const N: usize>(&mut self) -> Result<[u8; N], SpaceError> {
        let end = self
            .at
            .checked_add(N)
            .ok_or(SpaceError::TruncatedSnapshot)?;
        let out = self
            .bytes
            .get(self.at..end)
            .ok_or(SpaceError::TruncatedSnapshot)?;
        self.at = end;
        Ok(out.try_into().unwrap())
    }
    fn byte(&mut self) -> Result<u8, SpaceError> {
        Ok(self.take::<1>()?[0])
    }
    fn word(&mut self) -> Result<u32, SpaceError> {
        Ok(u32::from_le_bytes(self.take()?))
    }
    fn wide(&mut self) -> Result<u64, SpaceError> {
        Ok(u64::from_le_bytes(self.take()?))
    }
    fn float(&mut self) -> Result<f32, SpaceError> {
        Ok(f32::from_bits(self.word()?))
    }
    fn double(&mut self) -> Result<f64, SpaceError> {
        Ok(f64::from_bits(self.wide()?))
    }
    fn point(&mut self) -> Result<SpaceVec2, SpaceError> {
        Ok(SpaceVec2::new(self.float()?, self.float()?))
    }
    fn stock(&mut self) -> Result<[u32; 4], SpaceError> {
        Ok([self.word()?, self.word()?, self.word()?, self.word()?])
    }
    fn count(&mut self, max: usize) -> Result<usize, SpaceError> {
        let n = self.word()? as usize;
        if n > max {
            Err(SpaceError::TooManyEntities)
        } else {
            Ok(n)
        }
    }
    fn boolean(&mut self) -> Result<bool, SpaceError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(SpaceError::InvalidSnapshot),
        }
    }
    fn commodity(&mut self) -> Result<Commodity, SpaceError> {
        match self.byte()? {
            0 => Ok(Commodity::Ore),
            1 => Ok(Commodity::Food),
            2 => Ok(Commodity::FuelCell),
            3 => Ok(Commodity::TerraformKit),
            _ => Err(SpaceError::InvalidSnapshot),
        }
    }
    fn parameter(&mut self) -> Result<TerraformParameter, SpaceError> {
        match self.byte()? {
            0 => Ok(TerraformParameter::Atmosphere),
            1 => Ok(TerraformParameter::Temperature),
            2 => Ok(TerraformParameter::Water),
            _ => Err(SpaceError::InvalidSnapshot),
        }
    }
    fn location(&mut self) -> Result<SpaceLocation, SpaceError> {
        match self.byte()? {
            0 => Ok(SpaceLocation::Galaxy {
                system: self.word()?,
            }),
            1 => Ok(SpaceLocation::System(self.word()?)),
            2 => Ok(SpaceLocation::Planet {
                system: self.word()?,
                planet: self.word()?,
            }),
            _ => Err(SpaceError::InvalidSnapshot),
        }
    }
    fn mission(&mut self) -> Result<MissionKind, SpaceError> {
        match self.byte()? {
            0 => Ok(MissionKind::ReachSystem(self.word()?)),
            1 => Ok(MissionKind::SurveyPlanet(self.word()?)),
            2 => Ok(MissionKind::Deliver {
                planet: self.word()?,
                commodity: self.commodity()?,
                amount: self.word()?,
            }),
            3 => Ok(MissionKind::Terraform {
                planet: self.word()?,
                parameter: self.parameter()?,
                min: self.float()?,
                max: self.float()?,
            }),
            _ => Err(SpaceError::InvalidSnapshot),
        }
    }
}
impl SpaceSnapshot {
    pub fn to_bytes(&self) -> Result<Vec<u8>, SpaceError> {
        if self.version != 1 {
            return Err(SpaceError::UnsupportedVersion(self.version));
        }
        self.world.validate()?;
        let w = &self.world;
        let c = &w.config;
        let mut b = Writer(MAGIC.to_vec());
        b.word(1);
        b.word(c.system_count as u32);
        b.float(c.galaxy_radius);
        b.float(c.jump_range);
        b.float(c.max_fuel);
        b.word(c.cargo_capacity);
        b.word(c.objective_systems as u32);
        b.word(c.objective_colonies as u32);
        b.word(c.objective_missions as u32);
        b.wide(w.seed);
        b.wide(w.rng);
        b.word(w.next_colony_id);
        b.double(w.accumulator);
        b.wide(w.tick);
        b.byte(match w.state {
            SpaceState::Active => 0,
            SpaceState::Freeplay => 1,
            SpaceState::Disabled => 2,
        });
        b.byte(w.objective_complete as u8);
        b.word(w.systems.len() as u32);
        for s in &w.systems {
            b.word(s.id);
            b.point(s.position);
            b.word(s.planet_count);
            b.byte(s.visited as u8);
        }
        b.location(w.ship.location);
        b.float(w.ship.fuel);
        b.float(w.ship.health);
        b.word(w.ship.credits);
        b.stock(w.ship.cargo);
        b.word(w.visited_planets.len() as u32);
        for p in &w.visited_planets {
            b.word(p.id);
            b.word(p.system_id);
            b.word(p.ordinal);
            b.float(p.atmosphere);
            b.float(p.temperature);
            b.float(p.water);
            b.byte(p.life as u8);
            b.byte(p.surveyed as u8);
            b.word(p.terraform_actions);
            b.word(p.ore_remaining);
            b.word(p.initial_ore);
            b.stock(p.market.inventory);
            b.stock(p.market.prices);
            b.word(p.market.credits);
        }
        b.word(w.missions.len() as u32);
        for m in &w.missions {
            b.word(m.id);
            b.word(m.reward_credits);
            b.byte(match m.status {
                MissionStatus::Offered => 0,
                MissionStatus::Active => 1,
                MissionStatus::Claimed => 2,
            });
            b.mission(&m.kind);
        }
        b.word(w.colonies.len() as u32);
        for v in &w.colonies {
            b.word(v.id);
            b.word(v.planet_id);
            b.float(v.health);
            b.stock(v.inventory);
            b.word(v.credits);
            b.float(v.production_timer);
            b.float(v.maintenance_timer);
        }
        b.word(w.ai.len() as u32);
        for v in &w.ai {
            b.word(v.id);
            b.byte(match v.kind {
                SpaceAiKind::Merchant => 0,
                SpaceAiKind::Raider => 1,
            });
            b.word(v.system_id);
            b.stock(v.cargo);
            b.word(v.credits);
            b.float(v.timer);
        }
        Ok(b.0)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SpaceError> {
        if bytes.len() > 1024 * 1024 {
            return Err(SpaceError::TooManyEntities);
        }
        let mut r = Reader { bytes, at: 0 };
        if &r.take::<8>()? != MAGIC {
            return Err(SpaceError::InvalidSnapshot);
        }
        let version = r.word()?;
        if version != 1 {
            return Err(SpaceError::UnsupportedVersion(version));
        }
        let config = SpaceConfig {
            system_count: r.count(MAX_SYSTEMS)?,
            galaxy_radius: r.float()?,
            jump_range: r.float()?,
            max_fuel: r.float()?,
            cargo_capacity: r.word()?,
            objective_systems: r.count(MAX_SYSTEMS)?,
            objective_colonies: r.count(MAX_COLONIES)?,
            objective_missions: r.count(16)?,
        };
        validate_config(&config)?;
        let seed = r.wide()?;
        let rng = r.wide()?;
        let next_colony_id = r.word()?;
        let accumulator = r.double()?;
        let tick = r.wide()?;
        let state = match r.byte()? {
            0 => SpaceState::Active,
            1 => SpaceState::Freeplay,
            2 => SpaceState::Disabled,
            _ => return Err(SpaceError::InvalidSnapshot),
        };
        let objective_complete = r.boolean()?;
        let mut systems = vec![];
        for _ in 0..r.count(MAX_SYSTEMS)? {
            systems.push(StarSystem {
                id: r.word()?,
                position: r.point()?,
                planet_count: r.word()?,
                visited: r.boolean()?,
            });
        }
        let ship = SpaceShip {
            location: r.location()?,
            fuel: r.float()?,
            health: r.float()?,
            credits: r.word()?,
            cargo: r.stock()?,
        };
        let mut visited_planets = vec![];
        for _ in 0..r.count(MAX_PLANETS)? {
            visited_planets.push(SpacePlanet {
                id: r.word()?,
                system_id: r.word()?,
                ordinal: r.word()?,
                atmosphere: r.float()?,
                temperature: r.float()?,
                water: r.float()?,
                life: r.boolean()?,
                surveyed: r.boolean()?,
                terraform_actions: r.word()?,
                ore_remaining: r.word()?,
                initial_ore: r.word()?,
                market: SpaceMarket {
                    inventory: r.stock()?,
                    prices: r.stock()?,
                    credits: r.word()?,
                },
            });
        }
        let mut missions = vec![];
        for _ in 0..r.count(16)? {
            let id = r.word()?;
            let reward_credits = r.word()?;
            let status = match r.byte()? {
                0 => MissionStatus::Offered,
                1 => MissionStatus::Active,
                2 => MissionStatus::Claimed,
                _ => return Err(SpaceError::InvalidSnapshot),
            };
            missions.push(SpaceMission {
                id,
                reward_credits,
                status,
                kind: r.mission()?,
            });
        }
        let mut colonies = vec![];
        for _ in 0..r.count(MAX_COLONIES)? {
            colonies.push(SpaceColony {
                id: r.word()?,
                planet_id: r.word()?,
                health: r.float()?,
                inventory: r.stock()?,
                credits: r.word()?,
                production_timer: r.float()?,
                maintenance_timer: r.float()?,
            });
        }
        let mut ai = vec![];
        for _ in 0..r.count(16)? {
            ai.push(SpaceAi {
                id: r.word()?,
                kind: match r.byte()? {
                    0 => SpaceAiKind::Merchant,
                    1 => SpaceAiKind::Raider,
                    _ => return Err(SpaceError::InvalidSnapshot),
                },
                system_id: r.word()?,
                cargo: r.stock()?,
                credits: r.word()?,
                timer: r.float()?,
            });
        }
        if r.at != bytes.len() {
            return Err(SpaceError::TrailingBytes);
        }
        let world = SpaceWorld {
            config,
            systems,
            visited_planets,
            ship,
            missions,
            colonies,
            ai,
            state,
            objective_complete,
            tick,
            seed,
            rng,
            next_colony_id,
            accumulator,
        };
        world.validate()?;
        Ok(Self { version, world })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn home() -> SpaceWorld {
        let mut w = SpaceWorld::new(14);
        w.command(SpaceCommand::EnterPlanet(1)).unwrap();
        w.command(SpaceCommand::Survey).unwrap();
        w
    }
    fn colony() -> SpaceWorld {
        let mut w = home();
        w.command(SpaceCommand::FoundColony).unwrap();
        w
    }
    fn jump(w: &mut SpaceWorld, id: u32) {
        let cost = w.jump_cost(id).unwrap();
        while w.ship.fuel < cost {
            w.command(SpaceCommand::Refuel(1)).unwrap();
        }
        w.command(SpaceCommand::JumpSystem(id)).unwrap();
    }
    fn tour() -> SpaceWorld {
        let mut w = SpaceWorld::new(14);
        for id in 1..=4 {
            w.command(SpaceCommand::AcceptMission(id)).unwrap();
        }
        w.command(SpaceCommand::EnterPlanet(1)).unwrap();
        w.command(SpaceCommand::Survey).unwrap();
        w.command(SpaceCommand::ClaimMission(2)).unwrap();
        w.command(SpaceCommand::Terraform {
            parameter: TerraformParameter::Atmosphere,
            delta: 0.1,
        })
        .unwrap();
        w.command(SpaceCommand::ClaimMission(4)).unwrap();
        w.command(SpaceCommand::FoundColony).unwrap();
        w.command(SpaceCommand::Buy {
            commodity: Commodity::Ore,
            amount: 4,
        })
        .unwrap();
        w.command(SpaceCommand::Buy {
            commodity: Commodity::Food,
            amount: 3,
        })
        .unwrap();
        w.command(SpaceCommand::LeavePlanet).unwrap();
        jump(&mut w, 2);
        w.command(SpaceCommand::ClaimMission(1)).unwrap();
        w.command(SpaceCommand::EnterPlanet(9)).unwrap();
        w.command(SpaceCommand::Survey).unwrap();
        w.command(SpaceCommand::ClaimMission(3)).unwrap();
        w.command(SpaceCommand::SeedLife).unwrap();
        w.command(SpaceCommand::FoundColony).unwrap();
        w
    }
    #[test]
    fn seeded_systems_are_stable_connected_and_planets_start_lazy() {
        let a = SpaceWorld::new(14);
        assert_eq!(a, SpaceWorld::new(14));
        assert_ne!(a.systems, SpaceWorld::new(15).systems);
        assert!(a.visited_planets.is_empty());
        for i in 0..a.systems.len() {
            assert!(
                a.systems[i]
                    .position
                    .distance(a.systems[(i + 1) % a.systems.len()].position)
                    <= a.config.jump_range
            );
        }
        a.snapshot().to_bytes().unwrap();
        let small = SpaceWorld::with_config(
            14,
            SpaceConfig {
                system_count: 3,
                ..Default::default()
            },
        )
        .unwrap();
        small.snapshot().to_bytes().unwrap();
    }
    #[test]
    fn visited_planets_keep_trade_and_terraform_changes_on_revisit() {
        let mut w = home();
        w.command(SpaceCommand::Buy {
            commodity: Commodity::Food,
            amount: 2,
        })
        .unwrap();
        w.command(SpaceCommand::Terraform {
            parameter: TerraformParameter::Water,
            delta: 0.1,
        })
        .unwrap();
        let planet = w.visited_planets[0].clone();
        w.command(SpaceCommand::LeavePlanet).unwrap();
        w.command(SpaceCommand::EnterPlanet(1)).unwrap();
        assert_eq!(w.visited_planets.len(), 1);
        assert_eq!(w.visited_planets[0], planet);
    }
    #[test]
    fn generation_is_independent_of_visit_order_and_simulation_rng() {
        let mut a = home();
        a.advance(21.);
        a.command(SpaceCommand::LeavePlanet).unwrap();
        a.command(SpaceCommand::EnterPlanet(2)).unwrap();
        let mut b = SpaceWorld::new(14);
        b.command(SpaceCommand::EnterPlanet(2)).unwrap();
        assert_eq!(
            a.visited_planets.iter().find(|p| p.id == 2).unwrap(),
            &b.visited_planets[0]
        );
    }
    #[test]
    fn galaxy_system_and_planet_location_gates_are_enforced() {
        let mut w = home();
        let before = w.clone();
        assert_eq!(
            w.command(SpaceCommand::JumpSystem(2)),
            Err(SpaceError::WrongLocation)
        );
        assert_eq!(w, before);
        w.command(SpaceCommand::LeavePlanet).unwrap();
        w.command(SpaceCommand::ViewGalaxy).unwrap();
        assert_eq!(
            w.command(SpaceCommand::EnterPlanet(1)),
            Err(SpaceError::WrongLocation)
        );
        w.command(SpaceCommand::EnterSystem).unwrap();
        w.command(SpaceCommand::EnterPlanet(1)).unwrap();
        assert_eq!(w.ship.fuel, 93.);
    }
    #[test]
    fn jump_range_and_fuel_failures_are_transactional() {
        let mut w = SpaceWorld::new(14);
        let before = w.clone();
        assert_eq!(
            w.command(SpaceCommand::JumpSystem(7)),
            Err(SpaceError::OutOfRange)
        );
        assert_eq!(w, before);
        w.ship.fuel = 1.;
        let before = w.clone();
        assert_eq!(
            w.command(SpaceCommand::JumpSystem(2)),
            Err(SpaceError::NotEnoughFuel)
        );
        assert_eq!(w, before);
        assert_eq!(
            w.command(SpaceCommand::JumpSystem(1)),
            Err(SpaceError::InvalidTarget)
        );
        assert_eq!(
            w.command(SpaceCommand::JumpSystem(999)),
            Err(SpaceError::InvalidTarget)
        );
    }
    #[test]
    fn fuel_cells_are_consumed_and_never_overfill() {
        let mut w = SpaceWorld::new(14);
        w.ship.fuel = 1.;
        w.command(SpaceCommand::Refuel(2)).unwrap();
        assert_eq!(w.ship.cargo[2], 2);
        assert_eq!(w.ship.fuel, 41.);
        let cost = w.jump_cost(2).unwrap();
        w.command(SpaceCommand::JumpSystem(2)).unwrap();
        assert!((w.ship.fuel - (41. - cost)).abs() < 1e-5);
        let before = w.clone();
        assert_eq!(
            w.command(SpaceCommand::Refuel(3)),
            Err(SpaceError::NotEnoughCargo)
        );
        assert_eq!(w, before);
    }
    #[test]
    fn trade_conserves_goods_and_both_sides_credits() {
        let mut w = home();
        let initial = w.clone();
        let credits = w.ship.credits + w.visited_planets[0].market.credits;
        let food = w.ship.cargo[1] + w.visited_planets[0].market.inventory[1];
        w.command(SpaceCommand::Buy {
            commodity: Commodity::Food,
            amount: 2,
        })
        .unwrap();
        assert_eq!(
            w.ship.credits + w.visited_planets[0].market.credits,
            credits
        );
        assert_eq!(
            w.ship.cargo[1] + w.visited_planets[0].market.inventory[1],
            food
        );
        w.command(SpaceCommand::Sell {
            commodity: Commodity::Food,
            amount: 2,
        })
        .unwrap();
        assert_eq!(w, initial);
    }
    #[test]
    fn finite_market_bank_stock_and_cargo_capacity_limit_trading() {
        let mut w = home();
        w.command(SpaceCommand::Buy {
            commodity: Commodity::Ore,
            amount: 10,
        })
        .unwrap();
        let before = w.clone();
        assert_eq!(
            w.command(SpaceCommand::Buy {
                commodity: Commodity::Food,
                amount: 1
            }),
            Err(SpaceError::CargoFull)
        );
        assert_eq!(w, before);
        let mut w = home();
        w.visited_planets[0].market.inventory[0] = 0;
        assert_eq!(
            w.command(SpaceCommand::Buy {
                commodity: Commodity::Ore,
                amount: 1
            }),
            Err(SpaceError::MarketEmpty)
        );
        w.visited_planets[0].market.credits = 0;
        assert_eq!(
            w.command(SpaceCommand::Sell {
                commodity: Commodity::Food,
                amount: 1
            }),
            Err(SpaceError::MarketBroke)
        );
        for amount in [0, u32::MAX] {
            let before = w.clone();
            assert_eq!(
                w.command(SpaceCommand::Buy {
                    commodity: Commodity::Ore,
                    amount
                }),
                Err(SpaceError::InvalidAmount)
            );
            assert_eq!(w, before);
        }
    }
    #[test]
    fn missions_require_real_objectives_and_reward_exactly_once() {
        let mut w = SpaceWorld::new(14);
        w.command(SpaceCommand::AcceptMission(2)).unwrap();
        assert_eq!(
            w.command(SpaceCommand::ClaimMission(2)),
            Err(SpaceError::MissionNotReady)
        );
        w.command(SpaceCommand::EnterPlanet(1)).unwrap();
        w.command(SpaceCommand::Survey).unwrap();
        let credits = w.ship.credits;
        w.command(SpaceCommand::ClaimMission(2)).unwrap();
        assert_eq!(w.ship.credits, credits + 100);
        let before = w.clone();
        assert_eq!(
            w.command(SpaceCommand::ClaimMission(2)),
            Err(SpaceError::AlreadyClaimed)
        );
        assert_eq!(w, before);
        assert!(w.command(SpaceCommand::Survey).unwrap().is_empty());
    }
    #[test]
    fn delivery_requires_destination_and_consumes_goods_once() {
        let mut w = home();
        w.command(SpaceCommand::AcceptMission(3)).unwrap();
        assert_eq!(
            w.command(SpaceCommand::ClaimMission(3)),
            Err(SpaceError::MissionNotReady)
        );
        w.command(SpaceCommand::LeavePlanet).unwrap();
        jump(&mut w, 2);
        w.command(SpaceCommand::EnterPlanet(9)).unwrap();
        let food = w.ship.cargo[1];
        let credits = w.ship.credits;
        w.command(SpaceCommand::ClaimMission(3)).unwrap();
        assert_eq!(w.ship.cargo[1], food - 3);
        assert_eq!(w.ship.credits, credits + 150);
        assert_eq!(
            w.command(SpaceCommand::ClaimMission(3)),
            Err(SpaceError::AlreadyClaimed)
        );
    }
    #[test]
    fn terraforming_is_bounded_costed_and_completes_measured_goal() {
        let mut w = home();
        w.command(SpaceCommand::AcceptMission(4)).unwrap();
        assert!(!w.mission_ready(&w.missions[3]));
        let credits = w.ship.credits;
        let kits = w.ship.cargo[3];
        w.command(SpaceCommand::Terraform {
            parameter: TerraformParameter::Atmosphere,
            delta: 0.1,
        })
        .unwrap();
        assert!((w.visited_planets[0].atmosphere - 0.62).abs() < 1e-5);
        assert_eq!(w.ship.credits, credits - 20);
        assert_eq!(w.ship.cargo[3], kits - 1);
        assert!(w.mission_ready(&w.missions[3]));
        for delta in [f32::NAN, f32::INFINITY, 0., 0.2, -0.2] {
            let before = w.clone();
            assert_eq!(
                w.command(SpaceCommand::Terraform {
                    parameter: TerraformParameter::Water,
                    delta
                }),
                Err(SpaceError::InvalidAmount)
            );
            assert_eq!(w, before);
        }
    }
    #[test]
    fn colony_needs_survey_life_and_paid_materials() {
        let mut w = SpaceWorld::new(14);
        jump(&mut w, 2);
        w.command(SpaceCommand::EnterPlanet(9)).unwrap();
        assert_eq!(
            w.command(SpaceCommand::FoundColony),
            Err(SpaceError::NotSurveyed)
        );
        w.command(SpaceCommand::Survey).unwrap();
        assert_eq!(
            w.command(SpaceCommand::FoundColony),
            Err(SpaceError::MissingLife)
        );
        let credits = w.ship.credits;
        w.command(SpaceCommand::SeedLife).unwrap();
        assert_eq!(w.ship.cargo[1], 8);
        w.command(SpaceCommand::FoundColony).unwrap();
        assert_eq!(w.ship.credits, credits - 25 - 150);
        assert_eq!(w.ship.cargo[0], 4);
        assert_eq!(w.ship.cargo[1], 4);
        assert_eq!(w.colonies[0].inventory[1], 4);
        assert_eq!(
            w.command(SpaceCommand::FoundColony),
            Err(SpaceError::AlreadyColonized)
        );
    }
    #[test]
    fn colony_production_uses_finite_deposit_and_maintenance_supplies() {
        let mut w = colony();
        w.visited_planets[0].ore_remaining = 1;
        let credits = w.colonies[0].credits;
        let events = w.advance(41.).events;
        assert_eq!(w.visited_planets[0].ore_remaining, 0);
        assert_eq!(w.colonies[0].inventory[0], 1);
        assert_eq!(w.colonies[0].inventory[1], 3);
        assert_eq!(w.colonies[0].credits, credits - 3);
        assert!(events
            .iter()
            .any(|e| matches!(e, SpaceEvent::ColonyProduced { amount: 1, .. })));
        w.advance(40.);
        assert_eq!(w.colonies[0].inventory[0], 1);
    }
    #[test]
    fn maintenance_failure_hurts_and_cargo_funding_restores_supplies() {
        let mut w = colony();
        w.colonies[0].inventory[1] = 0;
        w.colonies[0].credits = 0;
        w.advance(31.);
        assert_eq!(w.colonies[0].health, 92.);
        w.command(SpaceCommand::TransferColony {
            commodity: Commodity::Food,
            amount: 2,
            deposit: true,
        })
        .unwrap();
        w.command(SpaceCommand::FundColony(10)).unwrap();
        w.advance(31.);
        assert_eq!(w.colonies[0].health, 92.);
        assert_eq!(w.colonies[0].inventory[1], 1);
        assert_eq!(w.colonies[0].credits, 7);
    }
    #[test]
    fn loss_of_suitability_extinguishes_life_and_halts_production() {
        let mut w = colony();
        w.command(SpaceCommand::Terraform {
            parameter: TerraformParameter::Water,
            delta: 0.15,
        })
        .unwrap();
        let events = w
            .command(SpaceCommand::Terraform {
                parameter: TerraformParameter::Water,
                delta: 0.15,
            })
            .unwrap();
        assert!(!w.visited_planets[0].life);
        assert!(events.contains(&SpaceEvent::LifeLost { planet: 1 }));
        assert_eq!(
            w.command(SpaceCommand::SeedLife),
            Err(SpaceError::UnsuitablePlanet)
        );
        let ore = w.visited_planets[0].ore_remaining;
        w.advance(31.);
        assert_eq!(w.visited_planets[0].ore_remaining, ore);
        assert_eq!(w.colonies[0].health, 92.);
        w.command(SpaceCommand::Terraform {
            parameter: TerraformParameter::Water,
            delta: -0.15,
        })
        .unwrap();
        w.command(SpaceCommand::SeedLife).unwrap();
        assert!(w.visited_planets[0].life);
    }
    #[test]
    fn simulation_merchants_conserve_trade_resources_and_spend_travel_fuel() {
        let mut w = home();
        let total_credits = w.ship.credits
            + w.visited_planets[0].market.credits
            + w.ai.iter().map(|a| a.credits).sum::<u32>();
        let total_food = w.ship.cargo[1]
            + w.visited_planets[0].market.inventory[1]
            + w.ai.iter().map(|a| a.cargo[1]).sum::<u32>();
        w.ai[0].timer = 0.;
        let fuel = w.ai[0].cargo[2];
        let events = w.step();
        assert_eq!(
            w.ship.credits
                + w.visited_planets[0].market.credits
                + w.ai.iter().map(|a| a.credits).sum::<u32>(),
            total_credits
        );
        assert_eq!(
            w.ship.cargo[1]
                + w.visited_planets[0].market.inventory[1]
                + w.ai.iter().map(|a| a.cargo[1]).sum::<u32>(),
            total_food
        );
        assert_eq!(w.ai[0].cargo[2], fuel - 1);
        assert!(events
            .iter()
            .any(|e| matches!(e, SpaceEvent::AiTraded { .. })));
    }
    #[test]
    fn raiders_damage_in_space_and_station_repairs_cost_credits() {
        let mut w = SpaceWorld::new(14);
        for id in 2..=6 {
            jump(&mut w, id);
        }
        let events = w.advance(28.).events;
        assert!(events
            .iter()
            .any(|e| matches!(e, SpaceEvent::Raided { actor: 2, .. })));
        assert_eq!(w.ship.health, 92.);
        if w.ship.fuel < 3. {
            w.command(SpaceCommand::Refuel(1)).unwrap();
        }
        w.command(SpaceCommand::EnterPlanet(41)).unwrap();
        let credits = w.ship.credits;
        w.command(SpaceCommand::Repair(8)).unwrap();
        assert_eq!(w.ship.health, 100.);
        assert_eq!(w.ship.credits, credits - 16);
    }
    #[test]
    fn real_exploration_colonization_and_mission_gates_unlock_continuing_freeplay() {
        let mut w = tour();
        assert_eq!(w.state, SpaceState::Active);
        assert!(!w.objective_complete);
        w.command(SpaceCommand::LeavePlanet).unwrap();
        jump(&mut w, 3);
        assert_eq!(w.state, SpaceState::Freeplay);
        assert!(w.objective_complete);
        assert_eq!(w.viable_colonies(), 2);
        assert_eq!(
            w.missions
                .iter()
                .filter(|m| m.status == MissionStatus::Claimed)
                .count(),
            4
        );
        let tick = w.tick;
        w.advance(10.);
        assert!(w.tick > tick);
        assert_eq!(w.state, SpaceState::Freeplay);
        w.command(SpaceCommand::ViewGalaxy).unwrap();
        w.snapshot().to_bytes().unwrap();
    }
    #[test]
    fn binary_roundtrip_keeps_rng_jobs_time_markets_and_future_generation() {
        let mut a = colony();
        a.command(SpaceCommand::AcceptMission(3)).unwrap();
        a.command(SpaceCommand::Buy {
            commodity: Commodity::Food,
            amount: 1,
        })
        .unwrap();
        a.advance(17.35);
        let bytes = a.snapshot().to_bytes().unwrap();
        let mut b = SpaceWorld::from_snapshot(SpaceSnapshot::from_bytes(&bytes).unwrap()).unwrap();
        assert_eq!(a, b);
        for i in 0..400 {
            let dt = 0.05 + (i % 5) as f64 * 0.05;
            assert_eq!(a.advance(dt), b.advance(dt));
        }
        for w in [&mut a, &mut b] {
            w.command(SpaceCommand::LeavePlanet).unwrap();
            jump(w, 2);
            w.command(SpaceCommand::EnterPlanet(9)).unwrap();
        }
        assert_eq!(a, b);
        assert_eq!(
            a.snapshot().to_bytes().unwrap(),
            b.snapshot().to_bytes().unwrap()
        );
    }
    #[test]
    fn malformed_binary_prefixes_versions_counts_nan_and_trailing_rejected() {
        let w = SpaceWorld::new(14);
        let bytes = w.snapshot().to_bytes().unwrap();
        for n in 0..bytes.len() {
            assert!(
                SpaceSnapshot::from_bytes(&bytes[..n]).is_err(),
                "accepted prefix{n}"
            );
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert_eq!(
            SpaceSnapshot::from_bytes(&bad),
            Err(SpaceError::TrailingBytes)
        );
        let mut bad = bytes.clone();
        bad[8..12].copy_from_slice(&99u32.to_le_bytes());
        assert_eq!(
            SpaceSnapshot::from_bytes(&bad),
            Err(SpaceError::UnsupportedVersion(99))
        );
        let mut bad = bytes.clone();
        bad[82..86].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            SpaceSnapshot::from_bytes(&bad),
            Err(SpaceError::TooManyEntities)
        );
        let mut bad = bytes.clone();
        bad[16..20].copy_from_slice(&f32::NAN.to_le_bytes());
        assert_eq!(
            SpaceSnapshot::from_bytes(&bad),
            Err(SpaceError::InvalidConfig)
        );
        let mut bad = bytes;
        let fuel = 86 + w.systems.len() * 17 + 5;
        bad[fuel..fuel + 4].copy_from_slice(&f32::NAN.to_le_bytes());
        assert_eq!(
            SpaceSnapshot::from_bytes(&bad),
            Err(SpaceError::InvalidEntity)
        );
    }
    #[test]
    fn snapshot_ids_references_and_cargo_overflow_are_rejected_without_panic() {
        let mut w = home();
        w.visited_planets.push(w.visited_planets[0].clone());
        assert_eq!(w.snapshot().to_bytes(), Err(SpaceError::DuplicateId));
        let mut w = SpaceWorld::new(14);
        w.ship.cargo = [u32::MAX; 4];
        assert_eq!(w.snapshot().to_bytes(), Err(SpaceError::InvalidEntity));
        let mut w = colony();
        w.colonies[0].planet_id = 999;
        assert_eq!(w.snapshot().to_bytes(), Err(SpaceError::InvalidReference));
        let mut w = home();
        w.missions[0].kind = MissionKind::ReachSystem(999);
        assert_eq!(w.snapshot().to_bytes(), Err(SpaceError::InvalidReference));
    }
    #[test]
    fn invalid_elapsed_time_is_rejected_and_restart_restores_generation() {
        let mut w = home();
        let before = w.clone();
        for seconds in [f64::NAN, f64::INFINITY, -1., 61.] {
            assert!(w.advance(seconds).time_rejected);
            assert_eq!(w, before);
        }
        w.advance(60.);
        assert_eq!(w.tick, 600);
        w.restart();
        assert_eq!(w, SpaceWorld::new(14));
    }
}
fn parameter_value(p: &SpacePlanet, parameter: TerraformParameter) -> f32 {
    match parameter {
        TerraformParameter::Atmosphere => p.atmosphere,
        TerraformParameter::Temperature => p.temperature,
        TerraformParameter::Water => p.water,
    }
}

impl SpaceWorld {
    pub fn command(&mut self, command: SpaceCommand) -> Result<Vec<SpaceEvent>, SpaceError> {
        if self.state == SpaceState::Disabled && !matches!(command, SpaceCommand::Repair(_)) {
            return Err(SpaceError::ShipDisabled);
        }
        let mut events = vec![];
        match command {
            SpaceCommand::ViewGalaxy => {
                if self.ship.location.planet().is_some() {
                    return Err(SpaceError::WrongLocation);
                }
                self.ship.location = SpaceLocation::Galaxy {
                    system: self.ship.location.system(),
                };
            }
            SpaceCommand::EnterSystem => {
                if self.ship.location.planet().is_some() {
                    return Err(SpaceError::WrongLocation);
                }
                self.ship.location = SpaceLocation::System(self.ship.location.system());
            }
            SpaceCommand::JumpSystem(id) => {
                if self.ship.location.planet().is_some() {
                    return Err(SpaceError::WrongLocation);
                }
                let cost = self.jump_cost(id)?;
                if self.ship.fuel < cost {
                    return Err(SpaceError::NotEnoughFuel);
                }
                let target = self.system_index(id).unwrap();
                self.ship.fuel -= cost;
                self.ship.location = SpaceLocation::System(id);
                self.systems[target].visited = true;
                events.push(SpaceEvent::Jumped {
                    system: id,
                    fuel: cost,
                });
            }
            SpaceCommand::EnterPlanet(id) => {
                let SpaceLocation::System(system) = self.ship.location else {
                    return Err(SpaceError::WrongLocation);
                };
                if !self.valid_planet(id) || (id - 1) / 8 + 1 != system {
                    return Err(SpaceError::InvalidTarget);
                }
                if self.ship.fuel < 3. {
                    return Err(SpaceError::NotEnoughFuel);
                }
                if self.planet_index(id).is_none() {
                    if self.visited_planets.len() >= MAX_PLANETS {
                        return Err(SpaceError::TooManyEntities);
                    }
                    self.visited_planets
                        .push(generate_planet(self.seed, system, (id - 1) % 8));
                }
                self.ship.fuel -= 3.;
                self.ship.location = SpaceLocation::Planet { system, planet: id };
                events.push(SpaceEvent::Landed { planet: id });
            }
            SpaceCommand::LeavePlanet => {
                let SpaceLocation::Planet { system, .. } = self.ship.location else {
                    return Err(SpaceError::WrongLocation);
                };
                if self.ship.fuel < 1. {
                    return Err(SpaceError::NotEnoughFuel);
                }
                self.ship.fuel -= 1.;
                self.ship.location = SpaceLocation::System(system);
            }
            SpaceCommand::Survey => {
                let i = self.current_planet()?;
                if !self.visited_planets[i].surveyed {
                    self.visited_planets[i].surveyed = true;
                    events.push(SpaceEvent::Surveyed {
                        planet: self.visited_planets[i].id,
                    });
                }
            }
            SpaceCommand::AcceptMission(id) => {
                let m = self
                    .missions
                    .iter_mut()
                    .find(|m| m.id == id)
                    .ok_or(SpaceError::InvalidTarget)?;
                if m.status == MissionStatus::Claimed {
                    return Err(SpaceError::AlreadyClaimed);
                }
                if m.status != MissionStatus::Offered {
                    return Err(SpaceError::MissionNotActive);
                }
                m.status = MissionStatus::Active;
                events.push(SpaceEvent::MissionAccepted { mission: id });
            }
            SpaceCommand::ClaimMission(id) => {
                let i = self
                    .missions
                    .iter()
                    .position(|m| m.id == id)
                    .ok_or(SpaceError::InvalidTarget)?;
                let m = self.missions[i].clone();
                if m.status == MissionStatus::Claimed {
                    return Err(SpaceError::AlreadyClaimed);
                }
                if m.status != MissionStatus::Active {
                    return Err(SpaceError::MissionNotActive);
                }
                if !self.mission_ready(&m) {
                    return Err(SpaceError::MissionNotReady);
                }
                let credits = self
                    .ship
                    .credits
                    .checked_add(m.reward_credits)
                    .filter(|v| *v <= CREDIT_LIMIT)
                    .ok_or(SpaceError::InvalidAmount)?;
                if let MissionKind::Deliver {
                    commodity, amount, ..
                } = m.kind
                {
                    self.ship.cargo[commodity.index()] -= amount;
                }
                self.ship.credits = credits;
                self.missions[i].status = MissionStatus::Claimed;
                events.push(SpaceEvent::MissionClaimed {
                    mission: id,
                    reward: m.reward_credits,
                });
            }
            SpaceCommand::Buy { commodity, amount } => {
                valid_amount(amount)?;
                let i = self.current_planet()?;
                let c = commodity.index();
                let market = &self.visited_planets[i].market;
                if market.inventory[c] < amount {
                    return Err(SpaceError::MarketEmpty);
                }
                if self
                    .ship
                    .cargo_used()
                    .checked_add(amount)
                    .is_none_or(|n| n > self.config.cargo_capacity)
                {
                    return Err(SpaceError::CargoFull);
                }
                let cost = amount
                    .checked_mul(market.prices[c])
                    .ok_or(SpaceError::InvalidAmount)?;
                if self.ship.credits < cost {
                    return Err(SpaceError::NotEnoughCredits);
                }
                let bank = market
                    .credits
                    .checked_add(cost)
                    .filter(|n| *n <= CREDIT_LIMIT)
                    .ok_or(SpaceError::InvalidAmount)?;
                self.ship.credits -= cost;
                self.ship.cargo[c] += amount;
                self.visited_planets[i].market.inventory[c] -= amount;
                self.visited_planets[i].market.credits = bank;
                events.push(SpaceEvent::Trade {
                    commodity,
                    amount,
                    bought: true,
                    credits: cost,
                });
            }
            SpaceCommand::Sell { commodity, amount } => {
                valid_amount(amount)?;
                let i = self.current_planet()?;
                let c = commodity.index();
                if self.ship.cargo[c] < amount {
                    return Err(SpaceError::NotEnoughCargo);
                }
                let market = &self.visited_planets[i].market;
                if market.inventory[c]
                    .checked_add(amount)
                    .is_none_or(|n| n > 10000)
                {
                    return Err(SpaceError::MarketEmpty);
                }
                let price = amount
                    .checked_mul(market.prices[c])
                    .ok_or(SpaceError::InvalidAmount)?;
                if market.credits < price {
                    return Err(SpaceError::MarketBroke);
                }
                let credits = self
                    .ship
                    .credits
                    .checked_add(price)
                    .filter(|v| *v <= CREDIT_LIMIT)
                    .ok_or(SpaceError::InvalidAmount)?;
                self.ship.cargo[c] -= amount;
                self.ship.credits = credits;
                self.visited_planets[i].market.inventory[c] += amount;
                self.visited_planets[i].market.credits -= price;
                events.push(SpaceEvent::Trade {
                    commodity,
                    amount,
                    bought: false,
                    credits: price,
                });
            }
            SpaceCommand::Refuel(cells) => {
                valid_amount(cells)?;
                let c = Commodity::FuelCell.index();
                if self.ship.cargo[c] < cells {
                    return Err(SpaceError::NotEnoughCargo);
                }
                if self.ship.fuel >= self.config.max_fuel {
                    return Err(SpaceError::InvalidAmount);
                }
                let gained = (cells as f32 * 20.).min(self.config.max_fuel - self.ship.fuel);
                self.ship.cargo[c] -= cells;
                self.ship.fuel += gained;
                events.push(SpaceEvent::Refueled { cells, gained });
            }
            SpaceCommand::Repair(points) => {
                valid_amount(points)?;
                let i = self.current_planet()?;
                if self.ship.health >= 100. {
                    return Err(SpaceError::InvalidAmount);
                }
                let gained = (points as f32).min(100. - self.ship.health);
                let cost = (gained * 2.).ceil() as u32;
                if self.ship.credits < cost {
                    return Err(SpaceError::NotEnoughCredits);
                }
                let bank = self.visited_planets[i]
                    .market
                    .credits
                    .checked_add(cost)
                    .filter(|n| *n <= CREDIT_LIMIT)
                    .ok_or(SpaceError::InvalidAmount)?;
                self.ship.credits -= cost;
                self.visited_planets[i].market.credits = bank;
                self.ship.health += gained;
                events.push(SpaceEvent::Repaired { gained, cost });
            }
            SpaceCommand::FoundColony => {
                let i = self.current_planet()?;
                let p = &self.visited_planets[i];
                if !p.surveyed {
                    return Err(SpaceError::NotSurveyed);
                }
                if !p.suitable() {
                    return Err(SpaceError::UnsuitablePlanet);
                }
                if !p.life {
                    return Err(SpaceError::MissingLife);
                }
                if self.colonies.iter().any(|c| c.planet_id == p.id) {
                    return Err(SpaceError::AlreadyColonized);
                }
                if self.colonies.len() >= MAX_COLONIES {
                    return Err(SpaceError::TooManyEntities);
                }
                if self.ship.credits < 150 {
                    return Err(SpaceError::NotEnoughCredits);
                }
                if self.ship.cargo[0] < 8 || self.ship.cargo[1] < 4 {
                    return Err(SpaceError::NotEnoughCargo);
                }
                let next = self
                    .next_colony_id
                    .checked_add(1)
                    .ok_or(SpaceError::TooManyEntities)?;
                self.ship.credits -= 150;
                self.ship.cargo[0] -= 8;
                self.ship.cargo[1] -= 4;
                let id = self.next_colony_id;
                self.next_colony_id = next;
                self.colonies.push(SpaceColony {
                    id,
                    planet_id: p.id,
                    health: 100.,
                    inventory: [0, 4, 0, 0],
                    credits: 100,
                    production_timer: 0.,
                    maintenance_timer: 0.,
                });
                events.push(SpaceEvent::ColonyFounded {
                    colony: id,
                    planet: p.id,
                });
            }
            SpaceCommand::Terraform { parameter, delta } => {
                if !delta.is_finite() || delta == 0. || delta.abs() > 0.15 {
                    return Err(SpaceError::InvalidAmount);
                }
                let i = self.current_planet()?;
                let value = parameter_value(&self.visited_planets[i], parameter) + delta;
                if !(0.0..=1.).contains(&value) {
                    return Err(SpaceError::InvalidAmount);
                }
                if self.ship.credits < 20 {
                    return Err(SpaceError::NotEnoughCredits);
                }
                if self.ship.cargo[3] < 1 {
                    return Err(SpaceError::NotEnoughCargo);
                }
                let actions = self.visited_planets[i]
                    .terraform_actions
                    .checked_add(1)
                    .ok_or(SpaceError::InvalidAmount)?;
                self.ship.credits -= 20;
                self.ship.cargo[3] -= 1;
                let p = &mut self.visited_planets[i];
                match parameter {
                    TerraformParameter::Atmosphere => p.atmosphere = value,
                    TerraformParameter::Temperature => p.temperature = value,
                    TerraformParameter::Water => p.water = value,
                }
                p.terraform_actions = actions;
                if p.life && !p.suitable() {
                    p.life = false;
                    events.push(SpaceEvent::LifeLost { planet: p.id });
                }
                events.push(SpaceEvent::TerraformChanged {
                    planet: p.id,
                    parameter,
                    value,
                });
            }
            SpaceCommand::SeedLife => {
                let i = self.current_planet()?;
                let p = &self.visited_planets[i];
                if p.life {
                    return Err(SpaceError::InvalidTarget);
                }
                if !p.suitable() {
                    return Err(SpaceError::UnsuitablePlanet);
                }
                if self.ship.credits < 25 {
                    return Err(SpaceError::NotEnoughCredits);
                }
                if self.ship.cargo[1] < 2 {
                    return Err(SpaceError::NotEnoughCargo);
                }
                self.ship.credits -= 25;
                self.ship.cargo[1] -= 2;
                self.visited_planets[i].life = true;
                events.push(SpaceEvent::LifeSeeded {
                    planet: self.visited_planets[i].id,
                });
            }
            SpaceCommand::TransferColony {
                commodity,
                amount,
                deposit,
            } => {
                valid_amount(amount)?;
                let p = self.current_planet()?;
                let planet = self.visited_planets[p].id;
                let i = self
                    .colonies
                    .iter()
                    .position(|c| c.planet_id == planet && c.health > 0.)
                    .ok_or(SpaceError::InvalidTarget)?;
                let c = commodity.index();
                if deposit {
                    if self.ship.cargo[c] < amount {
                        return Err(SpaceError::NotEnoughCargo);
                    }
                    if self.colonies[i]
                        .inventory
                        .iter()
                        .sum::<u32>()
                        .checked_add(amount)
                        .is_none_or(|n| n > 128)
                    {
                        return Err(SpaceError::CargoFull);
                    }
                    self.ship.cargo[c] -= amount;
                    self.colonies[i].inventory[c] += amount;
                } else {
                    if self.colonies[i].inventory[c] < amount {
                        return Err(SpaceError::NotEnoughCargo);
                    }
                    if self
                        .ship
                        .cargo_used()
                        .checked_add(amount)
                        .is_none_or(|n| n > self.config.cargo_capacity)
                    {
                        return Err(SpaceError::CargoFull);
                    }
                    self.colonies[i].inventory[c] -= amount;
                    self.ship.cargo[c] += amount;
                }
            }
            SpaceCommand::FundColony(amount) => {
                valid_amount(amount)?;
                let p = self.current_planet()?;
                let planet = self.visited_planets[p].id;
                let i = self
                    .colonies
                    .iter()
                    .position(|c| c.planet_id == planet && c.health > 0.)
                    .ok_or(SpaceError::InvalidTarget)?;
                if self.ship.credits < amount {
                    return Err(SpaceError::NotEnoughCredits);
                }
                let bank = self.colonies[i]
                    .credits
                    .checked_add(amount)
                    .filter(|n| *n <= CREDIT_LIMIT)
                    .ok_or(SpaceError::InvalidAmount)?;
                self.ship.credits -= amount;
                self.colonies[i].credits = bank;
            }
        }
        self.update_state(&mut events);
        Ok(events)
    }
}
fn valid_amount(amount: u32) -> Result<(), SpaceError> {
    if amount == 0 || amount > 10000 {
        Err(SpaceError::InvalidAmount)
    } else {
        Ok(())
    }
}

impl SpaceWorld {
    pub fn advance(&mut self, seconds: f64) -> SpaceStepReport {
        let rejected = !seconds.is_finite()
            || seconds < 0.
            || seconds > 60.
            || self.accumulator + seconds > 600.;
        let mut events = vec![];
        let mut steps = 0;
        if !rejected {
            self.accumulator += seconds;
            while self.accumulator + 1e-10 >= SPACE_FIXED_DT && steps < 600 {
                self.accumulator = (self.accumulator - SPACE_FIXED_DT).max(0.);
                events.extend(self.step());
                steps += 1;
            }
        }
        SpaceStepReport {
            steps,
            backlog_seconds: self.accumulator,
            interpolation: (self.accumulator / SPACE_FIXED_DT).min(1.) as f32,
            state: self.state,
            events,
            time_rejected: rejected,
        }
    }
    pub fn step(&mut self) -> Vec<SpaceEvent> {
        let mut events = vec![];
        if self.state == SpaceState::Disabled {
            return events;
        }
        self.tick = self.tick.saturating_add(1);
        for i in 0..self.colonies.len() {
            if self.colonies[i].health <= 0. {
                continue;
            }
            let Some(pi) = self.planet_index(self.colonies[i].planet_id) else {
                continue;
            };
            let suitable = self.visited_planets[pi].suitable() && self.visited_planets[pi].life;
            self.colonies[i].production_timer += DT;
            self.colonies[i].maintenance_timer += DT;
            if self.colonies[i].production_timer >= 20. {
                self.colonies[i].production_timer -= 20.;
                let c = &mut self.colonies[i];
                if suitable
                    && c.inventory[1] > 0
                    && c.inventory.iter().sum::<u32>() < 128
                    && self.visited_planets[pi].ore_remaining > 0
                {
                    let amount = 2
                        .min(self.visited_planets[pi].ore_remaining)
                        .min(128 - c.inventory.iter().sum::<u32>());
                    self.visited_planets[pi].ore_remaining -= amount;
                    c.inventory[0] += amount;
                    events.push(SpaceEvent::ColonyProduced {
                        colony: c.id,
                        amount,
                    });
                }
            }
            if self.colonies[i].maintenance_timer >= 30. {
                self.colonies[i].maintenance_timer -= 30.;
                let c = &mut self.colonies[i];
                let supplied = c.inventory[1] > 0 && c.credits >= 3;
                if supplied {
                    c.inventory[1] -= 1;
                    c.credits -= 3;
                }
                if !supplied || !suitable {
                    c.health = (c.health - 8.).max(0.);
                }
                events.push(SpaceEvent::ColonyMaintenance {
                    colony: c.id,
                    supplied: supplied && suitable,
                });
            }
        }
        for i in 0..self.ai.len() {
            self.ai[i].timer -= DT;
            if self.ai[i].timer > 0. {
                continue;
            }
            let random = self.random();
            self.ai[i].timer = 20. + (random % 61) as f32 / 10.;
            let id = self.ai[i].id;
            let system = self.ai[i].system_id;
            if self.ai[i].kind == SpaceAiKind::Raider
                && self.ship.location.system() == system
                && self.ship.location.planet().is_none()
            {
                let damage = 8f32.min(self.ship.health);
                self.ship.health = (self.ship.health - damage).max(0.);
                events.push(SpaceEvent::Raided { actor: id, damage });
            }
            if self.ai[i].kind == SpaceAiKind::Merchant {
                if let Some(pi) = self
                    .visited_planets
                    .iter()
                    .position(|p| p.system_id == system)
                {
                    let market = &mut self.visited_planets[pi].market;
                    let actor = &mut self.ai[i];
                    let c = if actor.cargo[0] >= 12 {
                        0
                    } else if actor.cargo[1] > 0 && market.inventory[1] < 100 {
                        1
                    } else {
                        0
                    };
                    let price = market.prices[c];
                    if c == 1 || actor.cargo[0] >= 12 {
                        if actor.cargo[c] > 0
                            && market.credits >= price
                            && market.inventory[c] < 10000
                            && actor.credits <= CREDIT_LIMIT - price
                        {
                            actor.cargo[c] -= 1;
                            actor.credits += price;
                            market.inventory[c] += 1;
                            market.credits -= price;
                            events.push(SpaceEvent::AiTraded {
                                actor: id,
                                planet: self.visited_planets[pi].id,
                            });
                        }
                    } else if market.inventory[c] > 0
                        && actor.credits >= price
                        && actor.cargo.iter().sum::<u32>() < 64
                        && market.credits <= CREDIT_LIMIT - price
                    {
                        market.inventory[c] -= 1;
                        market.credits += price;
                        actor.cargo[c] += 1;
                        actor.credits -= price;
                        events.push(SpaceEvent::AiTraded {
                            actor: id,
                            planet: self.visited_planets[pi].id,
                        });
                    }
                }
            }
            if self.ai[i].cargo[2] > 0 {
                self.ai[i].cargo[2] -= 1;
                self.ai[i].system_id = system % self.systems.len() as u32 + 1;
                events.push(SpaceEvent::AiMoved {
                    actor: id,
                    system: self.ai[i].system_id,
                });
            }
        }
        self.update_state(&mut events);
        events
    }
    pub fn snapshot(&self) -> SpaceSnapshot {
        SpaceSnapshot {
            version: 1,
            world: self.clone(),
        }
    }
    pub fn from_snapshot(snapshot: SpaceSnapshot) -> Result<Self, SpaceError> {
        if snapshot.version != 1 {
            return Err(SpaceError::UnsupportedVersion(snapshot.version));
        }
        snapshot.world.validate()?;
        Ok(snapshot.world)
    }
    fn validate(&self) -> Result<(), SpaceError> {
        validate_config(&self.config)?;
        if self.systems.len() != self.config.system_count
            || self.visited_planets.len() > MAX_PLANETS
            || self.colonies.len() > MAX_COLONIES
            || self.missions.len() > 16
            || self.ai.len() > 16
        {
            return Err(SpaceError::TooManyEntities);
        }
        if self.rng == 0
            || self.next_colony_id == 0
            || !self.accumulator.is_finite()
            || !(0.0..=600.).contains(&self.accumulator)
        {
            return Err(SpaceError::InvalidSnapshot);
        }
        let expected = systems(self.seed, &self.config);
        if expected.iter().enumerate().any(|(i, s)| {
            2. + s
                .position
                .distance(expected[(i + 1) % expected.len()].position)
                * 1.5
                > self.config.max_fuel
        }) {
            return Err(SpaceError::InvalidConfig);
        }
        for (i, s) in self.systems.iter().enumerate() {
            if s.id != expected[i].id
                || s.position != expected[i].position
                || s.planet_count != expected[i].planet_count
            {
                return Err(SpaceError::InvalidEntity);
            }
        }
        if !self.systems[0].visited
            || self.system_index(self.ship.location.system()).is_none()
            || !self.systems[self.system_index(self.ship.location.system()).unwrap()].visited
        {
            return Err(SpaceError::InvalidReference);
        }
        if !self.ship.health.is_finite()
            || !(0.0..=100.).contains(&self.ship.health)
            || !self.ship.fuel.is_finite()
            || !(0.0..=self.config.max_fuel).contains(&self.ship.fuel)
            || self.ship.credits > CREDIT_LIMIT
            || self
                .ship
                .cargo
                .iter()
                .any(|n| *n > self.config.cargo_capacity)
            || self.ship.cargo_used() > self.config.cargo_capacity
        {
            return Err(SpaceError::InvalidEntity);
        }
        let mut ids = HashSet::new();
        for p in &self.visited_planets {
            if !ids.insert(p.id) {
                return Err(SpaceError::DuplicateId);
            }
            if !self.valid_planet(p.id)
                || p.system_id != (p.id - 1) / 8 + 1
                || p.ordinal != (p.id - 1) % 8
                || !self.systems[self.system_index(p.system_id).unwrap()].visited
            {
                return Err(SpaceError::InvalidReference);
            }
            let natural = generate_planet(self.seed, p.system_id, p.ordinal);
            if p.initial_ore != natural.initial_ore || p.market.prices != natural.market.prices {
                return Err(SpaceError::InvalidEntity);
            }
            if [p.atmosphere, p.temperature, p.water]
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.).contains(v))
                || (p.life && !p.suitable())
                || p.initial_ore > 2000
                || p.ore_remaining > p.initial_ore
                || p.market.inventory.iter().any(|n| *n > 10000)
                || p.market.prices.iter().any(|n| !(1..=1000).contains(n))
                || p.market.credits > CREDIT_LIMIT
            {
                return Err(SpaceError::InvalidEntity);
            }
        }
        if let SpaceLocation::Planet { system, planet } = self.ship.location {
            if !self
                .visited_planets
                .iter()
                .any(|p| p.id == planet && p.system_id == system)
            {
                return Err(SpaceError::InvalidReference);
            }
        }
        let mut colony_ids = HashSet::new();
        let mut occupied = HashSet::new();
        for c in &self.colonies {
            if c.id == 0 || !colony_ids.insert(c.id) || !occupied.insert(c.planet_id) {
                return Err(SpaceError::DuplicateId);
            }
            if c.id >= self.next_colony_id || self.planet_index(c.planet_id).is_none() {
                return Err(SpaceError::InvalidReference);
            }
            if !c.health.is_finite()
                || !(0.0..=100.).contains(&c.health)
                || c.inventory.iter().any(|n| *n > 128)
                || c.inventory.iter().sum::<u32>() > 128
                || c.credits > CREDIT_LIMIT
                || !c.production_timer.is_finite()
                || !(0.0..=20.1).contains(&c.production_timer)
                || !c.maintenance_timer.is_finite()
                || !(0.0..=30.1).contains(&c.maintenance_timer)
            {
                return Err(SpaceError::InvalidEntity);
            }
        }
        let mut missions = HashSet::new();
        for m in &self.missions {
            if m.id == 0 || !missions.insert(m.id) {
                return Err(SpaceError::DuplicateId);
            }
            if m.reward_credits > CREDIT_LIMIT {
                return Err(SpaceError::InvalidEntity);
            }
            let valid = match m.kind {
                MissionKind::ReachSystem(id) => self.system_index(id).is_some(),
                MissionKind::SurveyPlanet(id) => self.valid_planet(id),
                MissionKind::Deliver { planet, amount, .. } => {
                    self.valid_planet(planet) && (1..=self.config.cargo_capacity).contains(&amount)
                }
                MissionKind::Terraform {
                    planet, min, max, ..
                } => {
                    self.valid_planet(planet)
                        && min.is_finite()
                        && max.is_finite()
                        && (0.0..=1.).contains(&min)
                        && (min..=1.).contains(&max)
                }
            };
            if !valid {
                return Err(SpaceError::InvalidReference);
            }
        }
        let mut ai_ids = HashSet::new();
        for a in &self.ai {
            if a.id == 0 || !ai_ids.insert(a.id) {
                return Err(SpaceError::DuplicateId);
            }
            if self.system_index(a.system_id).is_none() {
                return Err(SpaceError::InvalidReference);
            }
            if a.cargo.iter().any(|n| *n > 64)
                || a.cargo.iter().sum::<u32>() > 64
                || a.credits > CREDIT_LIMIT
                || !a.timer.is_finite()
                || !(-0.1..=60.).contains(&a.timer)
            {
                return Err(SpaceError::InvalidEntity);
            }
        }
        if self.objective_complete
            && (self.systems.iter().filter(|s| s.visited).count() < self.config.objective_systems
                || self.colonies.len() < self.config.objective_colonies
                || self
                    .missions
                    .iter()
                    .filter(|m| m.status == MissionStatus::Claimed)
                    .count()
                    < self.config.objective_missions)
        {
            return Err(SpaceError::InvalidSnapshot);
        }
        let expected_state = if self.ship.health <= 0. {
            SpaceState::Disabled
        } else if self.objective_complete {
            SpaceState::Freeplay
        } else {
            SpaceState::Active
        };
        if expected_state != self.state {
            return Err(SpaceError::InvalidSnapshot);
        }
        Ok(())
    }
}
