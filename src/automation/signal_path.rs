// Copyright (c) 2024 Mike Tsao. All rights reserved.

use crate::{prelude::*, util::Rng};
use core::ops::Range;
use delegate::delegate;
use derive_builder::Builder;
use nonoverlapping_interval_tree::NonOverlappingIntervalTree;
use serde::{Deserialize, Serialize};
use synonym::Synonym;

/// A representation of a single point in a [SignalPath].
#[derive(Clone, Debug, Default, Serialize, Deserialize, Builder, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SignalPoint {
    /// The time at which the signal should have the given value.
    pub(crate) when: MusicalTime,
    /// The value the signal should have at the given time.
    #[builder(default)]
    pub(crate) value: BipolarNormal,
}

/// A representation of a line segment connecting two [SignalPoint]s. In the
/// case of leftmost and rightmost points, there is a virtual extra line
/// segment, level, connecting time zero to the leftmost, and time max to the
/// rightmost.
#[derive(Clone, Debug)]
pub struct SignalStep {
    when: Range<MusicalTime>,
    value: Range<BipolarNormal>,
}
impl SignalStep {
    fn interpolated_value(&self, percent: f64) -> BipolarNormal {
        let value_range = (self.value.end - self.value.start).0;
        let interpolated_value = self.value.start.0 + value_range * percent;
        interpolated_value.into()
    }
}

/// Emits a signal that varies over time. The signal is defined by a set of
/// distinct points, ordered by time. The signal moves linearly from point
/// to point.
#[derive(Debug, Default, Serialize, Deserialize, Builder)]
#[serde(rename_all = "kebab-case")]
#[builder(build_fn(private, name = "build_from_builder"))]
pub struct SignalPath {
    /// Each point in the signal.
    #[builder(default, setter(each(name = "point", into)))]
    pub(crate) points: Vec<SignalPoint>,

    #[builder(setter(skip))]
    #[serde(skip)]
    e: SignalPathEphemerals,
}
#[derive(Debug, Default)]
pub struct SignalPathEphemerals {
    time_range: TimeRange,
    steps: NonOverlappingIntervalTree<MusicalTime, SignalStep>,
    value: Option<BipolarNormal>,
    broadcasted_value: Option<BipolarNormal>,
}
impl SignalPathBuilder {
    /// Builds the item.
    pub fn build(&self) -> Result<SignalPath, SignalPathBuilderError> {
        match self.build_from_builder() {
            Ok(mut s) => {
                Self::verify_point_ordering(&s)?;
                s.after_deser();
                Ok(s)
            }
            Err(e) => Err(e),
        }
    }

    /// Construct a random path. Useful for debugging and prototyping.
    pub fn random(&mut self, rng: &mut Rng) -> &mut Self {
        let mut cursor = MusicalTime::START;
        for _ in 0..8 {
            let point = SignalPoint {
                when: cursor,
                value: BipolarNormal::new(rng.rand_i64() as f64 / i64::MAX as f64),
            };
            self.point(point);
            cursor += MusicalTime::DURATION_QUARTER;
        }
        self
    }

    /// Ensures that all points were added in ascending time order. Consecutive
    /// points with the same time are OK, but they can't go back in time.
    fn verify_point_ordering(s: &SignalPath) -> Result<(), SignalPathBuilderError> {
        let mut last_point_time = MusicalTime::START;
        for p in s.points.iter() {
            if p.when < last_point_time {
                return Err(SignalPathBuilderError::ValidationError(format!(
                    "Point {p:?}'s time field is out of order"
                )));
            }
            last_point_time = p.when;
        }
        Ok(())
    }
}
impl Serializable for SignalPath {
    fn after_deser(&mut self) {
        self.e.steps.clear();
        let mut last_when = MusicalTime::START;
        let mut last_value = None;
        self.points.iter().for_each(|p| {
            let when = last_when..p.when;
            let step = SignalStep {
                when: when.clone(),
                value: last_value.unwrap_or_else(|| p.value)..p.value,
            };
            if when.end != MusicalTime::START {
                self.e.steps.insert(when, step);
            }
            last_when = p.when;
            last_value = Some(p.value);
        });
        if !self.points.is_empty() && last_when != MusicalTime::TIME_MAX {
            let when = last_when..MusicalTime::TIME_MAX;
            let last_value = last_value.unwrap_or_default();
            let step = SignalStep {
                when: when.clone(),
                value: last_value..last_value,
            };
            self.e.steps.insert(when, step);
        }
    }
}
impl Controls for SignalPath {
    fn time_range(&self) -> Option<TimeRange> {
        None
    }

    fn update_time_range(&mut self, time_range: &TimeRange) {
        self.e.time_range = time_range.clone();
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        self.update_value();
        self.broadcast_value(control_events_fn);
    }

    fn is_finished(&self) -> bool {
        true
    }

    fn play(&mut self) {}

    fn stop(&mut self) {}

    fn skip_to_start(&mut self) {}

    fn is_performing(&self) -> bool {
        false
    }
}
impl Configurable for SignalPath {
    fn reset(&mut self) {
        // reset() could mean that we've seeked, so we can't assume that the
        // targets are set to the current value.
        self.e.broadcasted_value = None;
    }
}
impl SignalPath {
    fn update_value(&mut self) {
        if self.e.steps.is_empty() {
            return;
        }
        self.e.value = self.calculate_value(self.e.time_range.start());
    }

    fn calculate_value(&self, when: MusicalTime) -> Option<BipolarNormal> {
        if self.e.steps.is_empty() {
            return None;
        }
        if let Some(current_step) = self.e.steps.get(&when) {
            let step_duration = current_step.when.end - current_step.when.start;
            debug_assert!(
                step_duration.total_units() != 0,
                "Zero-duration steps aren't allowed"
            );
            let step_elapsed = when - current_step.when.start;
            let percent_elapsed =
                step_elapsed.total_units() as f64 / step_duration.total_units() as f64;
            Some(current_step.interpolated_value(percent_elapsed))
        } else {
            None
        }
    }

    fn broadcast_value(&mut self, control_events_fn: &mut ControlEventsFn) {
        if self.e.value != self.e.broadcasted_value {
            if let Some(value) = self.e.value {
                control_events_fn(WorkEvent::Control(value.into()));
            }
            self.e.broadcasted_value = self.e.value;
        }
    }

    pub(crate) fn remove_point(&mut self, point: SignalPoint) {
        self.points.retain(|p| *p != point);
        self.after_deser();
    }

    pub(crate) fn add_point(&mut self, when: MusicalTime) {
        let value = self.calculate_value(when).unwrap_or_default();
        let new_signal_point = SignalPoint { when, value };
        if let Some((next_index, _)) = self
            .points
            .iter()
            .enumerate()
            .find(|(_, point)| point.when >= when)
        {
            self.points.insert(next_index, new_signal_point);
        } else {
            self.points.push(new_signal_point);
        }
        // TODO: if this gets expensive, break it down into point-by-point
        // and call the subroutine from here.
        self.after_deser();
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// A [PathUid] identifies a [SignalPath].
#[derive(Synonym, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PathUid(pub usize);
impl IsUid for PathUid {
    fn as_usize(&self) -> usize {
        self.0
    }
}

/// A factory that generates unique [PathUid]s.
#[derive(Synonym, Debug, Serialize, Deserialize)]
pub struct PathUidFactory(UidFactory<PathUid>);
impl Default for PathUidFactory {
    fn default() -> Self {
        Self(UidFactory::<PathUid>::new(1024))
    }
}
impl PathUidFactory {
    delegate! {
        to self.0 {
            /// Generates the next unique [PathUid].
            pub fn mint_next(&self) -> PathUid;
        }
    }
}
