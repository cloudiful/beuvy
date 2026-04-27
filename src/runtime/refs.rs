use bevy::prelude::*;
use std::sync::OnceLock;

pub type ResolveDeclarativeRef = fn(&mut Commands, Entity, &str);

static REF_RESOLVER: OnceLock<ResolveDeclarativeRef> = OnceLock::new();

pub fn set_ref_resolver(resolver: ResolveDeclarativeRef) {
    let _ = REF_RESOLVER.set(resolver);
}

pub(crate) fn resolve_ref(commands: &mut Commands, entity: Entity, ref_id: &str) {
    if let Some(resolver) = REF_RESOLVER.get() {
        resolver(commands, entity, ref_id);
    }
}
