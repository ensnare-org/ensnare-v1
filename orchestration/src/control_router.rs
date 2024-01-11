// Copyright (c) 2023 Mike Tsao. All rights reserved.

use ensnare_core::prelude::*;

/// Routes automation control events.
///
/// An [Entity] that produces control events can be linked to one or more
/// surfaces of other entities. An example of an event producer is an LFO that
/// generates an audio signal, and an example of an event consumer is a synth
/// that exposes its low-pass filter cutoff as a controllable parameter. Linking
/// them means that the cutoff should follow the LFO. When the LFO's value
/// changes, the synth receives a notification of the new [ControlValue] and
/// responds by updating its cutoff.
#[derive(Debug, Default, PartialEq)]
pub struct ControlRouter {
    uid_to_control: std::collections::HashMap<Uid, Vec<(Uid, ControlIndex)>>,
}
impl ControlRouter {
    /// Registers a link between a source [Entity] and a controllable surface on
    /// a target [Entity].
    pub fn link_control(&mut self, source_uid: Uid, target_uid: Uid, control_index: ControlIndex) {
        self.uid_to_control
            .entry(source_uid)
            .or_default()
            .push((target_uid, control_index));
    }

    /// Removes a control link matching the source/target [Uid] and
    /// [ControlIndex].
    pub fn unlink_control(
        &mut self,
        source_uid: Uid,
        target_uid: Uid,
        control_index: ControlIndex,
    ) {
        self.uid_to_control
            .entry(source_uid)
            .or_default()
            .retain(|(uid, index)| !(*uid == target_uid && *index == control_index));
    }

    /// Returns all the control links for a given [Entity].
    pub fn control_links(&self, source_uid: Uid) -> Option<&[(Uid, ControlIndex)]> {
        match self.uid_to_control.get(&source_uid) {
            Some(r) => Some(r),
            None => None,
        }
    }

    /// Given a control event consisting of a source [Entity] and a
    /// [ControlValue], routes that event to the control surfaces linked to it.
    pub fn route(
        &self,
        entity_store_fn: &mut dyn FnMut(&Uid, ControlIndex, ControlValue),
        source_uid: Uid,
        value: ControlValue,
    ) -> anyhow::Result<()> {
        if let Some(control_links) = self.control_links(source_uid) {
            control_links.iter().for_each(|(target_uid, index)| {
                entity_store_fn(target_uid, *index, value);
            });
        }
        Ok(())
    }
}
