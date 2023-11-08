# Good practices

- Aim for components to have just the required eframe/egui imports, and for the
  bulk of the functionality to live in widgets. I've changed the Displays trait
  to qualify rather than use, so the ideal component won't import anything.
- Where possible, parameters should be normalized, and then scaled to the needed
  range at the last possible moment. A great example is envelope
  attack/decay/release times. Don't store as seconds; store as a Normal, and
  then make time range a late-binding scalar.
  <https://github.com/sowbug/groove/issues/130>
- For any given physical parameter, make sure it's correctly represented as
  linear or logarithmic. <https://github.com/sowbug/groove/issues/44>

# Ideas

- Get in the habit of defining traits for major functional components, like
  Orchestrator. This leads to naturally thinking in terms of contracts, testable
  behavior, etc. It also makes it easier to swap in experiments or outright
  overhauls.

# Widgets, Displays, DisplaysInTimeline

I've noticed my egui work tends to sprawl all over the place, and it's
inconsistent. I'm making good headway in turning reusable things into widgets,
but I think I've gone too far, and things that didn't need to be widgets have
become them. So here are some guidelines.

1. If something can be used by more than one owner, then it's probably a Widget.
   This guideline focuses on how tightly bound a thing is to an Entity. If you
   always need a certain Entity to construct the Widget, and the Widget's main
   purpose is to render the Entity, then it's probably better for it to be a
   Displays::ui() implementation instead.
2. Another way to put rule #1: if you made the Widget private to the module
   containing the struct that it draws, would anyone notice?
3. I'm undecided whether single-consumer widgets (as in #2) should be in the
   widgets module or in the consumer's file. In the commit adding this text to
   the notes doc, I show why having that code in widgets pulls in needless
   dependencies to that module.

# The Sequencer and ControlAtlas integration problem

Sequencer replays MIDI events. ControlAtlas replays control events. They feel
like modular components, but they are also an essential part of the system. They
both render differently from other components. I'm going back and forth between
letting them be generic IsControllers and baking concrete instances of them into
Track or Orchestrator.

Generic pros: they fit nicely; they work.

Generic cons: it's not reasonable to code for cases where they aren't there;
they render differently; the problem's only going to get worse as we integrate
them more into the UI. "They work" is not entirely true; right now I can't
figure out how to draw ControlAtlas because it needs a temporary reference to
ControlRouter to do its job.

Specific pros: Easy.

Specific cons: I have written something like 5 sequencers, and they've all
sucked. So when I commit to a specific one, I worry that I'll be doing a lot of
surgery to replace it.

The solution might be Sequencer and ControlAtlas traits. To instantiate a Track,
we need to provide impls of each. The Track can be coded for the specific
interfaces. I can swap in/out new ones as long as they implement the traits. And
Track and Orchestrator can more easily provide shims that delegate to the real
ones, so they effectively inherit the functionality (at the cost of a bunch of
boilerplate).

Still concerned about the ControlAtlas/ControlRouter problem, but maybe that's
actually another trait. Or maybe they shouldn't be separate at all.

# Widgets Volume 2: How widgets can respond when the user does something

Adapted from
<https://github.com/ensnare-org/ensnare/commit/95db90375b6d31707dc92658aa6f046cb6716b4e>

Option #1: Just do it. For example, you're a slider. You mutate the variable and
return Response::changed(). End of story.

Option #2: The widget acts on a particular struct. The struct should implement
Acts. The widget calls Acts::set_action(). There might be two widgets that
operate on a single struct (e.g., TrackWidget and SignalChainWidget), and in
that case they'll share the same Action enum; in other words, the struct owns
the Action type, not the widget.

Option #3: The widget requires a &mut specifically to pass back the action. You
could think of this as a special case of Option #2 except the struct contains
only the Action.

These are all similar approaches, in a sense. The widget needs something to act
on, which might be the actual action, or else just a marker that the action
needs to happen.

# Uids

I'm not sure that Entities need to know their own Uids. Entities do not appear
to use Uids for anything, and the things that do use them (ControlAtlas,
ControlRouter, EntityStore) store their own copies. In other words, Uids
establish relationships, but Entities do not care about relationships, so
Entities probably do not need to care about Uids, either.

# ControlAtlas

ControlAtlas might turn out to be a bad idea, or perhaps not sufficiently
designed. Its value is that it contains a Track's ControlTrips, and that it
proxies for them when Orchestrator asks it to do work. So it's basically a list.
But it cannot render itself (a.k.a. render the ControlTrips it contains) without
asking ControlRouter for help (since ControlRouter knows the relationships), and
I haven't figured out a non-clunky way to let them share that knowledge.

Another path:

- EntityStore stores ControlTrips directly. This simply moves storage rather
    than adding or removing, no net change.
- Orchestrator treats ControlTrips like any other IsController (except for
    drawing, because they draw into the timeline). A tiny efficiency loss in a
    Vec of ControlTrips becoming a set of hash lookups, but that's
    insignificant.
- SignalChainWidget needs to be taught not to ask DisplaysInTimeline entities to
    draw. It currently checks to see what's in track.timeline_entities, but
    ControlTrips are not full timeline entities (ugh, this is getting brittle).
- ControlAtlas's drawing code becomes a ControlRouter widget. This solves the
    problem of ControlAtlas needing access to ControlRouter during drawing. (The
    widget would need access to EntityStore, though.)
- After writing this out, I don't think this improves anything.

Yet another approach:

- Keep ControlAtlas, but Track is in charge of drawing it. So it creates the
    widget, and because it has access to ControlRouter, it can gracefully share
    access. This undoes some of the work I did to genericize
    DisplaysWithTimeline entities, but I haven't really seen any benefits from
    that, so maybe it wasn't actually valuable.

I could also unify ControlAtlas and ControlRouter.

# Drag and drop

I can see that DnD is going to be a problem. My current code is based on the
egui DnD sample code. I have to manually instrument every drag source and drop
target, and because not everyone knows enough about themselves (e.g., entity
uids and absolute parameter indexes for control links), there is a lot of
passing of nearly identical messages up and down the rendering call stack.

The ideal DnD solution:

  1. For every source, let me declare a tag that uniquely identifies it.
  2. For every target, same -- generate a tag.
  3. A central area gets a notification when a source is dropped on a target,
     and it kicks off whatever needs to happen.

Usage examples:

- Automation: an Entity generates a signal that should be linked to another
    Entity's indexed parameter. source_uid, target_uid, index.
- Instruments: add a new Entity of this type to this Track at this position.
    EntityKey, TrackUid, index.
- Instruments: move this entity to this new Track position. EntityUid, TrackUid,
    index.

Problems:

- I need to come up with a way for something like a DCA widget to create its
    drop-target description without breaking encapsulation, and without the
    caller needing to know everything about the DCA widget (and its potential
    children). I think this means that the widget gets passed a Uid and a
    parameter index base, and it assembles its drop-target description from
    that.

Observations:

- I wish that egui did this for us.

# egui and the ui() method

With the commit that added this comment, I carried out an egui pattern that is
starting to emerge. There are some entities that shouldn't have a Displays
trait. It's possible that these things are actually not Entities. An example is
ControlAtlas, which at this instant is owned by Track, which puts a fake Uid
into the timeline_entities list in an attempt to make it act a little like an
Entity. The atlas widget needs some extra arguments to draw itself, so it's
inappropriate for Displays, which means that ControlAtlas (the 1:1 parent of
that widget) shouldn't have Displays, either. Rather, TrackWidget handles it
specially.

The rule or pattern is something like this: if an Entity's widget requires
special arguments, then it isn't a generic Entity that's asked to draw itself
through Displays, and we know that the only time it'll be asked to display
itself is when custom code that knows that concrete Entity is doing the asking.
So that thing isn't an Entity by the current definition of that thing, and it
might make sense to either change Entity's definition or make these special
Entities' Displays implementation explode.

# Composition and serialization

My very first design for this project had the core instrument functionality
isolated and very streamlined, and then I layered the Entity concept and serde
serialization on top of it in other places. I moved away from that because it
was a lot of work, and I was getting a YAGNI feeling that I was optimizing for a
need that would never really arise.

I am now thinking that the original instinct was correct.

**Entity**: I am pretty sure I could compose Entities like this:

```rust
struct MyInstrument {
  ...
}

#[derive(Metadata)]
struct MyInstrumentEntity {
  uid: Uid,
  inner: MyInstrument
}
```

This is what I'm trying now with PianoRoll, which doesn't need to be an Entity
in the real world, but which is easier to manage in the Entity GUI Explorer
example. So far it doesn't seem to be any more work. It feels cleaner because
the core functionality once again has very little boilerplate.

**Serialization**: the reason for separating this out is different (though it
was also the reason I originally did it separately). It's becoming very clear
that the disk serialization format will be a real nightmare to maintain if it's
just a #[derive(Serialize, Deserialize)] on top of the core struct. I'm not
paying that price now because I don't yet care about it, but I can see it's
going to be awful, not just because things move around a lot, but also because
the serialized structs aren't even the right way to serialize the project.

I don't think I need to do a major all-at-once refactor right now, but I do
think it's the way forward, incrementally.

# 2023-11-01: more on composition/serialization

(I've decided this document is more like a journal than topically organized
thoughts, so I'm adding a date to headers from now on.)

After further reflection on composition/serialization, I'm thinking it might be
more nuanced.

I can't get rid of #[derive(Serialize, Deserialize)] on the core struct because
I'll need it if I compose anything out of it. I could use Serde's [remote
derivation](https://serde.rs/remote-derive.html) facility, but that crosses the
YAGNI line for me.

For the Entity side of things, the proposal to wrap MyInstrument in
MyInstrumentEntity looks about as complicated as the Serde remote approach, and
I already had a lot of that code autogenerated by macros anyway. Other than the
required `uid` field, there isn't a whole lot of superfluous stuff.

That said, I'm still enamored of the idea that core structs might not have to
know anything about egui. Maybe that can be the dividing line.

**Update after thinking more**: "can't get rid of #[derive(Serialize,
Deserialize)]" is incorrect if the serialized version is explicitly distinct
from the in-memory result. I won't be composing anything out of that; that's the
point. Rather, the serialization struct exists only as scaffolding for Serde,
and it presumably implements appropriate From<> traits to transform to/from the
in-memory versions. (Yet another reason why the `Params` macro needs to die).

So this means we could have the following:

- The core struct that implements the actual audio business logic.
- A wrapper that turns the core struct into an Entity: it has a Uid, it maps
  from Control to fields, maybe it provides a buffer so that we don't have to
  keep flipping the core struct back and forth between internal/external
  buffers, and so on. I am still unsure whether this it better than how it is
  now.
- A settings struct that is Serde-friendly and transforms to/from working
  structs.

# 2023-11-01: more on ControlAtlas

I had a new thought after the prior entry on this topic. ControlRouter *is* the
repository of ControlTrips. It doesn't own the actual Entities, but it does know
their Uids. It could work like this:

- Something creates a ControlTrip with a Uid. Maybe it's a right-click on a
  controllable thing. When it creates it, it hands it to EntityStore, and it
  also adds source/target/index to ControlRouter.
- When it's appropriate, we render the ControlTrip within a DisplaysInTimeline
  context, which includes editing it. This is probably the TrackWidget.
- "Appropriate" could include the controlled thing being highlighted, or cycling
  through all a track's automations. The way we do that would be (1) find all
  the entities in the track, (2) for each entity, ask ControlRouter for links
  where that entity is the target, (3) render those links.

Basically this means kill ControlAtlas ASAP. It also might be the end of
DisplaysInTimeline, because we seem to be converging on specialized widgets that
were never going to be asked to Displays::ui() in a generic context.

# 2023-11-03: Widget vs. component

Another test: drop targets. If it can have multiple, then it must be a
component. If it can only be a single drop target, then it's probably a widget.

# 2023-11-07: More on concrete vs trait

I just noticed that I broke all the integration tests with the switch to
integrated LivePatternSequencer. They are all adding their own programmed
sequencers, so we ended up with two sequencers. They broke because not everyone
was using the same singleton PianoRoll. I updated a test to use the integrated
sequencer, but I'm not happy with how it turned out; there are too many
intricate and fragile ordering requirements.

So now for the Nth time I'm rethinking everything. Maybe this will work:

- `Orchestrator`'s core job is to accept the Entities it is given, and keep
  calling them in the right sequence until they're done.
- `Orchestrator`'s non-jobs: egui rendering, serialization.
- `OrchestratorHelper` takes an `Orchestrates` and does stuff like rendering to
  audio.
- `Project`'s job is to convert serialized Project to/from Orchestrator and
  anything else, like PianoRoll.
- `OrchestratorMegaWidget`'s job is to render the entire Orchestrator view and
  allow editing.

What I did today:

- Made a new `Orchestrator` and renamed the old one to `OldOrchestrator`. The
  new one *almost* does nothing but implement the trait.
- Made `Project` that doesn't quite do anything yet.
- Started separating out egui code into a new crate.
- Created `OrchestratorHelper` and moved the rendering methods there.

I'm going to commit this on a branch because it has a lot of roughness, and I'll
lose track of which still need work if I commit on main.
