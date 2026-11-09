//! Explicit synthetic source-actor snapshot handoff. This does not construct native
//! actors, resolve packages, model registration lifetime, or implement gameplay.
#[path = "cell_source_frame_fixture_v1.rs"]
mod frame_fixture;
use bevy::prelude::*;
use spore_core::cell_actor_pool_checked_v1::{CheckedActorPool, PoolError};
use spore_core::cell_source_sweep_pairs_v1::{self as sweep, SweepError, SweepSlot};
use std::sync::atomic::{AtomicU64, Ordering};
use sha2::{Digest, Sha256};

static NEXT_REGISTRY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActorHandle { registry: u64, raw: u32 }

/// These are separately admitted fixture identities, not native pointers.
#[derive(Clone, Debug, PartialEq, Eq)]
struct DefinitionIdentity { package: String, key: [u32; 3], digest: [u8; 32], raw: Vec<u8> }
#[derive(Clone,Debug)]
struct ControlledStagedSource {triple:[u32;3],cache_first:u32,creation_sha:String,rows:Vec<spore_core::cell_part_geometry::NativeCellPart>,geometry_admission:&'static str}
#[derive(Clone, Debug)]
struct ActorSnapshot { staged:Option<ControlledStagedSource>, definition: DefinitionIdentity, health_244: i32, tag: u32, min: [f32; 2], max: [f32; 2], geometry: spore_core::cell_actor_geometry_projection_v1::Output, position_words: [u32; 3] }

#[derive(Debug)]
enum Error { Pool(PoolError), Sweep(SweepError), ForeignOrStaleActor, MissingPlayer }
impl From<PoolError> for Error { fn from(e: PoolError) -> Self { Self::Pool(e) } }
impl From<SweepError> for Error { fn from(e: SweepError) -> Self { Self::Sweep(e) } }

struct Registry { id: u64, pool: CheckedActorPool<ActorSnapshot>, player: Option<ActorHandle> }
impl Registry {
    fn new(capacity: usize, counter: u32) -> Result<Self, Error> {
        let id = NEXT_REGISTRY.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| Error::ForeignOrStaleActor)?;
        Ok(Self { id, pool: CheckedActorPool::new(capacity, counter)?, player: None })
    }
    fn insert(&mut self, actor: ActorSnapshot) -> Result<ActorHandle, Error> {
        Ok(ActorHandle { registry: self.id, raw: self.pool.allocate(actor)? })
    }
    fn resolve(&self, h: ActorHandle) -> Result<&ActorSnapshot, Error> {
        if h.registry != self.id { return Err(Error::ForeignOrStaleActor); }
        self.pool.get(h.raw).ok_or(Error::ForeignOrStaleActor)
    }
    fn set_player(&mut self, h: ActorHandle) -> Result<(), Error> { self.resolve(h)?; self.player = Some(h); Ok(()) }
    /// Explicit player owner and selected actor are independent fixture inputs.
    /// Pool ownership is checked before passing a pool-local raw selection to
    /// the bounded target-copy model; stale same-pool selections may clear.
    fn update_target(&self, player_owner: ActorHandle, selected: &mut Option<ActorHandle>,
        player: &mut spore_core::cell_player_target_bridge_v1::PlayerTarget)
        -> Result<spore_core::cell_player_target_bridge_v1::TargetResult, Error> {
        self.resolve(player_owner)?;
        if self.player != Some(player_owner) { return Err(Error::ForeignOrStaleActor); }
        if selected.is_some_and(|h| h.registry != self.id) { return Err(Error::ForeignOrStaleActor); }
        let mut raw = selected.map_or(0, |h| h.raw);
        let outcome = spore_core::cell_player_target_bridge_v1::update_target(&mut raw, player, &self.pool, |actor| actor.position_words);
        *selected = if raw == 0 { None } else { Some(ActorHandle { registry: self.id, raw }) };
        Ok(outcome)
    }
    fn release(&mut self, h: ActorHandle) -> Result<(), Error> {
        self.resolve(h)?;
        self.pool.release(h.raw)?;
        if self.player == Some(h) { self.player = None; }
        Ok(())
    }
    /// Fresh source-shaped append snapshot; no persisted source collection or
    /// registration lifetime is inferred. Every actor comes from this registry.
    fn update_fresh_registration(&self,capacity:usize)->Result<(Vec<(ActorHandle,u32)>,Vec<(ActorHandle,ActorHandle,Result<bool,spore_core::cell_broadphase_gate_v1::Error>)>,usize,spore_core::cell_fresh_sweep_registration_v1::SortedFresh),String>{
        self.resolve(self.player.ok_or("fixture player not admitted")?).map_err(|e|format!("player: {e:?}"))?;
        let mut collection=spore_core::cell_fresh_sweep_registration_v1::FreshCollection::new(self.pool.live_count()).map_err(|e|format!("collection: {e:?}"))?;
        let mut slots=Vec::new();
        for (raw,actor) in self.pool.iter(){
            let handle=ActorHandle{registry:self.id,raw};self.resolve(handle).map_err(|e|format!("actor: {e:?}"))?;
            let index=collection.append(raw,actor.tag,actor.min.map(f32::to_bits),actor.max.map(f32::to_bits)).map_err(|e|format!("registration: {e:?}"))?;
            slots.push((handle,index));
        }
        // Check the fresh source-shaped rank handoff before the independent
        // bounded sweep; no persisted collection is mutated here.
        let consumer=collection.consume_fresh(capacity).map_err(|e|format!("fresh consumer: {e:?}"))?;
        let pairs=consumer.pairs;
        let mut out=Vec::new();
        for (a,b) in pairs.dispatch_order(){let a=ActorHandle{registry:self.id,raw:a as u32};let b=ActorHandle{registry:self.id,raw:b as u32};
            let first=self.resolve(a).map_err(|e|format!("first: {e:?}"))?;let second=self.resolve(b).map_err(|e|format!("second: {e:?}"))?;
            let gate=spore_core::cell_broadphase_gate_v1::evaluate(first.geometry,second.geometry,spore_core::native_sse::Policy::Mxcsr1f80).map(|g|g.passes);
            out.push((a,b,gate));
        }
        Ok((slots,out,pairs.suppressed,consumer.sorted))
    }

    /// A non-mutating admitted snapshot. Coordinates, tags and definition data
    /// are caller supplied; no persistent registration or 3D contact is modeled.
    fn update(&self, capacity: usize) -> Result<(Vec<(ActorHandle, ActorHandle, Result<bool, spore_core::cell_broadphase_gate_v1::Error>)>, usize), Error> {
        self.resolve(self.player.ok_or(Error::MissingPlayer)?)?;
        let slots: Vec<_> = self.pool.iter().map(|(raw, actor)| SweepSlot {
            id: raw as i32, tag: actor.tag, min: actor.min, max: actor.max,
        }).collect();
        let pairs = sweep::rebuild(&slots, capacity)?;
        let mut dispatch = Vec::new();
        for (a, b) in pairs.dispatch_order() {
            let a = ActorHandle { registry: self.id, raw: a as u32 };
            let b = ActorHandle { registry: self.id, raw: b as u32 };
            let first = self.resolve(a)?; let second = self.resolve(b)?;
            let gate = spore_core::cell_broadphase_gate_v1::evaluate(first.geometry, second.geometry,
                spore_core::native_sse::Policy::Mxcsr1f80).map(|gate| gate.passes);
            dispatch.push((a, b, gate));
        }
        Ok((dispatch, pairs.suppressed))
    }
}

fn fixture_actor(n: u32) -> ActorSnapshot {
    ActorSnapshot { staged:None, definition: DefinitionIdentity { package: "synthetic".into(), key: [n, 0, 0], digest: [0; 32], raw: Vec::new() },
        health_244: 6, tag: 0, min: [n as f32, n as f32], max: [10.0 + n as f32, 20.0 + n as f32],
        geometry: spore_core::cell_actor_geometry_projection_v1::Output { center: [(n as f32).to_bits(), 0, 0], scalar: 1f32.to_bits() }, position_words: [(n as f32).to_bits(), 0, 0] }
}

#[derive(Resource, Default)]
struct FixtureResult(Option<Result<String, String>>);
#[derive(Resource)]
struct FixtureDefinition(Option<DefinitionIdentity>);
fn fixture_update(mut result: ResMut<FixtureResult>, definition: Res<FixtureDefinition>) {
    let run = || -> Result<String, Error> {
        let mut registry = Registry::new(3, 0x7ffe)?;
        let mut actor = fixture_actor(1);
        if let Some(definition) = &definition.0 { actor.definition = definition.clone(); }
        let player = registry.insert(actor)?;
        registry.set_player(player)?;
        let other = registry.insert(fixture_actor(2))?;
        let mut selected = Some(other);
        let mut target = spore_core::cell_player_target_bridge_v1::PlayerTarget { words: [0; 3], byte_04: 1 };
        let target_result = registry.update_target(player, &mut selected, &mut target)?;
        let (pairs, suppressed) = registry.update(4)?;
        // Consume identities through the same checked registry, not prototype IDs.
        for (a, b, _) in &pairs {
            let first = registry.resolve(*a)?; let second = registry.resolve(*b)?;
            let _ = (&first.definition, first.health_244, &second.definition, second.health_244);
        }
        registry.release(other)?;
        Ok(format!("SOURCE_SWEEP_FIXTURE synthetic player={:08X} selected_before_release={selected:?} target={target:?} target_result={target_result:?} dispatch={pairs:?} suppressed={suppressed}; player owner, selected actor and target fields independently admitted; 2D bounds and 3D geometry independently supplied; no native actor, contact, or gameplay claim", player.raw))
    };
    result.0 = Some(run().map_err(|e| match e {
        Error::Pool(p) => format!("pool: {p:?}"), Error::Sweep(s) => format!("sweep: {s:?}"),
        Error::ForeignOrStaleActor => "foreign or stale actor".into(), Error::MissingPlayer => "missing player".into(),
    }));
}

pub fn run_frame(args: &[String]) -> Result<(), String> { frame_fixture::run(args) }
pub fn run_installed_frame(args: &[String]) -> Result<(), String> { frame_fixture::run_installed(args) }

pub fn run(args: &[String]) -> Result<(), String> {
    let definition = if args.is_empty() { None } else {
        if args.len() != 4 || args[0] != "--install" || args[2] != "--cell" {
            return Err("expected --install <SPORE dir> --cell <hex>, or no arguments".into());
        }
        let id = u32::try_from(crate::parse_hex_arg(&args[3], "Cell ID")?).map_err(|_| "Cell ID exceeds u32")?;
        let library = crate::source_cell::SourceCellLibrary::open_install(std::path::Path::new(&args[1]))?;
        let (identity, record) = library.definition_record(id)?;
        spore_core::cell_definition::parse(&record.raw).map_err(|e| e.to_string())?;
        let digest: [u8; 32] = Sha256::digest(&record.raw).into();
        println!("SOURCE_SWEEP_DEFINITION inspection_order package={} key={} sha256={:x} bytes={}; opaque provenance only, no behavior derived; package path must remain trusted and stable; selection is explicit", identity.package.display(), identity.key, Sha256::digest(&record.raw), record.raw.len());
        Some(DefinitionIdentity { package: identity.package.to_string_lossy().into_owned(),
            key: [identity.key.instance_id, identity.key.type_id, identity.key.group_id], digest, raw: record.raw })
    };
    let mut app = App::new();
    app.insert_resource(FixtureDefinition(definition)).init_resource::<FixtureResult>().add_systems(Update, fixture_update);
    app.update();
    let result = app.world_mut().resource_mut::<FixtureResult>().0.take().ok_or("fixture update did not execute")??;
    println!("{result}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_and_generation_are_checked() {
        let mut a = Registry::new(2, 0).unwrap(); let mut b = Registry::new(2, 0).unwrap();
        let old = a.insert(fixture_actor(1)).unwrap(); let foreign = b.insert(fixture_actor(1)).unwrap();
        assert_eq!(old.raw, foreign.raw); assert!(matches!(a.resolve(foreign), Err(Error::ForeignOrStaleActor)));
        a.set_player(old).unwrap(); a.release(old).unwrap();
        assert!(matches!(a.update(2), Err(Error::MissingPlayer)));
        let replacement = a.insert(fixture_actor(2)).unwrap();
        assert_ne!(old.raw, replacement.raw); assert!(matches!(a.resolve(old), Err(Error::ForeignOrStaleActor)));
    }
    #[test]
    fn signed_order_capacity_and_tie_rejection() {
        let mut r = Registry::new(3, 0x7ffe).unwrap();
        let p = r.insert(fixture_actor(1)).unwrap(); r.set_player(p).unwrap();
        let n = r.insert(fixture_actor(2)).unwrap();
        assert!((n.raw as i32) < 0); assert_eq!(r.update(1).unwrap().0, vec![(n,p,Ok(true))]);
        assert_eq!(r.update(0).unwrap().1, 1);
        r.insert(fixture_actor(2)).unwrap();
        assert!(matches!(r.update(4), Err(Error::Sweep(SweepError::EndpointTie))));
    }
    #[test]
    fn headless_application_runs_fixture_update() { run(&[]).unwrap(); }
    #[test]
    fn target_copy_uses_separate_checked_player_and_selected_actor() {
        use spore_core::cell_player_target_bridge_v1::{PlayerTarget, TargetResult};
        let mut r = Registry::new(3, 0).unwrap();
        let player = r.insert(fixture_actor(1)).unwrap(); r.set_player(player).unwrap();
        let mut actor = fixture_actor(2); actor.position_words = [0x8000_0000, 0x7fc0_1234, 42];
        let selected_actor = r.insert(actor).unwrap();
        let mut selected = Some(selected_actor);
        let mut target = PlayerTarget { words: [1, 2, 3], byte_04: 9 };
        assert_eq!(r.update_target(player, &mut selected, &mut target).unwrap(), TargetResult::Copied);
        assert_eq!(selected, Some(selected_actor));
        assert_eq!(target, PlayerTarget { words: [0x8000_0000, 0x7fc0_1234, 42], byte_04: 0 });
        r.release(selected_actor).unwrap();
        let replacement = r.insert(fixture_actor(3)).unwrap(); assert_ne!(replacement.raw, selected_actor.raw);
        let previous = target;
        assert_eq!(r.update_target(player, &mut selected, &mut target).unwrap(), TargetResult::SelectionCleared);
        assert_eq!(selected, None); assert_eq!(target, previous);
        let mut foreign_registry = Registry::new(1, 0).unwrap();
        let foreign = foreign_registry.insert(fixture_actor(4)).unwrap();
        selected = Some(foreign);
        assert!(matches!(r.update_target(player, &mut selected, &mut target), Err(Error::ForeignOrStaleActor)));
        assert_eq!(selected, Some(foreign)); assert_eq!(target, previous);
        assert!(matches!(r.update_target(replacement, &mut selected, &mut target), Err(Error::ForeignOrStaleActor)));
    }
    #[test]
    fn overlapping_2d_bounds_keep_strict_3d_gate_decisions_and_errors() {
        let mut r = Registry::new(2, 0).unwrap();
        let p = r.insert(fixture_actor(1)).unwrap(); r.set_player(p).unwrap();
        let mut other = fixture_actor(2); other.geometry.center[0] = 3f32.to_bits();
        let n = r.insert(other).unwrap();
        assert_eq!(r.update(1).unwrap().0[0].2, Ok(false)); // equal radius sum and distance
        r.pool.get_mut(n.raw).unwrap().geometry.center[0] = 4f32.to_bits();
        assert_eq!(r.update(1).unwrap().0[0].2, Ok(false));
        r.pool.get_mut(n.raw).unwrap().geometry.center[0] = 2f32.to_bits();
        assert_eq!(r.update(1).unwrap().0[0].2, Ok(true));
        r.pool.get_mut(n.raw).unwrap().geometry.center[0] = f32::NAN.to_bits();
        assert!(r.update(1).unwrap().0[0].2.is_err());
    }
    #[test]
    #[ignore = "requires local SPORE installation"]
    fn installed_definition_is_admitted_to_headless_fixture() {
        let install = std::env::var("RUSTSPORE_INSTALL").unwrap_or_else(|_| "D:/Games/SPORE Collection".into());
        run(&["--install".into(), install, "--cell".into(), "CBCD1287".into()]).unwrap();
    }
}
