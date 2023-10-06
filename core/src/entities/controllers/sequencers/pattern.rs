// Copyright (c) 2023 Mike Tsao. All rights reserved.

use super::MidiSequencer;
use crate::{
    midi::{MidiChannel, MidiEvent},
    piano_roll::{Note, Pattern, PatternUid, PianoRoll},
    time::MusicalTime,
    traits::{
        Configurable, ControlEventsFn, Controls, Displays, DisplaysInTimeline, HandlesMidi,
        Sequences, SequencesMidi, Serializable,
    },
    types::ChannelPair,
    uid::Uid,
};
use anyhow::anyhow;
use crossbeam_channel::Sender;
use eframe::{
    egui::{style::WidgetVisuals, Sense},
    emath::RectTransform,
    epaint::{pos2, vec2, Rect, RectShape, Shape},
};
use ensnare_proc_macros::{IsControllerWithTimelineDisplay, Uid};
use serde::{Deserialize, Serialize};
use std::{
    ops::Range,
    sync::{Arc, RwLock},
};

/// A sequencer that works in terms of static copies of [Pattern]s. Recording a
/// [Pattern] and then later changing it won't change what's recorded in this
/// sequencer.
///
/// This makes remove() a little weird. You can't remove a pattern that you've
/// changed, because the sequencer won't recognize that the new pattern was
/// meant to refer to the old pattern.
///
/// This sequencer is nice for certain test cases, but I don't think it's useful
/// in a production environment. [LivePatternSequencer] is better.
#[derive(Debug, Default)]
pub struct PatternSequencer {
    inner: MidiSequencer,
    patterns: Vec<Pattern>,
}
impl Sequences for PatternSequencer {
    type MU = Pattern;

    fn record(
        &mut self,
        channel: MidiChannel,
        pattern: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let pattern = pattern.clone() + position;
        let events: Vec<MidiEvent> = pattern.clone().into();
        events.iter().for_each(|&e| {
            let _ = self.inner.record_midi_event(channel, e);
        });
        self.patterns.push(pattern);
        Ok(())
    }

    fn remove(
        &mut self,
        channel: MidiChannel,
        pattern: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let pattern = pattern.clone() + position;
        let events: Vec<MidiEvent> = pattern.clone().into();
        events.iter().for_each(|&e| {
            let _ = self.inner.remove_midi_event(channel, e);
        });
        self.patterns.retain(|p| p != &pattern);
        Ok(())
    }

    fn clear(&mut self) {
        self.patterns.clear();
        self.inner.clear();
    }
}
impl Controls for PatternSequencer {
    fn update_time(&mut self, range: &Range<MusicalTime>) {
        self.inner.update_time(range)
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        self.inner.work(control_events_fn)
    }

    fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }

    fn play(&mut self) {
        self.inner.play()
    }

    fn stop(&mut self) {
        self.inner.stop()
    }

    fn skip_to_start(&mut self) {
        self.inner.skip_to_start()
    }

    fn is_performing(&self) -> bool {
        self.inner.is_performing()
    }
}
impl Configurable for PatternSequencer {}
impl PatternSequencer {
    fn shape_for_note(to_screen: &RectTransform, visuals: &WidgetVisuals, note: &Note) -> Shape {
        Shape::Rect(RectShape {
            rect: Rect::from_two_pos(
                to_screen * pos2(note.range.start.total_units() as f32, note.key as f32),
                to_screen * pos2(note.range.end.total_units() as f32, note.key as f32),
            ),
            rounding: visuals.rounding,
            fill: visuals.bg_fill,
            stroke: visuals.fg_stroke,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LivePatternArrangement {
    pattern_uid: PatternUid,
    range: Range<MusicalTime>,
}

#[derive(Debug)]
pub enum LivePatternEvent {
    Add(PatternUid, MusicalTime),
}

#[derive(Debug, Default, IsControllerWithTimelineDisplay, Uid, Serialize, Deserialize)]
pub struct LivePatternSequencer {
    uid: Uid,
    arrangements: Vec<LivePatternArrangement>,

    #[serde(skip)]
    inner: PatternSequencer,
    #[serde(skip)]
    piano_roll: Arc<RwLock<PianoRoll>>,
    #[serde(skip)]
    view_range: Range<MusicalTime>,

    #[serde(skip)]
    channel: ChannelPair<LivePatternEvent>,
}
impl Sequences for LivePatternSequencer {
    type MU = PatternUid;

    fn record(
        &mut self,
        channel: MidiChannel,
        pattern_uid: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        let piano_roll = self.piano_roll.read().unwrap();
        if let Some(pattern) = piano_roll.get_pattern(pattern_uid) {
            let _ = self.inner.record(channel, &pattern, position);
            self.arrangements.push(LivePatternArrangement {
                pattern_uid: *pattern_uid,
                range: position..position + pattern.duration(),
            });
            Ok(())
        } else {
            Err(anyhow!("couldn't find pattern {pattern_uid}"))
        }
    }

    fn remove(
        &mut self,
        channel: MidiChannel,
        pattern_uid: &Self::MU,
        position: MusicalTime,
    ) -> anyhow::Result<()> {
        self.arrangements
            .retain(|a| a.pattern_uid != *pattern_uid && a.range.start != position);
        self.inner.clear();
        self.replay();
        Ok(())
    }

    fn clear(&mut self) {
        self.arrangements.clear();
        self.inner.clear();
    }
}
impl Controls for LivePatternSequencer {
    fn update_time(&mut self, range: &Range<MusicalTime>) {
        self.inner.update_time(range)
    }

    fn work(&mut self, control_events_fn: &mut ControlEventsFn) {
        if let Ok(event) = self.channel.receiver.try_recv() {
            match event {
                LivePatternEvent::Add(pattern_uid, position) => {
                    let _ = self.record(MidiChannel(0), &pattern_uid, position);
                }
            }
        }
        self.inner.work(control_events_fn)
    }

    fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }

    fn play(&mut self) {
        self.inner.play()
    }

    fn stop(&mut self) {
        self.inner.stop()
    }

    fn skip_to_start(&mut self) {
        self.inner.skip_to_start()
    }

    fn is_performing(&self) -> bool {
        self.inner.is_performing()
    }
}
impl Serializable for LivePatternSequencer {
    fn after_deser(&mut self) {
        self.replay();
    }
}
impl Configurable for LivePatternSequencer {}
impl HandlesMidi for LivePatternSequencer {}
impl Displays for LivePatternSequencer {
    fn ui(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        ui.allocate_ui(vec2(ui.available_width(), 64.0), |ui| {
            let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
            let x_range_f32 = self.view_range.start.total_units() as f32
                ..=self.view_range.end.total_units() as f32;
            let y_range = i8::MAX as f32..=u8::MIN as f32;
            let local_space_rect = Rect::from_x_y_ranges(x_range_f32, y_range);
            let to_screen = RectTransform::from_to(local_space_rect, response.rect);
            let from_screen = to_screen.inverse();

            // Check whether we edited the sequence
            if response.clicked() {
                if let Some(click_pos) = ui.ctx().pointer_interact_pos() {
                    let local_pos = from_screen * click_pos;
                    let time = MusicalTime::new_with_units(local_pos.x as usize).quantized();
                    let key = local_pos.y as u8;
                    let note = Note::new_with(key, time, MusicalTime::DURATION_QUARTER);
                    eprintln!("Saw a click at {time}, note {note:?}");
                    // self.sequencer.toggle_note(note);
                    // self.sequencer.calculate_events();
                }
            }

            let visuals = if ui.is_enabled() {
                ui.ctx().style().visuals.widgets.active
            } else {
                ui.ctx().style().visuals.widgets.inactive
            };

            // Generate all the note shapes
            // let note_shapes: Vec<Shape> = self
            //     .sequencer
            //     .notes()
            //     .iter()
            //     .map(|note| self.shape_for_note(&to_screen, &visuals, note))
            //     .collect();

            // Generate all the pattern note shapes
            let pattern_shapes: Vec<Shape> =
                self.inner
                    .patterns
                    .iter()
                    .fold(Vec::default(), |mut v, pattern| {
                        pattern.notes().iter().for_each(|note| {
                            let note = Note {
                                key: note.key,
                                range: (note.range.start)..(note.range.end),
                            };
                            v.push(PatternSequencer::shape_for_note(
                                &to_screen, &visuals, &note,
                            ));
                        });
                        v
                    });

            // Paint all the shapes
            //            painter.extend(note_shapes);
            painter.extend(pattern_shapes);

            response
        })
        .inner
    }
}
impl DisplaysInTimeline for LivePatternSequencer {
    fn set_view_range(&mut self, view_range: &Range<MusicalTime>) {
        self.view_range = view_range.clone();
    }
}
impl LivePatternSequencer {
    pub fn new_with(piano_roll: Arc<RwLock<PianoRoll>>) -> Self {
        Self {
            uid: Default::default(),
            arrangements: Default::default(),
            inner: Default::default(),
            piano_roll,
            view_range: Default::default(),
            channel: Default::default(),
        }
    }

    fn replay(&mut self) {
        let piano_roll = self.piano_roll.read().unwrap();
        self.arrangements.iter().for_each(|arrangement| {
            if let Some(pattern) = piano_roll.get_pattern(&arrangement.pattern_uid) {
                let _ = self
                    .inner
                    .record(MidiChannel(0), pattern, arrangement.range.start);
            }
        });
    }

    pub fn pattern_uid_for_position(&self, position: MusicalTime) -> Option<PatternUid> {
        if let Some(arrangement) = self
            .arrangements
            .iter()
            .find(|a| a.range.contains(&position))
        {
            Some(arrangement.pattern_uid)
        } else {
            None
        }
    }

    pub fn sender(&self) -> &Sender<LivePatternEvent> {
        &self.channel.sender
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        midi::MidiNote,
        piano_roll::{Note, PatternBuilder},
        traits::tests::{
            validate_sequences_live_patterns_trait, validate_sequences_patterns_trait,
        },
    };
    use std::sync::{Arc, RwLock};

    #[test]
    fn sequencer_works() {
        let mut s = PatternSequencer::default();

        validate_sequences_patterns_trait(&mut s);
    }

    #[test]
    fn live_sequencer_works() {
        let piano_roll = Arc::new(RwLock::new(PianoRoll::default()));
        let mut s = LivePatternSequencer::new_with(Arc::clone(&piano_roll));

        validate_sequences_live_patterns_trait(piano_roll, &mut s);
    }

    #[test]
    fn live_sequencer_can_find_patterns() {
        let piano_roll = Arc::new(RwLock::new(PianoRoll::default()));
        let pattern_uid = piano_roll.write().unwrap().insert(
            PatternBuilder::default()
                .note(Note::new_with_midi_note(
                    MidiNote::C0,
                    MusicalTime::new_with_beats(0),
                    MusicalTime::DURATION_WHOLE,
                ))
                .note(Note::new_with_midi_note(
                    MidiNote::C0,
                    MusicalTime::new_with_beats(1),
                    MusicalTime::DURATION_WHOLE,
                ))
                .note(Note::new_with_midi_note(
                    MidiNote::C0,
                    MusicalTime::new_with_beats(2),
                    MusicalTime::DURATION_WHOLE,
                ))
                .note(Note::new_with_midi_note(
                    MidiNote::C0,
                    MusicalTime::new_with_beats(3),
                    MusicalTime::DURATION_WHOLE,
                ))
                .build()
                .unwrap(),
        );

        let mut s = LivePatternSequencer::new_with(Arc::clone(&piano_roll));
        let _ = s.record(
            MidiChannel(0),
            &pattern_uid,
            MusicalTime::new_with_beats(20),
        );

        assert!(s.pattern_uid_for_position(MusicalTime::START).is_none());
        assert!(s.pattern_uid_for_position(MusicalTime::TIME_MAX).is_none());
        assert!(s
            .pattern_uid_for_position(MusicalTime::new_with_beats(19))
            .is_none());
        // I manually counted the length of the pattern to figure out that it was four beats long.
        assert!(s
            .pattern_uid_for_position(
                MusicalTime::new_with_beats(20) + MusicalTime::new_with_beats(4)
            )
            .is_none());

        assert!(s
            .pattern_uid_for_position(MusicalTime::new_with_beats(20))
            .is_some());
        assert!(s
            .pattern_uid_for_position(
                MusicalTime::new_with_beats(24) - MusicalTime::new_with_units(1)
            )
            .is_some());

        s.clear();
    }
}
