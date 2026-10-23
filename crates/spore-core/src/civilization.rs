//! Original deterministic Civilization RTS baseline, not measured retail rules.
//! Costs, layouts, domains and AI below are explicit prototype tuning. Fixed ticks
//! and owned versioned saves require no renderer or external assets.

pub use crate::cell::CellVec2 as CivilizationVec2;
use std::{collections::HashSet, fmt};
pub const CIVILIZATION_FIXED_DT: f64 = 1.0 / 60.0;
const DT: f32 = CIVILIZATION_FIXED_DT as f32;
const LIMIT: usize = 4096;
const MAGIC: &[u8; 8] = b"RSCIV001";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CivilizationState {
    Playing,
    SpaceReady,
    Defeated,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleRole {
    Military,
    Religious,
    Economic,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleDomain {
    Land,
    Sea,
    Air,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildingKind {
    House,
    Factory,
    Entertainment,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CivilizationTarget {
    City(u32),
    Vehicle(u32),
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VehicleOrder {
    Idle,
    Move { destination: CivilizationVec2 },
    Claim { geyser: u32 },
    Attack { target: CivilizationTarget },
    Convert { city: u32 },
    Trade { city: u32, returning: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CivilizationFaction {
    pub id: u8,
    pub name: String,
    pub currency: f64,
    pub ai: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CityBuilding {
    pub id: u32,
    pub kind: BuildingKind,
    pub slot: u8,
}
#[derive(Debug, Clone, PartialEq)]
pub struct ProductionJob {
    pub id: u32,
    pub role: VehicleRole,
    pub domain: VehicleDomain,
    pub remaining: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CivilizationCity {
    pub id: u32,
    pub name: String,
    pub position: CivilizationVec2,
    pub owner: u8,
    pub allied_to: Option<u8>,
    pub specialty: VehicleRole,
    pub coastal: bool,
    pub population: u32,
    pub industry: f32,
    pub happiness: f32,
    pub health: f32,
    pub max_health: f32,
    pub faith: f32,
    pub influence_owner: u8,
    pub trade: f32,
    pub trade_owner: u8,
    pub weapon_cooldown: f32,
    pub build_cooldown: f32,
    pub buildings: Vec<CityBuilding>,
    pub production: Vec<ProductionJob>,
}
impl CivilizationCity {
    pub fn controller(&self) -> u8 {
        self.allied_to.unwrap_or(self.owner)
    }
    pub fn income_per_second(&self) -> f64 {
        ((2. + self.population as f32 * 0.25) * self.industry * self.happiness) as f64
    }
    pub fn building_position(&self, slot: u8) -> CivilizationVec2 {
        let a = slot as f32 * std::f32::consts::TAU / 12.;
        self.position
            .add(CivilizationVec2::new(a.cos(), a.sin()).scale(3.5))
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct CivilizationVehicle {
    pub id: u32,
    pub owner: u8,
    pub role: VehicleRole,
    pub domain: VehicleDomain,
    pub home_city: u32,
    pub position: CivilizationVec2,
    pub heading: f32,
    pub health: f32,
    pub max_health: f32,
    pub action_cooldown: f32,
    pub order: VehicleOrder,
}
impl CivilizationVehicle {
    pub fn radius(&self) -> f32 {
        match self.domain {
            VehicleDomain::Land => 0.65,
            VehicleDomain::Sea => 0.9,
            VehicleDomain::Air => 0.75,
        }
    }
    pub fn speed(&self) -> f32 {
        match self.domain {
            VehicleDomain::Land => 6.,
            VehicleDomain::Sea => 5.,
            VehicleDomain::Air => 10.,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct CivilizationGeyser {
    pub id: u32,
    pub position: CivilizationVec2,
    pub owner: Option<u8>,
    pub reserves: f32,
    pub max_reserves: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CivilizationTerritory {
    pub id: u32,
    pub city: u32,
    pub center: CivilizationVec2,
    pub radius: f32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CivilizationConfig {
    pub half_extents: CivilizationVec2,
    pub city_count: usize,
    pub geyser_count: usize,
    pub starting_currency: f64,
}
impl Default for CivilizationConfig {
    fn default() -> Self {
        Self {
            half_extents: CivilizationVec2::new(80., 60.),
            city_count: 4,
            geyser_count: 8,
            starting_currency: 400.,
        }
    }
}
impl CivilizationConfig {
    fn validate(&self) -> Result<(), CivilizationError> {
        if !finite(self.half_extents)
            || !(45.0..=500.).contains(&self.half_extents.x)
            || !(35.0..=500.).contains(&self.half_extents.y)
            || !(2..=8).contains(&self.city_count)
            || self.geyser_count > 128
            || !self.starting_currency.is_finite()
            || !(0.0..=1e7).contains(&self.starting_currency)
        {
            Err(CivilizationError::InvalidConfig)
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CivilizationCommand {
    Build {
        city: u32,
        kind: BuildingKind,
        slot: u8,
    },
    Demolish {
        city: u32,
        slot: u8,
    },
    Produce {
        city: u32,
        role: VehicleRole,
        domain: VehicleDomain,
    },
    Move {
        destination: CivilizationVec2,
    },
    Claim {
        geyser: u32,
    },
    Attack {
        target: CivilizationTarget,
    },
    Convert {
        city: u32,
    },
    Trade {
        city: u32,
    },
    Purchase {
        city: u32,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum CivilizationEvent {
    Built {
        city: u32,
        building: u32,
    },
    Produced {
        city: u32,
        vehicle: u32,
    },
    Claimed {
        geyser: u32,
        owner: u8,
    },
    Damaged {
        target: CivilizationTarget,
        amount: f32,
    },
    Destroyed {
        vehicle: u32,
    },
    Captured {
        city: u32,
        owner: u8,
    },
    Converted {
        city: u32,
        ally: u8,
    },
    Traded {
        city: u32,
        trader: u8,
    },
    Purchased {
        city: u32,
        owner: u8,
    },
    GeyserDepleted {
        geyser: u32,
    },
    SpaceEligible,
    PlayerDefeated,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CivilizationStepReport {
    pub steps: u32,
    pub backlog_seconds: f64,
    pub interpolation: f32,
    pub state: CivilizationState,
    pub events: Vec<CivilizationEvent>,
    pub time_rejected: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CivilizationWorld {
    pub config: CivilizationConfig,
    pub factions: Vec<CivilizationFaction>,
    pub cities: Vec<CivilizationCity>,
    pub territories: Vec<CivilizationTerritory>,
    pub vehicles: Vec<CivilizationVehicle>,
    pub geysers: Vec<CivilizationGeyser>,
    pub selected: Vec<u32>,
    pub state: CivilizationState,
    pub tick: u64,
    seed: u64,
    rng: u64,
    next_id: u32,
    accumulator: f64,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CivilizationSnapshot {
    pub version: u32,
    pub world: CivilizationWorld,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CivilizationError {
    InvalidConfig,
    InvalidEntity,
    InvalidReference,
    DuplicateId,
    InvalidOrder,
    InvalidSelection,
    NotOwned,
    InsufficientFunds,
    InvalidPlacement,
    Cooldown,
    QueueFull,
    WrongRole,
    WrongDomain,
    TradeRequired,
    Terminal,
    TooManyEntities,
    InvalidSnapshot,
    UnsupportedVersion(u32),
    TruncatedSnapshot,
    TrailingBytes,
}
impl fmt::Display for CivilizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CivilizationError {}

pub fn building_cost(kind: BuildingKind) -> f64 {
    match kind {
        BuildingKind::House => 50.,
        BuildingKind::Factory => 80.,
        BuildingKind::Entertainment => 60.,
    }
}
pub fn vehicle_cost(role: VehicleRole, domain: VehicleDomain) -> f64 {
    (match domain {
        VehicleDomain::Land => 60.,
        VehicleDomain::Sea => 75.,
        VehicleDomain::Air => 90.,
    }) + match role {
        VehicleRole::Military => 20.,
        VehicleRole::Religious => 15.,
        VehicleRole::Economic => 10.,
    }
}
pub fn purchase_cost(city: &CivilizationCity) -> f64 {
    200. + city.population as f64 * 30.
}

impl CivilizationWorld {
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, CivilizationConfig::default()).expect("default Civilization config")
    }
    pub fn with_config(seed: u64, config: CivilizationConfig) -> Result<Self, CivilizationError> {
        config.validate()?;
        let mut world = Self {
            config,
            factions: Vec::new(),
            cities: Vec::new(),
            territories: Vec::new(),
            vehicles: Vec::new(),
            geysers: Vec::new(),
            selected: Vec::new(),
            state: CivilizationState::Playing,
            tick: 0,
            seed,
            rng: if seed == 0 {
                0xb794_ae31_504a_a227
            } else {
                seed
            },
            next_id: 1,
            accumulator: 0.,
        };
        for owner in 0..world.config.city_count {
            world.factions.push(CivilizationFaction {
                id: owner as u8,
                name: if owner == 0 {
                    "Your nation".into()
                } else {
                    format!("Nation {owner}")
                },
                currency: world.config.starting_currency,
                ai: owner != 0,
            });
            let angle = owner as f32 * std::f32::consts::TAU / world.config.city_count as f32
                + std::f32::consts::PI;
            let coastal = owner % 2 == 1;
            let position = if coastal {
                CivilizationVec2::new(
                    if owner % 4 == 1 { -12. } else { 12. },
                    if world.config.city_count / 2 <= 1 {
                        -world.config.half_extents.y * 0.6
                    } else {
                        (-0.65
                            + (owner / 2) as f32 / (world.config.city_count / 2 - 1) as f32 * 1.3)
                            * world.config.half_extents.y
                    },
                )
            } else {
                CivilizationVec2::new(
                    (angle.cos() * world.config.half_extents.x * 0.58)
                        .abs()
                        .max(18.)
                        * if angle.cos() >= 0. { 1. } else { -1. },
                    angle.sin() * world.config.half_extents.y * 0.58,
                )
            };
            let id = world.id();
            let specialty = match owner % 3 {
                0 => VehicleRole::Economic,
                1 => VehicleRole::Military,
                _ => VehicleRole::Religious,
            };
            let mut buildings = Vec::new();
            for (slot, kind) in [
                (0, BuildingKind::House),
                (4, BuildingKind::Factory),
                (8, BuildingKind::Entertainment),
            ] {
                buildings.push(CityBuilding {
                    id: world.id(),
                    kind,
                    slot,
                });
            }
            let mut city = CivilizationCity {
                id,
                name: if owner == 0 {
                    "Firstlight".into()
                } else {
                    format!("City {owner}")
                },
                position,
                owner: owner as u8,
                allied_to: None,
                specialty,
                coastal,
                population: 0,
                industry: 0.,
                happiness: 0.,
                health: 250.,
                max_health: 250.,
                faith: 0.,
                influence_owner: owner as u8,
                trade: 0.,
                trade_owner: owner as u8,
                weapon_cooldown: 0.,
                build_cooldown: 0.,
                buildings,
                production: Vec::new(),
            };
            recalculate(&mut city);
            world.cities.push(city);
            let territory = world.id();
            world.territories.push(CivilizationTerritory {
                id: territory,
                city: id,
                center: position,
                radius: 14.,
            });
        }
        for index in 0..world.config.geyser_count {
            let position = if index == 0 {
                world.cities[0].position.add(CivilizationVec2::new(7., 3.))
            } else if index % 4 == 0 {
                CivilizationVec2::new(
                    0.,
                    (index as f32 * 0.7).sin() * (world.config.half_extents.y - 10.),
                )
            } else {
                let x = (world.random() * 2. - 1.) * (world.config.half_extents.x - 8.);
                let y = (world.random() * 2. - 1.) * (world.config.half_extents.y - 8.);
                CivilizationVec2::new(x, y)
            };
            let id = world.id();
            world.geysers.push(CivilizationGeyser {
                id,
                position,
                owner: None,
                reserves: 300.,
                max_reserves: 300.,
            });
        }
        world.validate()?;
        Ok(world)
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn restart(&mut self) {
        *self = Self::with_config(self.seed, self.config.clone())
            .expect("validated Civilization config");
    }
    pub fn progress(&self) -> f32 {
        self.cities
            .iter()
            .filter(|city| city.controller() == 0)
            .count() as f32
            / self.cities.len().max(1) as f32
    }
    pub fn currency(&self) -> f64 {
        self.factions[0].currency
    }
    pub fn territory_owner(&self, id: u32) -> Option<u8> {
        self.territories
            .iter()
            .find(|territory| territory.id == id)
            .and_then(|territory| self.cities.iter().find(|city| city.id == territory.city))
            .map(CivilizationCity::controller)
    }
    /// Vertical river with a twelve-unit-wide land bridge at the world's centre.
    pub fn is_water(&self, position: CivilizationVec2) -> bool {
        position.x.abs() < 8. && position.y.abs() > 6.
    }
    pub fn select(&mut self, ids: Vec<u32>) -> Result<(), CivilizationError> {
        let mut seen = HashSet::new();
        if ids.len() > 256
            || ids.iter().any(|id| {
                !seen.insert(*id)
                    || !self
                        .vehicles
                        .iter()
                        .any(|vehicle| vehicle.id == *id && vehicle.owner == 0)
            })
        {
            return Err(CivilizationError::InvalidSelection);
        }
        self.selected = ids;
        Ok(())
    }
    pub fn issue_command(&mut self, command: CivilizationCommand) -> Result<(), CivilizationError> {
        if self.state != CivilizationState::Playing {
            return Err(CivilizationError::Terminal);
        }
        self.command(0, command, &self.selected.clone())?;
        let owned = self
            .cities
            .iter()
            .filter(|city| city.controller() == 0)
            .count();
        if owned == self.cities.len() {
            self.state = CivilizationState::SpaceReady;
        } else if owned == 0 {
            self.state = CivilizationState::Defeated;
        }
        Ok(())
    }
    fn command(
        &mut self,
        owner: u8,
        command: CivilizationCommand,
        selected: &[u32],
    ) -> Result<(), CivilizationError> {
        match command {
            CivilizationCommand::Build { city, kind, slot } => {
                let at = self.city(city)?;
                if self.cities[at].controller() != owner {
                    return Err(CivilizationError::NotOwned);
                }
                if slot >= 12
                    || self.cities[at]
                        .buildings
                        .iter()
                        .any(|building| building.slot == slot)
                {
                    return Err(CivilizationError::InvalidPlacement);
                }
                if self.cities[at].build_cooldown > 0. {
                    return Err(CivilizationError::Cooldown);
                }
                self.pay(owner, building_cost(kind))?;
                let id = self.id();
                self.cities[at]
                    .buildings
                    .push(CityBuilding { id, kind, slot });
                self.cities[at].build_cooldown = 1.;
                recalculate(&mut self.cities[at]);
            }
            CivilizationCommand::Demolish { city, slot } => {
                let at = self.city(city)?;
                if self.cities[at].controller() != owner {
                    return Err(CivilizationError::NotOwned);
                }
                if self.cities[at].build_cooldown > 0. {
                    return Err(CivilizationError::Cooldown);
                }
                let index = self.cities[at]
                    .buildings
                    .iter()
                    .position(|building| building.slot == slot)
                    .ok_or(CivilizationError::InvalidPlacement)?;
                self.cities[at].buildings.remove(index);
                self.cities[at].build_cooldown = 0.5;
                recalculate(&mut self.cities[at]);
            }
            CivilizationCommand::Produce { city, role, domain } => {
                let at = self.city(city)?;
                if self.cities[at].controller() != owner {
                    return Err(CivilizationError::NotOwned);
                }
                if domain == VehicleDomain::Sea && !self.cities[at].coastal {
                    return Err(CivilizationError::WrongDomain);
                }
                if self.cities[at].production.len() >= 6
                    || self.vehicles.len()
                        + self
                            .cities
                            .iter()
                            .map(|city| city.production.len())
                            .sum::<usize>()
                        >= 256
                {
                    return Err(CivilizationError::QueueFull);
                }
                self.pay(owner, vehicle_cost(role, domain))?;
                let id = self.id();
                self.cities[at].production.push(ProductionJob {
                    id,
                    role,
                    domain,
                    remaining: 3.,
                });
            }
            CivilizationCommand::Purchase { city } => {
                let at = self.city(city)?;
                if self.cities[at].controller() == owner {
                    return Err(CivilizationError::NotOwned);
                }
                if self.cities[at].trade_owner != owner || self.cities[at].trade < 60. {
                    return Err(CivilizationError::TradeRequired);
                }
                let cost = purchase_cost(&self.cities[at]);
                self.pay(owner, cost)?;
                capture(&mut self.cities[at], owner);
            }
            action => {
                if selected.is_empty() {
                    return Err(CivilizationError::InvalidSelection);
                }
                let mut updates = Vec::new();
                for id in selected {
                    let at = self
                        .vehicles
                        .iter()
                        .position(|vehicle| vehicle.id == *id && vehicle.owner == owner)
                        .ok_or(CivilizationError::InvalidSelection)?;
                    let vehicle = &self.vehicles[at];
                    let order = match action {
                        CivilizationCommand::Move { destination } => {
                            if !inside(destination, self.config.half_extents)
                                || !self.accessible(vehicle.domain, destination)
                            {
                                return Err(CivilizationError::WrongDomain);
                            }
                            VehicleOrder::Move { destination }
                        }
                        CivilizationCommand::Claim { geyser } => {
                            let field = self
                                .geysers
                                .iter()
                                .find(|field| field.id == geyser)
                                .ok_or(CivilizationError::InvalidReference)?;
                            if field.reserves <= 0. || field.owner == Some(owner) {
                                return Err(CivilizationError::InvalidOrder);
                            }
                            if !self.accessible(vehicle.domain, field.position) {
                                return Err(CivilizationError::WrongDomain);
                            }
                            VehicleOrder::Claim { geyser }
                        }
                        CivilizationCommand::Attack { target } => {
                            if vehicle.role != VehicleRole::Military {
                                return Err(CivilizationError::WrongRole);
                            }
                            let (position, target_owner) = self.target(target)?;
                            if target_owner == owner {
                                return Err(CivilizationError::NotOwned);
                            }
                            if !self.reachable(vehicle.domain, position, target) {
                                return Err(CivilizationError::WrongDomain);
                            }
                            VehicleOrder::Attack { target }
                        }
                        CivilizationCommand::Convert { city }
                        | CivilizationCommand::Trade { city } => {
                            let at = self.city(city)?;
                            let target = &self.cities[at];
                            if target.controller() == owner {
                                return Err(CivilizationError::NotOwned);
                            }
                            if !self.reachable(
                                vehicle.domain,
                                target.position,
                                CivilizationTarget::City(city),
                            ) {
                                return Err(CivilizationError::WrongDomain);
                            }
                            if matches!(action, CivilizationCommand::Convert { .. }) {
                                if vehicle.role != VehicleRole::Religious {
                                    return Err(CivilizationError::WrongRole);
                                }
                                VehicleOrder::Convert { city }
                            } else {
                                if vehicle.role != VehicleRole::Economic {
                                    return Err(CivilizationError::WrongRole);
                                }
                                VehicleOrder::Trade {
                                    city,
                                    returning: false,
                                }
                            }
                        }
                        _ => return Err(CivilizationError::InvalidOrder),
                    };
                    updates.push((at, order));
                }
                for (at, order) in updates {
                    self.vehicles[at].order = order;
                }
            }
        }
        Ok(())
    }
    fn pay(&mut self, owner: u8, cost: f64) -> Result<(), CivilizationError> {
        let faction = self
            .factions
            .iter_mut()
            .find(|faction| faction.id == owner)
            .ok_or(CivilizationError::InvalidReference)?;
        if faction.currency + 1e-9 < cost {
            return Err(CivilizationError::InsufficientFunds);
        }
        faction.currency = (faction.currency - cost).max(0.);
        Ok(())
    }
    fn city(&self, id: u32) -> Result<usize, CivilizationError> {
        self.cities
            .iter()
            .position(|city| city.id == id)
            .ok_or(CivilizationError::InvalidReference)
    }
    fn target(
        &self,
        target: CivilizationTarget,
    ) -> Result<(CivilizationVec2, u8), CivilizationError> {
        match target {
            CivilizationTarget::City(id) => {
                let city = &self.cities[self.city(id)?];
                Ok((city.position, city.controller()))
            }
            CivilizationTarget::Vehicle(id) => self
                .vehicles
                .iter()
                .find(|vehicle| vehicle.id == id)
                .map(|vehicle| (vehicle.position, vehicle.owner))
                .ok_or(CivilizationError::InvalidReference),
        }
    }
    fn accessible(&self, domain: VehicleDomain, position: CivilizationVec2) -> bool {
        match domain {
            VehicleDomain::Air => true,
            VehicleDomain::Land => !self.is_water(position),
            // Boats pass under the central land bridge.
            VehicleDomain::Sea => position.x.abs() < 8.,
        }
    }
    fn reachable(
        &self,
        domain: VehicleDomain,
        position: CivilizationVec2,
        target: CivilizationTarget,
    ) -> bool {
        domain != VehicleDomain::Sea
            || match target {
                CivilizationTarget::City(id) => {
                    self.cities.iter().any(|city| city.id == id && city.coastal)
                }
                CivilizationTarget::Vehicle(_) => position.x.abs() < 8.,
            }
    }
    pub fn advance(&mut self, seconds: f64) -> CivilizationStepReport {
        let rejected = !seconds.is_finite() || seconds < 0. || seconds > 86400.;
        if !rejected && self.state == CivilizationState::Playing {
            self.accumulator += seconds;
        }
        let mut steps = 0;
        let mut events = Vec::new();
        while self.accumulator + 1e-12 >= CIVILIZATION_FIXED_DT
            && steps < 240
            && self.state == CivilizationState::Playing
        {
            self.accumulator = (self.accumulator - CIVILIZATION_FIXED_DT).max(0.);
            events.extend(self.step());
            steps += 1;
        }
        if self.state != CivilizationState::Playing {
            self.accumulator = 0.;
        }
        CivilizationStepReport {
            steps,
            backlog_seconds: self.accumulator,
            interpolation: (self.accumulator / CIVILIZATION_FIXED_DT).min(1.) as f32,
            state: self.state,
            events,
            time_rejected: rejected,
        }
    }
    pub fn step(&mut self) -> Vec<CivilizationEvent> {
        let mut events = Vec::new();
        if self.state != CivilizationState::Playing {
            return events;
        }
        self.tick = self.tick.saturating_add(1);
        for city in &mut self.cities {
            city.build_cooldown = (city.build_cooldown - DT).max(0.);
            city.weapon_cooldown = (city.weapon_cooldown - DT).max(0.);
            let owner = city.controller() as usize;
            self.factions[owner].currency = (self.factions[owner].currency
                + city.income_per_second() * CIVILIZATION_FIXED_DT)
                .min(1e9);
            if let Some(job) = city.production.first_mut() {
                job.remaining = (job.remaining - DT * city.industry).max(0.);
            }
            if city
                .production
                .first()
                .is_some_and(|job| job.remaining <= 0.)
            {
                let job = city.production.remove(0);
                let mut position = if job.domain == VehicleDomain::Sea {
                    CivilizationVec2::new(0., city.position.y)
                } else {
                    city.position.add(CivilizationVec2::new(
                        if city.position.x < 0. { -4.5 } else { 4.5 },
                        0.,
                    ))
                };
                position.x = position.x.clamp(
                    -self.config.half_extents.x + 1.,
                    self.config.half_extents.x - 1.,
                );
                position.y = position.y.clamp(
                    -self.config.half_extents.y + 1.,
                    self.config.half_extents.y - 1.,
                );
                self.vehicles.push(CivilizationVehicle {
                    id: job.id,
                    owner: owner as u8,
                    role: job.role,
                    domain: job.domain,
                    home_city: city.id,
                    position,
                    heading: 0.,
                    health: 80.,
                    max_health: 80.,
                    action_cooldown: 0.,
                    order: VehicleOrder::Idle,
                });
                events.push(CivilizationEvent::Produced {
                    city: city.id,
                    vehicle: job.id,
                });
            }
        }
        for field in &mut self.geysers {
            if let Some(owner) = field.owner {
                let extraction = (4. * DT).min(field.reserves);
                field.reserves = (field.reserves - extraction).max(0.);
                self.factions[owner as usize].currency =
                    (self.factions[owner as usize].currency + extraction as f64).min(1e9);
                if extraction > 0. && field.reserves == 0. {
                    events.push(CivilizationEvent::GeyserDepleted { geyser: field.id });
                }
            }
        }
        if self.tick % 120 == 0 {
            self.ai();
        }
        for index in 0..self.vehicles.len() {
            if self.vehicles[index].health <= 0. {
                continue;
            }
            self.vehicles[index].action_cooldown =
                (self.vehicles[index].action_cooldown - DT).max(0.);
            self.act(index, &mut events);
        }
        for city in &mut self.cities {
            if city.weapon_cooldown > 0. {
                continue;
            }
            if let Some(vehicle) = self
                .vehicles
                .iter_mut()
                .filter(|vehicle| {
                    vehicle.health > 0.
                        && vehicle.owner != city.controller()
                        && vehicle.position.distance(city.position) < 7.
                })
                .min_by(|a, b| {
                    a.position
                        .distance(city.position)
                        .total_cmp(&b.position.distance(city.position))
                })
            {
                let amount = 8_f32.min(vehicle.health);
                vehicle.health -= amount;
                city.weapon_cooldown = 1.;
                events.push(CivilizationEvent::Damaged {
                    target: CivilizationTarget::Vehicle(vehicle.id),
                    amount,
                });
            }
        }
        for vehicle in self.vehicles.iter().filter(|vehicle| vehicle.health <= 0.) {
            events.push(CivilizationEvent::Destroyed {
                vehicle: vehicle.id,
            });
        }
        self.vehicles.retain(|vehicle| vehicle.health > 0.);
        self.selected.retain(|id| {
            self.vehicles
                .iter()
                .any(|vehicle| vehicle.id == *id && vehicle.owner == 0)
        });
        let live: HashSet<u32> = self.vehicles.iter().map(|vehicle| vehicle.id).collect();
        for vehicle in &mut self.vehicles {
            if matches!(vehicle.order,VehicleOrder::Attack{target:CivilizationTarget::Vehicle(id)}if !live.contains(&id))
            {
                vehicle.order = VehicleOrder::Idle;
            }
        }
        let controlled = self
            .cities
            .iter()
            .filter(|city| city.controller() == 0)
            .count();
        if controlled == 0 {
            self.state = CivilizationState::Defeated;
            events.push(CivilizationEvent::PlayerDefeated);
        } else if controlled == self.cities.len() {
            self.state = CivilizationState::SpaceReady;
            events.push(CivilizationEvent::SpaceEligible);
        }
        events
    }
    fn act(&mut self, index: usize, events: &mut Vec<CivilizationEvent>) {
        let vehicle = self.vehicles[index].clone();
        let order = vehicle.order.clone();
        let owner = vehicle.owner;
        let (destination, range) = match order {
            VehicleOrder::Idle => return,
            VehicleOrder::Move { destination } => (destination, 0.3),
            VehicleOrder::Claim { geyser } => {
                let Some(field) = self.geysers.iter().find(|field| field.id == geyser) else {
                    self.vehicles[index].order = VehicleOrder::Idle;
                    return;
                };
                if field.owner == Some(owner) || field.reserves <= 0. {
                    self.vehicles[index].order = VehicleOrder::Idle;
                    return;
                }
                (field.position, 1.8)
            }
            VehicleOrder::Attack { target } => {
                let Ok((position, target_owner)) = self.target(target) else {
                    self.vehicles[index].order = VehicleOrder::Idle;
                    return;
                };
                if target_owner == owner {
                    self.vehicles[index].order = VehicleOrder::Idle;
                    return;
                }
                (
                    position,
                    if vehicle.domain == VehicleDomain::Sea {
                        15.
                    } else if vehicle.domain == VehicleDomain::Air {
                        6.
                    } else {
                        4.
                    },
                )
            }
            VehicleOrder::Convert { city }
            | VehicleOrder::Trade {
                city,
                returning: false,
            } => {
                let Ok(at) = self.city(city) else {
                    return;
                };
                if self.cities[at].controller() == owner {
                    self.vehicles[index].order = VehicleOrder::Idle;
                    return;
                }
                (
                    self.cities[at].position,
                    if vehicle.domain == VehicleDomain::Sea {
                        15.
                    } else {
                        5.
                    },
                )
            }
            VehicleOrder::Trade {
                returning: true, ..
            } => {
                let Ok(at) = self.city(vehicle.home_city) else {
                    return;
                };
                if self.cities[at].controller() != owner {
                    self.vehicles[index].order = VehicleOrder::Idle;
                    return;
                }
                (
                    if vehicle.domain == VehicleDomain::Sea {
                        CivilizationVec2::new(0., self.cities[at].position.y)
                    } else {
                        self.cities[at].position
                    },
                    3.,
                )
            }
        };
        if vehicle.position.distance(destination) > range {
            let mut waypoint = destination;
            if vehicle.domain == VehicleDomain::Land {
                if vehicle.position.x * destination.x < 0. {
                    waypoint = if vehicle.position.x.abs() > 9. && vehicle.position.y.abs() > 4. {
                        CivilizationVec2::new(vehicle.position.x.signum() * 10., 0.)
                    } else {
                        CivilizationVec2::new(destination.x.signum() * 10., 0.)
                    };
                } else if vehicle.position.x.abs() < 9. && destination.x.abs() >= 8. {
                    waypoint = CivilizationVec2::new(destination.x.signum() * 10., 0.);
                }
            }
            if vehicle.domain == VehicleDomain::Sea {
                waypoint.x = waypoint.x.clamp(-7., 7.);
            }
            let delta = waypoint.sub(vehicle.position);
            let length = delta.length();
            if length > 1e-5 {
                let next = vehicle
                    .position
                    .add(delta.scale((vehicle.speed() * DT).min(length) / length));
                if self.accessible(vehicle.domain, next) {
                    self.vehicles[index].position = next;
                    self.vehicles[index].heading = delta.y.atan2(delta.x);
                }
            }
            return;
        }
        if vehicle.action_cooldown > 0. {
            return;
        }
        self.vehicles[index].action_cooldown = 0.75;
        match order {
            VehicleOrder::Move { .. } => self.vehicles[index].order = VehicleOrder::Idle,
            VehicleOrder::Claim { geyser } => {
                if let Some(field) = self.geysers.iter_mut().find(|field| field.id == geyser) {
                    field.owner = Some(owner);
                    events.push(CivilizationEvent::Claimed { geyser, owner });
                }
                self.vehicles[index].order = VehicleOrder::Idle;
            }
            VehicleOrder::Attack { target } => match target {
                CivilizationTarget::Vehicle(id) => {
                    if let Some(target) = self.vehicles.iter_mut().find(|vehicle| vehicle.id == id)
                    {
                        let amount = 20_f32.min(target.health.max(0.));
                        target.health -= amount;
                        events.push(CivilizationEvent::Damaged {
                            target: CivilizationTarget::Vehicle(id),
                            amount,
                        });
                    }
                }
                CivilizationTarget::City(id) => {
                    if let Ok(at) = self.city(id) {
                        let city = &mut self.cities[at];
                        let amount = 20_f32.min(city.health);
                        city.health -= amount;
                        events.push(CivilizationEvent::Damaged {
                            target: CivilizationTarget::City(id),
                            amount,
                        });
                        if city.health <= 0. {
                            capture(city, owner);
                            events.push(CivilizationEvent::Captured { city: id, owner });
                        }
                    }
                }
            },
            VehicleOrder::Convert { city } => {
                if let Ok(at) = self.city(city) {
                    let target = &mut self.cities[at];
                    if target.influence_owner != owner {
                        target.faith = 0.;
                        target.influence_owner = owner;
                    }
                    target.faith = (target.faith + 12.).min(100.);
                    if target.faith >= 100. {
                        target.allied_to = if target.owner == owner {
                            None
                        } else {
                            Some(owner)
                        };
                        target.production.clear();
                        events.push(CivilizationEvent::Converted { city, ally: owner });
                        self.vehicles[index].order = VehicleOrder::Idle;
                    }
                }
            }
            VehicleOrder::Trade { city, returning } => {
                if returning {
                    self.vehicles[index].order = VehicleOrder::Trade {
                        city,
                        returning: false,
                    };
                } else if let Ok(at) = self.city(city) {
                    let target = &mut self.cities[at];
                    if target.trade_owner != owner {
                        target.trade = 0.;
                        target.trade_owner = owner;
                    }
                    target.trade = (target.trade + 20.).min(100.);
                    self.factions[owner as usize].currency =
                        (self.factions[owner as usize].currency + 15.).min(1e9);
                    self.vehicles[index].order = VehicleOrder::Trade {
                        city,
                        returning: true,
                    };
                    events.push(CivilizationEvent::Traded {
                        city,
                        trader: owner,
                    });
                }
            }
            VehicleOrder::Idle => {}
        }
    }
    fn ai(&mut self) {
        for owner in 1..self.factions.len() as u8 {
            if !self.factions[owner as usize].ai {
                continue;
            }
            let cities: Vec<u32> = self
                .cities
                .iter()
                .filter(|city| city.controller() == owner)
                .map(|city| city.id)
                .collect();
            for id in cities {
                let at = self.city(id).unwrap();
                let role = self.cities[at].specialty;
                if self.tick % 600 == 0 {
                    if let Some(slot) = (0..12).find(|slot| {
                        !self.cities[at]
                            .buildings
                            .iter()
                            .any(|building| building.slot == *slot)
                    }) {
                        let kind = match (self.tick / 600 + owner as u64) % 3 {
                            0 => BuildingKind::House,
                            1 => BuildingKind::Factory,
                            _ => BuildingKind::Entertainment,
                        };
                        let _ = self.command(
                            owner,
                            CivilizationCommand::Build {
                                city: id,
                                kind,
                                slot,
                            },
                            &[],
                        );
                    }
                }
                if self
                    .vehicles
                    .iter()
                    .filter(|vehicle| vehicle.owner == owner)
                    .count()
                    + self.cities[at].production.len()
                    < 4
                {
                    let domain = if self.cities[at].coastal && self.tick % 360 == 0 {
                        VehicleDomain::Sea
                    } else if self.tick % 480 == 0 {
                        VehicleDomain::Air
                    } else {
                        VehicleDomain::Land
                    };
                    let _ = self.command(
                        owner,
                        CivilizationCommand::Produce {
                            city: id,
                            role,
                            domain,
                        },
                        &[],
                    );
                }
            }
            let ids: Vec<u32> = self
                .vehicles
                .iter()
                .filter(|vehicle| {
                    vehicle.owner == owner && matches!(vehicle.order, VehicleOrder::Idle)
                })
                .map(|vehicle| vehicle.id)
                .collect();
            for id in ids {
                let vehicle = self
                    .vehicles
                    .iter()
                    .find(|vehicle| vehicle.id == id)
                    .unwrap()
                    .clone();
                let field = self
                    .geysers
                    .iter()
                    .filter(|field| {
                        field.owner.is_none()
                            && field.reserves > 0.
                            && self.accessible(vehicle.domain, field.position)
                    })
                    .min_by(|a, b| {
                        vehicle
                            .position
                            .distance(a.position)
                            .total_cmp(&vehicle.position.distance(b.position))
                    })
                    .map(|field| field.id);
                // Only a military detachment scouts fields. Assigning this by
                // id modulo three starved the entire religious faction because
                // interleaved production gave every convert the same residue.
                if vehicle.role == VehicleRole::Military && id % 2 == 0 {
                    if let Some(geyser) = field {
                        let _ = self.command(owner, CivilizationCommand::Claim { geyser }, &[id]);
                        continue;
                    }
                }
                let target = self
                    .cities
                    .iter()
                    .filter(|city| {
                        city.controller() != owner
                            && self.reachable(
                                vehicle.domain,
                                city.position,
                                CivilizationTarget::City(city.id),
                            )
                    })
                    .min_by(|a, b| {
                        vehicle
                            .position
                            .distance(a.position)
                            .total_cmp(&vehicle.position.distance(b.position))
                    })
                    .map(|city| city.id);
                if let Some(city) = target {
                    let command = match vehicle.role {
                        VehicleRole::Military => CivilizationCommand::Attack {
                            target: CivilizationTarget::City(city),
                        },
                        VehicleRole::Religious => CivilizationCommand::Convert { city },
                        VehicleRole::Economic => CivilizationCommand::Trade { city },
                    };
                    let _ = self.command(owner, command, &[id]);
                }
            }
            let purchases: Vec<u32> = self
                .cities
                .iter()
                .filter(|city| {
                    city.controller() != owner && city.trade_owner == owner && city.trade >= 60.
                })
                .map(|city| city.id)
                .collect();
            for city in purchases {
                let _ = self.command(owner, CivilizationCommand::Purchase { city }, &[]);
            }
        }
    }
    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        ((self.rng >> 40) as u32) as f32 / 16777216.
    }
    fn id(&mut self) -> u32 {
        loop {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1).max(1);
            if id != 0
                && !self.cities.iter().any(|city| {
                    city.id == id
                        || city.buildings.iter().any(|building| building.id == id)
                        || city.production.iter().any(|job| job.id == id)
                })
                && !self.vehicles.iter().any(|vehicle| vehicle.id == id)
                && !self.geysers.iter().any(|field| field.id == id)
                && !self.territories.iter().any(|territory| territory.id == id)
            {
                return id;
            }
        }
    }
    pub fn snapshot(&self) -> CivilizationSnapshot {
        CivilizationSnapshot {
            version: 1,
            world: self.clone(),
        }
    }
    pub fn from_snapshot(snapshot: CivilizationSnapshot) -> Result<Self, CivilizationError> {
        if snapshot.version != 1 {
            return Err(CivilizationError::UnsupportedVersion(snapshot.version));
        }
        snapshot.world.validate()?;
        Ok(snapshot.world)
    }
    fn validate(&self) -> Result<(), CivilizationError> {
        self.config.validate()?;
        if self.rng == 0
            || self.next_id == 0
            || !self.accumulator.is_finite()
            || !(0.0..=86400.).contains(&self.accumulator)
            || self.cities.len() != self.config.city_count
            || self.factions.len() != self.config.city_count
            || self.territories.len() != self.cities.len()
            || self.geysers.len() != self.config.geyser_count
            || self.vehicles.len() > 256
        {
            return Err(CivilizationError::InvalidSnapshot);
        }
        let mut ids = HashSet::new();
        for (index, faction) in self.factions.iter().enumerate() {
            if faction.id as usize != index
                || !valid_name(&faction.name)
                || !faction.currency.is_finite()
                || !(0.0..=1e9).contains(&faction.currency)
            {
                return Err(CivilizationError::InvalidEntity);
            }
        }
        for city in &self.cities {
            unique(&mut ids, city.id)?;
            if !inside(city.position, self.config.half_extents)
                || self.is_water(city.position)
                || (city.coastal && city.position.x.abs() > 15.)
                || !valid_name(&city.name)
                || city.owner as usize >= self.factions.len()
                || city
                    .allied_to
                    .is_some_and(|id| id as usize >= self.factions.len() || id == city.owner)
                || city.influence_owner as usize >= self.factions.len()
                || city.trade_owner as usize >= self.factions.len()
                || !range(city.health, 0.001, 10000.)
                || !range(city.max_health, 1., 10000.)
                || city.health > city.max_health
                || !range(city.faith, 0., 100.)
                || !range(city.trade, 0., 100.)
                || !range(city.weapon_cooldown, 0., 2.)
                || !range(city.build_cooldown, 0., 2.)
                || city.buildings.len() > 12
                || city.production.len() > 6
            {
                return Err(CivilizationError::InvalidEntity);
            }
            if city.allied_to.is_some()
                && (city.faith < 100. || city.allied_to != Some(city.influence_owner))
            {
                return Err(CivilizationError::InvalidReference);
            }
            let mut slots = HashSet::new();
            for building in &city.buildings {
                unique(&mut ids, building.id)?;
                if building.slot >= 12 || !slots.insert(building.slot) {
                    return Err(CivilizationError::InvalidPlacement);
                }
            }
            for job in &city.production {
                unique(&mut ids, job.id)?;
                if !range(job.remaining, 0., 3.)
                    || (job.domain == VehicleDomain::Sea && !city.coastal)
                {
                    return Err(CivilizationError::InvalidEntity);
                }
            }
            let mut expected = city.clone();
            recalculate(&mut expected);
            if expected.population != city.population
                || !close(expected.industry, city.industry)
                || !close(expected.happiness, city.happiness)
            {
                return Err(CivilizationError::InvalidEntity);
            }
        }
        let mut territories = HashSet::new();
        for territory in &self.territories {
            unique(&mut ids, territory.id)?;
            if !territories.insert(territory.city)
                || !self
                    .cities
                    .iter()
                    .any(|city| city.id == territory.city && city.position == territory.center)
                || !range(territory.radius, 1., 50.)
            {
                return Err(CivilizationError::InvalidReference);
            }
        }
        for field in &self.geysers {
            unique(&mut ids, field.id)?;
            if !inside(field.position, self.config.half_extents)
                || field
                    .owner
                    .is_some_and(|owner| owner as usize >= self.factions.len())
                || !range(field.max_reserves, 1., 1e6)
                || !range(field.reserves, 0., field.max_reserves)
            {
                return Err(CivilizationError::InvalidEntity);
            }
        }
        let jobs: usize = self.cities.iter().map(|city| city.production.len()).sum();
        if jobs + self.vehicles.len() > 256 {
            return Err(CivilizationError::TooManyEntities);
        }
        for vehicle in &self.vehicles {
            unique(&mut ids, vehicle.id)?;
            if vehicle.owner as usize >= self.factions.len()
                || !self.cities.iter().any(|city| {
                    city.id == vehicle.home_city
                        && (vehicle.domain != VehicleDomain::Sea || city.coastal)
                })
                || !inside(vehicle.position, self.config.half_extents)
                || !self.accessible(vehicle.domain, vehicle.position)
                || !vehicle.heading.is_finite()
                || !range(vehicle.max_health, 1., 10000.)
                || !range(vehicle.health, 0.001, vehicle.max_health)
                || !range(vehicle.action_cooldown, 0., 2.)
            {
                return Err(CivilizationError::InvalidEntity);
            }
            match vehicle.order {
                VehicleOrder::Idle => {}
                VehicleOrder::Move { destination } => {
                    if !inside(destination, self.config.half_extents)
                        || !self.accessible(vehicle.domain, destination)
                    {
                        return Err(CivilizationError::InvalidOrder);
                    }
                }
                VehicleOrder::Claim { geyser } => {
                    if !self.geysers.iter().any(|field| {
                        field.id == geyser && self.accessible(vehicle.domain, field.position)
                    }) {
                        return Err(CivilizationError::InvalidReference);
                    }
                }
                VehicleOrder::Attack { target } => {
                    let (position, _) = self.target(target)?;
                    if vehicle.role != VehicleRole::Military
                        || !self.reachable(vehicle.domain, position, target)
                    {
                        return Err(CivilizationError::InvalidOrder);
                    }
                }
                VehicleOrder::Convert { city } | VehicleOrder::Trade { city, .. } => {
                    let target = &self.cities[self.city(city)?];
                    if !self.reachable(
                        vehicle.domain,
                        target.position,
                        CivilizationTarget::City(city),
                    ) || matches!(vehicle.order, VehicleOrder::Convert { .. })
                        && vehicle.role != VehicleRole::Religious
                        || matches!(vehicle.order, VehicleOrder::Trade { .. })
                            && vehicle.role != VehicleRole::Economic
                    {
                        return Err(CivilizationError::InvalidOrder);
                    }
                }
            }
        }
        let mut selected = HashSet::new();
        for id in &self.selected {
            if !selected.insert(*id)
                || !self
                    .vehicles
                    .iter()
                    .any(|vehicle| vehicle.id == *id && vehicle.owner == 0)
            {
                return Err(CivilizationError::InvalidSelection);
            }
        }
        let own = self
            .cities
            .iter()
            .filter(|city| city.controller() == 0)
            .count();
        if !match self.state {
            CivilizationState::Playing => own > 0 && own < self.cities.len(),
            CivilizationState::SpaceReady => own == self.cities.len(),
            CivilizationState::Defeated => own == 0,
        } {
            return Err(CivilizationError::InvalidSnapshot);
        }
        Ok(())
    }
}
fn capture(city: &mut CivilizationCity, owner: u8) {
    city.owner = owner;
    city.allied_to = None;
    city.health = city.max_health * 0.6;
    city.faith = 0.;
    city.influence_owner = owner;
    city.trade = 0.;
    city.trade_owner = owner;
    city.production.clear();
}
fn recalculate(city: &mut CivilizationCity) {
    let houses = city
        .buildings
        .iter()
        .filter(|building| building.kind == BuildingKind::House)
        .count() as f32;
    let factories = city
        .buildings
        .iter()
        .filter(|building| building.kind == BuildingKind::Factory)
        .count() as f32;
    let fun = city
        .buildings
        .iter()
        .filter(|building| building.kind == BuildingKind::Entertainment)
        .count() as f32;
    let mut adjacency = 0.;
    for house in city
        .buildings
        .iter()
        .filter(|building| building.kind == BuildingKind::House)
    {
        for other in &city.buildings {
            let distance = (house.slot as i16 - other.slot as i16).unsigned_abs();
            if distance == 1 || distance == 11 {
                adjacency += match other.kind {
                    BuildingKind::Factory => -0.08,
                    BuildingKind::Entertainment => 0.1,
                    _ => 0.,
                };
            }
        }
    }
    city.population = 3 + houses as u32 * 2;
    city.industry = 1. + factories * 0.75;
    city.happiness =
        (0.6 + fun * 0.12 - houses * 0.04 - factories * 0.06 + adjacency).clamp(0.15, 1.);
}
fn finite(value: CivilizationVec2) -> bool {
    value.x.is_finite() && value.y.is_finite()
}
fn inside(value: CivilizationVec2, half: CivilizationVec2) -> bool {
    finite(value) && value.x.abs() <= half.x && value.y.abs() <= half.y
}
fn range(value: f32, min: f32, max: f32) -> bool {
    value.is_finite() && (min..=max).contains(&value)
}
fn close(a: f32, b: f32) -> bool {
    a.is_finite() && b.is_finite() && (a - b).abs() < 1e-5
}
fn valid_name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && !value.chars().any(char::is_control)
}
fn unique(ids: &mut HashSet<u32>, id: u32) -> Result<(), CivilizationError> {
    if id == 0 || !ids.insert(id) {
        Err(CivilizationError::DuplicateId)
    } else {
        Ok(())
    }
}

impl CivilizationSnapshot {
    pub fn to_bytes(&self) -> Result<Vec<u8>, CivilizationError> {
        let w = CivilizationWorld::from_snapshot(self.clone())?;
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        u32o(&mut out, self.version);
        vo(&mut out, w.config.half_extents);
        u32o(&mut out, w.config.city_count as u32);
        u32o(&mut out, w.config.geyser_count as u32);
        f64o(&mut out, w.config.starting_currency);
        u64o(&mut out, w.seed);
        u64o(&mut out, w.rng);
        u32o(&mut out, w.next_id);
        u64o(&mut out, w.tick);
        f64o(&mut out, w.accumulator);
        out.push(match w.state {
            CivilizationState::Playing => 0,
            CivilizationState::SpaceReady => 1,
            CivilizationState::Defeated => 2,
        });
        u32o(&mut out, w.factions.len() as u32);
        for f in &w.factions {
            out.push(f.id);
            so(&mut out, &f.name);
            f64o(&mut out, f.currency);
            out.push(u8::from(f.ai));
        }
        u32o(&mut out, w.cities.len() as u32);
        for c in &w.cities {
            u32o(&mut out, c.id);
            so(&mut out, &c.name);
            vo(&mut out, c.position);
            out.push(c.owner);
            out.push(c.allied_to.unwrap_or(255));
            out.push(role(c.specialty));
            out.push(u8::from(c.coastal));
            u32o(&mut out, c.population);
            for v in [
                c.industry,
                c.happiness,
                c.health,
                c.max_health,
                c.faith,
                c.trade,
                c.weapon_cooldown,
                c.build_cooldown,
            ] {
                fo(&mut out, v);
            }
            out.push(c.influence_owner);
            out.push(c.trade_owner);
            u32o(&mut out, c.buildings.len() as u32);
            for b in &c.buildings {
                u32o(&mut out, b.id);
                out.push(match b.kind {
                    BuildingKind::House => 0,
                    BuildingKind::Factory => 1,
                    BuildingKind::Entertainment => 2,
                });
                out.push(b.slot);
            }
            u32o(&mut out, c.production.len() as u32);
            for j in &c.production {
                u32o(&mut out, j.id);
                out.push(role(j.role));
                out.push(domain(j.domain));
                fo(&mut out, j.remaining);
            }
        }
        u32o(&mut out, w.territories.len() as u32);
        for t in &w.territories {
            u32o(&mut out, t.id);
            u32o(&mut out, t.city);
            vo(&mut out, t.center);
            fo(&mut out, t.radius);
        }
        u32o(&mut out, w.geysers.len() as u32);
        for g in &w.geysers {
            u32o(&mut out, g.id);
            vo(&mut out, g.position);
            out.push(g.owner.unwrap_or(255));
            fo(&mut out, g.reserves);
            fo(&mut out, g.max_reserves);
        }
        u32o(&mut out, w.vehicles.len() as u32);
        for v in &w.vehicles {
            u32o(&mut out, v.id);
            out.push(v.owner);
            out.push(role(v.role));
            out.push(domain(v.domain));
            u32o(&mut out, v.home_city);
            vo(&mut out, v.position);
            for f in [v.heading, v.health, v.max_health, v.action_cooldown] {
                fo(&mut out, f);
            }
            match v.order {
                VehicleOrder::Idle => out.push(0),
                VehicleOrder::Move { destination } => {
                    out.push(1);
                    vo(&mut out, destination);
                }
                VehicleOrder::Claim { geyser } => {
                    out.push(2);
                    u32o(&mut out, geyser);
                }
                VehicleOrder::Attack { target } => {
                    out.push(3);
                    match target {
                        CivilizationTarget::City(id) => {
                            out.push(0);
                            u32o(&mut out, id);
                        }
                        CivilizationTarget::Vehicle(id) => {
                            out.push(1);
                            u32o(&mut out, id);
                        }
                    }
                }
                VehicleOrder::Convert { city } => {
                    out.push(4);
                    u32o(&mut out, city);
                }
                VehicleOrder::Trade { city, returning } => {
                    out.push(5);
                    u32o(&mut out, city);
                    out.push(u8::from(returning));
                }
            }
        }
        u32o(&mut out, w.selected.len() as u32);
        for id in w.selected {
            u32o(&mut out, id);
        }
        Ok(out)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CivilizationError> {
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(CivilizationError::TooManyEntities);
        }
        let mut r = Reader { bytes, at: 0 };
        if r.take(8)? != MAGIC {
            return Err(CivilizationError::InvalidSnapshot);
        }
        let version = r.u32()?;
        if version != 1 {
            return Err(CivilizationError::UnsupportedVersion(version));
        }
        let config = CivilizationConfig {
            half_extents: r.vec()?,
            city_count: r.count(8)?,
            geyser_count: r.count(128)?,
            starting_currency: r.f64()?,
        };
        config.validate()?;
        let seed = r.u64()?;
        let rng = r.u64()?;
        let next_id = r.u32()?;
        let tick = r.u64()?;
        let accumulator = r.f64()?;
        let state = match r.byte()? {
            0 => CivilizationState::Playing,
            1 => CivilizationState::SpaceReady,
            2 => CivilizationState::Defeated,
            _ => return Err(CivilizationError::InvalidSnapshot),
        };
        let mut factions = Vec::new();
        for _ in 0..r.count(8)? {
            factions.push(CivilizationFaction {
                id: r.byte()?,
                name: r.string()?,
                currency: r.f64()?,
                ai: r.boolean()?,
            });
        }
        let mut cities = Vec::new();
        for _ in 0..r.count(8)? {
            let id = r.u32()?;
            let name = r.string()?;
            let position = r.vec()?;
            let owner = r.byte()?;
            let allied_to = r.optional_owner()?;
            let specialty = r.role()?;
            let coastal = r.boolean()?;
            let population = r.u32()?;
            let industry = r.f32()?;
            let happiness = r.f32()?;
            let health = r.f32()?;
            let max_health = r.f32()?;
            let faith = r.f32()?;
            let trade = r.f32()?;
            let weapon_cooldown = r.f32()?;
            let build_cooldown = r.f32()?;
            let influence_owner = r.byte()?;
            let trade_owner = r.byte()?;
            let mut buildings = Vec::new();
            for _ in 0..r.count(12)? {
                let id = r.u32()?;
                let kind = match r.byte()? {
                    0 => BuildingKind::House,
                    1 => BuildingKind::Factory,
                    2 => BuildingKind::Entertainment,
                    _ => return Err(CivilizationError::InvalidEntity),
                };
                buildings.push(CityBuilding {
                    id,
                    kind,
                    slot: r.byte()?,
                });
            }
            let mut production = Vec::new();
            for _ in 0..r.count(6)? {
                production.push(ProductionJob {
                    id: r.u32()?,
                    role: r.role()?,
                    domain: r.domain()?,
                    remaining: r.f32()?,
                });
            }
            cities.push(CivilizationCity {
                id,
                name,
                position,
                owner,
                allied_to,
                specialty,
                coastal,
                population,
                industry,
                happiness,
                health,
                max_health,
                faith,
                influence_owner,
                trade,
                trade_owner,
                weapon_cooldown,
                build_cooldown,
                buildings,
                production,
            });
        }
        let mut territories = Vec::new();
        for _ in 0..r.count(8)? {
            territories.push(CivilizationTerritory {
                id: r.u32()?,
                city: r.u32()?,
                center: r.vec()?,
                radius: r.f32()?,
            });
        }
        let mut geysers = Vec::new();
        for _ in 0..r.count(128)? {
            geysers.push(CivilizationGeyser {
                id: r.u32()?,
                position: r.vec()?,
                owner: r.optional_owner()?,
                reserves: r.f32()?,
                max_reserves: r.f32()?,
            });
        }
        let mut vehicles = Vec::new();
        for _ in 0..r.count(256)? {
            let id = r.u32()?;
            let owner = r.byte()?;
            let role = r.role()?;
            let domain = r.domain()?;
            let home_city = r.u32()?;
            let position = r.vec()?;
            let heading = r.f32()?;
            let health = r.f32()?;
            let max_health = r.f32()?;
            let action_cooldown = r.f32()?;
            let order = match r.byte()? {
                0 => VehicleOrder::Idle,
                1 => VehicleOrder::Move {
                    destination: r.vec()?,
                },
                2 => VehicleOrder::Claim { geyser: r.u32()? },
                3 => VehicleOrder::Attack {
                    target: match r.byte()? {
                        0 => CivilizationTarget::City(r.u32()?),
                        1 => CivilizationTarget::Vehicle(r.u32()?),
                        _ => return Err(CivilizationError::InvalidOrder),
                    },
                },
                4 => VehicleOrder::Convert { city: r.u32()? },
                5 => VehicleOrder::Trade {
                    city: r.u32()?,
                    returning: r.boolean()?,
                },
                _ => return Err(CivilizationError::InvalidOrder),
            };
            vehicles.push(CivilizationVehicle {
                id,
                owner,
                role,
                domain,
                home_city,
                position,
                heading,
                health,
                max_health,
                action_cooldown,
                order,
            });
        }
        let mut selected = Vec::new();
        for _ in 0..r.count(256)? {
            selected.push(r.u32()?);
        }
        if r.at != bytes.len() {
            return Err(CivilizationError::TrailingBytes);
        }
        let world = CivilizationWorld {
            config,
            factions,
            cities,
            territories,
            vehicles,
            geysers,
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
fn role(value: VehicleRole) -> u8 {
    match value {
        VehicleRole::Military => 0,
        VehicleRole::Religious => 1,
        VehicleRole::Economic => 2,
    }
}
fn domain(value: VehicleDomain) -> u8 {
    match value {
        VehicleDomain::Land => 0,
        VehicleDomain::Sea => 1,
        VehicleDomain::Air => 2,
    }
}
fn u32o(o: &mut Vec<u8>, v: u32) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn u64o(o: &mut Vec<u8>, v: u64) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn fo(o: &mut Vec<u8>, v: f32) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn f64o(o: &mut Vec<u8>, v: f64) {
    o.extend_from_slice(&v.to_le_bytes());
}
fn vo(o: &mut Vec<u8>, v: CivilizationVec2) {
    fo(o, v.x);
    fo(o, v.y);
}
fn so(o: &mut Vec<u8>, v: &str) {
    u32o(o, v.len() as u32);
    o.extend_from_slice(v.as_bytes());
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], CivilizationError> {
        let end = self
            .at
            .checked_add(n)
            .ok_or(CivilizationError::TruncatedSnapshot)?;
        let bytes = self
            .bytes
            .get(self.at..end)
            .ok_or(CivilizationError::TruncatedSnapshot)?;
        self.at = end;
        Ok(bytes)
    }
    fn byte(&mut self) -> Result<u8, CivilizationError> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, CivilizationError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, CivilizationError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, CivilizationError> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn f64(&mut self) -> Result<f64, CivilizationError> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn vec(&mut self) -> Result<CivilizationVec2, CivilizationError> {
        Ok(CivilizationVec2::new(self.f32()?, self.f32()?))
    }
    fn count(&mut self, max: usize) -> Result<usize, CivilizationError> {
        let n = self.u32()? as usize;
        if n > max || n > LIMIT {
            Err(CivilizationError::TooManyEntities)
        } else {
            Ok(n)
        }
    }
    fn string(&mut self) -> Result<String, CivilizationError> {
        let n = self.count(64)?;
        Ok(std::str::from_utf8(self.take(n)?)
            .map_err(|_| CivilizationError::InvalidEntity)?
            .to_owned())
    }
    fn boolean(&mut self) -> Result<bool, CivilizationError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(CivilizationError::InvalidEntity),
        }
    }
    fn optional_owner(&mut self) -> Result<Option<u8>, CivilizationError> {
        let b = self.byte()?;
        Ok(if b == 255 { None } else { Some(b) })
    }
    fn role(&mut self) -> Result<VehicleRole, CivilizationError> {
        match self.byte()? {
            0 => Ok(VehicleRole::Military),
            1 => Ok(VehicleRole::Religious),
            2 => Ok(VehicleRole::Economic),
            _ => Err(CivilizationError::InvalidEntity),
        }
    }
    fn domain(&mut self) -> Result<VehicleDomain, CivilizationError> {
        match self.byte()? {
            0 => Ok(VehicleDomain::Land),
            1 => Ok(VehicleDomain::Sea),
            2 => Ok(VehicleDomain::Air),
            _ => Err(CivilizationError::InvalidEntity),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn quiet() -> CivilizationWorld {
        let mut world = CivilizationWorld::new(42);
        for faction in &mut world.factions {
            faction.ai = false;
        }
        world
    }
    fn unit(
        world: &mut CivilizationWorld,
        role: VehicleRole,
        domain: VehicleDomain,
        position: CivilizationVec2,
    ) -> u32 {
        let id = world.id();
        world.vehicles.push(CivilizationVehicle {
            id,
            owner: 0,
            role,
            domain,
            home_city: if domain == VehicleDomain::Sea {
                world.cities.iter().find(|city| city.coastal).unwrap().id
            } else {
                world.cities[0].id
            },
            position,
            heading: 0.,
            health: 80.,
            max_health: 80.,
            action_cooldown: 0.,
            order: VehicleOrder::Idle,
        });
        id
    }
    fn ticks(world: &mut CivilizationWorld, count: usize) {
        for _ in 0..count {
            world.step();
        }
    }
    fn only_one_enemy(world: &mut CivilizationWorld) {
        for index in 2..world.cities.len() {
            capture(&mut world.cities[index], 0);
        }
    }

    #[test]
    fn seeds_configs_territories_and_layouts_form_valid_bounded_worlds() {
        let a = CivilizationWorld::new(17);
        assert_eq!(a, CivilizationWorld::new(17));
        assert_ne!(a.geysers, CivilizationWorld::new(18).geysers);
        assert_eq!(a.progress(), 0.25);
        assert_eq!(a.territory_owner(a.territories[0].id), Some(0));
        for count in 2..=8 {
            let world = CivilizationWorld::with_config(
                31,
                CivilizationConfig {
                    city_count: count,
                    geyser_count: 128,
                    half_extents: CivilizationVec2::new(45., 35.),
                    ..Default::default()
                },
            )
            .unwrap();
            world.validate().unwrap();
            let points: HashSet<_> = world
                .cities
                .iter()
                .map(|city| (city.position.x.to_bits(), city.position.y.to_bits()))
                .collect();
            assert_eq!(points.len(), count);
        }
    }

    #[test]
    fn production_spends_real_money_waits_for_industry_and_respects_sea_docks() {
        let mut world = quiet();
        let home = world.cities[0].id;
        let before = world.currency();
        world
            .issue_command(CivilizationCommand::Produce {
                city: home,
                role: VehicleRole::Military,
                domain: VehicleDomain::Land,
            })
            .unwrap();
        assert_eq!(world.currency(), before - 80.);
        assert!(world.vehicles.is_empty());
        assert_eq!(world.cities[0].production.len(), 1);
        ticks(&mut world, 120);
        assert_eq!(world.vehicles.len(), 1);
        assert!(world.cities[0].production.is_empty());
        let before = world.clone();
        assert_eq!(
            world.issue_command(CivilizationCommand::Produce {
                city: home,
                role: VehicleRole::Economic,
                domain: VehicleDomain::Sea
            }),
            Err(CivilizationError::WrongDomain)
        );
        assert_eq!(world, before);
        capture(&mut world.cities[1], 0);
        let coast = world.cities[1].id;
        world
            .issue_command(CivilizationCommand::Produce {
                city: coast,
                role: VehicleRole::Economic,
                domain: VehicleDomain::Sea,
            })
            .unwrap();
        ticks(&mut world, 120);
        assert!(world
            .vehicles
            .iter()
            .any(|vehicle| vehicle.domain == VehicleDomain::Sea
                && world.accessible(vehicle.domain, vehicle.position)));
        world.validate().unwrap();
    }

    #[test]
    fn building_cost_slots_adjacency_population_industry_happiness_and_cooldown_matter() {
        let mut world = quiet();
        let city = world.cities[0].id;
        let income = world.cities[0].income_per_second();
        let happy = world.cities[0].happiness;
        let before = world.currency();
        world
            .issue_command(CivilizationCommand::Build {
                city,
                kind: BuildingKind::House,
                slot: 5,
            })
            .unwrap();
        assert_eq!(world.currency(), before - 50.);
        assert_eq!(world.cities[0].population, 7);
        assert!(world.cities[0].happiness < happy);
        assert_ne!(world.cities[0].income_per_second(), income);
        let before = world.clone();
        assert_eq!(
            world.issue_command(CivilizationCommand::Build {
                city,
                kind: BuildingKind::Factory,
                slot: 5
            }),
            Err(CivilizationError::InvalidPlacement)
        );
        assert_eq!(world, before);
        assert_eq!(
            world.issue_command(CivilizationCommand::Build {
                city,
                kind: BuildingKind::Entertainment,
                slot: 6
            }),
            Err(CivilizationError::Cooldown)
        );
        ticks(&mut world, 61);
        world
            .issue_command(CivilizationCommand::Build {
                city,
                kind: BuildingKind::Entertainment,
                slot: 6,
            })
            .unwrap();
        assert!(world.cities[0].happiness > happy - 0.1);
        ticks(&mut world, 61);
        world
            .issue_command(CivilizationCommand::Demolish { city, slot: 5 })
            .unwrap();
        assert_eq!(world.cities[0].population, 5);
        world.validate().unwrap();
    }

    #[test]
    fn insufficient_funds_wrong_roles_and_mixed_selection_reject_without_partial_changes() {
        let mut world = quiet();
        world.factions[0].currency = 0.;
        let before = world.clone();
        assert_eq!(
            world.issue_command(CivilizationCommand::Produce {
                city: world.cities[0].id,
                role: VehicleRole::Military,
                domain: VehicleDomain::Land
            }),
            Err(CivilizationError::InsufficientFunds)
        );
        assert_eq!(world, before);
        let position = world.cities[0].position;
        let a = unit(
            &mut world,
            VehicleRole::Military,
            VehicleDomain::Land,
            position,
        );
        let b = unit(
            &mut world,
            VehicleRole::Economic,
            VehicleDomain::Air,
            position,
        );
        world.select(vec![a, b]).unwrap();
        let before = world.clone();
        assert_eq!(
            world.issue_command(CivilizationCommand::Attack {
                target: CivilizationTarget::City(world.cities[1].id)
            }),
            Err(CivilizationError::WrongRole)
        );
        assert_eq!(world, before);
        assert_eq!(
            world.select(vec![a, a]),
            Err(CivilizationError::InvalidSelection)
        );
        assert_eq!(world, before);
    }

    #[test]
    fn land_routes_across_bridge_sea_passes_below_and_air_crosses_directly() {
        let mut world = quiet();
        let land = unit(
            &mut world,
            VehicleRole::Economic,
            VehicleDomain::Land,
            CivilizationVec2::new(-25., 20.),
        );
        let destination = CivilizationVec2::new(25., 20.);
        world.select(vec![land]).unwrap();
        world
            .issue_command(CivilizationCommand::Move { destination })
            .unwrap();
        for _ in 0..1200 {
            world.step();
            let actor = world
                .vehicles
                .iter()
                .find(|vehicle| vehicle.id == land)
                .unwrap();
            assert!(!world.is_water(actor.position));
            if matches!(actor.order, VehicleOrder::Idle) {
                break;
            }
        }
        assert!(world.vehicles[0].position.distance(destination) < 0.4);
        let boat = unit(
            &mut world,
            VehicleRole::Economic,
            VehicleDomain::Sea,
            CivilizationVec2::new(0., -20.),
        );
        world.select(vec![boat]).unwrap();
        world
            .issue_command(CivilizationCommand::Move {
                destination: CivilizationVec2::new(0., 20.),
            })
            .unwrap();
        ticks(&mut world, 600);
        assert!(
            world
                .vehicles
                .iter()
                .find(|vehicle| vehicle.id == boat)
                .unwrap()
                .position
                .y
                > 19.
        );
        let air = unit(
            &mut world,
            VehicleRole::Economic,
            VehicleDomain::Air,
            CivilizationVec2::new(-20., 25.),
        );
        world.select(vec![air]).unwrap();
        world
            .issue_command(CivilizationCommand::Move {
                destination: CivilizationVec2::new(20., 25.),
            })
            .unwrap();
        ticks(&mut world, 300);
        assert!(
            world
                .vehicles
                .iter()
                .find(|vehicle| vehicle.id == air)
                .unwrap()
                .position
                .x
                > 19.
        );
    }

    #[test]
    fn geyser_claim_requires_travel_and_income_stops_at_finite_depletion() {
        let mut world = quiet();
        let field = world.geysers[0].clone();
        let position = world.cities[0].position;
        let id = unit(
            &mut world,
            VehicleRole::Economic,
            VehicleDomain::Land,
            position,
        );
        world.select(vec![id]).unwrap();
        world
            .issue_command(CivilizationCommand::Claim { geyser: field.id })
            .unwrap();
        world.step();
        assert_eq!(world.geysers[0].owner, None);
        ticks(&mut world, 120);
        assert_eq!(world.geysers[0].owner, Some(0));
        world.geysers[0].reserves = 0.1;
        let before = world.currency();
        let base = world.cities[0].income_per_second() * CIVILIZATION_FIXED_DT;
        let events = world.step();
        let second = world.step();
        assert!(events.iter().chain(&second).any(
            |event| matches!(event,CivilizationEvent::GeyserDepleted{geyser}if *geyser==field.id)
        ));
        assert_eq!(world.geysers[0].reserves, 0.);
        assert!((world.currency() - before - 2. * base - 0.1).abs() < 1e-5);
        world.validate().unwrap();
    }

    #[test]
    fn military_city_conquest_completes_real_space_gate_and_is_terminal() {
        let mut world = quiet();
        only_one_enemy(&mut world);
        let target = world.cities[1].id;
        world.cities[1].health = 15.;
        let position = world.cities[1].position.add(CivilizationVec2::new(5., 0.));
        let id = unit(
            &mut world,
            VehicleRole::Military,
            VehicleDomain::Air,
            position,
        );
        world.select(vec![id]).unwrap();
        world
            .issue_command(CivilizationCommand::Attack {
                target: CivilizationTarget::City(target),
            })
            .unwrap();
        let events = world.step();
        assert_eq!(world.cities[1].owner, 0);
        assert_eq!(world.state, CivilizationState::SpaceReady);
        assert!(events.contains(&CivilizationEvent::SpaceEligible));
        assert_eq!(world.progress(), 1.);
        let before = world.clone();
        assert!(world.step().is_empty());
        assert_eq!(world, before);
        world.snapshot().to_bytes().unwrap();
    }

    #[test]
    fn religious_conversion_allies_native_city_and_unlocks_space_without_ownership_change() {
        let mut world = quiet();
        only_one_enemy(&mut world);
        let target = world.cities[1].id;
        let native = world.cities[1].owner;
        let position = world.cities[1].position.add(CivilizationVec2::new(5., 0.));
        let id = unit(
            &mut world,
            VehicleRole::Religious,
            VehicleDomain::Air,
            position,
        );
        world.select(vec![id]).unwrap();
        world
            .issue_command(CivilizationCommand::Convert { city: target })
            .unwrap();
        ticks(&mut world, 500);
        assert_eq!(world.cities[1].owner, native);
        assert_eq!(world.cities[1].allied_to, Some(0));
        assert_eq!(world.cities[1].controller(), 0);
        assert_eq!(world.state, CivilizationState::SpaceReady);
        world.snapshot().to_bytes().unwrap();
    }

    #[test]
    fn economic_route_requires_completed_trade_visits_and_purchase_budget() {
        let mut world = quiet();
        only_one_enemy(&mut world);
        let target = world.cities[1].id;
        let home = world.cities[0].id;
        let position = world.cities[1].position.add(CivilizationVec2::new(5., 0.));
        let id = unit(
            &mut world,
            VehicleRole::Economic,
            VehicleDomain::Air,
            position,
        );
        world.select(vec![id]).unwrap();
        assert_eq!(
            world.issue_command(CivilizationCommand::Purchase { city: target }),
            Err(CivilizationError::TradeRequired)
        );
        world
            .issue_command(CivilizationCommand::Trade { city: target })
            .unwrap();
        let mut visited_home = false;
        for _ in 0..3000 {
            world.step();
            let vehicle = world
                .vehicles
                .iter()
                .find(|vehicle| vehicle.id == id)
                .expect("trader survives three real round trips");
            visited_home |= vehicle.position.distance(world.cities[0].position) < 4.;
            if world.cities[1].trade >= 60. {
                break;
            }
        }
        assert!(visited_home);
        assert_eq!(world.cities[1].trade, 60.);
        assert!(world.cities.iter().any(|city| city.id == home));
        world.factions[0].currency = 0.;
        let before = world.clone();
        assert_eq!(
            world.issue_command(CivilizationCommand::Purchase { city: target }),
            Err(CivilizationError::InsufficientFunds)
        );
        assert_eq!(world, before);
        world.factions[0].currency = 1000.;
        let cost = purchase_cost(&world.cities[1]);
        world
            .issue_command(CivilizationCommand::Purchase { city: target })
            .unwrap();
        assert_eq!(world.currency(), 1000. - cost);
        assert_eq!(world.state, CivilizationState::SpaceReady);
        world.snapshot().to_bytes().unwrap();
    }

    #[test]
    fn vehicle_combat_destroyed_target_cleanup_and_defeat_restart_work() {
        let mut world = quiet();
        let a = unit(
            &mut world,
            VehicleRole::Military,
            VehicleDomain::Air,
            CivilizationVec2::new(-30., 20.),
        );
        let b = unit(
            &mut world,
            VehicleRole::Economic,
            VehicleDomain::Air,
            CivilizationVec2::new(-28., 20.),
        );
        world.vehicles[1].owner = 1;
        world.vehicles[1].health = 10.;
        world.select(vec![a]).unwrap();
        world
            .issue_command(CivilizationCommand::Attack {
                target: CivilizationTarget::Vehicle(b),
            })
            .unwrap();
        let events = world.step();
        assert!(events.contains(&CivilizationEvent::Destroyed { vehicle: b }));
        assert_eq!(world.vehicles[0].order, VehicleOrder::Idle);
        world.validate().unwrap();
        capture(&mut world.cities[0], 1);
        world.step();
        assert_eq!(world.state, CivilizationState::Defeated);
        world.restart();
        assert_eq!(world, CivilizationWorld::new(42));
    }

    #[test]
    fn ai_builds_produces_and_operates_military_religious_and_trade_orders() {
        let mut world = CivilizationWorld::new(72);
        ticks(&mut world, 720);
        assert!(world
            .vehicles
            .iter()
            .any(|vehicle| vehicle.owner == 1 && vehicle.role == VehicleRole::Military));
        assert!(world
            .vehicles
            .iter()
            .any(|vehicle| vehicle.owner == 2 && vehicle.role == VehicleRole::Religious));
        assert!(world
            .vehicles
            .iter()
            .any(|vehicle| vehicle.owner == 3 && vehicle.role == VehicleRole::Economic));
        assert!(world
            .vehicles
            .iter()
            .any(|vehicle| matches!(vehicle.order, VehicleOrder::Attack { .. })));
        assert!(world
            .vehicles
            .iter()
            .any(|vehicle| matches!(vehicle.order, VehicleOrder::Convert { .. })));
        assert!(world
            .vehicles
            .iter()
            .any(|vehicle| matches!(vehicle.order, VehicleOrder::Trade { .. })));
        assert!(world
            .cities
            .iter()
            .skip(1)
            .any(|city| city.buildings.len() > 3));
        world.validate().unwrap();
    }

    #[test]
    fn fixed_tick_partitions_save_jobs_ai_and_deterministic_continuation_match() {
        let mut a = CivilizationWorld::new(88);
        let city = a.cities[0].id;
        a.issue_command(CivilizationCommand::Produce {
            city,
            role: VehicleRole::Military,
            domain: VehicleDomain::Air,
        })
        .unwrap();
        let mut b = a.clone();
        for _ in 0..120 {
            a.advance(1. / 120.);
        }
        for _ in 0..20 {
            b.advance(1. / 20.);
        }
        assert_eq!(a.tick, b.tick);
        assert_eq!(a.cities, b.cities);
        assert_eq!(a.vehicles, b.vehicles);
        a.advance(0.127);
        let bytes = a.snapshot().to_bytes().unwrap();
        let mut loaded = CivilizationSnapshot::from_bytes(&bytes).unwrap().world;
        assert_eq!(loaded, a);
        for _ in 0..800 {
            assert_eq!(loaded.advance(0.021), a.advance(0.021));
        }
        assert_eq!(loaded, a);
        assert_eq!(
            loaded.snapshot().to_bytes().unwrap(),
            a.snapshot().to_bytes().unwrap()
        );
    }

    #[test]
    fn malformed_saves_reject_truncation_trailing_versions_counts_nonfinite_and_references() {
        let bytes = quiet().snapshot().to_bytes().unwrap();
        for length in 0..bytes.len() {
            assert!(
                CivilizationSnapshot::from_bytes(&bytes[..length]).is_err(),
                "accepted prefix {length}"
            );
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert_eq!(
            CivilizationSnapshot::from_bytes(&bad),
            Err(CivilizationError::TrailingBytes)
        );
        let mut bad = bytes.clone();
        bad[8..12].copy_from_slice(&2u32.to_le_bytes());
        assert_eq!(
            CivilizationSnapshot::from_bytes(&bad),
            Err(CivilizationError::UnsupportedVersion(2))
        );
        let mut bad = bytes.clone();
        bad[20..24].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            CivilizationSnapshot::from_bytes(&bad),
            Err(CivilizationError::TooManyEntities)
        );
        let mut bad = bytes;
        bad[12..16].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(CivilizationSnapshot::from_bytes(&bad).is_err());
        let world = quiet();
        let mut bad = world.snapshot();
        bad.world.cities[1].id = bad.world.cities[0].id;
        assert_eq!(bad.to_bytes(), Err(CivilizationError::DuplicateId));
        let mut bad = world.snapshot();
        bad.world.territories[0].city = u32::MAX;
        assert_eq!(bad.to_bytes(), Err(CivilizationError::InvalidReference));
        let mut bad = world.snapshot();
        bad.world.factions[0].currency = f64::NAN;
        assert!(bad.to_bytes().is_err());
        let mut bad = world.snapshot();
        bad.world.cities[0].industry = 100.;
        assert!(bad.to_bytes().is_err());
        let mut bad = world.snapshot();
        bad.world.selected.push(u32::MAX);
        assert!(bad.to_bytes().is_err());
    }

    #[test]
    fn catchup_debt_and_invalid_time_do_not_drop_ticks_or_poison_simulation() {
        let mut world = quiet();
        let report = world.advance(5.);
        assert_eq!(report.steps, 240);
        assert!((report.backlog_seconds - 1.).abs() < 1e-10);
        assert_eq!(world.advance(0.).steps, 60);
        let before = world.clone();
        assert!(world.advance(f64::NAN).time_rejected);
        assert!(world.advance(-1.).time_rejected);
        assert_eq!(world, before);
    }
}
