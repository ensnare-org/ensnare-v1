// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::prelude::*;
use strum_macros::Display;

/// Something that is [Controllable] exposes a set of attributes, each with a
/// text name, that an [IsController] can change. If you're familiar with DAWs,
/// this is typically called automation.
///
/// The [Controllable] trait is more powerful than ordinary getters/setters
/// because it allows runtime binding of an [IsController] to a [Controllable].
#[allow(unused_variables)]
pub trait Controllable {
    // See https://stackoverflow.com/a/71988904/344467 to show that we could
    // have made these functions rather than methods (no self). But then we'd
    // lose the ability to query an object without knowing its struct, which is
    // important for the loose binding that the automation system provides.

    /// The number of controllable parameters.
    fn control_index_count(&self) -> usize {
        unimplemented!()
    }
    /// Given a parameter name, return the corresponding index.
    fn control_index_for_name(&self, name: &str) -> Option<ControlIndex> {
        unimplemented!("Controllable trait methods are implemented by the Control #derive macro")
    }
    /// Given a parameter index, return the corresponding name.
    fn control_name_for_index(&self, index: ControlIndex) -> Option<String> {
        unimplemented!()
    }
    /// Given a parameter name and a new value for it, set that parameter's
    /// value.
    fn control_set_param_by_name(&mut self, name: &str, value: ControlValue) {
        unimplemented!()
    }
    /// Given a parameter index and a new value for it, set that parameter's
    /// value.
    fn control_set_param_by_index(&mut self, index: ControlIndex, value: ControlValue) {
        unimplemented!()
    }
}

/// Passes [WorkEvent]s to the caller. Used in [Controls::work()].
pub type ControlEventsFn<'a> = dyn FnMut(WorkEvent) + 'a;

/// A device that [Controls] produces [EntityEvent]s that control other things.
/// It also has a concept of a performance that has a beginning and an end. It
/// knows how to respond to requests to start, stop, restart, and seek within
/// the performance.
#[allow(unused_variables)]
pub trait Controls: Send {
    /// Returns the current [MusicalTime] range, or [None] if not performing or
    /// not applicable.
    fn time_range(&self) -> Option<TimeRange> {
        None
    }

    /// Sets the range of [MusicalTime] to which the next [Controls::work()]
    /// method applies.
    ///
    /// Because a project performance often groups many audio frames into a
    /// single batch of work for efficiency reasons, the [TimeRange] is not
    /// necessarily the same as the current audio frame being rendered. Instead,
    /// it is a window that covers the current batch of frames.
    fn update_time_range(&mut self, time_range: &TimeRange) {}

    /// Performs work for the time range specified in the previous
    /// [Controls::update_time_range()]. If the work produces any events,
    /// calling [control_events_fn] asks the system to queue them. They might be
    /// handled right away, or later.
    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {}

    /// Indicates whether this entity has completed all its scheduled work.
    ///
    /// The framework ends the performance only when all entities indicate that
    /// they're finished. Thus, an entity should return false only if it knows
    /// that it has more work to do (such as a sequencer that has not yet
    /// reached the end of its arranged sequences). An entity that performs work
    /// only on command, such as a synthesizer, should always return true;
    /// otherwise, the performance would never end.
    fn is_finished(&self) -> bool {
        true
    }

    /// Tells the entity to play its performance from the current location. A
    /// device *must* refresh [Controls::is_finished()] during this method.
    fn play(&mut self) {}

    /// Tells the device to stop playing its performance. It shouldn't change
    /// its cursor location, so that a [Controls::play()] after a
    /// [Controls::stop()] acts like a resume.
    fn stop(&mut self) {}

    /// Resets cursors to the beginning.
    fn skip_to_start(&mut self) {}

    /// Whether the entity is currently playing.
    //
    // TODO: This is part of the trait so that implementers don't have to leak
    // their internal state to unit test code. Consider removing.
    fn is_performing(&self) -> bool {
        false
    }
}

/// A wrapper for identifiers of ControlLink sources. Both entities and paths
/// can generate Control events, so we express them here as variants.
#[derive(Debug, Display, Copy, Clone)]
pub enum ControlLinkSource {
    /// An Entity source.
    Entity(Uid),
    /// A Path source.
    Path(PathUid),
}
impl From<Uid> for ControlLinkSource {
    fn from(uid: Uid) -> Self {
        Self::Entity(uid)
    }
}
impl From<PathUid> for ControlLinkSource {
    fn from(path_uid: PathUid) -> Self {
        Self::Path(path_uid)
    }
}

/// Callback for [ControlsAsProxy::work_as_proxy()].
pub type ControlProxyEventsFn<'a> = dyn FnMut(ControlLinkSource, WorkEvent) + 'a;

/// A version of [Controls] for collections of entities.
#[allow(unused_variables)]
pub trait ControlsAsProxy: Controls {
    /// Allows a collection of entities to do work.
    fn work_as_proxy(&mut self, control_events_fn: &mut ControlProxyEventsFn) {}
}
