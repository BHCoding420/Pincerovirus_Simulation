//! [`Validator`] trait for testing.

use crate::simulation::PersonId;

/// [`Validator`] trait for testing.
///
/// We use this trait to gain insights into your program and automatically validate
/// some requirements of the project.
///
/// For the purpose of this trait, patches are enumerated left-to-right and
/// top-to-bottom, i.e., the top-left patch has the id `0`, it's right neighbor
/// has the id `1`, and so on.
pub trait Validator: 'static + Send + Sync {
    /// Call this method before processing a tick on a patch.
    ///
    /// - `tick`: The tick that is about to be processed on the given patch.
    /// - `patch_id`: The id of the patch the tick is processed on.
    fn on_patch_tick(&self, tick: usize, patch_id: usize) {
        let _ = (tick, patch_id);
    }

    /// Call this method before calling `tick` on a person.
    ///
    /// - `tick`: The tick that is about to be processed on the given person.
    /// - `patch_id`: The id of the patch the tick is processed on.
    /// - `person_id`: The id of the person the tick is processed on.
    fn on_person_tick(&self, tick: usize, patch_id: usize, person_id: PersonId) {
        let _ = (tick, patch_id, person_id);
    }
}

/// A dummy validator that does nothing.
pub struct DummyValidator;

impl Validator for DummyValidator {}
