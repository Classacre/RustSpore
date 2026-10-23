//! Original, deterministic Creature-stage gameplay baseline.
//!
//! This module provides a playable ruleset, not measured retail-game fidelity.
//! It owns no renderer, assets, animation, editor UI or stage-transition storage.
//! All simulation and save decisions use fixed ticks and owned Rust data.

pub use crate::cell::{CellVec2 as CreatureVec2, Diet, FoodKind};
use crate::editor::{
    Creation, CreationKind, CreatureLoadout, EditorSnapshot, GameplayLoadout, MAX_SAVE_BYTES,
};
use std::collections::HashSet;
use std::fmt;

pub const CREATURE_FIXED_DT: f64 = 1.0 / 60.0;
const DT: f32 = CREATURE_FIXED_DT as f32;
const MAX_STEPS: u32 = 240;
const MAX_ENTITIES: usize = 8192;
const MAGIC: &[u8; 8] = b"RSCREA01";
const ALLIANCE: i16 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocialAction {
    Sing,
    Dance,
    Charm,
}
impl SocialAction {
    pub fn next(self) -> Self {
        match self {
            Self::Sing => Self::Dance,
            Self::Dance => Self::Charm,
            Self::Charm => Self::Sing,
        }
    }
}

/// Collectibles change the player's movement, attack, social and defence stats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartUnlock {
    Stride,
    Claws,
    Voice,
    Armor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureState {
    Playing,
    TribeReady,
    Dead,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CreatureInput {
    /// Forward/reverse acceleration and counterclockwise turning, clamped -1..1.
    pub thrust: f32,
    pub turn: f32,
    pub sprint: bool,
    pub attack: bool,
    /// Match the target's visible cue. Attack takes precedence when both are set.
    pub social: Option<SocialAction>,
    /// Eat, collect nearby bones and recruit a nearby allied creature.
    pub interact: bool,
    /// Hold still at the home nest to heal; hunger still passes during rest.
    pub rest: bool,
    /// Explicit NPC ID, or the nearest eligible NPC within the action's range.
    pub target: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureActor {
    pub id: u32,
    /// Zero identifies the player's species. NPC species have nonzero IDs.
    pub species_id: u32,
    pub position: CreatureVec2,
    pub velocity: CreatureVec2,
    /// Radians; zero faces positive x.
    pub heading: f32,
    pub health: f32,
    pub max_health: f32,
    /// Normalized 0..1 reserves; hunger 1 means fully fed.
    pub stamina: f32,
    pub hunger: f32,
    pub action_cooldown: f32,
    pub social_cue: SocialAction,
}
impl CreatureActor {
    fn new(id: u32, species_id: u32, position: CreatureVec2) -> Self {
        let max_health = if species_id == 0 { 100.0 } else { 60.0 };
        Self {
            id,
            species_id,
            position,
            velocity: CreatureVec2::ZERO,
            heading: 0.0,
            health: max_health,
            max_health,
            stamina: 1.0,
            hunger: 1.0,
            action_cooldown: 0.0,
            social_cue: SocialAction::Sing,
        }
    }
    pub fn radius(&self) -> f32 {
        if self.species_id == 0 {
            0.9
        } else {
            0.85
        }
    }
    pub fn forward(&self) -> CreatureVec2 {
        CreatureVec2::new(self.heading.cos(), self.heading.sin())
    }
    pub fn alive(&self) -> bool {
        self.health > 0.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureSpecies {
    pub id: u32,
    pub name: String,
    pub diet: Diet,
    /// -100..100. At 60 the species allies and permits pack recruitment.
    pub relationship: i16,
    pub kills: u32,
    pub extinct: bool,
    /// A species grants resolution DNA once, even if an alliance is later broken.
    pub reward_claimed: bool,
}
impl CreatureSpecies {
    pub fn allied(&self) -> bool {
        self.relationship >= ALLIANCE && !self.extinct
    }
    pub fn resolved(&self) -> bool {
        self.extinct || self.allied()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureNest {
    pub id: u32,
    pub species_id: u32,
    pub position: CreatureVec2,
    pub radius: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureFood {
    pub id: u32,
    pub position: CreatureVec2,
    pub kind: FoodKind,
    pub nutrition: f32,
}
impl CreatureFood {
    pub fn radius(&self) -> f32 {
        0.45
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreaturePickup {
    pub id: u32,
    pub position: CreatureVec2,
    pub part: PartUnlock,
    pub dna: u32,
}
impl CreaturePickup {
    pub fn radius(&self) -> f32 {
        0.55
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureConfig {
    pub half_extents: CreatureVec2,
    pub species_count: usize,
    pub members_per_species: usize,
    pub food_count: usize,
    pub pickup_count: usize,
    pub pack_capacity: usize,
    pub tribe_dna: u32,
    pub tribe_parts: usize,
    pub tribe_resolved_species: usize,
}
impl Default for CreatureConfig {
    fn default() -> Self {
        Self {
            half_extents: CreatureVec2::new(64.0, 48.0),
            species_count: 4,
            members_per_species: 3,
            food_count: 48,
            pickup_count: 8,
            pack_capacity: 3,
            tribe_dna: 100,
            tribe_parts: 2,
            tribe_resolved_species: 2,
        }
    }
}
impl CreatureConfig {
    fn validate(&self) -> Result<(), CreatureError> {
        if !finite(self.half_extents)
            || self.half_extents.x < 24.0
            || self.half_extents.y < 24.0
            || self.half_extents.x > 1000.0
            || self.half_extents.y > 1000.0
            || !(2..=12).contains(&self.species_count)
            || !(1..=16).contains(&self.members_per_species)
            || self.food_count > 2000
            || self.pickup_count > 128
            || self.pack_capacity > 8
            || self.tribe_dna == 0
            || self.tribe_dna > 1_000_000
            || !(1..=4).contains(&self.tribe_parts)
            || self.tribe_resolved_species == 0
            || self.tribe_resolved_species > self.species_count
        {
            return Err(CreatureError::InvalidConfig);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreatureEvent {
    Ate {
        actor: u32,
        food: u32,
    },
    PartFound {
        part: PartUnlock,
        newly_unlocked: bool,
        dna: u32,
    },
    Damaged {
        actor: u32,
        amount: f32,
    },
    Killed {
        actor: u32,
        species: u32,
    },
    Social {
        actor: u32,
        matched: bool,
        relationship: i16,
    },
    Allied {
        species: u32,
    },
    Extinct {
        species: u32,
    },
    Recruited {
        actor: u32,
    },
    PlayerDied,
    TribeEligible,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureStepReport {
    pub steps: u32,
    pub backlog_seconds: f64,
    pub interpolation: f32,
    pub state: CreatureState,
    pub events: Vec<CreatureEvent>,
    pub time_rejected: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureWorld {
    pub config: CreatureConfig,
    pub player: CreatureActor,
    pub player_diet: Diet,
    pub creatures: Vec<CreatureActor>,
    pub species: Vec<CreatureSpecies>,
    pub nests: Vec<CreatureNest>,
    pub food: Vec<CreatureFood>,
    pub pickups: Vec<CreaturePickup>,
    /// Existing allied NPC IDs; followers retain normal health and can die.
    pub pack: Vec<u32>,
    pub state: CreatureState,
    pub tick: u64,
    pub dna: u32,
    pub unlocked_parts: Vec<PartUnlock>,
    /// Applied original editor stats; None preserves the unmodified baseline.
    pub loadout: Option<CreatureLoadout>,
    pub creation: Option<Creation>,
    seed: u64,
    rng: u64,
    next_id: u32,
    accumulator: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatureSnapshot {
    pub version: u32,
    pub world: CreatureWorld,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureError {
    InvalidConfig,
    InvalidEntity,
    InvalidReference,
    DuplicateId,
    TooManyEntities,
    InvalidSnapshot,
    UnsupportedVersion(u32),
    TruncatedSnapshot,
    TrailingBytes,
    InvalidLoadout,
    UnsupportedCapability,
    InvalidCreation,
}
impl fmt::Display for CreatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for CreatureError {}

impl CreatureWorld {
    pub fn new(seed: u64, diet: Diet) -> Self {
        Self::with_config(seed, diet, CreatureConfig::default()).expect("default Creature config")
    }
    pub fn with_config(
        seed: u64,
        diet: Diet,
        config: CreatureConfig,
    ) -> Result<Self, CreatureError> {
        config.validate()?;
        let mut world = Self {
            config,
            player: CreatureActor::new(1, 0, CreatureVec2::ZERO),
            player_diet: diet,
            creatures: Vec::new(),
            species: Vec::new(),
            nests: Vec::new(),
            food: Vec::new(),
            pickups: Vec::new(),
            pack: Vec::new(),
            state: CreatureState::Playing,
            tick: 0,
            dna: 0,
            unlocked_parts: Vec::new(),
            loadout: None,
            creation: None,
            seed,
            rng: if seed == 0 {
                0x5899_65cc_7537_4cc3
            } else {
                seed
            },
            next_id: 2,
            accumulator: 0.0,
        };
        let home_id = world.id();
        world.nests.push(CreatureNest {
            id: home_id,
            species_id: 0,
            position: CreatureVec2::ZERO,
            radius: 3.5,
        });
        const NAMES: [&str; 12] = [
            "Mossbacks",
            "Reedwalkers",
            "Emberjaws",
            "Tidekin",
            "Stonehoppers",
            "Fernwings",
            "Cinderlings",
            "Dunestalkers",
            "Bloomhorns",
            "Driftclaws",
            "Poolrunners",
            "Raincallers",
        ];
        for (index, name) in NAMES.iter().enumerate().take(world.config.species_count) {
            let species_id = index as u32 + 1;
            let angle = index as f32 * std::f32::consts::TAU / world.config.species_count as f32
                + world.random() * 0.35;
            let position = CreatureVec2::new(
                angle.cos() * world.config.half_extents.x * 0.58,
                angle.sin() * world.config.half_extents.y * 0.58,
            );
            let species_diet = match index % 3 {
                0 => Diet::Herbivore,
                1 => Diet::Omnivore,
                _ => Diet::Carnivore,
            };
            world.species.push(CreatureSpecies {
                id: species_id,
                name: (*name).to_owned(),
                diet: species_diet,
                relationship: if index == 2 { -35 } else { 0 },
                kills: 0,
                extinct: false,
                reward_claimed: false,
            });
            let nest_id = world.id();
            world.nests.push(CreatureNest {
                id: nest_id,
                species_id,
                position,
                radius: 3.0,
            });
            for member in 0..world.config.members_per_species {
                let offset =
                    member as f32 * std::f32::consts::TAU / world.config.members_per_species as f32;
                let id = world.id();
                let mut actor = CreatureActor::new(
                    id,
                    species_id,
                    position.add(CreatureVec2::new(offset.cos(), offset.sin()).scale(3.5)),
                );
                actor.heading = angle;
                actor.social_cue = match member % 3 {
                    0 => SocialAction::Sing,
                    1 => SocialAction::Dance,
                    _ => SocialAction::Charm,
                };
                world.creatures.push(actor);
            }
        }
        for index in 0..world.config.food_count {
            let position = if index < 6 {
                CreatureVec2::new(3.0 + index as f32 * 1.25, -3.0)
            } else {
                world.random_position(2.0)
            };
            let kind = if index < 6 {
                if diet == Diet::Carnivore {
                    FoodKind::Meat
                } else {
                    FoodKind::Plant
                }
            } else if index % 2 == 0 {
                FoodKind::Plant
            } else {
                FoodKind::Meat
            };
            let id = world.id();
            world.food.push(CreatureFood {
                id,
                position,
                kind,
                nutrition: 0.4,
            });
        }
        for index in 0..world.config.pickup_count {
            // First two discoverable bones are close enough to teach exploration.
            let position = match index {
                0 => CreatureVec2::new(5.0, 2.0),
                1 => CreatureVec2::new(11.0, 5.0),
                _ => world.random_position(3.0),
            };
            let part = match index % 4 {
                0 => PartUnlock::Stride,
                1 => PartUnlock::Claws,
                2 => PartUnlock::Voice,
                _ => PartUnlock::Armor,
            };
            let id = world.id();
            world.pickups.push(CreaturePickup {
                id,
                position,
                part,
                dna: 15,
            });
        }
        world.validate()?;
        Ok(world)
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn restart(&mut self) {
        let loadout = self.loadout.clone();
        let creation = self.creation.clone();
        *self = Self::with_config(self.seed, self.player_diet, self.config.clone())
            .expect("existing valid Creature config");
        if let Some(loadout) = loadout {
            self.apply_loadout(loadout)
                .expect("validated authored Creature loadout");
        }
        self.creation = creation;
    }
    /// Apply original stats atomically, preserving current health percentage.
    /// Gliding remains unsupported and is explicitly rejected, never cosmetic.
    pub fn apply_loadout(&mut self, loadout: CreatureLoadout) -> Result<(), CreatureError> {
        validate_loadout(&loadout)?;
        if !self.player.max_health.is_finite()
            || self.player.max_health <= 0.0
            || !self.player.health.is_finite()
            || !(0.0..=self.player.max_health).contains(&self.player.health)
        {
            return Err(CreatureError::InvalidEntity);
        }
        let fraction = self.player.health / self.player.max_health;
        self.player.max_health = loadout.health;
        self.player.health = loadout.health * fraction;
        self.player_diet = loadout.diet;
        self.loadout = Some(loadout);
        self.creation = None;
        Ok(())
    }
    pub fn apply_creation(&mut self, creation: Creation) -> Result<(), CreatureError> {
        if creation.kind != CreationKind::Creature {
            return Err(CreatureError::InvalidCreation);
        }
        let GameplayLoadout::Creature(loadout) = creation
            .gameplay_loadout()
            .map_err(|_| CreatureError::InvalidCreation)?
        else {
            return Err(CreatureError::InvalidCreation);
        };
        self.apply_loadout(loadout)?;
        self.creation = Some(creation);
        Ok(())
    }
    pub fn resolved_species(&self) -> usize {
        self.species
            .iter()
            .filter(|species| species.resolved())
            .count()
    }
    pub fn tribe_eligible(&self) -> bool {
        self.player.alive()
            && self.dna >= self.config.tribe_dna
            && self.unlocked_parts.len() >= self.config.tribe_parts
            && self.resolved_species() >= self.config.tribe_resolved_species
    }
    /// Lowest of the three independent progression gates, normalized 0..1.
    pub fn progress(&self) -> f32 {
        (self.dna as f32 / self.config.tribe_dna.max(1) as f32)
            .min(self.unlocked_parts.len() as f32 / self.config.tribe_parts.max(1) as f32)
            .min(self.resolved_species() as f32 / self.config.tribe_resolved_species.max(1) as f32)
            .clamp(0.0, 1.0)
    }
    pub fn snapshot(&self) -> CreatureSnapshot {
        CreatureSnapshot {
            version: 2,
            world: self.clone(),
        }
    }
    pub fn from_snapshot(snapshot: CreatureSnapshot) -> Result<Self, CreatureError> {
        if !matches!(snapshot.version, 1 | 2) {
            return Err(CreatureError::UnsupportedVersion(snapshot.version));
        }
        if snapshot.version == 1
            && (snapshot.world.loadout.is_some() || snapshot.world.creation.is_some())
        {
            return Err(CreatureError::InvalidSnapshot);
        }
        snapshot.world.validate()?;
        Ok(snapshot.world)
    }

    /// Debt beyond the work budget is retained. Partitioning elapsed time changes
    /// no fixed-tick rules when inputs are held constant and debt is eventually drained.
    pub fn advance(&mut self, seconds: f64, input: CreatureInput) -> CreatureStepReport {
        let rejected = !seconds.is_finite() || seconds < 0.0 || seconds > 86_400.0;
        if !rejected && self.state == CreatureState::Playing {
            self.accumulator += seconds;
        }
        let mut steps = 0;
        let mut events = Vec::new();
        while self.accumulator + 1e-12 >= CREATURE_FIXED_DT
            && steps < MAX_STEPS
            && self.state == CreatureState::Playing
        {
            self.accumulator = (self.accumulator - CREATURE_FIXED_DT).max(0.0);
            events.extend(self.step(input));
            steps += 1;
        }
        if self.state != CreatureState::Playing {
            self.accumulator = 0.0;
        }
        CreatureStepReport {
            steps,
            backlog_seconds: self.accumulator,
            interpolation: (self.accumulator / CREATURE_FIXED_DT).min(1.0) as f32,
            state: self.state,
            events,
            time_rejected: rejected,
        }
    }

    pub fn step(&mut self, input: CreatureInput) -> Vec<CreatureEvent> {
        let mut events = Vec::new();
        if self.state != CreatureState::Playing {
            return events;
        }
        self.tick = self.tick.saturating_add(1);
        if !self.player.alive() {
            self.state = CreatureState::Dead;
            events.push(CreatureEvent::PlayerDied);
            return events;
        }
        let resting = input.rest && self.at_home();
        let reserve = self
            .loadout
            .as_ref()
            .map(|loadout| loadout.stamina)
            .unwrap_or(1.0);
        let base_speed = self
            .loadout
            .as_ref()
            .map(|loadout| loadout.speed)
            .unwrap_or(5.5);
        metabolism_tuned(&mut self.player, resting, reserve);
        let stride = self.unlocked_parts.contains(&PartUnlock::Stride);
        move_player(
            &mut self.player,
            input,
            resting,
            stride,
            base_speed,
            reserve,
        );
        if resting && self.player.hunger > 0.1 {
            self.player.health = (self.player.health + 9.0 * DT).min(self.player.max_health);
        }
        let relationships: Vec<(u32, i16)> = self
            .species
            .iter()
            .map(|species| (species.id, species.relationship))
            .collect();
        self.pack.retain(|id| {
            self.creatures.iter().any(|actor| {
                actor.id == *id
                    && actor.alive()
                    && relationships.iter().any(|(species, relationship)| {
                        *species == actor.species_id && *relationship >= ALLIANCE
                    })
            })
        });
        for index in 0..self.creatures.len() {
            if !self.creatures[index].alive() {
                continue;
            }
            let actor = &self.creatures[index];
            let follower = self.pack.iter().position(|id| *id == actor.id);
            let relationship = relationships
                .iter()
                .find(|(id, _)| *id == actor.species_id)
                .map(|(_, value)| *value)
                .unwrap_or(0);
            let distance = actor.position.distance(self.player.position);
            let hostile = relationship <= -30 && distance < 14.0;
            let target = if let Some(slot) = follower {
                let angle = self.player.heading + std::f32::consts::PI + (slot as f32 - 1.0) * 0.8;
                self.player
                    .position
                    .add(CreatureVec2::new(angle.cos(), angle.sin()).scale(2.7))
            } else if hostile {
                self.player.position
            } else if actor.hunger < 0.55 {
                let diet = self
                    .species
                    .iter()
                    .find(|species| species.id == actor.species_id)
                    .map(|species| species.diet)
                    .unwrap_or(Diet::Omnivore);
                self.food
                    .iter()
                    .filter(|food| diet.can_eat(food.kind))
                    .min_by(|a, b| {
                        actor
                            .position
                            .distance(a.position)
                            .total_cmp(&actor.position.distance(b.position))
                    })
                    .map(|food| food.position)
                    .unwrap_or(actor.position)
            } else {
                let nest = self
                    .nests
                    .iter()
                    .find(|nest| nest.species_id == actor.species_id)
                    .map(|nest| nest.position)
                    .unwrap_or(actor.position);
                let angle = self.tick as f32 * 0.003 + actor.id as f32;
                nest.add(CreatureVec2::new(angle.cos(), angle.sin()).scale(4.5))
            };
            let actor = &mut self.creatures[index];
            metabolism(actor, false);
            steer_to(
                actor,
                target,
                if follower.is_some() {
                    6.5
                } else if hostile {
                    5.0
                } else {
                    2.0
                },
            );
            if hostile
                && actor.alive()
                && distance < 2.6
                && actor.action_cooldown <= 0.0
                && actor.stamina >= 0.1
            {
                actor.action_cooldown = 0.9;
                actor.stamina -= 0.1;
                let defense = (self
                    .loadout
                    .as_ref()
                    .map(|loadout| loadout.defense)
                    .unwrap_or(0.0)
                    + if self.unlocked_parts.contains(&PartUnlock::Armor) {
                        0.3
                    } else {
                        0.0
                    })
                .min(0.85);
                let amount = 10.0 * (1.0 - defense);
                damage(&mut self.player, amount, &mut events);
            }
        }
        // Corpses cannot act or be recruited, even if they died from starvation.
        if self.player.alive() {
            if input.attack {
                self.player_attack(input.target, &mut events);
            } else if let Some(action) = input.social {
                self.player_social(input.target, action, &mut events);
            }
            if input.interact {
                self.interact(input.target, &mut events);
            }
        }
        self.pack_attack(&mut events);
        self.npc_feed(&mut events);
        self.remove_dead(&mut events);
        for actor in &mut self.creatures {
            separate(&mut self.player, actor);
        }
        for index in 0..self.creatures.len() {
            let (before, after) = self.creatures.split_at_mut(index + 1);
            for other in after {
                separate(&mut before[index], other);
            }
        }
        bound(&mut self.player, self.config.half_extents);
        for actor in &mut self.creatures {
            bound(actor, self.config.half_extents);
        }
        if self.tick % 180 == 0 && self.food.len() < self.config.food_count {
            self.replenish_food();
        }
        if !self.player.alive() {
            self.state = CreatureState::Dead;
            events.push(CreatureEvent::PlayerDied);
        } else if self.tribe_eligible() {
            self.state = CreatureState::TribeReady;
            events.push(CreatureEvent::TribeEligible);
        }
        events
    }

    fn at_home(&self) -> bool {
        self.nests.iter().any(|nest| {
            nest.species_id == 0 && nest.position.distance(self.player.position) <= nest.radius
        })
    }
    fn target(&self, requested: Option<u32>, range: f32, social: bool) -> Option<usize> {
        self.creatures
            .iter()
            .enumerate()
            .filter(|(_, actor)| {
                actor.alive()
                    && !self.pack.contains(&actor.id)
                    && requested.map(|id| id == actor.id).unwrap_or(true)
                    && self.player.position.distance(actor.position) <= range
                    && (!social
                        || self
                            .species
                            .iter()
                            .any(|species| species.id == actor.species_id && !species.allied()))
            })
            .min_by(|(_, a), (_, b)| {
                self.player
                    .position
                    .distance(a.position)
                    .total_cmp(&self.player.position.distance(b.position))
            })
            .map(|(index, _)| index)
    }
    fn player_attack(&mut self, target: Option<u32>, events: &mut Vec<CreatureEvent>) {
        let cost = 0.12
            / self
                .loadout
                .as_ref()
                .map(|loadout| loadout.stamina)
                .unwrap_or(1.0);
        if self.player.action_cooldown > 0.0 || self.player.stamina < cost {
            return;
        }
        let Some(index) = self.target(target, 3.1, false) else {
            return;
        };
        self.player.action_cooldown = 0.65;
        self.player.stamina -= cost;
        let amount = self
            .loadout
            .as_ref()
            .map(|loadout| loadout.attack_damage)
            .unwrap_or(22.0)
            + if self.unlocked_parts.contains(&PartUnlock::Claws) {
                10.0
            } else {
                0.0
            };
        let species_id = self.creatures[index].species_id;
        damage(&mut self.creatures[index], amount, events);
        if let Some(species) = self
            .species
            .iter_mut()
            .find(|species| species.id == species_id)
        {
            species.relationship = (species.relationship - 35).max(-100);
        }
        self.pack.retain(|id| {
            self.creatures
                .iter()
                .any(|actor| actor.id == *id && actor.species_id != species_id)
        });
    }
    fn player_social(
        &mut self,
        target: Option<u32>,
        action: SocialAction,
        events: &mut Vec<CreatureEvent>,
    ) {
        let cost = 0.1
            / self
                .loadout
                .as_ref()
                .map(|loadout| loadout.stamina)
                .unwrap_or(1.0);
        if self.player.action_cooldown > 0.0 || self.player.stamina < cost {
            return;
        }
        let Some(index) = self.target(target, 4.5, true) else {
            return;
        };
        self.player.action_cooldown = 0.8;
        self.player.stamina -= cost;
        let actor = &mut self.creatures[index];
        let matched = actor.social_cue == action;
        if matched {
            actor.social_cue = actor.social_cue.next();
        }
        let Some(species) = self
            .species
            .iter_mut()
            .find(|species| species.id == actor.species_id)
        else {
            return;
        };
        let was_allied = species.allied();
        let gain = (self
            .loadout
            .as_ref()
            .map(|loadout| 20.0 * loadout.social_power)
            .unwrap_or(20.0)
            + if self.unlocked_parts.contains(&PartUnlock::Voice) {
                10.0
            } else {
                0.0
            })
        .round()
        .clamp(1.0, 100.0) as i16;
        species.relationship =
            (species.relationship + if matched { gain } else { -8 }).clamp(-100, 100);
        events.push(CreatureEvent::Social {
            actor: actor.id,
            matched,
            relationship: species.relationship,
        });
        if !was_allied && species.allied() {
            if !species.reward_claimed {
                self.dna = self.dna.saturating_add(35);
                species.reward_claimed = true;
            }
            events.push(CreatureEvent::Allied {
                species: species.id,
            });
        }
    }
    fn interact(&mut self, target: Option<u32>, events: &mut Vec<CreatureEvent>) {
        if let Some(index) = self.food.iter().position(|food| {
            self.player_diet.can_eat(food.kind)
                && self.player.position.distance(food.position) < 2.0
                && self.player.hunger < 0.98
        }) {
            let food = self.food.remove(index);
            self.player.hunger = (self.player.hunger + food.nutrition).min(1.0);
            self.player.health = (self.player.health + 3.0).min(self.player.max_health);
            events.push(CreatureEvent::Ate {
                actor: self.player.id,
                food: food.id,
            });
        }
        let mut index = 0;
        while index < self.pickups.len() {
            if self.player.position.distance(self.pickups[index].position) <= 2.0 {
                let pickup = self.pickups.remove(index);
                let newly_unlocked = !self.unlocked_parts.contains(&pickup.part);
                if newly_unlocked {
                    self.unlocked_parts.push(pickup.part);
                }
                self.dna = self.dna.saturating_add(pickup.dna);
                events.push(CreatureEvent::PartFound {
                    part: pickup.part,
                    newly_unlocked,
                    dna: pickup.dna,
                });
            } else {
                index += 1;
            }
        }
        if self.pack.len() < self.config.pack_capacity {
            if let Some(actor) = self
                .creatures
                .iter()
                .filter(|actor| {
                    actor.alive()
                        && !self.pack.contains(&actor.id)
                        && target.map(|id| id == actor.id).unwrap_or(true)
                        && self.player.position.distance(actor.position) <= 3.5
                        && self
                            .species
                            .iter()
                            .any(|species| species.id == actor.species_id && species.allied())
                })
                .min_by(|a, b| {
                    self.player
                        .position
                        .distance(a.position)
                        .total_cmp(&self.player.position.distance(b.position))
                })
            {
                self.pack.push(actor.id);
                events.push(CreatureEvent::Recruited { actor: actor.id });
            }
        }
    }
    fn pack_attack(&mut self, events: &mut Vec<CreatureEvent>) {
        for id in self.pack.clone() {
            let Some(index) = self.creatures.iter().position(|actor| {
                actor.id == id
                    && actor.alive()
                    && actor.action_cooldown <= 0.0
                    && actor.stamina >= 0.12
            }) else {
                continue;
            };
            let position = self.creatures[index].position;
            let target = self
                .creatures
                .iter()
                .enumerate()
                .filter(|(_, actor)| {
                    actor.alive()
                        && actor.id != id
                        && !self.pack.contains(&actor.id)
                        && actor.position.distance(position) < 3.0
                        && actor.position.distance(self.player.position) < 9.0
                        && self.species.iter().any(|species| {
                            species.id == actor.species_id && species.relationship <= -30
                        })
                })
                .min_by(|(_, a), (_, b)| {
                    position
                        .distance(a.position)
                        .total_cmp(&position.distance(b.position))
                })
                .map(|(index, _)| index);
            if let Some(target) = target {
                self.creatures[index].action_cooldown = 0.9;
                self.creatures[index].stamina -= 0.12;
                damage(&mut self.creatures[target], 16.0, events);
            }
        }
    }
    fn npc_feed(&mut self, events: &mut Vec<CreatureEvent>) {
        for actor in &mut self.creatures {
            if !actor.alive() || actor.hunger >= 0.75 {
                continue;
            }
            let diet = self
                .species
                .iter()
                .find(|species| species.id == actor.species_id)
                .map(|species| species.diet)
                .unwrap_or(Diet::Omnivore);
            if let Some(index) = self.food.iter().position(|food| {
                diet.can_eat(food.kind) && actor.position.distance(food.position) < 1.5
            }) {
                let food = self.food.remove(index);
                actor.hunger = (actor.hunger + food.nutrition).min(1.0);
                events.push(CreatureEvent::Ate {
                    actor: actor.id,
                    food: food.id,
                });
            }
        }
    }
    fn remove_dead(&mut self, events: &mut Vec<CreatureEvent>) {
        let dead: Vec<CreatureActor> = self
            .creatures
            .iter()
            .filter(|actor| !actor.alive())
            .cloned()
            .collect();
        self.creatures.retain(|actor| actor.alive());
        self.pack
            .retain(|id| self.creatures.iter().any(|actor| actor.id == *id));
        for actor in dead {
            events.push(CreatureEvent::Killed {
                actor: actor.id,
                species: actor.species_id,
            });
            let id = self.id();
            if self.food.len() < MAX_ENTITIES / 2 {
                self.food.push(CreatureFood {
                    id,
                    position: actor.position,
                    kind: FoodKind::Meat,
                    nutrition: 0.55,
                });
            }
            if let Some(species) = self
                .species
                .iter_mut()
                .find(|species| species.id == actor.species_id)
            {
                species.kills = species.kills.saturating_add(1);
            }
        }
        for species in &mut self.species {
            if !species.extinct
                && !self
                    .creatures
                    .iter()
                    .any(|actor| actor.species_id == species.id)
            {
                species.extinct = true;
                species.relationship = -100;
                if !species.reward_claimed {
                    self.dna = self.dna.saturating_add(35);
                    species.reward_claimed = true;
                }
                events.push(CreatureEvent::Extinct {
                    species: species.id,
                });
            }
        }
    }
    fn replenish_food(&mut self) {
        let position = self.random_position(2.0);
        let kind = if self.random() < 0.5 {
            FoodKind::Plant
        } else {
            FoodKind::Meat
        };
        let id = self.id();
        self.food.push(CreatureFood {
            id,
            position,
            kind,
            nutrition: 0.4,
        });
    }
    fn random(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        ((self.rng >> 40) as u32) as f32 / 16_777_216.0
    }
    fn random_position(&mut self, margin: f32) -> CreatureVec2 {
        CreatureVec2::new(
            (self.random() * 2.0 - 1.0) * (self.config.half_extents.x - margin),
            (self.random() * 2.0 - 1.0) * (self.config.half_extents.y - margin),
        )
    }
    fn id(&mut self) -> u32 {
        loop {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1).max(1);
            if id != 0
                && id != self.player.id
                && !self.creatures.iter().any(|item| item.id == id)
                && !self.nests.iter().any(|item| item.id == id)
                && !self.food.iter().any(|item| item.id == id)
                && !self.pickups.iter().any(|item| item.id == id)
            {
                return id;
            }
        }
    }

    fn validate(&self) -> Result<(), CreatureError> {
        self.config.validate()?;
        if let Some(creation) = &self.creation {
            if creation.kind != CreationKind::Creature
                || creation
                    .gameplay_loadout()
                    .map_err(|_| CreatureError::InvalidCreation)?
                    != GameplayLoadout::Creature(
                        self.loadout.clone().ok_or(CreatureError::InvalidCreation)?,
                    )
            {
                return Err(CreatureError::InvalidCreation);
            }
        }
        if let Some(loadout) = &self.loadout {
            validate_loadout(loadout)?;
            if self.player_diet != loadout.diet
                || (self.player.max_health - loadout.health).abs() > 0.001
            {
                return Err(CreatureError::InvalidLoadout);
            }
        }
        if self.rng == 0
            || self.next_id == 0
            || !self.accumulator.is_finite()
            || !(0.0..=86_400.0).contains(&self.accumulator)
            || self.species.len() != self.config.species_count
            || self.nests.len() != self.species.len() + 1
            || self.creatures.len() > self.config.species_count * self.config.members_per_species
            || self.food.len() > MAX_ENTITIES / 2
            || self.pickups.len() > self.config.pickup_count
            || self.pack.len() > self.config.pack_capacity
            || self.unlocked_parts.len() > 4
        {
            return Err(CreatureError::InvalidSnapshot);
        }
        let mut ids = HashSet::new();
        let mut species_ids = HashSet::new();
        for species in &self.species {
            if species.id == 0 || !species_ids.insert(species.id) {
                return Err(CreatureError::DuplicateId);
            }
            if species.name.is_empty()
                || species.name.len() > 48
                || species.name.chars().any(char::is_control)
                || !(-100..=100).contains(&species.relationship)
            {
                return Err(CreatureError::InvalidEntity);
            }
            let living = self
                .creatures
                .iter()
                .filter(|actor| actor.species_id == species.id)
                .count();
            if species.extinct != (living == 0)
                || living + species.kills as usize != self.config.members_per_species
            {
                return Err(CreatureError::InvalidReference);
            }
        }
        if self.player.species_id != 0 {
            return Err(CreatureError::InvalidReference);
        }
        for actor in std::iter::once(&self.player).chain(&self.creatures) {
            unique(&mut ids, actor.id)?;
            if actor.species_id != 0 && !species_ids.contains(&actor.species_id) {
                return Err(CreatureError::InvalidReference);
            }
            if actor.id != self.player.id && actor.species_id == 0 {
                return Err(CreatureError::InvalidReference);
            }
            if !inside(actor.position, self.config.half_extents)
                || !finite(actor.velocity)
                || actor.velocity.length() > 100.0
                || !actor.heading.is_finite()
                || actor.heading.abs() > 1_000_000.0
                || !actor.max_health.is_finite()
                || !(1.0..=10_000.0).contains(&actor.max_health)
                || !actor.health.is_finite()
                || !(0.0..=actor.max_health).contains(&actor.health)
                || !actor.stamina.is_finite()
                || !(0.0..=1.0).contains(&actor.stamina)
                || !actor.hunger.is_finite()
                || !(0.0..=1.0).contains(&actor.hunger)
                || !actor.action_cooldown.is_finite()
                || !(0.0..=10.0).contains(&actor.action_cooldown)
                || (actor.id != self.player.id && !actor.alive())
            {
                return Err(CreatureError::InvalidEntity);
            }
        }
        let mut nest_species = HashSet::new();
        for nest in &self.nests {
            unique(&mut ids, nest.id)?;
            if !nest_species.insert(nest.species_id)
                || (nest.species_id != 0 && !species_ids.contains(&nest.species_id))
            {
                return Err(CreatureError::InvalidReference);
            }
            if !inside(nest.position, self.config.half_extents)
                || !nest.radius.is_finite()
                || !(1.0..=10.0).contains(&nest.radius)
            {
                return Err(CreatureError::InvalidEntity);
            }
        }
        for food in &self.food {
            unique(&mut ids, food.id)?;
            if !inside(food.position, self.config.half_extents)
                || !food.nutrition.is_finite()
                || !(0.01..=1.0).contains(&food.nutrition)
            {
                return Err(CreatureError::InvalidEntity);
            }
        }
        for pickup in &self.pickups {
            unique(&mut ids, pickup.id)?;
            if !inside(pickup.position, self.config.half_extents)
                || pickup.dna == 0
                || pickup.dna > 1000
            {
                return Err(CreatureError::InvalidEntity);
            }
        }
        let mut pack = HashSet::new();
        for id in &self.pack {
            if !pack.insert(*id) {
                return Err(CreatureError::DuplicateId);
            }
            if !self.creatures.iter().any(|actor| {
                actor.id == *id
                    && self
                        .species
                        .iter()
                        .any(|species| species.id == actor.species_id && species.allied())
            }) {
                return Err(CreatureError::InvalidReference);
            }
        }
        let parts: HashSet<PartUnlock> = self.unlocked_parts.iter().copied().collect();
        if parts.len() != self.unlocked_parts.len() {
            return Err(CreatureError::DuplicateId);
        }
        let valid_state = match self.state {
            CreatureState::Playing => self.player.alive() && !self.tribe_eligible(),
            CreatureState::TribeReady => self.tribe_eligible(),
            CreatureState::Dead => !self.player.alive(),
        };
        if !valid_state {
            return Err(CreatureError::InvalidSnapshot);
        }
        Ok(())
    }
}

fn finite(value: CreatureVec2) -> bool {
    value.x.is_finite() && value.y.is_finite()
}
fn inside(value: CreatureVec2, half: CreatureVec2) -> bool {
    finite(value) && value.x.abs() <= half.x && value.y.abs() <= half.y
}
fn normalized(value: CreatureVec2) -> CreatureVec2 {
    let length = value.length();
    if length > 1e-6 {
        value.scale(1.0 / length)
    } else {
        CreatureVec2::new(1.0, 0.0)
    }
}
fn unique(ids: &mut HashSet<u32>, id: u32) -> Result<(), CreatureError> {
    if id == 0 || !ids.insert(id) {
        Err(CreatureError::DuplicateId)
    } else {
        Ok(())
    }
}
fn metabolism(actor: &mut CreatureActor, resting: bool) {
    metabolism_tuned(actor, resting, 1.0);
}
fn metabolism_tuned(actor: &mut CreatureActor, resting: bool, reserve: f32) {
    actor.action_cooldown = (actor.action_cooldown - DT).max(0.0);
    actor.hunger = (actor.hunger - DT * 0.003).max(0.0);
    actor.stamina = (actor.stamina + DT * if resting { 0.45 } else { 0.16 } / reserve).min(1.0);
    if actor.hunger <= 0.0 {
        actor.health = (actor.health - DT * 5.0).max(0.0);
    }
}
fn move_player(
    actor: &mut CreatureActor,
    input: CreatureInput,
    resting: bool,
    stride: bool,
    base_speed: f32,
    reserve: f32,
) {
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
    actor.heading = (actor.heading + turn * 2.8 * DT).rem_euclid(std::f32::consts::TAU);
    if resting {
        actor.velocity = CreatureVec2::ZERO;
        return;
    }
    let sprinting = input.sprint && actor.stamina > 0.05 && thrust.abs() > 0.01;
    if sprinting {
        actor.stamina = (actor.stamina - DT * 0.46 / reserve).max(0.0);
    }
    let speed =
        base_speed * (if sprinting { 9.0 / 5.5 } else { 1.0 }) * if stride { 1.2 } else { 1.0 };
    let acceleration = 24.0
        * (base_speed / 5.5)
        * if sprinting { 1.7 } else { 1.0 }
        * if stride { 1.2 } else { 1.0 };
    actor.velocity = actor
        .velocity
        .add(actor.forward().scale(thrust * acceleration * DT))
        .scale(0.94);
    if actor.velocity.length() > speed {
        actor.velocity = normalized(actor.velocity).scale(speed);
    }
    actor.position = actor.position.add(actor.velocity.scale(DT));
}
fn steer_to(actor: &mut CreatureActor, target: CreatureVec2, speed: f32) {
    let delta = target.sub(actor.position);
    let desired = if delta.length() < 0.4 {
        CreatureVec2::ZERO
    } else {
        normalized(delta).scale(speed.min(delta.length() * 2.0))
    };
    actor.velocity = actor.velocity.scale(0.9).add(desired.scale(0.1));
    if actor.velocity.length() > 0.01 {
        actor.heading = actor.velocity.y.atan2(actor.velocity.x);
    }
    actor.position = actor.position.add(actor.velocity.scale(DT));
}
fn damage(actor: &mut CreatureActor, amount: f32, events: &mut Vec<CreatureEvent>) {
    if !actor.alive() {
        return;
    }
    let amount = amount.min(actor.health);
    actor.health -= amount;
    events.push(CreatureEvent::Damaged {
        actor: actor.id,
        amount,
    });
}
fn bound(actor: &mut CreatureActor, half: CreatureVec2) {
    let x = half.x - actor.radius();
    let y = half.y - actor.radius();
    if actor.position.x.abs() > x {
        actor.position.x = actor.position.x.clamp(-x, x);
        actor.velocity.x = 0.0;
    }
    if actor.position.y.abs() > y {
        actor.position.y = actor.position.y.clamp(-y, y);
        actor.velocity.y = 0.0;
    }
}
fn separate(a: &mut CreatureActor, b: &mut CreatureActor) {
    let delta = b.position.sub(a.position);
    let distance = delta.length();
    let limit = a.radius() + b.radius();
    if distance < limit {
        let shift = normalized(delta).scale((limit - distance) * 0.5);
        a.position = a.position.sub(shift);
        b.position = b.position.add(shift);
    }
}

impl CreatureSnapshot {
    /// Versioned little-endian data including RNG, cooldowns and fractional tick debt.
    pub fn to_bytes(&self) -> Result<Vec<u8>, CreatureError> {
        let world = CreatureWorld::from_snapshot(self.clone())?;
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        u32_out(&mut out, self.version);
        vec_out(&mut out, world.config.half_extents);
        for value in [
            world.config.species_count,
            world.config.members_per_species,
            world.config.food_count,
            world.config.pickup_count,
            world.config.pack_capacity,
        ] {
            u32_out(&mut out, value as u32);
        }
        u32_out(&mut out, world.config.tribe_dna);
        u32_out(&mut out, world.config.tribe_parts as u32);
        u32_out(&mut out, world.config.tribe_resolved_species as u32);
        u64_out(&mut out, world.seed);
        u64_out(&mut out, world.rng);
        u32_out(&mut out, world.next_id);
        u64_out(&mut out, world.tick);
        out.extend_from_slice(&world.accumulator.to_le_bytes());
        u32_out(&mut out, world.dna);
        out.push(match world.state {
            CreatureState::Playing => 0,
            CreatureState::TribeReady => 1,
            CreatureState::Dead => 2,
        });
        out.push(diet_byte(world.player_diet));
        actor_out(&mut out, &world.player);
        u32_out(&mut out, world.creatures.len() as u32);
        for actor in &world.creatures {
            actor_out(&mut out, actor);
        }
        u32_out(&mut out, world.species.len() as u32);
        for species in &world.species {
            u32_out(&mut out, species.id);
            u32_out(&mut out, species.name.len() as u32);
            out.extend_from_slice(species.name.as_bytes());
            out.push(diet_byte(species.diet));
            out.extend_from_slice(&species.relationship.to_le_bytes());
            u32_out(&mut out, species.kills);
            out.push(u8::from(species.extinct));
            out.push(u8::from(species.reward_claimed));
        }
        u32_out(&mut out, world.nests.len() as u32);
        for nest in &world.nests {
            u32_out(&mut out, nest.id);
            u32_out(&mut out, nest.species_id);
            vec_out(&mut out, nest.position);
            f32_out(&mut out, nest.radius);
        }
        u32_out(&mut out, world.food.len() as u32);
        for food in &world.food {
            u32_out(&mut out, food.id);
            vec_out(&mut out, food.position);
            out.push(food_byte(food.kind));
            f32_out(&mut out, food.nutrition);
        }
        u32_out(&mut out, world.pickups.len() as u32);
        for pickup in &world.pickups {
            u32_out(&mut out, pickup.id);
            vec_out(&mut out, pickup.position);
            out.push(part_byte(pickup.part));
            u32_out(&mut out, pickup.dna);
        }
        u32_out(&mut out, world.pack.len() as u32);
        for id in &world.pack {
            u32_out(&mut out, *id);
        }
        u32_out(&mut out, world.unlocked_parts.len() as u32);
        for part in &world.unlocked_parts {
            out.push(part_byte(*part));
        }
        if self.version == 2 {
            out.push(u8::from(world.loadout.is_some()));
            if let Some(loadout) = &world.loadout {
                out.push(diet_byte(loadout.diet));
                for value in [
                    loadout.health,
                    loadout.speed,
                    loadout.attack_damage,
                    loadout.social_power,
                    loadout.stamina,
                    loadout.defense,
                ] {
                    f32_out(&mut out, value);
                }
                out.push(u8::from(loadout.can_glide));
            }
            out.push(u8::from(world.creation.is_some()));
            if let Some(creation) = &world.creation {
                let bytes = EditorSnapshot {
                    version: 1,
                    revision: 0,
                    creation: creation.clone(),
                }
                .to_bytes()
                .map_err(|_| CreatureError::InvalidCreation)?;
                u32_out(&mut out, bytes.len() as u32);
                out.extend_from_slice(&bytes);
            }
        }
        Ok(out)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CreatureError> {
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(CreatureError::TooManyEntities);
        }
        let mut read = Reader { bytes, at: 0 };
        if read.take(8)? != MAGIC {
            return Err(CreatureError::InvalidSnapshot);
        }
        let version = read.u32()?;
        if !matches!(version, 1 | 2) {
            return Err(CreatureError::UnsupportedVersion(version));
        }
        let config = CreatureConfig {
            half_extents: read.vec()?,
            species_count: read.count(12)?,
            members_per_species: read.count(16)?,
            food_count: read.count(2000)?,
            pickup_count: read.count(128)?,
            pack_capacity: read.count(8)?,
            tribe_dna: read.u32()?,
            tribe_parts: read.count(4)?,
            tribe_resolved_species: read.count(12)?,
        };
        config.validate()?;
        let seed = read.u64()?;
        let rng = read.u64()?;
        let next_id = read.u32()?;
        let tick = read.u64()?;
        let accumulator = f64::from_le_bytes(read.take(8)?.try_into().unwrap());
        let dna = read.u32()?;
        let state = match read.byte()? {
            0 => CreatureState::Playing,
            1 => CreatureState::TribeReady,
            2 => CreatureState::Dead,
            _ => return Err(CreatureError::InvalidSnapshot),
        };
        let player_diet = read.diet()?;
        let player = read.actor()?;
        let mut creatures = Vec::new();
        for _ in 0..read.count(192)? {
            creatures.push(read.actor()?);
        }
        let mut species = Vec::new();
        for _ in 0..read.count(12)? {
            let id = read.u32()?;
            let length = read.count(48)?;
            let name = std::str::from_utf8(read.take(length)?)
                .map_err(|_| CreatureError::InvalidSnapshot)?
                .to_owned();
            let diet = read.diet()?;
            let relationship = i16::from_le_bytes(read.take(2)?.try_into().unwrap());
            let kills = read.u32()?;
            let extinct = read.boolean()?;
            let reward_claimed = read.boolean()?;
            species.push(CreatureSpecies {
                id,
                name,
                diet,
                relationship,
                kills,
                extinct,
                reward_claimed,
            });
        }
        let mut nests = Vec::new();
        for _ in 0..read.count(13)? {
            nests.push(CreatureNest {
                id: read.u32()?,
                species_id: read.u32()?,
                position: read.vec()?,
                radius: read.f32()?,
            });
        }
        let mut food = Vec::new();
        for _ in 0..read.count(MAX_ENTITIES / 2)? {
            food.push(CreatureFood {
                id: read.u32()?,
                position: read.vec()?,
                kind: read.food_kind()?,
                nutrition: read.f32()?,
            });
        }
        let mut pickups = Vec::new();
        for _ in 0..read.count(128)? {
            pickups.push(CreaturePickup {
                id: read.u32()?,
                position: read.vec()?,
                part: read.part()?,
                dna: read.u32()?,
            });
        }
        let mut pack = Vec::new();
        for _ in 0..read.count(8)? {
            pack.push(read.u32()?);
        }
        let mut unlocked_parts = Vec::new();
        for _ in 0..read.count(4)? {
            unlocked_parts.push(read.part()?);
        }
        let loadout = if version == 2 {
            match read.byte()? {
                0 => None,
                1 => Some(CreatureLoadout {
                    diet: read.diet()?,
                    health: read.f32()?,
                    speed: read.f32()?,
                    attack_damage: read.f32()?,
                    social_power: read.f32()?,
                    stamina: read.f32()?,
                    defense: read.f32()?,
                    can_glide: read.boolean()?,
                }),
                _ => return Err(CreatureError::InvalidLoadout),
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
                        return Err(CreatureError::InvalidCreation);
                    }
                    Some(
                        EditorSnapshot::from_bytes(read.take(count)?)
                            .map_err(|_| CreatureError::InvalidCreation)?
                            .creation,
                    )
                }
                _ => return Err(CreatureError::InvalidCreation),
            }
        } else {
            None
        };
        if read.at != bytes.len() {
            return Err(CreatureError::TrailingBytes);
        }
        let world = CreatureWorld {
            config,
            player,
            player_diet,
            creatures,
            species,
            nests,
            food,
            pickups,
            pack,
            state,
            tick,
            dna,
            unlocked_parts,
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

fn validate_loadout(loadout: &CreatureLoadout) -> Result<(), CreatureError> {
    if ![
        (loadout.health, 1.0, 1500.0),
        (loadout.speed, 0.1, 18.0),
        (loadout.attack_damage, 0.0, 250.0),
        (loadout.social_power, 0.1, 100.0),
        (loadout.stamina, 0.1, 5.0),
        (loadout.defense, 0.0, 0.75),
    ]
    .iter()
    .all(|(value, min, max)| value.is_finite() && (*min..=*max).contains(value))
    {
        return Err(CreatureError::InvalidLoadout);
    }
    if loadout.can_glide {
        return Err(CreatureError::UnsupportedCapability);
    }
    Ok(())
}

fn u32_out(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn u64_out(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn f32_out(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}
fn vec_out(out: &mut Vec<u8>, value: CreatureVec2) {
    f32_out(out, value.x);
    f32_out(out, value.y);
}
fn diet_byte(diet: Diet) -> u8 {
    match diet {
        Diet::Herbivore => 0,
        Diet::Carnivore => 1,
        Diet::Omnivore => 2,
    }
}
fn food_byte(kind: FoodKind) -> u8 {
    if kind == FoodKind::Plant {
        0
    } else {
        1
    }
}
fn part_byte(part: PartUnlock) -> u8 {
    match part {
        PartUnlock::Stride => 0,
        PartUnlock::Claws => 1,
        PartUnlock::Voice => 2,
        PartUnlock::Armor => 3,
    }
}
fn social_byte(action: SocialAction) -> u8 {
    match action {
        SocialAction::Sing => 0,
        SocialAction::Dance => 1,
        SocialAction::Charm => 2,
    }
}
fn actor_out(out: &mut Vec<u8>, actor: &CreatureActor) {
    u32_out(out, actor.id);
    u32_out(out, actor.species_id);
    vec_out(out, actor.position);
    vec_out(out, actor.velocity);
    for value in [
        actor.heading,
        actor.health,
        actor.max_health,
        actor.stamina,
        actor.hunger,
        actor.action_cooldown,
    ] {
        f32_out(out, value);
    }
    out.push(social_byte(actor.social_cue));
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], CreatureError> {
        let end = self
            .at
            .checked_add(length)
            .ok_or(CreatureError::TruncatedSnapshot)?;
        let result = self
            .bytes
            .get(self.at..end)
            .ok_or(CreatureError::TruncatedSnapshot)?;
        self.at = end;
        Ok(result)
    }
    fn byte(&mut self) -> Result<u8, CreatureError> {
        Ok(self.take(1)?[0])
    }
    fn boolean(&mut self) -> Result<bool, CreatureError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(CreatureError::InvalidSnapshot),
        }
    }
    fn u32(&mut self) -> Result<u32, CreatureError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, CreatureError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, CreatureError> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn vec(&mut self) -> Result<CreatureVec2, CreatureError> {
        Ok(CreatureVec2::new(self.f32()?, self.f32()?))
    }
    fn count(&mut self, limit: usize) -> Result<usize, CreatureError> {
        let count = self.u32()? as usize;
        if count > limit {
            Err(CreatureError::TooManyEntities)
        } else {
            Ok(count)
        }
    }
    fn diet(&mut self) -> Result<Diet, CreatureError> {
        match self.byte()? {
            0 => Ok(Diet::Herbivore),
            1 => Ok(Diet::Carnivore),
            2 => Ok(Diet::Omnivore),
            _ => Err(CreatureError::InvalidSnapshot),
        }
    }
    fn food_kind(&mut self) -> Result<FoodKind, CreatureError> {
        match self.byte()? {
            0 => Ok(FoodKind::Plant),
            1 => Ok(FoodKind::Meat),
            _ => Err(CreatureError::InvalidSnapshot),
        }
    }
    fn part(&mut self) -> Result<PartUnlock, CreatureError> {
        match self.byte()? {
            0 => Ok(PartUnlock::Stride),
            1 => Ok(PartUnlock::Claws),
            2 => Ok(PartUnlock::Voice),
            3 => Ok(PartUnlock::Armor),
            _ => Err(CreatureError::InvalidSnapshot),
        }
    }
    fn social(&mut self) -> Result<SocialAction, CreatureError> {
        match self.byte()? {
            0 => Ok(SocialAction::Sing),
            1 => Ok(SocialAction::Dance),
            2 => Ok(SocialAction::Charm),
            _ => Err(CreatureError::InvalidSnapshot),
        }
    }
    fn actor(&mut self) -> Result<CreatureActor, CreatureError> {
        Ok(CreatureActor {
            id: self.u32()?,
            species_id: self.u32()?,
            position: self.vec()?,
            velocity: self.vec()?,
            heading: self.f32()?,
            health: self.f32()?,
            max_health: self.f32()?,
            stamina: self.f32()?,
            hunger: self.f32()?,
            action_cooldown: self.f32()?,
            social_cue: self.social()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quiet(diet: Diet) -> CreatureWorld {
        let mut world = CreatureWorld::with_config(
            42,
            diet,
            CreatureConfig {
                members_per_species: 1,
                food_count: 0,
                pickup_count: 0,
                ..Default::default()
            },
        )
        .unwrap();
        for species in &mut world.species {
            species.relationship = 0;
        }
        world
    }
    fn nearby(world: &mut CreatureWorld, index: usize) -> u32 {
        world.creatures[index].position = world.player.position.add(CreatureVec2::new(2.2, 0.0));
        world.creatures[index].velocity = CreatureVec2::ZERO;
        world.creatures[index].id
    }
    fn food(world: &mut CreatureWorld, kind: FoodKind) -> u32 {
        let id = world.id();
        world.food.push(CreatureFood {
            id,
            position: world.player.position,
            kind,
            nutrition: 0.4,
        });
        id
    }

    #[test]
    fn seeded_world_has_species_nests_and_reachable_learning_pickups() {
        let world = CreatureWorld::new(93, Diet::Carnivore);
        assert_eq!(world, CreatureWorld::new(93, Diet::Carnivore));
        assert_ne!(world.nests, CreatureWorld::new(94, Diet::Carnivore).nests);
        assert_eq!(world.species.len(), 4);
        assert_eq!(world.creatures.len(), 12);
        assert_eq!(world.nests.len(), 5);
        assert!(world
            .food
            .iter()
            .take(6)
            .all(|item| item.kind == FoodKind::Meat && item.position.length() < 11.0));
        assert!(world.pickups[0].position.length() < 6.0);
        world.validate().unwrap();
    }

    #[test]
    fn identical_ticks_and_different_frame_partitions_replay() {
        let mut a = quiet(Diet::Omnivore);
        let mut b = a.clone();
        let input = CreatureInput {
            thrust: 0.8,
            turn: 0.2,
            sprint: true,
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
        assert_eq!(a.creatures, b.creatures);
        for tick in 0..300 {
            let input = CreatureInput {
                thrust: 0.5,
                turn: if tick < 100 { 0.3 } else { -0.2 },
                ..Default::default()
            };
            assert_eq!(a.step(input), b.step(input));
        }
        assert_eq!(a.player, b.player);
        assert_eq!(a.food, b.food);
    }

    #[test]
    fn work_budget_retains_debt_and_invalid_time_does_not_poison_world() {
        let mut world = quiet(Diet::Herbivore);
        let report = world.advance(5.0, CreatureInput::default());
        assert_eq!(report.steps, 240);
        assert!((report.backlog_seconds - 1.0).abs() < 1e-10);
        assert_eq!(world.advance(0.0, CreatureInput::default()).steps, 60);
        let before = world.clone();
        assert!(
            world
                .advance(f64::NAN, CreatureInput::default())
                .time_rejected
        );
        assert!(world.advance(-1.0, CreatureInput::default()).time_rejected);
        assert!(
            world
                .advance(86_401.0, CreatureInput::default())
                .time_rejected
        );
        assert_eq!(before, world);
    }

    #[test]
    fn sprint_and_stride_change_distance_and_reserve() {
        let mut walk = quiet(Diet::Herbivore);
        let mut sprint = walk.clone();
        let mut stride = walk.clone();
        stride.unlocked_parts.push(PartUnlock::Stride);
        for _ in 0..60 {
            walk.step(CreatureInput {
                thrust: 1.0,
                ..Default::default()
            });
            sprint.step(CreatureInput {
                thrust: 1.0,
                sprint: true,
                ..Default::default()
            });
            stride.step(CreatureInput {
                thrust: 1.0,
                ..Default::default()
            });
        }
        assert!(sprint.player.position.x > walk.player.position.x * 1.4);
        assert!(stride.player.position.x > walk.player.position.x * 1.1);
        assert!(sprint.player.stamina < walk.player.stamina);
    }

    #[test]
    fn feeding_requires_interaction_correct_diet_and_appetite() {
        for diet in [Diet::Herbivore, Diet::Carnivore, Diet::Omnivore] {
            let mut world = quiet(diet);
            world.player.hunger = 0.2;
            let plant = food(&mut world, FoodKind::Plant);
            let meat = food(&mut world, FoodKind::Meat);
            world.step(CreatureInput::default());
            assert_eq!(world.food.len(), 2);
            let events = world.step(CreatureInput {
                interact: true,
                ..Default::default()
            });
            assert!(world.player.hunger > 0.59);
            let consumed = if diet == Diet::Carnivore { meat } else { plant };
            assert!(events.contains(&CreatureEvent::Ate {
                actor: world.player.id,
                food: consumed
            }));
            assert_eq!(world.food.len(), 1);
        }
        let mut world = quiet(Diet::Herbivore);
        food(&mut world, FoodKind::Plant);
        world.step(CreatureInput {
            interact: true,
            ..Default::default()
        });
        assert_eq!(world.food.len(), 1);
    }

    #[test]
    fn starvation_death_terminal_then_restart_restores_seeded_state() {
        let mut world = quiet(Diet::Carnivore);
        world.player.hunger = 0.0;
        world.player.health = 0.01;
        let events = world.step(CreatureInput {
            interact: true,
            ..Default::default()
        });
        assert!(events.contains(&CreatureEvent::PlayerDied));
        assert_eq!(world.state, CreatureState::Dead);
        let dead = world.clone();
        assert!(world
            .step(CreatureInput {
                thrust: 1.0,
                ..Default::default()
            })
            .is_empty());
        assert_eq!(world, dead);
        world.snapshot().to_bytes().unwrap();
        let expected =
            CreatureWorld::with_config(world.seed(), world.player_diet, world.config.clone())
                .unwrap();
        world.restart();
        assert_eq!(world, expected);
    }

    #[test]
    fn rest_heals_only_at_home_and_stops_movement() {
        let mut home = quiet(Diet::Herbivore);
        home.player.health = 50.0;
        home.player.stamina = 0.2;
        let mut away = home.clone();
        away.player.position = CreatureVec2::new(12.0, 0.0);
        home.step(CreatureInput {
            rest: true,
            thrust: 1.0,
            ..Default::default()
        });
        away.step(CreatureInput {
            rest: true,
            ..Default::default()
        });
        assert!(home.player.health > 50.0);
        assert_eq!(home.player.position, CreatureVec2::ZERO);
        assert_eq!(away.player.health, 50.0);
        assert!(home.player.stamina > away.player.stamina);
    }

    #[test]
    fn matching_social_cues_build_alliance_wrong_cues_hurt_and_cooldown_applies() {
        let mut world = quiet(Diet::Herbivore);
        let id = nearby(&mut world, 0);
        world.step(CreatureInput {
            target: Some(id),
            social: Some(SocialAction::Dance),
            ..Default::default()
        });
        assert_eq!(world.species[0].relationship, -8);
        world.player.action_cooldown = 0.0;
        world.step(CreatureInput {
            target: Some(id),
            social: Some(SocialAction::Sing),
            ..Default::default()
        });
        assert_eq!(world.species[0].relationship, 12);
        assert_eq!(world.creatures[0].social_cue, SocialAction::Dance);
        world.step(CreatureInput {
            target: Some(id),
            social: Some(SocialAction::Dance),
            ..Default::default()
        });
        assert_eq!(world.species[0].relationship, 12);
        for _ in 0..3 {
            nearby(&mut world, 0);
            world.player.action_cooldown = 0.0;
            let action = world.creatures[0].social_cue;
            world.step(CreatureInput {
                target: Some(id),
                social: Some(action),
                ..Default::default()
            });
        }
        assert!(world.species[0].allied());
        assert_eq!(world.dna, 35);
        world.player.action_cooldown = 0.0;
        world.step(CreatureInput {
            target: Some(id),
            social: Some(SocialAction::Sing),
            ..Default::default()
        });
        assert_eq!(world.dna, 35);
    }

    #[test]
    fn social_voice_unlock_improves_success_and_invalid_target_cannot_hit_bystander() {
        let mut world = quiet(Diet::Herbivore);
        nearby(&mut world, 0);
        world.unlocked_parts.push(PartUnlock::Voice);
        world.step(CreatureInput {
            social: Some(SocialAction::Sing),
            ..Default::default()
        });
        assert_eq!(world.species[0].relationship, 30);
        world.player.action_cooldown = 0.0;
        let health = world.creatures[0].health;
        world.step(CreatureInput {
            attack: true,
            target: Some(u32::MAX),
            ..Default::default()
        });
        assert_eq!(world.creatures[0].health, health);
    }

    #[test]
    fn combat_kill_extinction_reward_corpse_and_exhaustion_are_real() {
        let mut world = quiet(Diet::Carnivore);
        let id = nearby(&mut world, 0);
        world.creatures[0].health = 20.0;
        world.player.stamina = 0.0;
        world.step(CreatureInput {
            attack: true,
            target: Some(id),
            ..Default::default()
        });
        assert!(world.creatures.iter().any(|actor| actor.id == id));
        world.player.stamina = 1.0;
        let events = world.step(CreatureInput {
            attack: true,
            target: Some(id),
            ..Default::default()
        });
        assert!(!world.creatures.iter().any(|actor| actor.id == id));
        assert!(world.species[0].extinct);
        assert_eq!(world.species[0].kills, 1);
        assert_eq!(world.dna, 35);
        assert!(world.food.iter().any(|food| food.kind == FoodKind::Meat));
        assert!(events.contains(&CreatureEvent::Extinct {
            species: world.species[0].id
        }));
        world.validate().unwrap();
    }

    #[test]
    fn claws_and_armor_have_gameplay_effects_and_hostiles_can_kill_player() {
        let mut normal = quiet(Diet::Herbivore);
        let id = nearby(&mut normal, 0);
        let mut upgraded = normal.clone();
        upgraded
            .unlocked_parts
            .extend([PartUnlock::Claws, PartUnlock::Armor]);
        normal.step(CreatureInput {
            attack: true,
            target: Some(id),
            ..Default::default()
        });
        upgraded.step(CreatureInput {
            attack: true,
            target: Some(id),
            ..Default::default()
        });
        assert!(upgraded.creatures[0].health < normal.creatures[0].health);
        let mut armor = quiet(Diet::Herbivore);
        nearby(&mut armor, 0);
        armor.species[0].relationship = -100;
        let mut bare = armor.clone();
        armor.unlocked_parts.push(PartUnlock::Armor);
        armor.step(CreatureInput::default());
        bare.step(CreatureInput::default());
        assert_eq!(armor.player.health, 93.0);
        assert_eq!(bare.player.health, 90.0);
        bare.player.health = 1.0;
        bare.creatures[0].action_cooldown = 0.0;
        let events = bare.step(CreatureInput::default());
        assert!(events.contains(&CreatureEvent::PlayerDied));
    }

    #[test]
    fn allied_recruit_follows_and_can_defend_player() {
        let mut world = quiet(Diet::Herbivore);
        let follower = nearby(&mut world, 0);
        world.species[0].relationship = 60;
        world.species[0].reward_claimed = true;
        assert!(world
            .step(CreatureInput {
                interact: true,
                target: Some(follower),
                ..Default::default()
            })
            .contains(&CreatureEvent::Recruited { actor: follower }));
        world.step(CreatureInput {
            interact: true,
            target: Some(follower),
            ..Default::default()
        });
        assert_eq!(world.pack, vec![follower]);
        world.player.position = CreatureVec2::new(10.0, 0.0);
        let start = world.creatures[0].position.distance(world.player.position);
        for _ in 0..60 {
            world.step(CreatureInput::default());
        }
        assert!(world.creatures[0].position.distance(world.player.position) < start);
        world.creatures[1].position = world.creatures[0].position.add(CreatureVec2::new(0.0, 2.0));
        world.species[1].relationship = -100;
        let before = world.creatures[1].health;
        world.step(CreatureInput::default());
        assert!(world.creatures[1].health < before);
        world.snapshot().to_bytes().unwrap();
    }

    #[test]
    fn killing_pack_member_removes_reference_and_breaking_alliance_dismisses_pack() {
        let mut world = quiet(Diet::Herbivore);
        let follower = nearby(&mut world, 0);
        world.species[0].relationship = 60;
        world.pack.push(follower);
        world.creatures[0].health = 0.0;
        world.step(CreatureInput::default());
        assert!(world.pack.is_empty());
        let second = nearby(&mut world, 0);
        world.species[1].relationship = 60;
        world.pack.push(second);
        world.species[1].relationship = 59;
        world.step(CreatureInput::default());
        assert!(world.pack.is_empty());
    }

    #[test]
    fn bones_unlock_unique_parts_and_all_three_tribe_gates_are_required() {
        let mut world = quiet(Diet::Herbivore);
        world.config.pickup_count = 3;
        for part in [PartUnlock::Stride, PartUnlock::Stride, PartUnlock::Claws] {
            let id = world.id();
            world.pickups.push(CreaturePickup {
                id,
                position: CreatureVec2::ZERO,
                part,
                dna: 15,
            });
        }
        let events = world.step(CreatureInput {
            interact: true,
            ..Default::default()
        });
        assert_eq!(world.dna, 45);
        assert_eq!(world.unlocked_parts.len(), 2);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    CreatureEvent::PartFound {
                        newly_unlocked: false,
                        ..
                    }
                ))
                .count(),
            1
        );
        assert_eq!(world.state, CreatureState::Playing);
        world.dna = 100;
        assert!(!world.tribe_eligible());
        world.species[0].relationship = 60;
        assert!(!world.tribe_eligible());
        world.species[1].relationship = 60;
        assert!(world.tribe_eligible());
        let events = world.step(CreatureInput::default());
        assert!(events.contains(&CreatureEvent::TribeEligible));
        assert_eq!(world.state, CreatureState::TribeReady);
        assert_eq!(world.progress(), 1.0);
        assert!(world.step(CreatureInput::default()).is_empty());
        world.snapshot().to_bytes().unwrap();
    }

    #[test]
    fn arena_bounds_and_actor_collision_remain_finite() {
        let mut world = quiet(Diet::Herbivore);
        world.creatures[0].position = CreatureVec2::ZERO;
        world.step(CreatureInput {
            thrust: f32::NAN,
            turn: f32::INFINITY,
            ..Default::default()
        });
        assert!(world.player.position.distance(world.creatures[0].position) > 1.7);
        world.player.position = CreatureVec2::new(world.config.half_extents.x, 0.0);
        for _ in 0..120 {
            world.step(CreatureInput {
                thrust: 1.0,
                ..Default::default()
            });
        }
        assert!(world.player.position.x <= world.config.half_extents.x - world.player.radius());
        world.validate().unwrap();
    }

    #[test]
    fn binary_save_preserves_partial_tick_rng_and_future_simulation() {
        let mut world = CreatureWorld::new(777, Diet::Omnivore);
        world.food.pop();
        world.advance(
            0.127,
            CreatureInput {
                thrust: 1.0,
                ..Default::default()
            },
        );
        let bytes = world.snapshot().to_bytes().unwrap();
        let snapshot = CreatureSnapshot::from_bytes(&bytes).unwrap();
        assert_eq!(snapshot.world, world);
        let mut restored = CreatureWorld::from_snapshot(snapshot).unwrap();
        for tick in 0..400 {
            let input = CreatureInput {
                thrust: 0.4,
                turn: (tick % 3) as f32 * 0.1,
                sprint: tick < 30,
                ..Default::default()
            };
            assert_eq!(world.advance(0.021, input), restored.advance(0.021, input));
        }
        assert_eq!(world, restored);
        assert_eq!(
            world.snapshot().to_bytes().unwrap(),
            restored.snapshot().to_bytes().unwrap()
        );
    }

    #[test]
    fn malformed_binary_saves_reject_prefixes_trailing_version_counts_and_enums() {
        let bytes = CreatureWorld::new(4, Diet::Herbivore)
            .snapshot()
            .to_bytes()
            .unwrap();
        for length in 0..bytes.len() {
            assert!(
                CreatureSnapshot::from_bytes(&bytes[..length]).is_err(),
                "accepted truncated save length {length}"
            );
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert_eq!(
            CreatureSnapshot::from_bytes(&bad),
            Err(CreatureError::TrailingBytes)
        );
        let mut bad = bytes.clone();
        bad[8..12].copy_from_slice(&3u32.to_le_bytes());
        assert_eq!(
            CreatureSnapshot::from_bytes(&bad),
            Err(CreatureError::UnsupportedVersion(3))
        );
        let mut bad = bytes.clone();
        bad[20..24].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            CreatureSnapshot::from_bytes(&bad),
            Err(CreatureError::TooManyEntities)
        );
        let mut bad = bytes.clone();
        bad[12..16].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(CreatureSnapshot::from_bytes(&bad).is_err());
        // Header ends with state and player diet at byte offsets 92 and 93.
        let mut bad = bytes.clone();
        bad[92] = 255;
        assert!(CreatureSnapshot::from_bytes(&bad).is_err());
        let mut bad = bytes;
        bad[0] = 0;
        assert_eq!(
            CreatureSnapshot::from_bytes(&bad),
            Err(CreatureError::InvalidSnapshot)
        );
    }

    #[test]
    fn invalid_owned_snapshots_reject_nonfinite_duplicate_and_dangling_references() {
        let world = quiet(Diet::Herbivore);
        let mut bad = world.snapshot();
        bad.world.player.health = f32::NAN;
        assert!(bad.to_bytes().is_err());
        let mut bad = world.snapshot();
        bad.world.creatures[0].id = bad.world.player.id;
        assert_eq!(bad.to_bytes(), Err(CreatureError::DuplicateId));
        let mut bad = world.snapshot();
        bad.world.pack.push(u32::MAX);
        assert_eq!(bad.to_bytes(), Err(CreatureError::InvalidReference));
        let mut bad = world.snapshot();
        bad.world.creatures[0].species_id = 200;
        assert!(bad.to_bytes().is_err());
        let mut bad = world.snapshot();
        bad.world.species[0].extinct = true;
        assert!(bad.to_bytes().is_err());
        let mut bad = world.snapshot();
        bad.world.state = CreatureState::TribeReady;
        assert!(bad.to_bytes().is_err());
        let mut bad = world.snapshot();
        bad.world.unlocked_parts = vec![PartUnlock::Voice, PartUnlock::Voice];
        assert_eq!(bad.to_bytes(), Err(CreatureError::DuplicateId));
        let mut bad = world.snapshot();
        bad.world.rng = 0;
        assert!(bad.to_bytes().is_err());
    }

    #[test]
    fn id_cursor_can_wrap_without_reusing_existing_id() {
        let mut world = quiet(Diet::Herbivore);
        world.next_id = u32::MAX;
        assert_eq!(world.id(), u32::MAX);
        let id = world.id();
        assert!(id > world.player.id);
        assert!(!world.creatures.iter().any(|actor| actor.id == id));
        assert!(!world.nests.iter().any(|nest| nest.id == id));
    }

    fn authored_loadout() -> CreatureLoadout {
        CreatureLoadout {
            diet: Diet::Carnivore,
            health: 200.,
            speed: 8.,
            attack_damage: 40.,
            social_power: 2.,
            stamina: 2.,
            defense: 0.5,
            can_glide: false,
        }
    }

    #[test]
    fn authored_health_diet_speed_and_stamina_change_gameplay() {
        let mut normal = quiet(Diet::Herbivore);
        let mut authored = normal.clone();
        authored.player.health = 50.;
        authored.apply_loadout(authored_loadout()).unwrap();
        assert_eq!(authored.player.max_health, 200.);
        assert_eq!(authored.player.health, 100.);
        assert_eq!(authored.player_diet, Diet::Carnivore);
        for _ in 0..60 {
            let input = CreatureInput {
                thrust: 1.,
                sprint: true,
                ..Default::default()
            };
            normal.step(input);
            authored.step(input);
        }
        assert!(authored.player.position.x > normal.player.position.x * 1.3);
        assert!(authored.player.stamina > normal.player.stamina + 0.1);
        authored.snapshot().to_bytes().unwrap();
    }

    #[test]
    fn authored_attack_defense_and_social_power_are_consumed_by_actions() {
        let mut normal = quiet(Diet::Herbivore);
        let id = nearby(&mut normal, 0);
        let mut authored = normal.clone();
        authored.apply_loadout(authored_loadout()).unwrap();
        let attack = CreatureInput {
            attack: true,
            target: Some(id),
            ..Default::default()
        };
        normal.step(attack);
        authored.step(attack);
        assert!(authored.creatures[0].health < normal.creatures[0].health - 15.);
        let mut normal = quiet(Diet::Herbivore);
        let id = nearby(&mut normal, 0);
        let mut authored = normal.clone();
        authored.apply_loadout(authored_loadout()).unwrap();
        let input = CreatureInput {
            social: Some(SocialAction::Sing),
            target: Some(id),
            ..Default::default()
        };
        normal.step(input);
        authored.step(input);
        assert_eq!(normal.species[0].relationship, 20);
        assert_eq!(authored.species[0].relationship, 40);
        assert!(authored.player.stamina > normal.player.stamina);
        let mut normal = quiet(Diet::Herbivore);
        nearby(&mut normal, 0);
        normal.species[0].relationship = -100;
        let mut authored = normal.clone();
        authored.apply_loadout(authored_loadout()).unwrap();
        let original_normal = normal.player.health;
        let original_authored = authored.player.health;
        normal.step(CreatureInput::default());
        authored.step(CreatureInput::default());
        assert_eq!(
            (original_authored - authored.player.health) * 2.,
            original_normal - normal.player.health
        );
    }

    #[test]
    fn authored_v2_save_and_restart_preserve_stats_and_v1_keeps_legacy_defaults() {
        let mut world = quiet(Diet::Herbivore);
        world.apply_loadout(authored_loadout()).unwrap();
        world.advance(
            0.127,
            CreatureInput {
                thrust: 1.,
                ..Default::default()
            },
        );
        let bytes = world.snapshot().to_bytes().unwrap();
        let mut loaded = CreatureSnapshot::from_bytes(&bytes).unwrap().world;
        assert_eq!(loaded, world);
        for _ in 0..150 {
            let input = CreatureInput {
                thrust: 0.5,
                turn: 0.1,
                ..Default::default()
            };
            assert_eq!(loaded.advance(0.021, input), world.advance(0.021, input));
        }
        assert_eq!(loaded, world);
        loaded.restart();
        assert_eq!(loaded.loadout, Some(authored_loadout()));
        assert_eq!(loaded.player.health, 200.);
        assert_eq!(loaded.player_diet, Diet::Carnivore);
        let original = quiet(Diet::Herbivore);
        let bytes = CreatureSnapshot {
            version: 1,
            world: original.clone(),
        }
        .to_bytes()
        .unwrap();
        let mut legacy = CreatureSnapshot::from_bytes(&bytes).unwrap().world;
        assert_eq!(legacy, original);
        assert!(legacy.loadout.is_none());
        let mut current = original;
        for _ in 0..60 {
            let input = CreatureInput {
                thrust: 1.,
                ..Default::default()
            };
            assert_eq!(legacy.step(input), current.step(input));
        }
        assert_eq!(legacy, current);
    }

    #[test]
    fn invalid_authored_stats_and_unsupported_glide_leave_world_unchanged() {
        let mut world = quiet(Diet::Herbivore);
        let before = world.clone();
        for value in [f32::NAN, 0., 1501.] {
            let mut loadout = authored_loadout();
            loadout.health = value;
            assert_eq!(
                world.apply_loadout(loadout),
                Err(CreatureError::InvalidLoadout)
            );
            assert_eq!(world, before);
        }
        let mut glide = authored_loadout();
        glide.can_glide = true;
        assert_eq!(
            world.apply_loadout(glide),
            Err(CreatureError::UnsupportedCapability)
        );
        assert_eq!(world, before);
        world.apply_loadout(authored_loadout()).unwrap();
        let bytes = world.snapshot().to_bytes().unwrap();
        let mut bad = bytes.clone();
        let size = bad.len();
        bad[size - 26..size - 22].copy_from_slice(&f32::NAN.to_le_bytes());
        assert!(CreatureSnapshot::from_bytes(&bad).is_err());
        assert!(CreatureSnapshot::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        let mut snapshot = world.snapshot();
        snapshot.version = 1;
        assert_eq!(snapshot.to_bytes(), Err(CreatureError::InvalidSnapshot));
    }

    #[test]
    fn authored_creation_palette_geometry_save_restart_and_rejection_are_authoritative() {
        use crate::editor::{AuthoredPart, PartArchetype};
        let mut creation = Creation::new(CreationKind::Creature, "Authored walker", 200).unwrap();
        for (id, archetype) in [
            (2, PartArchetype::HerbivoreMouth),
            (3, PartArchetype::Leg),
            (4, PartArchetype::Eye),
        ] {
            creation.parts.push(AuthoredPart::new(
                id,
                format!("Part {id}"),
                archetype,
                Some(1),
            ));
        }
        creation.parts[2].transform.translation = [0., -1., 0.7];
        creation.palette[0] = [0.8, 0.2, 0.6];
        let mut world = quiet(Diet::Carnivore);
        world.apply_creation(creation.clone()).unwrap();
        let bytes = world.snapshot().to_bytes().unwrap();
        let mut loaded = CreatureSnapshot::from_bytes(&bytes).unwrap().world;
        assert_eq!(loaded.creation, Some(creation.clone()));
        loaded.restart();
        assert_eq!(loaded.creation, Some(creation.clone()));
        assert_eq!(loaded.loadout, world.loadout);
        let mut bad = world.snapshot();
        bad.world.loadout.as_mut().unwrap().speed += 1.;
        assert_eq!(bad.to_bytes(), Err(CreatureError::InvalidCreation));
        let mut glider = creation.clone();
        glider
            .parts
            .push(AuthoredPart::new(5, "Wing A", PartArchetype::Wing, Some(1)));
        glider
            .parts
            .push(AuthoredPart::new(6, "Wing B", PartArchetype::Wing, Some(1)));
        let before = world.clone();
        assert_eq!(
            world.apply_creation(glider),
            Err(CreatureError::UnsupportedCapability)
        );
        assert_eq!(world, before);
        let mut corrupt = bytes;
        let start = corrupt
            .windows(8)
            .position(|bytes| bytes == b"RSEDIT01")
            .unwrap();
        corrupt[start - 4..start].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            CreatureSnapshot::from_bytes(&corrupt),
            Err(CreatureError::InvalidCreation)
        );
        world.apply_loadout(authored_loadout()).unwrap();
        assert!(world.creation.is_none());
    }
}
