//! Authored creations and transactional editing for the original gameplay baseline.
//!
//! Catalog costs, attachment rules and traits below are original prototype tuning.
//! An optional source resource is an opaque display/reference link. Its retail
//! cost, capabilities and attachment metadata remain unknown and are never inferred
//! from the selected prototype archetype. No shipped-part catalog is claimed here.

use crate::cell::Diet;
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

pub const MAX_PARTS: usize = 256;
pub const MAX_SAVE_BYTES: usize = 256 * 1024;
pub const MAX_HISTORY: usize = 128;
pub const DEFAULT_HISTORY: usize = 64;
const MAGIC: &[u8; 8] = b"RSEDIT01";
const NAME_LIMIT: usize = 64;
const MAX_DEPTH: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreationKind {
    Cell,
    Creature,
    Building,
    Vehicle,
    Adventure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PartArchetype {
    Body,
    HerbivoreMouth,
    CarnivoreMouth,
    OmnivoreMouth,
    Eye,
    Flagellum,
    Fin,
    Leg,
    Arm,
    Claw,
    Voice,
    Armor,
    Wing,
    Tail,
    Horn,
    Foundation,
    Wall,
    Roof,
    Door,
    Chassis,
    Wheel,
    Thruster,
    Cockpit,
    Weapon,
    Terrain,
    Platform,
    Goal,
    Prop,
    Spawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentClass {
    Body,
    Mouth,
    Sensor,
    Motion,
    Limb,
    Weapon,
    Social,
    Defense,
    Foundation,
    Wall,
    Roof,
    Access,
    Chassis,
    Drive,
    Cabin,
    Terrain,
    Platform,
    Logic,
    Prop,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PrototypeTraits {
    pub health: f32,
    pub speed: f32,
    pub attack: f32,
    pub social: f32,
    pub stamina: f32,
    pub defense: f32,
}
const fn traits(
    health: f32,
    speed: f32,
    attack: f32,
    social: f32,
    stamina: f32,
    defense: f32,
) -> PrototypeTraits {
    PrototypeTraits {
        health,
        speed,
        attack,
        social,
        stamina,
        defense,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartDefinition {
    pub archetype: PartArchetype,
    pub label: &'static str,
    pub cost: u32,
    pub kinds: &'static [CreationKind],
    pub attachment: AttachmentClass,
    /// An empty slice identifies a required root archetype.
    pub allowed_parents: &'static [AttachmentClass],
    pub traits: PrototypeTraits,
}

use AttachmentClass as A;
use CreationKind as K;
const BIO: &[CreationKind] = &[K::Cell, K::Creature];
const CREATURE: &[CreationKind] = &[K::Creature];
const CELL: &[CreationKind] = &[K::Cell];
const BUILDING: &[CreationKind] = &[K::Building];
const VEHICLE: &[CreationKind] = &[K::Vehicle];
const ADVENTURE: &[CreationKind] = &[K::Adventure];
const ALL_KINDS: &[CreationKind] = &[K::Cell, K::Creature, K::Building, K::Vehicle, K::Adventure];
macro_rules! part {
    ($archetype:ident,$label:literal,$cost:literal,$kinds:expr,$class:ident,$parents:expr,$traits:expr) => {
        PartDefinition {
            archetype: PartArchetype::$archetype,
            label: $label,
            cost: $cost,
            kinds: $kinds,
            attachment: A::$class,
            allowed_parents: $parents,
            traits: $traits,
        }
    };
}

/// Original part definitions. Numeric stats describe this project's prototype only.
pub const PART_CATALOG: &[PartDefinition] = &[
    part!(
        Body,
        "Body",
        10,
        BIO,
        Body,
        &[],
        traits(40., 0., 0., 0., 0., 0.)
    ),
    part!(
        HerbivoreMouth,
        "Leaf mouth",
        15,
        BIO,
        Mouth,
        &[A::Body],
        traits(0., 0., 2., 0.5, 0., 0.)
    ),
    part!(
        CarnivoreMouth,
        "Biting mouth",
        15,
        BIO,
        Mouth,
        &[A::Body],
        traits(0., 0., 8., 0., 0., 0.)
    ),
    part!(
        OmnivoreMouth,
        "Mixed mouth",
        20,
        BIO,
        Mouth,
        &[A::Body],
        traits(0., 0., 5., 0.25, 0., 0.)
    ),
    part!(
        Eye,
        "Eye",
        4,
        BIO,
        Sensor,
        &[A::Body, A::Mouth],
        traits(0., 0., 0., 0.2, 0., 0.)
    ),
    part!(
        Flagellum,
        "Flagellum",
        12,
        CELL,
        Motion,
        &[A::Body],
        traits(0., 1.4, 0., 0., 0.15, 0.)
    ),
    part!(
        Fin,
        "Fin",
        8,
        CELL,
        Motion,
        &[A::Body],
        traits(0., 0.6, 0., 0., 0.1, 0.)
    ),
    part!(
        Leg,
        "Leg",
        12,
        CREATURE,
        Limb,
        &[A::Body],
        traits(0., 1.3, 0., 0., 0.1, 0.)
    ),
    part!(
        Arm,
        "Arm",
        10,
        CREATURE,
        Limb,
        &[A::Body],
        traits(0., 0., 1., 0.5, 0., 0.)
    ),
    part!(
        Claw,
        "Claw",
        12,
        CREATURE,
        Weapon,
        &[A::Limb],
        traits(0., 0., 8., 0., 0., 0.)
    ),
    part!(
        Voice,
        "Voice crest",
        14,
        CREATURE,
        Social,
        &[A::Body],
        traits(0., 0., 0., 4., 0., 0.)
    ),
    part!(
        Armor,
        "Armor",
        15,
        BIO,
        Defense,
        &[A::Body, A::Limb],
        traits(20., 0., 0., 0., 0., 0.08)
    ),
    part!(
        Wing,
        "Gliding wing",
        20,
        CREATURE,
        Limb,
        &[A::Body],
        traits(0., 0.8, 0., 0., 0.15, 0.)
    ),
    part!(
        Tail,
        "Tail",
        8,
        CREATURE,
        Motion,
        &[A::Body],
        traits(0., 0.25, 1., 0., 0.05, 0.)
    ),
    part!(
        Horn,
        "Horn",
        10,
        BIO,
        Weapon,
        &[A::Body],
        traits(0., 0., 6., 0., 0., 0.)
    ),
    part!(
        Foundation,
        "Foundation",
        10,
        BUILDING,
        Foundation,
        &[],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Wall,
        "Wall",
        8,
        BUILDING,
        Wall,
        &[A::Foundation, A::Wall],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Roof,
        "Roof",
        10,
        BUILDING,
        Roof,
        &[A::Wall],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Door,
        "Door",
        5,
        BUILDING,
        Access,
        &[A::Wall],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Chassis,
        "Chassis",
        12,
        VEHICLE,
        Chassis,
        &[],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Wheel,
        "Wheel",
        8,
        VEHICLE,
        Drive,
        &[A::Chassis],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Thruster,
        "Thruster",
        18,
        VEHICLE,
        Drive,
        &[A::Chassis],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Cockpit,
        "Cockpit",
        10,
        VEHICLE,
        Cabin,
        &[A::Chassis],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Weapon,
        "Vehicle weapon",
        15,
        VEHICLE,
        Weapon,
        &[A::Chassis, A::Cabin],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Terrain,
        "Adventure ground",
        10,
        ADVENTURE,
        Terrain,
        &[],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Platform,
        "Platform",
        8,
        ADVENTURE,
        Platform,
        &[A::Terrain, A::Platform],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Goal,
        "Goal marker",
        5,
        ADVENTURE,
        Logic,
        &[A::Terrain, A::Platform],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Prop,
        "Decoration",
        4,
        ALL_KINDS,
        Prop,
        &[
            A::Body,
            A::Foundation,
            A::Wall,
            A::Roof,
            A::Chassis,
            A::Terrain,
            A::Platform
        ],
        traits(0., 0., 0., 0., 0., 0.)
    ),
    part!(
        Spawn,
        "Spawn marker",
        5,
        ADVENTURE,
        Logic,
        &[A::Terrain, A::Platform],
        traits(0., 0., 0., 0., 0., 0.)
    ),
];
impl PartArchetype {
    pub fn definition(self) -> &'static PartDefinition {
        &PART_CATALOG[self as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PartTransform {
    /// Parent-local translation, each component -32..32.
    pub translation: [f32; 3],
    /// Parent-local Euler XYZ radians, each component -TAU..TAU.
    pub rotation: [f32; 3],
    /// Positive parent-local scale, each component 0.1..4.
    pub scale: [f32; 3],
}
impl Default for PartTransform {
    fn default() -> Self {
        Self {
            translation: [0.; 3],
            rotation: [0.; 3],
            scale: [1.; 3],
        }
    }
}
impl PartTransform {
    /// Bilateral reflection across the parent's local XY plane (z changes sign).
    pub fn mirrored(self) -> Self {
        Self {
            translation: [
                self.translation[0],
                self.translation[1],
                -self.translation[2],
            ],
            rotation: [-self.rotation[0], -self.rotation[1], self.rotation[2]],
            scale: self.scale,
        }
    }
    fn validate(self) -> Result<(), EditorError> {
        if self
            .translation
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 32.0)
            || self
                .rotation
                .iter()
                .any(|v| !v.is_finite() || v.abs() > std::f32::consts::TAU)
            || self
                .scale
                .iter()
                .any(|v| !v.is_finite() || !(0.1..=4.0).contains(v))
        {
            return Err(EditorError::InvalidTransform);
        }
        Ok(())
    }
}

/// Opaque resource identity only. Retail cost, traits and attachment rules unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceResource {
    pub type_id: u32,
    pub group_id: u32,
    pub instance_id: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthoredPart {
    pub id: u32,
    pub name: String,
    pub archetype: PartArchetype,
    pub parent: Option<u32>,
    /// Reciprocal pair with equal archetypes and reflected local transforms.
    pub mirror: Option<u32>,
    pub transform: PartTransform,
    pub source: Option<SourceResource>,
}
impl AuthoredPart {
    pub fn new(
        id: u32,
        name: impl Into<String>,
        archetype: PartArchetype,
        parent: Option<u32>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            archetype,
            parent,
            mirror: None,
            transform: PartTransform::default(),
            source: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Creation {
    pub kind: CreationKind,
    pub name: String,
    pub budget: u32,
    pub palette: [[f32; 3]; 3],
    pub parts: Vec<AuthoredPart>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CellLoadout {
    pub diet: Diet,
    pub health: f32,
    pub speed: f32,
    pub bite_damage: f32,
    pub boost_capacity: f32,
    pub defense: f32,
    pub has_eyes: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub struct CreatureLoadout {
    pub diet: Diet,
    pub health: f32,
    pub speed: f32,
    pub attack_damage: f32,
    pub social_power: f32,
    pub stamina: f32,
    pub defense: f32,
    pub can_glide: bool,
}
#[derive(Debug, Clone, PartialEq)]
pub enum GameplayLoadout {
    Cell(CellLoadout),
    Creature(CreatureLoadout),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorError {
    InvalidName,
    DuplicateName,
    DuplicateId,
    InvalidPalette,
    InvalidTransform,
    MissingRoot,
    MultipleRoots,
    InvalidRoot,
    InvalidParent,
    Cycle,
    GraphTooDeep,
    IncompatibleKind,
    IncompatibleAttachment,
    InvalidMirror,
    BudgetExceeded,
    InvalidBudget,
    TooManyParts,
    PartNotFound(u32),
    MissingMouth,
    ConflictingMouths,
    UnsupportedGameplayKind,
    InvalidHistoryLimit,
    RevisionExhausted,
    InvalidSnapshot,
    UnsupportedVersion(u32),
    TruncatedSnapshot,
    TrailingBytes,
    SaveTooLarge,
}
impl fmt::Display for EditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for EditorError {}

impl Creation {
    pub fn new(
        kind: CreationKind,
        name: impl Into<String>,
        budget: u32,
    ) -> Result<Self, EditorError> {
        let archetype = required_root(kind);
        let creation = Self {
            kind,
            name: name.into(),
            budget,
            palette: [[0.16, 0.68, 0.61], [0.87, 0.79, 0.45], [0.16, 0.22, 0.20]],
            parts: vec![AuthoredPart::new(
                1,
                archetype.definition().label,
                archetype,
                None,
            )],
        };
        creation.validate()?;
        Ok(creation)
    }
    pub fn spent(&self) -> u32 {
        self.parts
            .iter()
            .map(|part| part.archetype.definition().cost)
            .sum()
    }
    pub fn part(&self, id: u32) -> Option<&AuthoredPart> {
        self.parts.iter().find(|part| part.id == id)
    }
    pub fn next_part_id(&self) -> Result<u32, EditorError> {
        if self.parts.len() >= MAX_PARTS {
            return Err(EditorError::TooManyParts);
        }
        Ok((1..=MAX_PARTS as u32 + 1)
            .find(|id| self.parts.iter().all(|part| part.id != *id))
            .expect("bounded part count leaves free ID"))
    }
    pub fn validate(&self) -> Result<(), EditorError> {
        name(&self.name)?;
        if self.budget == 0 || self.budget > 100_000 {
            return Err(EditorError::InvalidBudget);
        }
        if self.parts.is_empty() {
            return Err(EditorError::MissingRoot);
        }
        if self.parts.len() > MAX_PARTS {
            return Err(EditorError::TooManyParts);
        }
        palette(self.palette)?;
        let mut ids = HashMap::new();
        let mut names = HashSet::new();
        for (index, part) in self.parts.iter().enumerate() {
            if part.id == 0 || ids.insert(part.id, index).is_some() {
                return Err(EditorError::DuplicateId);
            }
            name(&part.name)?;
            if !names.insert(part.name.to_lowercase()) {
                return Err(EditorError::DuplicateName);
            }
            part.transform.validate()?;
            if !part.archetype.definition().kinds.contains(&self.kind) {
                return Err(EditorError::IncompatibleKind);
            }
        }
        let roots: Vec<_> = self
            .parts
            .iter()
            .filter(|part| part.parent.is_none())
            .collect();
        if roots.is_empty() {
            return Err(EditorError::MissingRoot);
        }
        if roots.len() != 1 {
            return Err(EditorError::MultipleRoots);
        }
        if roots[0].archetype != required_root(self.kind) || roots[0].mirror.is_some() {
            return Err(EditorError::InvalidRoot);
        }
        for part in &self.parts {
            if let Some(parent_id) = part.parent {
                let parent = self.part(parent_id).ok_or(EditorError::InvalidParent)?;
                if !part
                    .archetype
                    .definition()
                    .allowed_parents
                    .contains(&parent.archetype.definition().attachment)
                {
                    return Err(EditorError::IncompatibleAttachment);
                }
            } else if part.archetype != required_root(self.kind) {
                return Err(EditorError::InvalidRoot);
            }
            if let Some(mirror_id) = part.mirror {
                let other = self.part(mirror_id).ok_or(EditorError::InvalidMirror)?;
                if mirror_id == part.id
                    || other.mirror != Some(part.id)
                    || part.archetype != other.archetype
                    || !near_transform(part.transform.mirrored(), other.transform)
                {
                    return Err(EditorError::InvalidMirror);
                }
                let parent = self
                    .part(part.parent.ok_or(EditorError::InvalidMirror)?)
                    .ok_or(EditorError::InvalidParent)?;
                if part.parent != other.parent && parent.mirror != other.parent {
                    return Err(EditorError::InvalidMirror);
                }
            }
        }
        // Check complete ancestry even when a parent's world matrix is cached.
        // This also bounds recursion independently of serialized part ordering.
        for part in &self.parts {
            let mut seen = HashSet::new();
            let mut cursor = Some(part.id);
            let mut depth = 0;
            while let Some(id) = cursor {
                if !seen.insert(id) {
                    return Err(EditorError::Cycle);
                }
                if depth > MAX_DEPTH {
                    return Err(EditorError::GraphTooDeep);
                }
                cursor = self.part(id).ok_or(EditorError::InvalidParent)?.parent;
                depth += 1;
            }
        }
        if self.spent() > self.budget {
            return Err(EditorError::BudgetExceeded);
        }
        self.resolve_matrices(&ids)?;
        Ok(())
    }
    /// Row-major local-to-world matrices, in the same order as `parts`.
    pub fn world_matrices(&self) -> Result<Vec<[[f32; 4]; 4]>, EditorError> {
        self.validate()?;
        let ids = self
            .parts
            .iter()
            .enumerate()
            .map(|(index, part)| (part.id, index))
            .collect();
        self.resolve_matrices(&ids)
    }
    fn resolve_matrices(
        &self,
        ids: &HashMap<u32, usize>,
    ) -> Result<Vec<[[f32; 4]; 4]>, EditorError> {
        let mut states = vec![0; self.parts.len()];
        let mut matrices = vec![[[0.; 4]; 4]; self.parts.len()];
        for index in 0..self.parts.len() {
            resolve(index, self, ids, &mut states, &mut matrices, 0)?;
        }
        Ok(matrices)
    }
    /// Transform and paint do not inflate stats. Counted catalog archetypes and
    /// their prototype costs/traits determine the loadout; source links do not.
    pub fn gameplay_loadout(&self) -> Result<GameplayLoadout, EditorError> {
        self.validate()?;
        if !matches!(self.kind, CreationKind::Cell | CreationKind::Creature) {
            return Err(EditorError::UnsupportedGameplayKind);
        }
        let mut diet = None;
        let mut total = PrototypeTraits::default();
        let mut wings = 0;
        let mut eyes = false;
        for part in &self.parts {
            let part_diet = match part.archetype {
                PartArchetype::HerbivoreMouth => Some(Diet::Herbivore),
                PartArchetype::CarnivoreMouth => Some(Diet::Carnivore),
                PartArchetype::OmnivoreMouth => Some(Diet::Omnivore),
                _ => None,
            };
            if let Some(part_diet) = part_diet {
                if diet.is_some_and(|diet| diet != part_diet) {
                    return Err(EditorError::ConflictingMouths);
                }
                diet = Some(part_diet);
            }
            let values = part.archetype.definition().traits;
            total.health += values.health;
            total.speed += values.speed;
            total.attack += values.attack;
            total.social += values.social;
            total.stamina += values.stamina;
            total.defense += values.defense;
            if part.archetype == PartArchetype::Wing {
                wings += 1;
            }
            if part.archetype == PartArchetype::Eye {
                eyes = true;
            }
        }
        let diet = diet.ok_or(EditorError::MissingMouth)?;
        if self.kind == CreationKind::Cell {
            Ok(GameplayLoadout::Cell(CellLoadout {
                diet,
                health: (20. + total.health).min(1000.),
                speed: (2. + total.speed).min(15.),
                bite_damage: (4. + total.attack).min(200.),
                boost_capacity: (1. + total.stamina).min(5.),
                defense: total.defense.min(0.75),
                has_eyes: eyes,
            }))
        } else {
            Ok(GameplayLoadout::Creature(CreatureLoadout {
                diet,
                health: (60. + total.health).min(1500.),
                speed: (1.5 + total.speed).min(18.),
                attack_damage: (6. + total.attack).min(250.),
                social_power: (1. + total.social).min(100.),
                stamina: (1. + total.stamina).min(5.),
                defense: total.defense.min(0.75),
                can_glide: wings >= 2,
            }))
        }
    }
}

fn required_root(kind: CreationKind) -> PartArchetype {
    match kind {
        K::Cell | K::Creature => PartArchetype::Body,
        K::Building => PartArchetype::Foundation,
        K::Vehicle => PartArchetype::Chassis,
        K::Adventure => PartArchetype::Terrain,
    }
}
fn name(value: &str) -> Result<(), EditorError> {
    if value.is_empty()
        || value.len() > NAME_LIMIT
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        Err(EditorError::InvalidName)
    } else {
        Ok(())
    }
}
fn palette(value: [[f32; 3]; 3]) -> Result<(), EditorError> {
    if value
        .iter()
        .flatten()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
    {
        Err(EditorError::InvalidPalette)
    } else {
        Ok(())
    }
}
fn near_transform(a: PartTransform, b: PartTransform) -> bool {
    a.translation
        .iter()
        .zip(b.translation)
        .chain(a.rotation.iter().zip(b.rotation))
        .chain(a.scale.iter().zip(b.scale))
        .all(|(a, b)| (*a - b).abs() < 1e-4)
}
fn multiply(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.; 4]; 4];
    for row in 0..4 {
        for col in 0..4 {
            for inner in 0..4 {
                out[row][col] += a[row][inner] * b[inner][col];
            }
        }
    }
    out
}
fn local_matrix(transform: PartTransform) -> [[f32; 4]; 4] {
    let [x, y, z] = transform.rotation;
    let (sx, cx) = x.sin_cos();
    let (sy, cy) = y.sin_cos();
    let (sz, cz) = z.sin_cos();
    let rx = [
        [1., 0., 0., 0.],
        [0., cx, -sx, 0.],
        [0., sx, cx, 0.],
        [0., 0., 0., 1.],
    ];
    let ry = [
        [cy, 0., sy, 0.],
        [0., 1., 0., 0.],
        [-sy, 0., cy, 0.],
        [0., 0., 0., 1.],
    ];
    let rz = [
        [cz, -sz, 0., 0.],
        [sz, cz, 0., 0.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ];
    let mut out = multiply(multiply(rx, ry), rz);
    for row in 0..3 {
        for col in 0..3 {
            out[row][col] *= transform.scale[col];
        }
        out[row][3] = transform.translation[row];
    }
    out
}
fn resolve(
    index: usize,
    creation: &Creation,
    ids: &HashMap<u32, usize>,
    states: &mut [u8],
    matrices: &mut [[[f32; 4]; 4]],
    depth: usize,
) -> Result<(), EditorError> {
    if states[index] == 2 {
        return Ok(());
    }
    if states[index] == 1 {
        return Err(EditorError::Cycle);
    }
    if depth > MAX_DEPTH {
        return Err(EditorError::GraphTooDeep);
    }
    states[index] = 1;
    let part = &creation.parts[index];
    let local = local_matrix(part.transform);
    let world = if let Some(parent) = part.parent {
        let parent = *ids.get(&parent).ok_or(EditorError::InvalidParent)?;
        resolve(parent, creation, ids, states, matrices, depth + 1)?;
        multiply(matrices[parent], local)
    } else {
        local
    };
    if world.iter().flatten().any(|value| !value.is_finite())
        || world.iter().take(3).any(|row| row[3].abs() > 128.)
    {
        return Err(EditorError::InvalidTransform);
    }
    for col in 0..3 {
        let scale = (world[0][col].powi(2) + world[1][col].powi(2) + world[2][col].powi(2)).sqrt();
        if !(0.01..=16.).contains(&scale) {
            return Err(EditorError::InvalidTransform);
        }
    }
    // Bounded column lengths alone do not bound the inverse of a sheared basis.
    // Evaluate its inverse/condition in f64 before any GPU normal inversion.
    let b: [[f64; 3]; 3] = std::array::from_fn(|r| std::array::from_fn(|c| world[r][c] as f64));
    let det = b[0][0] * (b[1][1] * b[2][2] - b[1][2] * b[2][1])
        - b[0][1] * (b[1][0] * b[2][2] - b[1][2] * b[2][0])
        + b[0][2] * (b[1][0] * b[2][1] - b[1][1] * b[2][0]);
    if !det.is_finite() || det.abs() < 1e-9 {
        return Err(EditorError::InvalidTransform);
    }
    let mut inverse_squared = 0.0;
    for r in 0..3 {
        for c in 0..3 {
            let rows: Vec<_> = (0..3).filter(|i| *i != r).collect();
            let cols: Vec<_> = (0..3).filter(|i| *i != c).collect();
            let cofactor = b[rows[0]][cols[0]] * b[rows[1]][cols[1]]
                - b[rows[0]][cols[1]] * b[rows[1]][cols[0]];
            inverse_squared += (cofactor / det).powi(2);
        }
    }
    let inverse_norm = inverse_squared.sqrt();
    let basis_norm = b.iter().flatten().map(|v| v * v).sum::<f64>().sqrt();
    if !inverse_norm.is_finite() || inverse_norm > 1000.0 || basis_norm * inverse_norm > 1000.0 {
        return Err(EditorError::InvalidTransform);
    }
    matrices[index] = world;
    states[index] = 2;
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorCommand {
    AddPart(AuthoredPart),
    AddMirrored {
        part: AuthoredPart,
        mirror_id: u32,
        mirror_name: String,
    },
    /// Removes descendants and reciprocal mirror subtrees as one transaction.
    RemoveSubtree(u32),
    /// Also reflects the reciprocal partner's local transform.
    SetTransform {
        id: u32,
        transform: PartTransform,
    },
    /// The partner's transform is reflected from `id`; old pairs are detached.
    SetMirror {
        id: u32,
        partner: Option<u32>,
    },
    /// A paired part reparents its partner to the mirrored parent or same parent.
    Reparent {
        id: u32,
        parent: u32,
    },
    SetPalette([[f32; 3]; 3]),
    RenamePart {
        id: u32,
        name: String,
    },
    RenameCreation(String),
    SetBudget(u32),
    SetSource {
        id: u32,
        source: Option<SourceResource>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorDocument {
    creation: Creation,
    revision: u64,
    past: Vec<Creation>,
    future: Vec<Creation>,
    history_limit: usize,
}
impl EditorDocument {
    pub fn new(
        kind: CreationKind,
        name: impl Into<String>,
        budget: u32,
    ) -> Result<Self, EditorError> {
        Self::from_creation(Creation::new(kind, name, budget)?)
    }
    pub fn from_creation(creation: Creation) -> Result<Self, EditorError> {
        creation.validate()?;
        Ok(Self {
            creation,
            revision: 0,
            past: Vec::new(),
            future: Vec::new(),
            history_limit: DEFAULT_HISTORY,
        })
    }
    pub fn creation(&self) -> &Creation {
        &self.creation
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn undo_len(&self) -> usize {
        self.past.len()
    }
    pub fn redo_len(&self) -> usize {
        self.future.len()
    }
    pub fn history_limit(&self) -> usize {
        self.history_limit
    }
    pub fn set_history_limit(&mut self, limit: usize) -> Result<(), EditorError> {
        if limit == 0 || limit > MAX_HISTORY {
            return Err(EditorError::InvalidHistoryLimit);
        }
        self.history_limit = limit;
        if self.past.len() > limit {
            self.past.drain(..self.past.len() - limit);
        }
        if self.future.len() > limit {
            self.future.drain(..self.future.len() - limit);
        }
        Ok(())
    }
    /// Invalid edits leave the creation, revision and both histories unchanged.
    /// No-op edits return false and preserve the redo branch.
    pub fn transact(&mut self, command: EditorCommand) -> Result<bool, EditorError> {
        let mut candidate = self.creation.clone();
        apply(&mut candidate, command)?;
        candidate.validate()?;
        if candidate == self.creation {
            return Ok(false);
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(EditorError::RevisionExhausted)?;
        self.push_past(self.creation.clone());
        self.future.clear();
        self.creation = candidate;
        self.revision = revision;
        Ok(true)
    }
    pub fn undo(&mut self) -> Result<bool, EditorError> {
        if self.past.is_empty() {
            return Ok(false);
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(EditorError::RevisionExhausted)?;
        let previous = self.past.pop().unwrap();
        self.future
            .push(std::mem::replace(&mut self.creation, previous));
        self.revision = revision;
        Ok(true)
    }
    pub fn redo(&mut self) -> Result<bool, EditorError> {
        if self.future.is_empty() {
            return Ok(false);
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(EditorError::RevisionExhausted)?;
        let next = self.future.pop().unwrap();
        let previous = std::mem::replace(&mut self.creation, next);
        self.push_past(previous);
        self.revision = revision;
        Ok(true)
    }
    fn push_past(&mut self, creation: Creation) {
        self.past.push(creation);
        if self.past.len() > self.history_limit {
            self.past.remove(0);
        }
    }
    /// Save files persist the creation and revision. Undo/redo are session-local.
    pub fn snapshot(&self) -> EditorSnapshot {
        EditorSnapshot {
            version: 1,
            revision: self.revision,
            creation: self.creation.clone(),
        }
    }
    pub fn from_snapshot(snapshot: EditorSnapshot) -> Result<Self, EditorError> {
        if snapshot.version != 1 {
            return Err(EditorError::UnsupportedVersion(snapshot.version));
        }
        let mut document = Self::from_creation(snapshot.creation)?;
        document.revision = snapshot.revision;
        Ok(document)
    }
}

fn index(creation: &Creation, id: u32) -> Result<usize, EditorError> {
    creation
        .parts
        .iter()
        .position(|part| part.id == id)
        .ok_or(EditorError::PartNotFound(id))
}
fn detach_mirror(creation: &mut Creation, id: u32) -> Result<(), EditorError> {
    let at = index(creation, id)?;
    if let Some(partner) = creation.parts[at].mirror {
        let other = index(creation, partner)?;
        creation.parts[other].mirror = None;
    }
    creation.parts[at].mirror = None;
    Ok(())
}
fn apply(creation: &mut Creation, command: EditorCommand) -> Result<(), EditorError> {
    match command {
        EditorCommand::AddPart(part) => {
            if creation.parts.len() >= MAX_PARTS {
                return Err(EditorError::TooManyParts);
            }
            creation.parts.push(part);
        }
        EditorCommand::AddMirrored {
            mut part,
            mirror_id,
            mirror_name,
        } => {
            if creation.parts.len() + 2 > MAX_PARTS {
                return Err(EditorError::TooManyParts);
            }
            if part.mirror.is_some() {
                return Err(EditorError::InvalidMirror);
            }
            let mut other = part.clone();
            other.id = mirror_id;
            other.name = mirror_name;
            other.transform = part.transform.mirrored();
            other.mirror = Some(part.id);
            if let Some(parent) = part.parent {
                other.parent = creation
                    .part(parent)
                    .ok_or(EditorError::InvalidParent)?
                    .mirror
                    .or(Some(parent));
            }
            part.mirror = Some(mirror_id);
            creation.parts.extend([part, other]);
        }
        EditorCommand::RemoveSubtree(id) => {
            if creation.parts[index(creation, id)?].parent.is_none() {
                return Err(EditorError::MissingRoot);
            }
            let mut removed = HashSet::from([id]);
            loop {
                let before = removed.len();
                for part in &creation.parts {
                    if removed.contains(&part.id)
                        || part.parent.is_some_and(|parent| removed.contains(&parent))
                        || part
                            .mirror
                            .is_some_and(|partner| removed.contains(&partner))
                    {
                        removed.insert(part.id);
                    }
                }
                if removed.len() == before {
                    break;
                }
            }
            creation.parts.retain(|part| !removed.contains(&part.id));
        }
        EditorCommand::SetTransform { id, transform } => {
            let at = index(creation, id)?;
            creation.parts[at].transform = transform;
            if let Some(partner) = creation.parts[at].mirror {
                let other = index(creation, partner)?;
                creation.parts[other].transform = transform.mirrored();
            }
        }
        EditorCommand::SetMirror { id, partner } => {
            let at = index(creation, id)?;
            if partner == Some(id) {
                return Err(EditorError::InvalidMirror);
            }
            detach_mirror(creation, id)?;
            if let Some(partner) = partner {
                let other = index(creation, partner)?;
                detach_mirror(creation, partner)?;
                creation.parts[at].mirror = Some(partner);
                creation.parts[other].mirror = Some(id);
                creation.parts[other].transform = creation.parts[at].transform.mirrored();
            }
        }
        EditorCommand::Reparent { id, parent } => {
            let at = index(creation, id)?;
            if creation.parts[at].parent.is_none() {
                return Err(EditorError::InvalidRoot);
            }
            let parent_at = index(creation, parent)?;
            let mirrored_parent = creation.parts[parent_at].mirror.unwrap_or(parent);
            creation.parts[at].parent = Some(parent);
            if let Some(partner) = creation.parts[at].mirror {
                let other = index(creation, partner)?;
                creation.parts[other].parent = Some(mirrored_parent);
            }
        }
        EditorCommand::SetPalette(value) => creation.palette = value,
        EditorCommand::RenamePart { id, name } => {
            let at = index(creation, id)?;
            creation.parts[at].name = name;
        }
        EditorCommand::RenameCreation(name) => creation.name = name,
        EditorCommand::SetBudget(budget) => creation.budget = budget,
        EditorCommand::SetSource { id, source } => {
            let at = index(creation, id)?;
            creation.parts[at].source = source;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorSnapshot {
    pub version: u32,
    pub revision: u64,
    pub creation: Creation,
}
impl EditorSnapshot {
    pub fn to_bytes(&self) -> Result<Vec<u8>, EditorError> {
        if self.version != 1 {
            return Err(EditorError::UnsupportedVersion(self.version));
        }
        self.creation.validate()?;
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        u32_out(&mut out, self.version);
        u64_out(&mut out, self.revision);
        out.push(kind_byte(self.creation.kind));
        string_out(&mut out, &self.creation.name);
        u32_out(&mut out, self.creation.budget);
        for color in self.creation.palette {
            for component in color {
                f32_out(&mut out, component);
            }
        }
        u32_out(&mut out, self.creation.parts.len() as u32);
        for part in &self.creation.parts {
            u32_out(&mut out, part.id);
            string_out(&mut out, &part.name);
            out.push(part.archetype as u8);
            u32_out(&mut out, part.parent.unwrap_or(0));
            u32_out(&mut out, part.mirror.unwrap_or(0));
            for component in part
                .transform
                .translation
                .into_iter()
                .chain(part.transform.rotation)
                .chain(part.transform.scale)
            {
                f32_out(&mut out, component);
            }
            out.push(u8::from(part.source.is_some()));
            if let Some(source) = part.source {
                u32_out(&mut out, source.type_id);
                u32_out(&mut out, source.group_id);
                u64_out(&mut out, source.instance_id);
            }
        }
        if out.len() > MAX_SAVE_BYTES {
            return Err(EditorError::SaveTooLarge);
        }
        Ok(out)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, EditorError> {
        if bytes.len() > MAX_SAVE_BYTES {
            return Err(EditorError::SaveTooLarge);
        }
        let mut read = Reader { bytes, at: 0 };
        if read.take(8)? != MAGIC {
            return Err(EditorError::InvalidSnapshot);
        }
        let version = read.u32()?;
        if version != 1 {
            return Err(EditorError::UnsupportedVersion(version));
        }
        let revision = read.u64()?;
        let kind = match read.byte()? {
            0 => K::Cell,
            1 => K::Creature,
            2 => K::Building,
            3 => K::Vehicle,
            4 => K::Adventure,
            _ => return Err(EditorError::InvalidSnapshot),
        };
        let name = read.string()?;
        let budget = read.u32()?;
        let mut palette = [[0.; 3]; 3];
        for color in &mut palette {
            for component in color {
                *component = read.f32()?;
            }
        }
        let count = read.u32()? as usize;
        if count > MAX_PARTS {
            return Err(EditorError::TooManyParts);
        }
        let mut parts = Vec::with_capacity(count);
        for _ in 0..count {
            let id = read.u32()?;
            let name = read.string()?;
            let archetype = PART_CATALOG
                .get(read.byte()? as usize)
                .ok_or(EditorError::InvalidSnapshot)?
                .archetype;
            let parent = nonzero(read.u32()?);
            let mirror = nonzero(read.u32()?);
            let mut transform = PartTransform::default();
            for component in transform
                .translation
                .iter_mut()
                .chain(&mut transform.rotation)
                .chain(&mut transform.scale)
            {
                *component = read.f32()?;
            }
            let source = match read.byte()? {
                0 => None,
                1 => Some(SourceResource {
                    type_id: read.u32()?,
                    group_id: read.u32()?,
                    instance_id: read.u64()?,
                }),
                _ => return Err(EditorError::InvalidSnapshot),
            };
            parts.push(AuthoredPart {
                id,
                name,
                archetype,
                parent,
                mirror,
                transform,
                source,
            });
        }
        if read.at != bytes.len() {
            return Err(EditorError::TrailingBytes);
        }
        let creation = Creation {
            kind,
            name,
            budget,
            palette,
            parts,
        };
        creation.validate()?;
        Ok(Self {
            version,
            revision,
            creation,
        })
    }
}
fn kind_byte(kind: CreationKind) -> u8 {
    match kind {
        K::Cell => 0,
        K::Creature => 1,
        K::Building => 2,
        K::Vehicle => 3,
        K::Adventure => 4,
    }
}
fn nonzero(value: u32) -> Option<u32> {
    if value == 0 {
        None
    } else {
        Some(value)
    }
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
fn string_out(out: &mut Vec<u8>, value: &str) {
    u32_out(out, value.len() as u32);
    out.extend_from_slice(value.as_bytes());
}
struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], EditorError> {
        let end = self
            .at
            .checked_add(count)
            .ok_or(EditorError::TruncatedSnapshot)?;
        let value = self
            .bytes
            .get(self.at..end)
            .ok_or(EditorError::TruncatedSnapshot)?;
        self.at = end;
        Ok(value)
    }
    fn byte(&mut self) -> Result<u8, EditorError> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, EditorError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, EditorError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, EditorError> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn string(&mut self) -> Result<String, EditorError> {
        let count = self.u32()? as usize;
        if count > NAME_LIMIT {
            return Err(EditorError::InvalidName);
        }
        Ok(std::str::from_utf8(self.take(count)?)
            .map_err(|_| EditorError::InvalidName)?
            .to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn creature() -> EditorDocument {
        EditorDocument::new(K::Creature, "Valley walker", 500).unwrap()
    }
    fn add(document: &mut EditorDocument, id: u32, archetype: PartArchetype, parent: u32) {
        document
            .transact(EditorCommand::AddPart(AuthoredPart::new(
                id,
                format!("Part {id}"),
                archetype,
                Some(parent),
            )))
            .unwrap();
    }
    fn paired(
        document: &mut EditorDocument,
        id: u32,
        other: u32,
        archetype: PartArchetype,
        parent: u32,
    ) {
        let mut part = AuthoredPart::new(id, format!("Left {id}"), archetype, Some(parent));
        part.transform.translation = [0., 0., 1.];
        document
            .transact(EditorCommand::AddMirrored {
                part,
                mirror_id: other,
                mirror_name: format!("Right {other}"),
            })
            .unwrap();
    }

    #[test]
    fn all_creation_kinds_have_required_roots_and_catalog_indices_are_stable() {
        for kind in ALL_KINDS {
            let creation = Creation::new(*kind, "Creation", 100).unwrap();
            assert_eq!(creation.parts[0].archetype, required_root(*kind));
            creation.validate().unwrap();
        }
        for (index, definition) in PART_CATALOG.iter().enumerate() {
            assert_eq!(definition.archetype as usize, index);
            assert_eq!(definition.archetype.definition(), definition);
            assert!(definition.cost > 0);
        }
    }

    #[test]
    fn cell_loadout_comes_from_authored_parts_and_source_link_does_not_invent_traits() {
        let mut document = EditorDocument::new(K::Cell, "Tide swimmer", 200).unwrap();
        add(&mut document, 2, PartArchetype::HerbivoreMouth, 1);
        add(&mut document, 3, PartArchetype::Flagellum, 1);
        add(&mut document, 4, PartArchetype::Eye, 1);
        let initial = document.creation().gameplay_loadout().unwrap();
        let GameplayLoadout::Cell(loadout) = &initial else {
            panic!("wrong kind")
        };
        assert_eq!(loadout.diet, Diet::Herbivore);
        assert_eq!(loadout.health, 60.);
        assert!((loadout.speed - 3.4).abs() < 1e-5);
        assert_eq!(loadout.bite_damage, 6.);
        assert!(loadout.has_eyes);
        assert_eq!(document.creation().spent(), 41);
        document
            .transact(EditorCommand::SetSource {
                id: 3,
                source: Some(SourceResource {
                    type_id: 0x1234,
                    group_id: 7,
                    instance_id: 0xfeed_face_dead_beef,
                }),
            })
            .unwrap();
        document
            .transact(EditorCommand::SetPalette([
                [0.1, 0.2, 0.3],
                [0.4, 0.5, 0.6],
                [0.7, 0.8, 0.9],
            ]))
            .unwrap();
        document
            .transact(EditorCommand::SetTransform {
                id: 3,
                transform: PartTransform {
                    scale: [2.; 3],
                    ..Default::default()
                },
            })
            .unwrap();
        assert_eq!(document.creation().gameplay_loadout().unwrap(), initial);
    }

    #[test]
    fn creature_traits_count_real_mirrored_parts_and_upgrades() {
        let mut document = creature();
        add(&mut document, 2, PartArchetype::CarnivoreMouth, 1);
        paired(&mut document, 3, 4, PartArchetype::Leg, 1);
        add(&mut document, 5, PartArchetype::Claw, 3);
        add(&mut document, 6, PartArchetype::Voice, 1);
        add(&mut document, 7, PartArchetype::Armor, 1);
        paired(&mut document, 8, 9, PartArchetype::Wing, 1);
        let GameplayLoadout::Creature(loadout) = document.creation().gameplay_loadout().unwrap()
        else {
            panic!("wrong kind")
        };
        assert_eq!(loadout.diet, Diet::Carnivore);
        assert_eq!(loadout.health, 120.);
        assert_eq!(loadout.attack_damage, 22.);
        assert_eq!(loadout.social_power, 5.);
        assert!((loadout.speed - 5.7).abs() < 1e-5);
        assert!((loadout.stamina - 1.5).abs() < 1e-5);
        assert_eq!(loadout.defense, 0.08);
        assert!(loadout.can_glide);
        assert_eq!(document.creation().parts.len(), 9);
    }

    #[test]
    fn gameplay_export_requires_a_consistent_mouth_and_supported_kind() {
        let mut document = creature();
        assert_eq!(
            document.creation().gameplay_loadout(),
            Err(EditorError::MissingMouth)
        );
        add(&mut document, 2, PartArchetype::HerbivoreMouth, 1);
        add(&mut document, 3, PartArchetype::CarnivoreMouth, 1);
        document.creation().validate().unwrap();
        assert_eq!(
            document.creation().gameplay_loadout(),
            Err(EditorError::ConflictingMouths)
        );
        assert_eq!(
            Creation::new(K::Building, "Shelter", 100)
                .unwrap()
                .gameplay_loadout(),
            Err(EditorError::UnsupportedGameplayKind)
        );
    }

    #[test]
    fn mirror_transform_edits_are_atomic_and_undoable() {
        let mut document = creature();
        paired(&mut document, 2, 3, PartArchetype::Leg, 1);
        let original = document.creation().clone();
        let transform = PartTransform {
            translation: [1., 2., -2.],
            rotation: [0.2, -0.3, 0.4],
            scale: [1.2, 1.1, 0.8],
        };
        document
            .transact(EditorCommand::SetTransform { id: 3, transform })
            .unwrap();
        assert_eq!(
            document.creation().part(2).unwrap().transform,
            transform.mirrored()
        );
        assert_eq!(document.creation().part(3).unwrap().transform, transform);
        let changed = document.creation().clone();
        document.undo().unwrap();
        assert_eq!(document.creation(), &original);
        document.redo().unwrap();
        assert_eq!(document.creation(), &changed);
        assert_eq!(document.revision(), 4);
    }

    #[test]
    fn subtree_removal_includes_mirrored_descendants_and_undo_restores_entire_graph() {
        let mut document = creature();
        paired(&mut document, 2, 3, PartArchetype::Arm, 1);
        paired(&mut document, 4, 5, PartArchetype::Claw, 2);
        assert_eq!(document.creation().part(4).unwrap().parent, Some(2));
        assert_eq!(document.creation().part(5).unwrap().parent, Some(3));
        let before = document.creation().clone();
        document.transact(EditorCommand::RemoveSubtree(2)).unwrap();
        assert_eq!(document.creation().parts.len(), 1);
        document.undo().unwrap();
        assert_eq!(document.creation(), &before);
        document.redo().unwrap();
        assert_eq!(document.creation().parts.len(), 1);
        let before = document.clone();
        assert_eq!(
            document.transact(EditorCommand::RemoveSubtree(1)),
            Err(EditorError::MissingRoot)
        );
        assert_eq!(document, before);
    }

    #[test]
    fn changing_mirror_pairs_detaches_previous_partners_and_reparent_tracks_parent_pair() {
        let mut document = creature();
        for id in 2..=4 {
            add(&mut document, id, PartArchetype::Leg, 1);
        }
        document
            .transact(EditorCommand::SetMirror {
                id: 2,
                partner: Some(3),
            })
            .unwrap();
        document
            .transact(EditorCommand::SetMirror {
                id: 2,
                partner: Some(4),
            })
            .unwrap();
        assert_eq!(document.creation().part(3).unwrap().mirror, None);
        assert_eq!(document.creation().part(4).unwrap().mirror, Some(2));
        document
            .transact(EditorCommand::SetMirror {
                id: 2,
                partner: None,
            })
            .unwrap();
        assert_eq!(document.creation().part(4).unwrap().mirror, None);
        let mut document = creature();
        paired(&mut document, 2, 3, PartArchetype::Arm, 1);
        paired(&mut document, 4, 5, PartArchetype::Claw, 2);
        document
            .transact(EditorCommand::Reparent { id: 4, parent: 3 })
            .unwrap();
        assert_eq!(document.creation().part(4).unwrap().parent, Some(3));
        assert_eq!(document.creation().part(5).unwrap().parent, Some(2));
    }

    #[test]
    fn invalid_transaction_preserves_document_revision_and_redo_branch() {
        let mut document = creature();
        add(&mut document, 2, PartArchetype::Leg, 1);
        document.undo().unwrap();
        let before = document.clone();
        assert_eq!(
            document.transact(EditorCommand::SetBudget(1)),
            Err(EditorError::BudgetExceeded)
        );
        assert_eq!(document, before);
        assert_eq!(
            document.transact(EditorCommand::AddPart(AuthoredPart::new(
                2,
                "Claw",
                PartArchetype::Claw,
                Some(1)
            ))),
            Err(EditorError::IncompatibleAttachment)
        );
        assert_eq!(document, before);
        assert_eq!(
            document.transact(EditorCommand::RenameCreation(" ".into())),
            Err(EditorError::InvalidName)
        );
        assert_eq!(document, before);
        assert_eq!(
            document.transact(EditorCommand::SetMirror {
                id: 1,
                partner: Some(1)
            }),
            Err(EditorError::InvalidMirror)
        );
        assert_eq!(document, before);
        document.redo().unwrap();
        assert_eq!(document.creation().parts.len(), 2);
    }

    #[test]
    fn undo_redo_history_is_bounded_and_new_edit_clears_future() {
        let mut document = creature();
        document.set_history_limit(2).unwrap();
        for label in ["A", "B", "C"] {
            document
                .transact(EditorCommand::RenameCreation(label.into()))
                .unwrap();
        }
        assert_eq!(document.undo_len(), 2);
        document.undo().unwrap();
        document.undo().unwrap();
        assert_eq!(document.creation().name, "A");
        assert!(!document.undo().unwrap());
        assert_eq!(document.redo_len(), 2);
        document.redo().unwrap();
        assert_eq!(document.creation().name, "B");
        document
            .transact(EditorCommand::RenameCreation("D".into()))
            .unwrap();
        assert_eq!(document.redo_len(), 0);
        assert!(!document.redo().unwrap());
        assert_eq!(document.revision(), 7);
        assert_eq!(
            document.set_history_limit(0),
            Err(EditorError::InvalidHistoryLimit)
        );
        assert_eq!(
            document.set_history_limit(MAX_HISTORY + 1),
            Err(EditorError::InvalidHistoryLimit)
        );
    }

    #[test]
    fn no_op_preserves_redo_and_revision_overflow_is_transactional() {
        let mut document = creature();
        add(&mut document, 2, PartArchetype::Leg, 1);
        document.undo().unwrap();
        let before = document.clone();
        assert!(!document
            .transact(EditorCommand::RenameCreation("Valley walker".into()))
            .unwrap());
        assert_eq!(document, before);
        let mut snapshot = document.snapshot();
        snapshot.revision = u64::MAX;
        let mut exhausted = EditorDocument::from_snapshot(snapshot).unwrap();
        let before = exhausted.clone();
        assert_eq!(
            exhausted.transact(EditorCommand::RenameCreation("Another".into())),
            Err(EditorError::RevisionExhausted)
        );
        assert_eq!(exhausted, before);
    }

    #[test]
    fn names_ids_attachment_kind_and_root_invariants_are_enforced() {
        let base = Creation::new(K::Creature, "Valid", 1000).unwrap();
        let mut bad = base.clone();
        bad.parts
            .push(AuthoredPart::new(1, "Another", PartArchetype::Eye, Some(1)));
        assert_eq!(bad.validate(), Err(EditorError::DuplicateId));
        let mut bad = base.clone();
        bad.parts
            .push(AuthoredPart::new(2, "body", PartArchetype::Eye, Some(1)));
        assert_eq!(bad.validate(), Err(EditorError::DuplicateName));
        let mut bad = base.clone();
        bad.parts[0].parent = Some(2);
        assert_eq!(bad.validate(), Err(EditorError::MissingRoot));
        let mut bad = base.clone();
        bad.parts.push(AuthoredPart::new(
            2,
            "Second body",
            PartArchetype::Body,
            None,
        ));
        assert_eq!(bad.validate(), Err(EditorError::MultipleRoots));
        let mut bad = base.clone();
        bad.parts.push(AuthoredPart::new(
            2,
            "Missing parent",
            PartArchetype::Eye,
            Some(300),
        ));
        assert_eq!(bad.validate(), Err(EditorError::InvalidParent));
        let mut bad = base;
        bad.parts.push(AuthoredPart::new(
            2,
            "Cell motion",
            PartArchetype::Flagellum,
            Some(1),
        ));
        assert_eq!(bad.validate(), Err(EditorError::IncompatibleKind));
    }

    #[test]
    fn cyclic_and_too_deep_graphs_are_rejected_independent_of_part_order() {
        let mut cycle = Creation::new(K::Building, "Loop", 1000).unwrap();
        cycle
            .parts
            .push(AuthoredPart::new(2, "Wall A", PartArchetype::Wall, Some(3)));
        cycle
            .parts
            .push(AuthoredPart::new(3, "Wall B", PartArchetype::Wall, Some(2)));
        assert_eq!(cycle.validate(), Err(EditorError::Cycle));
        let mut deep = Creation::new(K::Building, "Tower", 1000).unwrap();
        for id in 2..=35 {
            deep.parts.push(AuthoredPart::new(
                id,
                format!("Wall {id}"),
                PartArchetype::Wall,
                Some(id - 1),
            ));
        }
        assert_eq!(deep.validate(), Err(EditorError::GraphTooDeep));
        deep.parts.reverse();
        assert_eq!(deep.validate(), Err(EditorError::GraphTooDeep));
    }

    #[test]
    fn malformed_mirror_graphs_are_rejected() {
        let mut document = creature();
        paired(&mut document, 2, 3, PartArchetype::Leg, 1);
        let base = document.creation().clone();
        let mut bad = base.clone();
        bad.parts[1].mirror = Some(900);
        assert_eq!(bad.validate(), Err(EditorError::InvalidMirror));
        let mut bad = base.clone();
        bad.parts[2].mirror = None;
        assert_eq!(bad.validate(), Err(EditorError::InvalidMirror));
        let mut bad = base.clone();
        bad.parts[2].archetype = PartArchetype::Arm;
        assert_eq!(bad.validate(), Err(EditorError::InvalidMirror));
        let mut bad = base;
        bad.parts[2].transform.translation[2] = 9.;
        assert_eq!(bad.validate(), Err(EditorError::InvalidMirror));
    }

    #[test]
    fn palettes_and_local_or_composed_transforms_have_real_bounds() {
        let mut document = creature();
        add(&mut document, 2, PartArchetype::Eye, 1);
        for invalid in [f32::NAN, -0.1, 1.1] {
            let before = document.clone();
            let mut colors = document.creation().palette;
            colors[0][0] = invalid;
            assert_eq!(
                document.transact(EditorCommand::SetPalette(colors)),
                Err(EditorError::InvalidPalette)
            );
            assert_eq!(document, before);
        }
        for transform in [
            PartTransform {
                translation: [33., 0., 0.],
                ..Default::default()
            },
            PartTransform {
                rotation: [f32::INFINITY, 0., 0.],
                ..Default::default()
            },
            PartTransform {
                scale: [0., 1., 1.],
                ..Default::default()
            },
            PartTransform {
                scale: [4.1, 1., 1.],
                ..Default::default()
            },
        ] {
            let before = document.clone();
            assert_eq!(
                document.transact(EditorCommand::SetTransform { id: 2, transform }),
                Err(EditorError::InvalidTransform)
            );
            assert_eq!(document, before);
        }
        let mut building = EditorDocument::new(K::Building, "Scale tower", 1000).unwrap();
        add(&mut building, 2, PartArchetype::Wall, 1);
        add(&mut building, 3, PartArchetype::Wall, 2);
        building
            .transact(EditorCommand::SetTransform {
                id: 1,
                transform: PartTransform {
                    scale: [4.; 3],
                    ..Default::default()
                },
            })
            .unwrap();
        building
            .transact(EditorCommand::SetTransform {
                id: 2,
                transform: PartTransform {
                    scale: [4.; 3],
                    ..Default::default()
                },
            })
            .unwrap();
        let before = building.clone();
        assert_eq!(
            building.transact(EditorCommand::SetTransform {
                id: 3,
                transform: PartTransform {
                    scale: [4.; 3],
                    ..Default::default()
                }
            }),
            Err(EditorError::InvalidTransform)
        );
        assert_eq!(building, before);
    }

    #[test]
    fn world_matrices_apply_parent_scale_rotation_and_translation() {
        let mut document = creature();
        add(&mut document, 2, PartArchetype::Eye, 1);
        document
            .transact(EditorCommand::SetTransform {
                id: 1,
                transform: PartTransform {
                    translation: [1., 2., 3.],
                    rotation: [0., 0., std::f32::consts::FRAC_PI_2],
                    scale: [2.; 3],
                },
            })
            .unwrap();
        document
            .transact(EditorCommand::SetTransform {
                id: 2,
                transform: PartTransform {
                    translation: [1., 0., 0.],
                    ..Default::default()
                },
            })
            .unwrap();
        let matrices = document.creation().world_matrices().unwrap();
        assert!((matrices[1][0][3] - 1.).abs() < 1e-5);
        assert!((matrices[1][1][3] - 4.).abs() < 1e-5);
        assert!((matrices[1][2][3] - 3.).abs() < 1e-5);
    }

    #[test]
    fn nonuniform_parent_rotation_preserves_valid_shear_but_rejects_ill_conditioned_basis() {
        let mut document = creature();
        add(&mut document, 2, PartArchetype::Eye, 1);
        document
            .transact(EditorCommand::SetTransform {
                id: 1,
                transform: PartTransform {
                    scale: [2., 1., 1.],
                    ..Default::default()
                },
            })
            .unwrap();
        document
            .transact(EditorCommand::SetTransform {
                id: 2,
                transform: PartTransform {
                    rotation: [0., 0., std::f32::consts::FRAC_PI_4],
                    ..Default::default()
                },
            })
            .unwrap();
        let world = document.creation().world_matrices().unwrap()[1];
        let dot = world[0][0] * world[0][1] + world[1][0] * world[1][1] + world[2][0] * world[2][1];
        assert!(
            (dot + 1.5).abs() < 1e-5,
            "valid shear must not be orthogonalized"
        );
        let mut bad = Creation::new(K::Building, "Condition test", 1000).unwrap();
        bad.parts[0].transform.scale = [4., 0.1, 1.];
        for id in 2..=6 {
            let mut wall =
                AuthoredPart::new(id, format!("Wall {id}"), PartArchetype::Wall, Some(id - 1));
            wall.transform.rotation[2] = std::f32::consts::FRAC_PI_4;
            wall.transform.scale = [1., 0.1, 1.];
            bad.parts.push(wall);
        }
        assert_eq!(bad.validate(), Err(EditorError::InvalidTransform));
    }

    #[test]
    fn versioned_save_roundtrips_creation_revision_and_future_edits_without_session_history() {
        let mut document = creature();
        add(&mut document, 2, PartArchetype::OmnivoreMouth, 1);
        paired(&mut document, 3, 4, PartArchetype::Leg, 1);
        document
            .transact(EditorCommand::SetSource {
                id: 2,
                source: Some(SourceResource {
                    type_id: 1,
                    group_id: 2,
                    instance_id: u64::MAX,
                }),
            })
            .unwrap();
        let bytes = document.snapshot().to_bytes().unwrap();
        let snapshot = EditorSnapshot::from_bytes(&bytes).unwrap();
        assert_eq!(snapshot, document.snapshot());
        assert_eq!(snapshot.to_bytes().unwrap(), bytes);
        let mut loaded = EditorDocument::from_snapshot(snapshot).unwrap();
        assert_eq!(loaded.creation(), document.creation());
        assert_eq!(loaded.revision(), document.revision());
        assert_eq!(loaded.undo_len(), 0);
        let command = EditorCommand::SetTransform {
            id: 3,
            transform: PartTransform {
                translation: [2., 1., 1.5],
                ..Default::default()
            },
        };
        document.transact(command.clone()).unwrap();
        loaded.transact(command).unwrap();
        assert_eq!(loaded.snapshot(), document.snapshot());
    }

    #[test]
    fn malformed_saves_reject_truncation_trailing_oversize_version_counts_and_utf8() {
        let bytes = creature().snapshot().to_bytes().unwrap();
        for length in 0..bytes.len() {
            assert!(
                EditorSnapshot::from_bytes(&bytes[..length]).is_err(),
                "accepted prefix {length}"
            );
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert_eq!(
            EditorSnapshot::from_bytes(&bad),
            Err(EditorError::TrailingBytes)
        );
        let mut bad = bytes.clone();
        bad[8..12].copy_from_slice(&2u32.to_le_bytes());
        assert_eq!(
            EditorSnapshot::from_bytes(&bad),
            Err(EditorError::UnsupportedVersion(2))
        );
        let mut bad = bytes.clone();
        bad[21..25].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            EditorSnapshot::from_bytes(&bad),
            Err(EditorError::InvalidName)
        );
        let mut bad = bytes.clone();
        bad[25] = 255;
        assert_eq!(
            EditorSnapshot::from_bytes(&bad),
            Err(EditorError::InvalidName)
        );
        let mut bad = bytes.clone();
        let count_at = 25 + "Valley walker".len() + 4 + 36;
        bad[count_at..count_at + 4].copy_from_slice(&257u32.to_le_bytes());
        assert_eq!(
            EditorSnapshot::from_bytes(&bad),
            Err(EditorError::TooManyParts)
        );
        let mut bad = bytes.clone();
        bad[20] = 255;
        assert_eq!(
            EditorSnapshot::from_bytes(&bad),
            Err(EditorError::InvalidSnapshot)
        );
        assert_eq!(
            EditorSnapshot::from_bytes(&vec![0; MAX_SAVE_BYTES + 1]),
            Err(EditorError::SaveTooLarge)
        );
    }

    #[test]
    fn invalid_owned_saves_cannot_bypass_validation_or_budget() {
        let document = creature();
        let mut bad = document.snapshot();
        bad.creation.budget = 1;
        assert_eq!(bad.to_bytes(), Err(EditorError::BudgetExceeded));
        assert_eq!(
            EditorDocument::from_snapshot(bad),
            Err(EditorError::BudgetExceeded)
        );
        let mut bad = document.snapshot();
        bad.creation.palette[0][0] = f32::NAN;
        assert_eq!(bad.to_bytes(), Err(EditorError::InvalidPalette));
        let mut bad = document.snapshot();
        bad.version = 8;
        assert_eq!(bad.to_bytes(), Err(EditorError::UnsupportedVersion(8)));
        let mut bad = document.snapshot();
        bad.creation.parts[0].transform.scale[0] = -1.;
        assert_eq!(bad.to_bytes(), Err(EditorError::InvalidTransform));
    }
}
