//! Selected reference effects in original `009A3140`, not its full execution.
//!
//! Source: retained `target/rea/20261027-source-runtime-v4/38-analyze_function.json`;
//! installed base executable SHA-256
//! `17cf32b3e3adbcc7ed9c4e593c83b5d3deb3c675059b7ad9cbfd19192df5f87a`.
//! This projection admits only the non-null supplied-binding branch, after the
//! caller has established that execution reaches it. It does not perform the
//! intervening wrapper work, builder, allocation, resource lookup, or release
//! callbacks. Events are ordered source operations, not actual reference counts.

/// Opaque identity in the corresponding native pointer domain; zero is null.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Identity(pub u32);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InstanceRefs {
    pub resource: Identity, // instance +0
    pub owner: Identity,    // instance +4
    pub binding: Identity,  // instance +8
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    /// Direct increment of resource +114 at 009A315C.
    RetainResource(Identity),
    StoreResource(Identity),
    /// Call 0099CD90 on the replaced resource.
    ReleaseResource(Identity),
    /// Call 00A16FC0 on the incoming owner.
    RetainOwner(Identity),
    StoreOwner(Identity),
    /// Call 009C4B50 on the replaced owner.
    ReleaseOwner(Identity),
    /// Direct increment of binding +30 at 009A339D.
    RetainBinding(Identity),
    StoreBinding(Identity),
    /// Call 0099EDE0 on the replaced binding.
    ReleaseBinding(Identity),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Projection {
    pub refs: InstanceRefs,
    pub events: Vec<Event>,
}

/// Evidence supplied by the caller about the omitted work and callbacks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotSnapshotAdmission {
    Unestablished,
    /// Old fields remain equal to the supplied snapshot until each source
    /// read; refcount writes and omitted callbacks do not alias those fields.
    StableUntilEachSourceRead,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Null instead enters a different path, including a possible builder call.
    NullSuppliedBinding,
    /// The subsequent source path dereferences the resource at 009A31C5.
    NullResource,
    UnestablishedSlotSnapshots,
}

/// Project the selected operations in native order for the supplied-binding
/// path. Inputs must represent valid pointer identities and valid lifetimes;
/// notably this does not establish that arbitrary `u32` values are pointers.
/// The resource/owner operations occur before the omitted constructor work;
/// binding operations occur afterward if that work reaches 009A3396.
/// Source reads old owner after the resource release call, and old binding
/// after all intervening work. A single initial snapshot is sound only when
/// the caller establishes the stability and no-alias condition explicitly.
pub fn project_supplied_binding(
    old: InstanceRefs,
    resource: Identity,
    owner: Identity,
    binding: Identity,
    admission: SlotSnapshotAdmission,
) -> Result<Projection, Error> {
    if admission != SlotSnapshotAdmission::StableUntilEachSourceRead {
        return Err(Error::UnestablishedSlotSnapshots);
    }
    if binding.0 == 0 {
        return Err(Error::NullSuppliedBinding);
    }
    if resource.0 == 0 {
        return Err(Error::NullResource);
    }
    let mut events = Vec::new();
    if resource != old.resource {
        if resource.0 != 0 { events.push(Event::RetainResource(resource)); }
        events.push(Event::StoreResource(resource));
        if old.resource.0 != 0 { events.push(Event::ReleaseResource(old.resource)); }
    }
    if owner != old.owner {
        if owner.0 != 0 { events.push(Event::RetainOwner(owner)); }
        events.push(Event::StoreOwner(owner));
        if old.owner.0 != 0 { events.push(Event::ReleaseOwner(old.owner)); }
    }
    if binding != old.binding {
        events.push(Event::RetainBinding(binding));
        events.push(Event::StoreBinding(binding));
        if old.binding.0 != 0 { events.push(Event::ReleaseBinding(old.binding)); }
    }
    Ok(Projection { refs: InstanceRefs { resource, owner, binding }, events })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacement_retains_before_store_and_release() {
        let old = InstanceRefs { resource: Identity(1), owner: Identity(2), binding: Identity(3) };
        let result = project_supplied_binding(old, Identity(4), Identity(5), Identity(6), SlotSnapshotAdmission::StableUntilEachSourceRead).unwrap();
        assert_eq!(result.events, vec![
            Event::RetainResource(Identity(4)), Event::StoreResource(Identity(4)), Event::ReleaseResource(Identity(1)),
            Event::RetainOwner(Identity(5)), Event::StoreOwner(Identity(5)), Event::ReleaseOwner(Identity(2)),
            Event::RetainBinding(Identity(6)), Event::StoreBinding(Identity(6)), Event::ReleaseBinding(Identity(3)),
        ]);
        assert_eq!(result.refs, InstanceRefs { resource: Identity(4), owner: Identity(5), binding: Identity(6) });
    }

    #[test]
    fn same_pointer_has_no_reference_effect() {
        let old = InstanceRefs { resource: Identity(1), owner: Identity(2), binding: Identity(3) };
        assert!(project_supplied_binding(old, old.resource, old.owner, old.binding, SlotSnapshotAdmission::StableUntilEachSourceRead).unwrap().events.is_empty());
    }

    #[test]
    fn clearing_owner_releases_without_retaining_null() {
        let old = InstanceRefs { resource: Identity(1), owner: Identity(2), binding: Identity(3) };
        let p = project_supplied_binding(old, Identity(1), Identity(0), Identity(3), SlotSnapshotAdmission::StableUntilEachSourceRead).unwrap();
        assert_eq!(p.events, vec![Event::StoreOwner(Identity(0)), Event::ReleaseOwner(Identity(2))]);
    }

    #[test]
    fn null_binding_is_not_treated_as_an_empty_binding() {
        assert_eq!(project_supplied_binding(InstanceRefs::default(), Identity(1), Identity(2), Identity(0), SlotSnapshotAdmission::StableUntilEachSourceRead),
                   Err(Error::NullSuppliedBinding));
        assert_eq!(project_supplied_binding(InstanceRefs::default(), Identity(0), Identity(2), Identity(3), SlotSnapshotAdmission::StableUntilEachSourceRead),
                   Err(Error::NullResource));
    }

    #[test]
    fn snapshot_stability_must_be_established() {
        assert_eq!(project_supplied_binding(InstanceRefs::default(), Identity(1), Identity(2), Identity(3), SlotSnapshotAdmission::Unestablished),
                   Err(Error::UnestablishedSlotSnapshots));
    }
}
