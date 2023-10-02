// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::prelude::*;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::{ops::Add, vec::Vec};

/// A human-readable description of the parameter being controlled. Not suitable
/// for end-user viewing, but it's good for debugging.
#[derive(Debug, Serialize, Deserialize, Display)]
pub struct ControlName(pub String);

/// A zero-based index of the entity parameter being controlled. The index is
/// specific to the entity type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
pub struct ControlIndex(pub usize);
impl Add<usize> for ControlIndex {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

/// A standardized value range (0..=1.0) for Controls/Controllable traits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ControlValue(pub f64);
#[allow(missing_docs)]
impl ControlValue {
    pub const MIN: Self = Self(0.0);
    pub const MAX: Self = Self(1.0);
}
impl From<Normal> for ControlValue {
    fn from(value: Normal) -> Self {
        Self(value.0)
    }
}
impl From<ControlValue> for Normal {
    fn from(value: ControlValue) -> Self {
        Self::from(value.0)
    }
}
impl From<BipolarNormal> for ControlValue {
    fn from(value: BipolarNormal) -> Self {
        Self(Normal::from(value).into())
    }
}
impl From<ControlValue> for BipolarNormal {
    fn from(value: ControlValue) -> Self {
        Self::from(Normal::from(value))
    }
}
impl From<usize> for ControlValue {
    fn from(value: usize) -> Self {
        Self(value as f64)
    }
}
impl From<ControlValue> for usize {
    fn from(value: ControlValue) -> Self {
        value.0 as usize
    }
}
impl From<u8> for ControlValue {
    fn from(value: u8) -> Self {
        Self(value as f64 / u8::MAX as f64)
    }
}
impl From<ControlValue> for u8 {
    fn from(value: ControlValue) -> Self {
        (value.0 * u8::MAX as f64) as u8
    }
}
impl From<f32> for ControlValue {
    fn from(value: f32) -> Self {
        Self(value as f64)
    }
}
impl From<ControlValue> for f32 {
    fn from(value: ControlValue) -> Self {
        value.0 as f32
    }
}
impl From<f64> for ControlValue {
    fn from(value: f64) -> Self {
        Self(value)
    }
}
impl From<ControlValue> for f64 {
    fn from(value: ControlValue) -> Self {
        value.0
    }
}
impl From<FrequencyHz> for ControlValue {
    fn from(value: FrequencyHz) -> Self {
        FrequencyHz::frequency_to_percent(value.0).into()
    }
}
impl From<ControlValue> for FrequencyHz {
    fn from(value: ControlValue) -> Self {
        Self::percent_to_frequency(Normal::from(value)).into()
    }
}
impl From<bool> for ControlValue {
    fn from(value: bool) -> Self {
        ControlValue(if value { 1.0 } else { 0.0 })
    }
}
impl From<ControlValue> for bool {
    fn from(value: ControlValue) -> Self {
        value.0 != 0.0
    }
}
impl From<Ratio> for ControlValue {
    fn from(value: Ratio) -> Self {
        ControlValue(Normal::from(value).0)
    }
}
impl From<ControlValue> for Ratio {
    fn from(value: ControlValue) -> Self {
        Self::from(Normal::from(value))
    }
}
impl From<Tempo> for ControlValue {
    fn from(value: Tempo) -> Self {
        Self(value.0 / Tempo::MAX_VALUE)
    }
}
impl From<ControlValue> for Tempo {
    fn from(value: ControlValue) -> Self {
        Self(value.0 * Tempo::MAX_VALUE)
    }
}

/// Routes automation control events.
///
/// An [Entity] that produces control events can be linked to one or more
/// surfaces of other entities. An example of an event producer is an LFO that
/// generates an audio signal, and an example of an event consumer is a synth
/// that exposes its low-pass filter cutoff as a controllable parameter. Linking
/// them means that the cutoff should follow the LFO. When the LFO's value
/// changes, the synth receives a notification of the new [ControlValue] and
/// responds by updating its cutoff.
#[derive(Serialize, Deserialize, Debug, Default)]
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
    pub fn control_links(&self, source_uid: Uid) -> Option<&Vec<(Uid, ControlIndex)>> {
        self.uid_to_control.get(&source_uid)
    }

    /// Given a control event consisting of a source [Entity] and a
    /// [ControlValue], routes that event to the control surfaces linked to it.
    pub fn route(
        &mut self,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::factory::test_entities::TestControllable;
    use crate::traits::*;

    #[test]
    fn usize_ok() {
        let a = usize::MAX;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<usize>>::into(cv));

        let a = usize::MIN;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<usize>>::into(cv));
    }

    #[test]
    fn u8_ok() {
        let a = u8::MAX;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<u8>>::into(cv));

        let a = u8::MIN;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<u8>>::into(cv));
    }

    #[test]
    fn f32_ok() {
        let a = f32::MAX;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<f32>>::into(cv));

        let a = f32::MIN;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<f32>>::into(cv));
    }

    #[test]
    fn f64_ok() {
        let a = 1000000.0f64;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<f64>>::into(cv));

        let a = -1000000.0f64;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<f64>>::into(cv));
    }

    #[test]
    fn normal_ok() {
        let a = Normal::maximum();
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<Normal>>::into(cv));

        let a = Normal::minimum();
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<Normal>>::into(cv));
    }

    #[test]
    fn bipolar_normal_ok() {
        let a = BipolarNormal::maximum();
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<BipolarNormal>>::into(cv));

        let a = BipolarNormal::minimum();
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<BipolarNormal>>::into(cv));

        let a = BipolarNormal::zero();
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<BipolarNormal>>::into(cv));
    }

    #[test]
    fn bool_ok() {
        let a = true;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<bool>>::into(cv));

        let a = false;
        let cv: ControlValue = a.into();
        assert_eq!(a, <ControlValue as Into<bool>>::into(cv));
    }

    #[test]
    fn ratio_ok() {
        assert_eq!(Ratio::from(ControlValue(0.0)).value(), 0.125);
        assert_eq!(Ratio::from(ControlValue(0.5)).value(), 1.0);
        assert_eq!(Ratio::from(ControlValue(1.0)).value(), 8.0);

        assert_eq!(ControlValue::from(Ratio::from(0.125)).0, 0.0);
        assert_eq!(ControlValue::from(Ratio::from(1.0)).0, 0.5);
        assert_eq!(ControlValue::from(Ratio::from(8.0)).0, 1.0);

        assert_eq!(Ratio::from(BipolarNormal::from(-1.0)).value(), 0.125);
        assert_eq!(Ratio::from(BipolarNormal::from(0.0)).value(), 1.0);
        assert_eq!(Ratio::from(BipolarNormal::from(1.0)).value(), 8.0);

        assert_eq!(BipolarNormal::from(Ratio::from(0.125)).value(), -1.0);
        assert_eq!(BipolarNormal::from(Ratio::from(1.0)).value(), 0.0);
        assert_eq!(BipolarNormal::from(Ratio::from(8.0)).value(), 1.0);
    }

    #[test]
    fn crud_works() {
        let mut cr = ControlRouter::default();
        assert!(
            cr.uid_to_control.is_empty(),
            "new ControlRouter should be empty"
        );

        let source_1_uid = Uid(1);
        let source_2_uid = Uid(2);
        let target_1_uid = Uid(3);
        let target_2_uid = Uid(4);

        cr.link_control(source_1_uid, target_1_uid, ControlIndex(0));
        assert_eq!(
            cr.uid_to_control.len(),
            1,
            "there should be one vec after inserting one link"
        );
        cr.link_control(source_1_uid, target_2_uid, ControlIndex(1));
        assert_eq!(
            cr.uid_to_control.len(),
            1,
            "there should still be one vec after inserting a second link for same source_uid"
        );
        cr.link_control(source_2_uid, target_1_uid, ControlIndex(0));
        assert_eq!(
            cr.uid_to_control.len(),
            2,
            "there should be two vecs after inserting one link for a second Uid"
        );

        assert_eq!(
            cr.control_links(source_1_uid).unwrap().len(),
            2,
            "the first source's vec should have two entries"
        );
        assert_eq!(
            cr.control_links(source_2_uid).unwrap().len(),
            1,
            "the second source's vec should have one entry"
        );

        let tracker = std::sync::Arc::new(std::sync::RwLock::new(Vec::default()));
        let mut controllable_1 =
            TestControllable::new_with(target_1_uid, std::sync::Arc::clone(&tracker));
        let mut controllable_2 =
            TestControllable::new_with(target_2_uid, std::sync::Arc::clone(&tracker));

        // The closures are wooden and repetitive because we don't have access
        // to EntityStore in this crate, so we hardwired a simple version of it
        // here.
        let _ = cr.route(
            &mut |target_uid, index, value| match *target_uid {
                Uid(3) => {
                    controllable_1.control_set_param_by_index(index, value);
                }
                Uid(4) => {
                    controllable_2.control_set_param_by_index(index, value);
                }
                _ => panic!("Shouldn't have received target_uid {target_uid}"),
            },
            source_1_uid,
            ControlValue(0.5),
        );
        if let Ok(t) = tracker.read() {
            assert_eq!(
                t.len(),
                2,
                "there should be expected number of control events after the route {:#?}",
                t
            );
            assert_eq!(t[0], (target_1_uid, ControlIndex(0), ControlValue(0.5)));
            assert_eq!(t[1], (target_2_uid, ControlIndex(1), ControlValue(0.5)));
        };

        // Try removing links. Start with nonexistent link
        if let Ok(mut t) = tracker.write() {
            t.clear();
        }
        cr.unlink_control(source_1_uid, target_1_uid, ControlIndex(99));
        let _ = cr.route(
            &mut |target_uid, index, value| match *target_uid {
                Uid(3) => {
                    controllable_1.control_set_param_by_index(index, value);
                }
                Uid(4) => {
                    controllable_2.control_set_param_by_index(index, value);
                }
                _ => panic!("Shouldn't have received target_uid {target_uid}"),
            },
            source_1_uid,
            ControlValue(0.5),
        );
        if let Ok(t) = tracker.read() {
            assert_eq!(
                t.len(),
                2,
                "route results shouldn't change when removing nonexistent link {:#?}",
                t
            );
        };

        if let Ok(mut t) = tracker.write() {
            t.clear();
        }
        cr.unlink_control(source_1_uid, target_1_uid, ControlIndex(0));
        let _ = cr.route(
            &mut |target_uid, index, value| match *target_uid {
                Uid(3) => {
                    controllable_1.control_set_param_by_index(index, value);
                }
                Uid(4) => {
                    controllable_2.control_set_param_by_index(index, value);
                }
                _ => panic!("Shouldn't have received target_uid {target_uid}"),
            },
            source_1_uid,
            ControlValue(0.5),
        );
        if let Ok(t) = tracker.read() {
            assert_eq!(
                t.len(),
                1,
                "removing a link should continue routing to remaining ones {:#?}",
                t
            );
            assert_eq!(t[0], (target_2_uid, ControlIndex(1), ControlValue(0.5)));
        };
    }
}
