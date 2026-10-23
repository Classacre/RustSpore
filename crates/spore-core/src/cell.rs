//! Deterministic Cell-stage gameplay foundation.
//!
//! This is original prototype simulation and tuning, not a reconstruction of the
//! retail game's rules. Rendering, input devices, package assets and stage-to-stage
//! persistence belong to the application. Fixed ticks, snapshots and all gameplay
//! decisions here are engine-independent.

use crate::editor::{
    CellLoadout, Creation, CreationKind, EditorSnapshot, GameplayLoadout, MAX_SAVE_BYTES,
};
use std::collections::HashSet;
use std::fmt;

pub const CELL_FIXED_DT: f64 = 1.0 / 60.0;
const MAX_STEPS_PER_ADVANCE: u32 = 240;
const START_MASS: f32 = 4.0;
const MAX_ENTITIES: usize = 10_000;
const SNAPSHOT_MAGIC: &[u8; 8] = b"RSCELL01";

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CellVec2 {
    pub x: f32,
    pub y: f32,
}

impl CellVec2 {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }
    pub fn distance(self, other: Self) -> f32 {
        self.sub(other).length()
    }
    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
    pub fn scale(self, factor: f32) -> Self {
        Self::new(self.x * factor, self.y * factor)
    }
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }
    fn finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
    fn normalized(self) -> Self {
        let length = self.length();
        if length > 1e-6 {
            self.scale(1.0 / length)
        } else {
            Self::new(1.0, 0.0)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Diet {
    Herbivore,
    Carnivore,
    Omnivore,
}

impl Diet {
    pub fn can_eat(self, kind: FoodKind) -> bool {
        matches!(
            (self, kind),
            (Self::Omnivore, _)
                | (Self::Herbivore, FoodKind::Plant)
                | (Self::Carnivore, FoodKind::Meat)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodKind {
    Plant,
    Meat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellBehavior {
    Grazer,
    Scavenger,
    Predator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Playing,
    Evolved,
    Dead,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CellInput {
    /// Forward/reverse acceleration, clamped to -1..1.
    pub thrust: f32,
    /// Counterclockwise steering, clamped to -1..1.
    pub turn: f32,
    pub boost: bool,
    pub bite: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellActor {
    pub id: u32,
    pub position: CellVec2,
    pub velocity: CellVec2,
    /// Radians; zero points along positive x.
    pub heading: f32,
    pub mass: f32,
    pub health: f32,
    pub max_health: f32,
    /// Normalized 0..1 boost reserve.
    pub energy: f32,
    pub bite_cooldown: f32,
    pub diet: Diet,
    pub behavior: CellBehavior,
}

impl CellActor {
    fn new(id: u32, position: CellVec2, mass: f32, diet: Diet, behavior: CellBehavior) -> Self {
        let max_health = 40.0 + mass * 5.0;
        Self {
            id,
            position,
            velocity: CellVec2::ZERO,
            heading: 0.0,
            mass,
            health: max_health,
            max_health,
            energy: 1.0,
            bite_cooldown: 0.0,
            diet,
            behavior,
        }
    }
    pub fn radius(&self) -> f32 {
        0.55 + self.mass.sqrt() * 0.22
    }
    pub fn forward(&self) -> CellVec2 {
        CellVec2::new(self.heading.cos(), self.heading.sin())
    }
    pub fn mouth_position(&self) -> CellVec2 {
        self.position
            .add(self.forward().scale(self.radius() * 0.85))
    }
    pub fn alive(&self) -> bool {
        self.health > 0.0
    }
    fn grow(&mut self, mass: f32) {
        self.mass += mass;
        self.max_health += mass * 5.0;
        self.health = (self.health + mass * 2.0).min(self.max_health);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellFood {
    pub id: u32,
    pub position: CellVec2,
    pub kind: FoodKind,
    pub mass: f32,
}

impl CellFood {
    pub fn radius(&self) -> f32 {
        0.22 + self.mass.sqrt() * 0.12
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellObstacle {
    pub id: u32,
    pub position: CellVec2,
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellConfig {
    pub half_extents: CellVec2,
    pub evolution_mass: f32,
    pub food_count: usize,
    pub enemy_count: usize,
    pub obstacle_count: usize,
}

impl Default for CellConfig {
    fn default() -> Self {
        Self {
            half_extents: CellVec2::new(40.0, 28.0),
            evolution_mass: 30.0,
            food_count: 72,
            enemy_count: 10,
            obstacle_count: 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellEvent {
    Ate {
        actor: u32,
        food: u32,
        kind: FoodKind,
        mass: f32,
    },
    Damaged {
        actor: u32,
        amount: f32,
    },
    Killed {
        actor: u32,
    },
    Evolved,
    PlayerDied,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepReport {
    pub steps: u32,
    pub backlog_seconds: f64,
    pub interpolation: f32,
    pub state: CellState,
    pub events: Vec<CellEvent>,
    pub time_rejected: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellWorld {
    pub config: CellConfig,
    pub player: CellActor,
    pub enemies: Vec<CellActor>,
    pub food: Vec<CellFood>,
    pub obstacles: Vec<CellObstacle>,
    pub state: CellState,
    pub tick: u64,
    pub eaten: u32,
    pub kills: u32,
    /// Applied original editor stats; None retains the baseline rules.
    pub loadout: Option<CellLoadout>,
    pub creation: Option<Creation>,
    seed: u64,
    rng: u64,
    next_id: u32,
    accumulator: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellSnapshot {
    pub version: u32,
    pub world: CellWorld,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellError {
    InvalidConfig,
    InvalidEntity,
    DuplicateId,
    TooManyEntities,
    InvalidSnapshot,
    UnsupportedVersion(u32),
    TruncatedSnapshot,
    TrailingBytes,
    InvalidLoadout,
    InvalidCreation,
}

impl fmt::Display for CellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig => f.write_str("invalid Cell configuration"),
            Self::InvalidEntity => f.write_str("invalid Cell entity"),
            Self::DuplicateId => f.write_str("duplicate Cell entity ID"),
            Self::TooManyEntities => f.write_str("too many Cell entities"),
            Self::InvalidSnapshot => f.write_str("invalid Cell snapshot"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported Cell snapshot version {v}"),
            Self::TruncatedSnapshot => f.write_str("truncated Cell snapshot"),
            Self::TrailingBytes => f.write_str("trailing bytes in Cell snapshot"),
            Self::InvalidLoadout => f.write_str("invalid authored Cell loadout"),
            Self::InvalidCreation => f.write_str("invalid authored Cell creation"),
        }
    }
}
impl std::error::Error for CellError {}

impl CellConfig {
    fn validate(&self) -> Result<(), CellError> {
        if !self.half_extents.finite()
            || self.half_extents.x < 8.0
            || self.half_extents.y < 8.0
            || self.half_extents.x > 100_000.0
            || self.half_extents.y > 100_000.0
            || !self.evolution_mass.is_finite()
            || self.evolution_mass <= START_MASS
            || self.evolution_mass > 100_000.0
            || self.food_count > 5_000
            || self.enemy_count > 128
            || self.obstacle_count > 256
        {
            return Err(CellError::InvalidConfig);
        }
        Ok(())
    }
}

impl CellWorld {
    pub fn new(seed: u64, diet: Diet) -> Self {
        Self::with_config(seed, diet, CellConfig::default()).expect("default Cell config is valid")
    }

    pub fn with_config(seed: u64, diet: Diet, config: CellConfig) -> Result<Self, CellError> {
        config.validate()?;
        let mut world = Self {
            config,
            player: CellActor::new(1, CellVec2::ZERO, START_MASS, diet, CellBehavior::Grazer),
            enemies: Vec::new(),
            food: Vec::new(),
            obstacles: Vec::new(),
            state: CellState::Playing,
            tick: 0,
            eaten: 0,
            kills: 0,
            loadout: None,
            creation: None,
            seed,
            rng: if seed == 0 { 0x9e3779b97f4a7c15 } else { seed },
            next_id: 2,
            accumulator: 0.0,
        };
        for _ in 0..world.config.obstacle_count {
            let position = world.spawn_position(8.0);
            let radius = 0.9 + world.random() * 1.2;
            let id = world.allocate_id();
            world.obstacles.push(CellObstacle {
                id,
                position,
                radius,
            });
        }
        for index in 0..world.config.enemy_count {
            let position = world.spawn_position(12.0);
            let (enemy_diet, behavior, mass) = match index % 3 {
                0 => (
                    Diet::Herbivore,
                    CellBehavior::Grazer,
                    2.0 + world.random() * 4.0,
                ),
                1 => (
                    Diet::Carnivore,
                    CellBehavior::Predator,
                    6.0 + world.random() * 4.0,
                ),
                _ => (
                    Diet::Omnivore,
                    CellBehavior::Scavenger,
                    3.0 + world.random() * 4.0,
                ),
            };
            let id = world.allocate_id();
            let mut actor = CellActor::new(id, position, mass, enemy_diet, behavior);
            actor.heading = world.random() * std::f32::consts::TAU;
            world.enemies.push(actor);
        }
        // A short, safe feeding trail makes the first controls discoverable.
        for index in 0..world.config.food_count.min(8) {
            let angle = (index as f32 % 4.0 - 1.5) * 0.22;
            let distance = 2.8 + index as f32 * 0.35;
            let position = CellVec2::new(angle.cos() * distance, angle.sin() * distance);
            let kind = if diet == Diet::Carnivore {
                FoodKind::Meat
            } else {
                FoodKind::Plant
            };
            let id = world.allocate_id();
            world.food.push(CellFood {
                id,
                position,
                kind,
                mass: 0.8,
            });
        }
        world.replenish_food();
        Ok(world)
    }

    pub fn restart(&mut self) {
        let loadout = self.loadout.clone();
        let creation = self.creation.clone();
        *self = Self::with_config(self.seed, self.player.diet, self.config.clone())
            .expect("a validated world's config remains valid");
        if let Some(loadout) = loadout {
            self.apply_loadout(loadout)
                .expect("validated authored Cell loadout");
        }
        self.creation = creation;
    }

    /// Apply validated prototype stats atomically, preserving health percentage.
    pub fn apply_loadout(&mut self, loadout: CellLoadout) -> Result<(), CellError> {
        validate_loadout(&loadout)?;
        if !self.player.mass.is_finite()
            || self.player.mass < START_MASS
            || !self.player.max_health.is_finite()
            || self.player.max_health <= 0.0
            || !self.player.health.is_finite()
            || !(0.0..=self.player.max_health).contains(&self.player.health)
        {
            return Err(CellError::InvalidEntity);
        }
        let fraction = (self.player.health / self.player.max_health).clamp(0.0, 1.0);
        self.player.max_health = loadout.health + (self.player.mass - START_MASS) * 5.0;
        self.player.health = self.player.max_health * fraction;
        self.player.diet = loadout.diet;
        self.loadout = Some(loadout);
        self.creation = None;
        Ok(())
    }
    pub fn apply_creation(&mut self, creation: Creation) -> Result<(), CellError> {
        if creation.kind != CreationKind::Cell {
            return Err(CellError::InvalidCreation);
        }
        let GameplayLoadout::Cell(loadout) = creation
            .gameplay_loadout()
            .map_err(|_| CellError::InvalidCreation)?
        else {
            return Err(CellError::InvalidCreation);
        };
        self.apply_loadout(loadout)?;
        self.creation = Some(creation);
        Ok(())
    }
    /// Original perception tuning. Unauthored worlds retain their full arena view.
    pub fn perception_radius(&self) -> f32 {
        self.loadout
            .as_ref()
            .map(|loadout| if loadout.has_eyes { 36.0 } else { 8.0 })
            .unwrap_or_else(|| self.config.half_extents.length() * 2.0)
    }
    pub fn can_perceive(&self, position: CellVec2) -> bool {
        self.player.position.distance(position) <= self.perception_radius()
    }

    pub fn progress(&self) -> f32 {
        ((self.player.mass - START_MASS) / (self.config.evolution_mass - START_MASS))
            .clamp(0.0, 1.0)
    }

    /// The seed used when this world was created, retained through save/load.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Advance fixed ticks. Catch-up is bounded per call, but elapsed time is
    /// retained as backlog instead of silently discarded. Inputs are held for
    /// all ticks in this call. A replay should record input for each fixed tick.
    pub fn advance(&mut self, elapsed: f64, input: CellInput) -> StepReport {
        let invalid =
            !elapsed.is_finite() || elapsed < 0.0 || !(self.accumulator + elapsed).is_finite();
        let mut report = StepReport {
            steps: 0,
            backlog_seconds: self.accumulator,
            interpolation: (self.accumulator / CELL_FIXED_DT).clamp(0.0, 1.0) as f32,
            state: self.state,
            events: Vec::new(),
            time_rejected: invalid,
        };
        if invalid {
            return report;
        }
        if self.state != CellState::Playing {
            self.accumulator = 0.0;
            report.backlog_seconds = 0.0;
            report.interpolation = 0.0;
            return report;
        }
        self.accumulator += elapsed;
        while self.accumulator + 1e-12 >= CELL_FIXED_DT && report.steps < MAX_STEPS_PER_ADVANCE {
            self.accumulator = (self.accumulator - CELL_FIXED_DT).max(0.0);
            report.events.extend(self.step(input));
            report.steps += 1;
            if self.state != CellState::Playing {
                self.accumulator = 0.0;
                break;
            }
        }
        report.state = self.state;
        report.backlog_seconds = self.accumulator;
        report.interpolation = (self.accumulator / CELL_FIXED_DT).clamp(0.0, 1.0) as f32;
        report
    }

    /// Exactly one fixed gameplay tick, convenient for deterministic replays.
    pub fn step(&mut self, input: CellInput) -> Vec<CellEvent> {
        let mut events = Vec::new();
        if self.state != CellState::Playing {
            return events;
        }
        self.tick = self.tick.saturating_add(1);
        let dt = CELL_FIXED_DT as f32;
        let (speed, reserve) = self
            .loadout
            .as_ref()
            .map(|loadout| (loadout.speed, loadout.boost_capacity))
            .unwrap_or((5.5, 1.0));
        drive_tuned(&mut self.player, input, dt, speed, reserve);
        for enemy in &mut self.enemies {
            let to_player = self.player.position.sub(enemy.position);
            let player_distance = to_player.length();
            let target = if enemy.mass < self.player.mass * 0.65 && player_distance < 6.0 {
                enemy.position.sub(to_player)
            } else if enemy.behavior == CellBehavior::Predator
                && player_distance < 8.0
                && enemy.mass >= self.player.mass * 0.75
            {
                self.player.position
            } else {
                self.food
                    .iter()
                    .filter(|food| enemy.diet.can_eat(food.kind))
                    .min_by(|a, b| {
                        enemy
                            .position
                            .sub(a.position)
                            .length_squared()
                            .total_cmp(&enemy.position.sub(b.position).length_squared())
                    })
                    .map(|food| food.position)
                    .unwrap_or_else(|| {
                        let angle = self.tick as f32 * 0.003 + enemy.id as f32;
                        enemy
                            .position
                            .add(CellVec2::new(angle.cos(), angle.sin()).scale(4.0))
                    })
            };
            let delta = target.sub(enemy.position);
            let desired = delta.y.atan2(delta.x);
            let turn = wrap_angle(desired - enemy.heading).clamp(-1.0, 1.0);
            drive(
                enemy,
                CellInput {
                    thrust: 0.7,
                    turn,
                    boost: false,
                    bite: false,
                },
                dt,
            );
        }
        constrain(&mut self.player, self.config.half_extents, &self.obstacles);
        for enemy in &mut self.enemies {
            constrain(enemy, self.config.half_extents, &self.obstacles);
        }
        for enemy in &mut self.enemies {
            separate(&mut self.player, enemy);
        }
        for i in 0..self.enemies.len() {
            let (before, after) = self.enemies.split_at_mut(i + 1);
            for other in after {
                separate(&mut before[i], other);
            }
        }
        constrain(&mut self.player, self.config.half_extents, &self.obstacles);
        for enemy in &mut self.enemies {
            constrain(enemy, self.config.half_extents, &self.obstacles);
        }

        if input.bite && self.player.bite_cooldown <= 0.0 {
            self.player.bite_cooldown = 0.45;
            if let Some(enemy) = self
                .enemies
                .iter_mut()
                .filter(|enemy| enemy.alive())
                .find(|enemy| bite_contact(&self.player, enemy))
            {
                let amount = self
                    .loadout
                    .as_ref()
                    .map(|loadout| {
                        loadout.bite_damage + (self.player.mass - START_MASS).max(0.0) * 0.8
                    })
                    .unwrap_or(6.0 + self.player.mass * 0.8);
                enemy.health = (enemy.health - amount).max(0.0);
                events.push(CellEvent::Damaged {
                    actor: enemy.id,
                    amount,
                });
            }
        }
        for enemy in &mut self.enemies {
            if enemy.alive()
                && enemy.behavior == CellBehavior::Predator
                && enemy.bite_cooldown <= 0.0
                && enemy.mass >= self.player.mass * 0.75
                && bite_contact(enemy, &self.player)
            {
                enemy.bite_cooldown = 0.8;
                let amount = (3.0 + enemy.mass * 0.5)
                    * (1.0
                        - self
                            .loadout
                            .as_ref()
                            .map(|loadout| loadout.defense)
                            .unwrap_or(0.0));
                self.player.health = (self.player.health - amount).max(0.0);
                events.push(CellEvent::Damaged {
                    actor: self.player.id,
                    amount,
                });
            }
        }
        if self.player.alive() {
            feed(
                &mut self.player,
                &mut self.food,
                &mut events,
                &mut self.eaten,
            );
        }
        for enemy in &mut self.enemies {
            if enemy.alive() {
                let mut ignored = 0;
                feed(enemy, &mut self.food, &mut events, &mut ignored);
            }
        }
        let corpses: Vec<_> = self
            .enemies
            .iter()
            .filter(|enemy| !enemy.alive())
            .map(|enemy| (enemy.id, enemy.position, (enemy.mass * 0.7).min(8.0)))
            .collect();
        self.enemies.retain(CellActor::alive);
        for (actor, position, mass) in corpses {
            let id = self.allocate_id();
            self.food.push(CellFood {
                id,
                position,
                kind: FoodKind::Meat,
                mass,
            });
            self.kills = self.kills.saturating_add(1);
            events.push(CellEvent::Killed { actor });
        }
        if !self.player.alive() {
            self.state = CellState::Dead;
            events.push(CellEvent::PlayerDied);
        } else if self.player.mass >= self.config.evolution_mass {
            self.state = CellState::Evolved;
            events.push(CellEvent::Evolved);
        }
        if self.state == CellState::Playing {
            self.replenish_food();
        }
        events
    }

    pub fn add_food(
        &mut self,
        position: CellVec2,
        kind: FoodKind,
        mass: f32,
    ) -> Result<u32, CellError> {
        if !self.valid_position(position) || !mass.is_finite() || mass <= 0.0 || mass > 1_000.0 {
            return Err(CellError::InvalidEntity);
        }
        if self.food.len() + self.enemies.len() + self.obstacles.len() >= MAX_ENTITIES - 1 {
            return Err(CellError::TooManyEntities);
        }
        let id = self.allocate_id();
        self.food.push(CellFood {
            id,
            position,
            kind,
            mass,
        });
        Ok(id)
    }

    pub fn add_enemy(
        &mut self,
        position: CellVec2,
        mass: f32,
        diet: Diet,
        behavior: CellBehavior,
    ) -> Result<u32, CellError> {
        if !self.valid_position(position) || !mass.is_finite() || mass <= 0.0 || mass > 1_000.0 {
            return Err(CellError::InvalidEntity);
        }
        if self.food.len() + self.enemies.len() + self.obstacles.len() >= MAX_ENTITIES - 1 {
            return Err(CellError::TooManyEntities);
        }
        let id = self.allocate_id();
        self.enemies
            .push(CellActor::new(id, position, mass, diet, behavior));
        Ok(id)
    }

    pub fn snapshot(&self) -> CellSnapshot {
        CellSnapshot {
            version: 2,
            world: self.clone(),
        }
    }

    pub fn from_snapshot(snapshot: CellSnapshot) -> Result<Self, CellError> {
        if !matches!(snapshot.version, 1 | 2) {
            return Err(CellError::UnsupportedVersion(snapshot.version));
        }
        if snapshot.version == 1
            && (snapshot.world.loadout.is_some() || snapshot.world.creation.is_some())
        {
            return Err(CellError::InvalidSnapshot);
        }
        snapshot.world.validate()?;
        Ok(snapshot.world)
    }

    fn validate(&self) -> Result<(), CellError> {
        self.config.validate()?;
        if let Some(creation) = &self.creation {
            if creation.kind != CreationKind::Cell
                || creation
                    .gameplay_loadout()
                    .map_err(|_| CellError::InvalidCreation)?
                    != GameplayLoadout::Cell(
                        self.loadout.clone().ok_or(CellError::InvalidCreation)?,
                    )
            {
                return Err(CellError::InvalidCreation);
            }
        }
        if let Some(loadout) = &self.loadout {
            validate_loadout(loadout)?;
            let expected = loadout.health + (self.player.mass - START_MASS) * 5.0;
            if self.player.mass < START_MASS
                || self.player.diet != loadout.diet
                || (self.player.max_health - expected).abs() > expected.abs() * 0.001 + 0.001
            {
                return Err(CellError::InvalidLoadout);
            }
        }
        if self.rng == 0
            || !self.accumulator.is_finite()
            || self.accumulator < 0.0
            || self.next_id == 0
        {
            return Err(CellError::InvalidSnapshot);
        }
        if self.food.len() + self.enemies.len() + self.obstacles.len() + 1 > MAX_ENTITIES {
            return Err(CellError::TooManyEntities);
        }
        let mut ids = HashSet::new();
        for actor in std::iter::once(&self.player).chain(&self.enemies) {
            if actor.id == 0
                || !self.valid_position(actor.position)
                || !actor.velocity.finite()
                || !actor.heading.is_finite()
                || !actor.mass.is_finite()
                || actor.mass <= 0.0
                || actor.mass > 100_000.0
                || !actor.health.is_finite()
                || !actor.max_health.is_finite()
                || actor.max_health <= 0.0
                || actor.health < 0.0
                || actor.health > actor.max_health
                || !actor.energy.is_finite()
                || !(0.0..=1.0).contains(&actor.energy)
                || !actor.bite_cooldown.is_finite()
                || actor.bite_cooldown < 0.0
            {
                return Err(CellError::InvalidEntity);
            }
            if !ids.insert(actor.id) {
                return Err(CellError::DuplicateId);
            }
        }
        for food in &self.food {
            if food.id == 0
                || !self.valid_position(food.position)
                || !food.mass.is_finite()
                || food.mass <= 0.0
                || food.mass > 1_000.0
            {
                return Err(CellError::InvalidEntity);
            }
            if !ids.insert(food.id) {
                return Err(CellError::DuplicateId);
            }
        }
        for obstacle in &self.obstacles {
            if obstacle.id == 0
                || !self.valid_position(obstacle.position)
                || !obstacle.radius.is_finite()
                || obstacle.radius <= 0.0
                || obstacle.radius > self.config.half_extents.x.min(self.config.half_extents.y)
            {
                return Err(CellError::InvalidEntity);
            }
            if !ids.insert(obstacle.id) {
                return Err(CellError::DuplicateId);
            }
        }
        if (self.state == CellState::Dead) != !self.player.alive()
            || (self.state == CellState::Evolved && self.player.mass < self.config.evolution_mass)
        {
            return Err(CellError::InvalidSnapshot);
        }
        Ok(())
    }

    fn valid_position(&self, p: CellVec2) -> bool {
        p.finite()
            && p.x.abs() <= self.config.half_extents.x
            && p.y.abs() <= self.config.half_extents.y
    }
    fn allocate_id(&mut self) -> u32 {
        // IDs may wrap during a long session. At most MAX_ENTITIES are alive,
        // so a free u32 identifier always exists; never panic on a loaded cursor.
        let mut id = self.next_id.max(2);
        loop {
            if id != self.player.id
                && !self.enemies.iter().any(|e| e.id == id)
                && !self.food.iter().any(|e| e.id == id)
                && !self.obstacles.iter().any(|e| e.id == id)
            {
                self.next_id = id.wrapping_add(1).max(2);
                return id;
            }
            id = id.wrapping_add(1).max(2);
        }
    }
    fn random(&mut self) -> f32 {
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        let value = self.rng.wrapping_mul(0x2545f4914f6cdd1d);
        ((value >> 40) as u32) as f32 / 16_777_216.0
    }
    fn spawn_position(&mut self, minimum_radius: f32) -> CellVec2 {
        for _ in 0..64 {
            let position = CellVec2::new(
                (self.random() * 2.0 - 1.0) * (self.config.half_extents.x - 3.0),
                (self.random() * 2.0 - 1.0) * (self.config.half_extents.y - 3.0),
            );
            if position.length()
                >= minimum_radius
                    .min(self.config.half_extents.x.min(self.config.half_extents.y) - 3.0)
                && self
                    .obstacles
                    .iter()
                    .all(|o| position.distance(o.position) > o.radius + 2.0)
            {
                return position;
            }
        }
        CellVec2::new(
            self.config.half_extents.x - 3.0,
            self.config.half_extents.y - 3.0,
        )
    }
    fn replenish_food(&mut self) {
        while self.food.len() < self.config.food_count {
            let position = self.spawn_position(0.0);
            let kind = if self.random() < 0.6 {
                FoodKind::Plant
            } else {
                FoodKind::Meat
            };
            let mass = if kind == FoodKind::Plant { 0.8 } else { 1.2 };
            let id = self.allocate_id();
            self.food.push(CellFood {
                id,
                position,
                kind,
                mass,
            });
        }
    }
}

fn drive(actor: &mut CellActor, input: CellInput, dt: f32) {
    drive_tuned(actor, input, dt, 5.5, 1.0);
}
fn drive_tuned(actor: &mut CellActor, input: CellInput, dt: f32, base_speed: f32, reserve: f32) {
    let turn = if input.turn.is_finite() {
        input.turn.clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let thrust = if input.thrust.is_finite() {
        input.thrust.clamp(-1.0, 1.0)
    } else {
        0.0
    };
    actor.heading = wrap_angle(actor.heading + turn * 2.8 * dt);
    // Original baseline tuning: boost increases acceleration as well as the cap.
    // Holding an exhausted boost cannot alternate free bursts and recharging;
    // release boost (or stop thrusting) to recover the reserve.
    let wants_boost = input.boost && thrust > 0.0;
    let cost = 0.28 * dt / reserve;
    let boosting = wants_boost && actor.energy >= cost;
    actor.energy = if boosting {
        (actor.energy - cost).max(0.0)
    } else if wants_boost {
        0.0
    } else {
        (actor.energy + 0.18 * dt / reserve).min(1.0)
    };
    let speed =
        base_speed * (START_MASS / actor.mass).powf(0.12) * if boosting { 1.75 } else { 1.0 };
    let acceleration = 16.0 * (base_speed / 5.5) * if boosting { 1.75 } else { 1.0 };
    actor.velocity = actor
        .velocity
        .add(actor.forward().scale(thrust * acceleration * dt));
    actor.velocity = actor.velocity.scale((-2.8 * dt).exp());
    if actor.velocity.length_squared() > speed * speed {
        actor.velocity = actor.velocity.normalized().scale(speed);
    }
    actor.position = actor.position.add(actor.velocity.scale(dt));
    actor.bite_cooldown = (actor.bite_cooldown - dt).max(0.0);
}

fn wrap_angle(angle: f32) -> f32 {
    (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

fn constrain(actor: &mut CellActor, half: CellVec2, obstacles: &[CellObstacle]) {
    let radius = actor.radius();
    for obstacle in obstacles {
        let delta = actor.position.sub(obstacle.position);
        let distance = delta.length();
        let limit = radius + obstacle.radius;
        if distance < limit {
            let normal = delta.normalized();
            actor.position = obstacle.position.add(normal.scale(limit));
            let inward = actor.velocity.dot(normal).min(0.0);
            actor.velocity = actor.velocity.sub(normal.scale(inward));
        }
    }
    let max_x = (half.x - radius).max(0.0);
    let max_y = (half.y - radius).max(0.0);
    if actor.position.x.abs() > max_x {
        actor.position.x = actor.position.x.clamp(-max_x, max_x);
        actor.velocity.x = 0.0;
    }
    if actor.position.y.abs() > max_y {
        actor.position.y = actor.position.y.clamp(-max_y, max_y);
        actor.velocity.y = 0.0;
    }
}

fn separate(a: &mut CellActor, b: &mut CellActor) {
    let delta = b.position.sub(a.position);
    let distance = delta.length();
    let limit = a.radius() + b.radius();
    if distance >= limit {
        return;
    }
    let normal = delta.normalized();
    let penetration = limit - distance;
    let ratio = b.mass / (a.mass + b.mass);
    a.position = a.position.sub(normal.scale(penetration * ratio));
    b.position = b.position.add(normal.scale(penetration * (1.0 - ratio)));
    let closing = b.velocity.sub(a.velocity).dot(normal);
    if closing < 0.0 {
        a.velocity = a.velocity.add(normal.scale(closing * ratio));
        b.velocity = b.velocity.sub(normal.scale(closing * (1.0 - ratio)));
    }
}

fn bite_contact(a: &CellActor, b: &CellActor) -> bool {
    let delta = b.position.sub(a.position);
    let distance = delta.length();
    distance <= a.radius() + b.radius() + 0.25 && a.forward().dot(delta) >= distance * 0.3
}

fn feed(
    actor: &mut CellActor,
    food: &mut Vec<CellFood>,
    events: &mut Vec<CellEvent>,
    eaten: &mut u32,
) {
    let mouth = actor.mouth_position();
    let mouth_radius = actor.radius() * 0.5;
    food.retain(|item| {
        if actor.diet.can_eat(item.kind)
            && mouth.distance(item.position) <= mouth_radius + item.radius()
        {
            actor.grow(item.mass);
            *eaten = eaten.saturating_add(1);
            events.push(CellEvent::Ate {
                actor: actor.id,
                food: item.id,
                kind: item.kind,
                mass: item.mass,
            });
            false
        } else {
            true
        }
    });
}

impl CellSnapshot {
    /// Versioned little-endian save data, including RNG and fractional tick debt.
    pub fn to_bytes(&self) -> Result<Vec<u8>, CellError> {
        let world = CellWorld::from_snapshot(self.clone())?;
        let mut out = Vec::new();
        out.extend_from_slice(SNAPSHOT_MAGIC);
        put_u32(&mut out, self.version);
        put_vec(&mut out, world.config.half_extents);
        put_f32(&mut out, world.config.evolution_mass);
        for count in [
            world.config.food_count,
            world.config.enemy_count,
            world.config.obstacle_count,
        ] {
            put_u32(&mut out, count as u32);
        }
        put_u64(&mut out, world.seed);
        put_u64(&mut out, world.rng);
        put_u32(&mut out, world.next_id);
        put_u64(&mut out, world.tick);
        put_u32(&mut out, world.eaten);
        put_u32(&mut out, world.kills);
        out.push(match world.state {
            CellState::Playing => 0,
            CellState::Evolved => 1,
            CellState::Dead => 2,
        });
        out.extend_from_slice(&world.accumulator.to_le_bytes());
        put_actor(&mut out, &world.player);
        put_u32(&mut out, world.enemies.len() as u32);
        for actor in &world.enemies {
            put_actor(&mut out, actor);
        }
        put_u32(&mut out, world.food.len() as u32);
        for food in &world.food {
            put_u32(&mut out, food.id);
            put_vec(&mut out, food.position);
            out.push(if food.kind == FoodKind::Plant { 0 } else { 1 });
            put_f32(&mut out, food.mass);
        }
        put_u32(&mut out, world.obstacles.len() as u32);
        for obstacle in &world.obstacles {
            put_u32(&mut out, obstacle.id);
            put_vec(&mut out, obstacle.position);
            put_f32(&mut out, obstacle.radius);
        }
        if self.version == 2 {
            out.push(u8::from(world.loadout.is_some()));
            if let Some(loadout) = &world.loadout {
                out.push(match loadout.diet {
                    Diet::Herbivore => 0,
                    Diet::Carnivore => 1,
                    Diet::Omnivore => 2,
                });
                for value in [
                    loadout.health,
                    loadout.speed,
                    loadout.bite_damage,
                    loadout.boost_capacity,
                    loadout.defense,
                ] {
                    put_f32(&mut out, value);
                }
                out.push(u8::from(loadout.has_eyes));
            }
            out.push(u8::from(world.creation.is_some()));
            if let Some(creation) = &world.creation {
                let bytes = EditorSnapshot {
                    version: 1,
                    revision: 0,
                    creation: creation.clone(),
                }
                .to_bytes()
                .map_err(|_| CellError::InvalidCreation)?;
                put_u32(&mut out, bytes.len() as u32);
                out.extend_from_slice(&bytes);
            }
        }
        Ok(out)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CellError> {
        let mut read = Reader { bytes, at: 0 };
        if read.take(8)? != SNAPSHOT_MAGIC {
            return Err(CellError::InvalidSnapshot);
        }
        let version = read.u32()?;
        if !matches!(version, 1 | 2) {
            return Err(CellError::UnsupportedVersion(version));
        }
        let config = CellConfig {
            half_extents: read.vec()?,
            evolution_mass: read.f32()?,
            food_count: read.count()?,
            enemy_count: read.count()?,
            obstacle_count: read.count()?,
        };
        config.validate()?;
        let seed = read.u64()?;
        let rng = read.u64()?;
        let next_id = read.u32()?;
        let tick = read.u64()?;
        let eaten = read.u32()?;
        let kills = read.u32()?;
        let state = match read.byte()? {
            0 => CellState::Playing,
            1 => CellState::Evolved,
            2 => CellState::Dead,
            _ => return Err(CellError::InvalidSnapshot),
        };
        let accumulator = f64::from_le_bytes(read.take(8)?.try_into().unwrap());
        let player = read.actor()?;
        let enemy_count = read.count()?;
        let mut enemies = Vec::with_capacity(enemy_count);
        for _ in 0..enemy_count {
            enemies.push(read.actor()?);
        }
        let food_count = read.count()?;
        let mut food = Vec::with_capacity(food_count);
        for _ in 0..food_count {
            let id = read.u32()?;
            let position = read.vec()?;
            let kind = match read.byte()? {
                0 => FoodKind::Plant,
                1 => FoodKind::Meat,
                _ => return Err(CellError::InvalidSnapshot),
            };
            food.push(CellFood {
                id,
                position,
                kind,
                mass: read.f32()?,
            });
        }
        let obstacle_count = read.count()?;
        let mut obstacles = Vec::with_capacity(obstacle_count);
        for _ in 0..obstacle_count {
            obstacles.push(CellObstacle {
                id: read.u32()?,
                position: read.vec()?,
                radius: read.f32()?,
            });
        }
        let loadout = if version == 2 {
            match read.byte()? {
                0 => None,
                1 => {
                    let diet = match read.byte()? {
                        0 => Diet::Herbivore,
                        1 => Diet::Carnivore,
                        2 => Diet::Omnivore,
                        _ => return Err(CellError::InvalidLoadout),
                    };
                    let health = read.f32()?;
                    let speed = read.f32()?;
                    let bite_damage = read.f32()?;
                    let boost_capacity = read.f32()?;
                    let defense = read.f32()?;
                    let has_eyes = match read.byte()? {
                        0 => false,
                        1 => true,
                        _ => return Err(CellError::InvalidLoadout),
                    };
                    Some(CellLoadout {
                        diet,
                        health,
                        speed,
                        bite_damage,
                        boost_capacity,
                        defense,
                        has_eyes,
                    })
                }
                _ => return Err(CellError::InvalidLoadout),
            }
        } else {
            None
        };
        let creation = if version == 2 {
            match read.byte()? {
                0 => None,
                1 => {
                    let count = read.u32()? as usize;
                    if count > MAX_SAVE_BYTES {
                        return Err(CellError::InvalidCreation);
                    }
                    Some(
                        EditorSnapshot::from_bytes(read.take(count)?)
                            .map_err(|_| CellError::InvalidCreation)?
                            .creation,
                    )
                }
                _ => return Err(CellError::InvalidCreation),
            }
        } else {
            None
        };
        if read.at != bytes.len() {
            return Err(CellError::TrailingBytes);
        }
        let world = CellWorld {
            config,
            player,
            enemies,
            food,
            obstacles,
            state,
            tick,
            eaten,
            kills,
            loadout,
            creation,
            seed,
            rng,
            next_id,
            accumulator,
        };
        world.validate()?;
        Ok(Self { version, world })
    }
}

fn validate_loadout(loadout: &CellLoadout) -> Result<(), CellError> {
    if ![
        (loadout.health, 1.0, 1000.0),
        (loadout.speed, 0.1, 15.0),
        (loadout.bite_damage, 0.0, 200.0),
        (loadout.boost_capacity, 0.1, 5.0),
        (loadout.defense, 0.0, 0.75),
    ]
    .iter()
    .all(|(value, min, max)| value.is_finite() && (*min..=*max).contains(value))
    {
        return Err(CellError::InvalidLoadout);
    }
    Ok(())
}

fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn put_f32(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn put_vec(out: &mut Vec<u8>, value: CellVec2) {
    put_f32(out, value.x);
    put_f32(out, value.y);
}
fn put_actor(out: &mut Vec<u8>, a: &CellActor) {
    put_u32(out, a.id);
    put_vec(out, a.position);
    put_vec(out, a.velocity);
    for value in [
        a.heading,
        a.mass,
        a.health,
        a.max_health,
        a.energy,
        a.bite_cooldown,
    ] {
        put_f32(out, value);
    }
    out.push(match a.diet {
        Diet::Herbivore => 0,
        Diet::Carnivore => 1,
        Diet::Omnivore => 2,
    });
    out.push(match a.behavior {
        CellBehavior::Grazer => 0,
        CellBehavior::Scavenger => 1,
        CellBehavior::Predator => 2,
    });
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], CellError> {
        let end = self
            .at
            .checked_add(count)
            .ok_or(CellError::TruncatedSnapshot)?;
        let value = self
            .bytes
            .get(self.at..end)
            .ok_or(CellError::TruncatedSnapshot)?;
        self.at = end;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8, CellError> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, CellError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, CellError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, CellError> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn vec(&mut self) -> Result<CellVec2, CellError> {
        Ok(CellVec2::new(self.f32()?, self.f32()?))
    }
    fn count(&mut self) -> Result<usize, CellError> {
        let count = self.u32()? as usize;
        if count > MAX_ENTITIES {
            Err(CellError::TooManyEntities)
        } else {
            Ok(count)
        }
    }
    fn actor(&mut self) -> Result<CellActor, CellError> {
        let id = self.u32()?;
        let position = self.vec()?;
        let velocity = self.vec()?;
        let heading = self.f32()?;
        let mass = self.f32()?;
        let health = self.f32()?;
        let max_health = self.f32()?;
        let energy = self.f32()?;
        let bite_cooldown = self.f32()?;
        let diet = match self.byte()? {
            0 => Diet::Herbivore,
            1 => Diet::Carnivore,
            2 => Diet::Omnivore,
            _ => return Err(CellError::InvalidSnapshot),
        };
        let behavior = match self.byte()? {
            0 => CellBehavior::Grazer,
            1 => CellBehavior::Scavenger,
            2 => CellBehavior::Predator,
            _ => return Err(CellError::InvalidSnapshot),
        };
        Ok(CellActor {
            id,
            position,
            velocity,
            heading,
            mass,
            health,
            max_health,
            energy,
            bite_cooldown,
            diet,
            behavior,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty(diet: Diet) -> CellWorld {
        CellWorld::with_config(
            42,
            diet,
            CellConfig {
                food_count: 0,
                enemy_count: 0,
                obstacle_count: 0,
                ..Default::default()
            },
        )
        .unwrap()
    }

    #[test]
    fn equal_seed_and_inputs_replay_identically() {
        let mut a = CellWorld::new(1234, Diet::Omnivore);
        let mut b = a.clone();
        for tick in 0..600 {
            let input = CellInput {
                thrust: 0.8,
                turn: if tick < 200 { 0.2 } else { -0.1 },
                boost: tick < 90,
                bite: true,
            };
            assert_eq!(a.step(input), b.step(input));
        }
        assert_eq!(a, b);
    }

    #[test]
    fn fixed_ticks_do_not_depend_on_render_frame_partition() {
        let mut a = empty(Diet::Herbivore);
        let mut b = a.clone();
        let input = CellInput {
            thrust: 1.0,
            turn: 0.2,
            ..Default::default()
        };
        for _ in 0..120 {
            a.advance(1.0 / 120.0, input);
        }
        for _ in 0..20 {
            b.advance(1.0 / 20.0, input);
        }
        assert_eq!(a.tick, 60);
        assert_eq!(a.player, b.player);
        assert_eq!(a.tick, b.tick);
        assert!((a.accumulator - b.accumulator).abs() < 1e-12);
    }

    #[test]
    fn matching_food_grows_and_wrong_diet_food_remains() {
        let mut world = empty(Diet::Herbivore);
        let position = world.player.mouth_position();
        world.add_food(position, FoodKind::Plant, 1.0).unwrap();
        let meat = world.add_food(position, FoodKind::Meat, 2.0).unwrap();
        let events = world.step(CellInput::default());
        assert_eq!(world.player.mass, START_MASS + 1.0);
        assert_eq!(world.eaten, 1);
        assert_eq!(world.food.len(), 1);
        assert_eq!(world.food[0].id, meat);
        assert!(events.iter().any(|e| matches!(
            e,
            CellEvent::Ate {
                kind: FoodKind::Plant,
                ..
            }
        )));
    }

    #[test]
    fn meat_is_eaten_by_carnivores_and_omnivores() {
        for diet in [Diet::Carnivore, Diet::Omnivore] {
            let mut world = empty(diet);
            world
                .add_food(world.player.mouth_position(), FoodKind::Meat, 2.0)
                .unwrap();
            world.step(CellInput::default());
            assert_eq!(world.player.mass, START_MASS + 2.0);
        }
    }

    #[test]
    fn evolution_and_death_are_terminal_and_emit_once() {
        let mut world = empty(Diet::Herbivore);
        world.config.evolution_mass = 5.0;
        world
            .add_food(world.player.mouth_position(), FoodKind::Plant, 1.0)
            .unwrap();
        assert!(world
            .step(CellInput::default())
            .contains(&CellEvent::Evolved));
        assert_eq!(world.state, CellState::Evolved);
        let tick = world.tick;
        assert!(world.step(CellInput::default()).is_empty());
        assert_eq!(world.tick, tick);
        world.restart();
        world.player.health = 0.0;
        assert!(world
            .step(CellInput::default())
            .contains(&CellEvent::PlayerDied));
        assert_eq!(world.state, CellState::Dead);
        assert!(world.step(CellInput::default()).is_empty());
    }

    #[test]
    fn bite_damage_has_cooldown_and_dead_enemies_drop_meat() {
        let mut world = empty(Diet::Carnivore);
        let id = world
            .add_enemy(
                CellVec2::new(1.9, 0.0),
                3.0,
                Diet::Herbivore,
                CellBehavior::Grazer,
            )
            .unwrap();
        world.enemies[0].health = 15.0;
        let first = world.step(CellInput {
            bite: true,
            ..Default::default()
        });
        assert!(first
            .iter()
            .any(|e| matches!(e, CellEvent::Damaged { actor, .. } if *actor == id)));
        let health = world.enemies[0].health;
        world.step(CellInput {
            bite: true,
            ..Default::default()
        });
        assert_eq!(world.enemies[0].health, health);
        world.player.bite_cooldown = 0.0;
        world.enemies[0].position = CellVec2::new(1.9, 0.0);
        let killed = world.step(CellInput {
            bite: true,
            ..Default::default()
        });
        assert!(killed.contains(&CellEvent::Killed { actor: id }));
        assert!(world.enemies.is_empty());
        assert_eq!(world.kills, 1);
        assert!(world.food.iter().any(|food| food.kind == FoodKind::Meat));
    }

    #[test]
    fn predator_attack_can_kill_player() {
        let mut world = empty(Diet::Herbivore);
        world.player.health = 1.0;
        world
            .add_enemy(
                CellVec2::new(2.0, 0.0),
                8.0,
                Diet::Carnivore,
                CellBehavior::Predator,
            )
            .unwrap();
        world.enemies[0].heading = std::f32::consts::PI;
        let events = world.step(CellInput::default());
        assert_eq!(world.state, CellState::Dead);
        assert!(events.contains(&CellEvent::PlayerDied));
    }

    #[test]
    fn solid_obstacles_and_arena_bounds_resolve_penetration() {
        let mut world = empty(Diet::Herbivore);
        let id = world.allocate_id();
        world.obstacles.push(CellObstacle {
            id,
            position: CellVec2::ZERO,
            radius: 2.0,
        });
        world.step(CellInput::default());
        assert!(world.player.position.length() >= world.player.radius() + 2.0 - 1e-5);
        world.player.position = CellVec2::new(1_000.0, -1_000.0);
        world.step(CellInput::default());
        assert!(world.player.position.x <= world.config.half_extents.x - world.player.radius());
        assert!(world.player.position.y >= -world.config.half_extents.y + world.player.radius());
    }

    #[test]
    fn boost_uses_energy_and_invalid_time_is_rejected() {
        let mut world = empty(Diet::Herbivore);
        for _ in 0..60 {
            world.step(CellInput {
                thrust: 1.0,
                boost: true,
                ..Default::default()
            });
        }
        assert!(world.player.energy < 1.0);
        let before = world.clone();
        assert!(world.advance(f64::NAN, CellInput::default()).time_rejected);
        assert!(world.advance(-1.0, CellInput::default()).time_rejected);
        assert_eq!(world, before);
        for _ in 0..120 {
            world.step(CellInput::default());
        }
        assert!(world.player.energy > before.player.energy);
    }

    #[test]
    fn boost_increases_distance_and_exhaustion_requires_releasing_the_control() {
        let mut normal = empty(Diet::Herbivore);
        let mut boosted = normal.clone();
        for _ in 0..120 {
            normal.step(CellInput {
                thrust: 1.0,
                ..Default::default()
            });
            boosted.step(CellInput {
                thrust: 1.0,
                boost: true,
                ..Default::default()
            });
        }
        assert!(boosted.player.position.x > normal.player.position.x * 1.6);
        assert!(boosted.player.velocity.length() > normal.player.velocity.length() * 1.6);
        assert!(boosted.player.energy < normal.player.energy - 0.5);

        let mut exhausted = empty(Diet::Herbivore);
        exhausted.player.energy = 0.0;
        let mut unboosted = exhausted.clone();
        for _ in 0..60 {
            exhausted.step(CellInput {
                thrust: 1.0,
                boost: true,
                ..Default::default()
            });
            unboosted.step(CellInput {
                thrust: 1.0,
                ..Default::default()
            });
        }
        assert_eq!(exhausted.player.position, unboosted.player.position);
        assert_eq!(exhausted.player.velocity, unboosted.player.velocity);
        assert_eq!(exhausted.player.energy, 0.0);
        assert!(unboosted.player.energy > 0.0);
        for _ in 0..60 {
            exhausted.step(CellInput::default());
        }
        let recovered = exhausted.player.energy;
        exhausted.step(CellInput {
            thrust: 1.0,
            boost: true,
            ..Default::default()
        });
        assert!(exhausted.player.energy < recovered);
    }

    #[test]
    fn catch_up_cap_preserves_time_as_debt() {
        let mut world = empty(Diet::Herbivore);
        let first = world.advance(5.0, CellInput::default());
        assert_eq!(first.steps, MAX_STEPS_PER_ADVANCE);
        assert!((first.backlog_seconds - 1.0).abs() < 1e-10);
        let second = world.advance(0.0, CellInput::default());
        assert_eq!(second.steps, 60);
        assert_eq!(world.tick, 300);
    }

    #[test]
    fn snapshot_bytes_preserve_exact_state_and_future_rng() {
        let mut world = CellWorld::new(777, Diet::Omnivore);
        world.advance(
            0.123,
            CellInput {
                thrust: 1.0,
                ..Default::default()
            },
        );
        let bytes = world.snapshot().to_bytes().unwrap();
        let mut loaded =
            CellWorld::from_snapshot(CellSnapshot::from_bytes(&bytes).unwrap()).unwrap();
        assert_eq!(world, loaded);
        // Force replenishment after loading so future RNG state is exercised.
        world.food[0].position = world.player.mouth_position();
        loaded.food[0].position = loaded.player.mouth_position();
        for _ in 0..100 {
            let input = CellInput {
                thrust: 0.5,
                turn: -0.2,
                bite: true,
                ..Default::default()
            };
            assert_eq!(world.step(input), loaded.step(input));
        }
        assert!(world.eaten > 0);
        assert_eq!(world, loaded);
    }

    #[test]
    fn snapshots_reject_truncation_trailing_bytes_and_bad_values() {
        let bytes = empty(Diet::Herbivore).snapshot().to_bytes().unwrap();
        for length in 0..bytes.len() {
            assert!(
                CellSnapshot::from_bytes(&bytes[..length]).is_err(),
                "accepted prefix {length}"
            );
        }
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert_eq!(
            CellSnapshot::from_bytes(&trailing),
            Err(CellError::TrailingBytes)
        );
        let mut version = bytes.clone();
        version[8..12].copy_from_slice(&99u32.to_le_bytes());
        assert_eq!(
            CellSnapshot::from_bytes(&version),
            Err(CellError::UnsupportedVersion(99))
        );
        let mut nonfinite = bytes.clone();
        nonfinite[12..16].copy_from_slice(&f32::NAN.to_le_bytes());
        assert_eq!(
            CellSnapshot::from_bytes(&nonfinite),
            Err(CellError::InvalidConfig)
        );
        let mut snapshot = empty(Diet::Herbivore).snapshot();
        snapshot.world.player.position.x = f32::INFINITY;
        assert_eq!(snapshot.to_bytes(), Err(CellError::InvalidEntity));
    }

    #[test]
    fn snapshots_reject_duplicate_ids_and_excessive_counts() {
        let mut world = empty(Diet::Herbivore);
        world
            .add_food(CellVec2::new(5.0, 0.0), FoodKind::Plant, 1.0)
            .unwrap();
        world.food[0].id = world.player.id;
        assert_eq!(world.snapshot().to_bytes(), Err(CellError::DuplicateId));
        let mut bytes = empty(Diet::Herbivore).snapshot().to_bytes().unwrap();
        // Config's food-count word follows its two extents and evolution threshold.
        bytes[24..28].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            CellSnapshot::from_bytes(&bytes),
            Err(CellError::TooManyEntities)
        );
    }

    #[test]
    fn restart_rebuilds_original_seed_and_diet() {
        let mut world = CellWorld::new(55, Diet::Carnivore);
        let original = world.clone();
        world.advance(
            2.0,
            CellInput {
                thrust: 1.0,
                turn: 0.2,
                ..Default::default()
            },
        );
        world.restart();
        assert_eq!(world, original);
    }

    #[test]
    fn loaded_id_cursor_wraps_without_reusing_live_ids() {
        let mut world = empty(Diet::Herbivore);
        world.next_id = u32::MAX;
        let max = world
            .add_food(CellVec2::new(5.0, 0.0), FoodKind::Plant, 1.0)
            .unwrap();
        let next = world
            .add_food(CellVec2::new(6.0, 0.0), FoodKind::Plant, 1.0)
            .unwrap();
        assert_eq!(max, u32::MAX);
        assert_eq!(next, 2);
        assert_ne!(next, world.player.id);
        assert!(world.snapshot().to_bytes().is_ok());
    }

    fn authored_loadout() -> CellLoadout {
        CellLoadout {
            diet: Diet::Carnivore,
            health: 120.,
            speed: 8.,
            bite_damage: 40.,
            boost_capacity: 2.,
            defense: 0.5,
            has_eyes: true,
        }
    }

    #[test]
    fn authored_stats_change_health_diet_movement_and_boost_duration() {
        let mut normal = empty(Diet::Herbivore);
        let mut authored = normal.clone();
        authored.player.health = 30.;
        authored.apply_loadout(authored_loadout()).unwrap();
        assert_eq!(authored.player.max_health, 120.);
        assert_eq!(authored.player.health, 60.);
        assert_eq!(authored.player.diet, Diet::Carnivore);
        for _ in 0..120 {
            let input = CellInput {
                thrust: 1.,
                boost: true,
                ..Default::default()
            };
            normal.step(input);
            authored.step(input);
        }
        assert!(authored.player.position.x > normal.player.position.x * 1.3);
        assert!(authored.player.energy > normal.player.energy + 0.2);
    }

    #[test]
    fn authored_bite_damage_and_defense_change_combat() {
        let mut normal = empty(Diet::Carnivore);
        normal
            .add_enemy(
                CellVec2::new(1.9, 0.),
                3.,
                Diet::Herbivore,
                CellBehavior::Grazer,
            )
            .unwrap();
        let mut authored = normal.clone();
        authored.apply_loadout(authored_loadout()).unwrap();
        normal.step(CellInput {
            bite: true,
            ..Default::default()
        });
        authored.step(CellInput {
            bite: true,
            ..Default::default()
        });
        assert!(authored.enemies[0].health < normal.enemies[0].health - 25.);
        let mut normal = empty(Diet::Herbivore);
        normal
            .add_enemy(
                CellVec2::new(2., 0.),
                8.,
                Diet::Carnivore,
                CellBehavior::Predator,
            )
            .unwrap();
        normal.enemies[0].heading = std::f32::consts::PI;
        let mut authored = normal.clone();
        authored.apply_loadout(authored_loadout()).unwrap();
        let before_normal = normal.player.health;
        let before_authored = authored.player.health;
        normal.step(CellInput::default());
        authored.step(CellInput::default());
        assert!(
            ((before_authored - authored.player.health) * 2.
                - (before_normal - normal.player.health))
                .abs()
                < 1e-5
        );
    }

    #[test]
    fn authored_eyes_control_perception_and_unauthored_worlds_keep_full_arena() {
        let mut world = empty(Diet::Herbivore);
        assert!(world.can_perceive(CellVec2::new(40., 28.)));
        assert!(world.perception_radius() > 80.);
        let mut loadout = authored_loadout();
        loadout.has_eyes = false;
        world.apply_loadout(loadout).unwrap();
        assert_eq!(world.perception_radius(), 8.);
        assert!(!world.can_perceive(CellVec2::new(9., 0.)));
        world.apply_loadout(authored_loadout()).unwrap();
        assert_eq!(world.perception_radius(), 36.);
        assert!(world.can_perceive(CellVec2::new(35., 0.)));
        assert!(!world.can_perceive(CellVec2::new(37., 0.)));
    }

    #[test]
    fn authored_save_restart_and_v1_upgrade_preserve_deterministic_rules() {
        let mut world = empty(Diet::Herbivore);
        world.apply_loadout(authored_loadout()).unwrap();
        world.advance(
            0.127,
            CellInput {
                thrust: 1.,
                ..Default::default()
            },
        );
        let bytes = world.snapshot().to_bytes().unwrap();
        let mut loaded =
            CellWorld::from_snapshot(CellSnapshot::from_bytes(&bytes).unwrap()).unwrap();
        assert_eq!(loaded, world);
        for _ in 0..120 {
            let input = CellInput {
                thrust: 1.,
                turn: 0.1,
                boost: true,
                ..Default::default()
            };
            assert_eq!(loaded.advance(0.021, input), world.advance(0.021, input));
        }
        assert_eq!(loaded, world);
        loaded.restart();
        assert_eq!(loaded.loadout, Some(authored_loadout()));
        assert_eq!(loaded.player.health, 120.);
        assert_eq!(loaded.player.diet, Diet::Carnivore);
        let original = empty(Diet::Herbivore);
        let old = CellSnapshot {
            version: 1,
            world: original.clone(),
        }
        .to_bytes()
        .unwrap();
        let mut legacy = CellSnapshot::from_bytes(&old).unwrap().world;
        assert_eq!(legacy, original);
        assert!(legacy.loadout.is_none());
        let mut current = original;
        for _ in 0..60 {
            let input = CellInput {
                thrust: 1.,
                ..Default::default()
            };
            assert_eq!(legacy.step(input), current.step(input));
        }
        assert_eq!(legacy, current);
    }

    #[test]
    fn invalid_authored_stats_and_corrupt_v2_payload_reject_atomically() {
        let mut world = empty(Diet::Herbivore);
        let before = world.clone();
        for value in [f32::NAN, 0., 1001.] {
            let mut bad = authored_loadout();
            bad.health = value;
            assert_eq!(world.apply_loadout(bad), Err(CellError::InvalidLoadout));
            assert_eq!(world, before);
        }
        world.apply_loadout(authored_loadout()).unwrap();
        let bytes = world.snapshot().to_bytes().unwrap();
        let mut bad = bytes.clone();
        let length = bad.len();
        bad[length - 23] = 99;
        assert!(CellSnapshot::from_bytes(&bad).is_err());
        let mut bad = bytes.clone();
        let length = bad.len();
        bad[length - 22..length - 18].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(CellSnapshot::from_bytes(&bad).is_err());
        assert!(CellSnapshot::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        let mut snapshot = world.snapshot();
        snapshot.version = 1;
        assert_eq!(snapshot.to_bytes(), Err(CellError::InvalidSnapshot));
    }

    #[test]
    fn authored_creation_palette_and_parts_survive_save_restart_and_detect_stat_mismatch() {
        use crate::editor::{AuthoredPart, PartArchetype};
        let mut creation = Creation::new(CreationKind::Cell, "Authored swimmer", 100).unwrap();
        for (id, archetype) in [
            (2, PartArchetype::CarnivoreMouth),
            (3, PartArchetype::Flagellum),
            (4, PartArchetype::Eye),
        ] {
            creation.parts.push(AuthoredPart::new(
                id,
                format!("Part {id}"),
                archetype,
                Some(1),
            ));
        }
        creation.palette[0] = [0.7, 0.2, 0.4];
        let mut world = empty(Diet::Herbivore);
        world.apply_creation(creation.clone()).unwrap();
        let bytes = world.snapshot().to_bytes().unwrap();
        let mut loaded = CellSnapshot::from_bytes(&bytes).unwrap().world;
        assert_eq!(loaded.creation, Some(creation.clone()));
        assert_eq!(loaded.loadout, world.loadout);
        loaded.restart();
        assert_eq!(loaded.creation, Some(creation));
        assert_eq!(loaded.player.diet, Diet::Carnivore);
        loaded.snapshot().to_bytes().unwrap();
        let mut bad = world.snapshot();
        bad.world.loadout.as_mut().unwrap().speed += 1.;
        assert_eq!(bad.to_bytes(), Err(CellError::InvalidCreation));
        let mut corrupt = bytes;
        let start = corrupt
            .windows(8)
            .position(|bytes| bytes == b"RSEDIT01")
            .unwrap();
        corrupt[start - 4..start].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            CellSnapshot::from_bytes(&corrupt),
            Err(CellError::InvalidCreation)
        );
        world.apply_loadout(authored_loadout()).unwrap();
        assert!(world.creation.is_none());
        let before = world.clone();
        assert_eq!(
            world.apply_creation(Creation::new(CreationKind::Creature, "Wrong kind", 100).unwrap()),
            Err(CellError::InvalidCreation)
        );
        assert_eq!(world, before);
    }
}
