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

# 2023-11-16: displaying in timeline

I think things are coming together.

- `Orchestrator` is just entity CRUD plus management of the inner event loop.
- `OrchestratorHelper` renders `Orchestrator` as audio.
- `InMemoryProject` owns `Orchestrator`. Crucially, it *and not `Orchestrator`*
  knows about specific entities. So a method like `new_midi_track()` is
  implemented in `InMemoryProject`, not in `Orchestrator`.
- To fulfill its job, `InMemoryProject` gives only fully-functional entities to
  `Orchestrator`. For example, `LivePatternSequencer` needs a `PianoRoll`, so
  `InMemoryProject` manages a `PianoRoll` and supplies it when creating
  `LivePatternSequencer`. But it *also* asks `LivePatternSequencer` for a
  `crossbeam_channel` `Sender<SequencerInput>` so that it can pass along the
  drag-and-drop event that is supposed to drop a pattern onto the timeline.
  `InMemoryProject` is the mediator between entities that need to communicate
  with each other, *not* `Orchestrator`.
- If I stick to this pattern, then `ControlRouter` will need to become a shared
  entity like `PianoRoll`, so that each `ControlTrip` can maintain a reference
  to it when it's rendering itself. This shouldn't be too much trouble.

Why did this take so long? I was stuck in a mental loop that `Orchestrator` was
the backstop for all responsibilities, so I kept trying to cram things into it,
which violated all sorts of architectural boundaries. Once I realized that
`Orchestrator` should just orchestrate, I was able to imagine it not handling
pattern-adds. Unfortunately I implemented that by creating the `Orchestrates`
trait and thinking that each concrete `Orchestrator` would have more
responsibilities than just the basic trait, which was an improvement but not
quite the breakthrough that *Orchestrator should only orchestrate*. When I took
a break to work on serialization, developing `Project` and then
`InMemoryProject`, I started to see the difference between orchestrating and
providing `Orchestrator` with *complete* entities that it could generically
orchestrate. Now I think (hope) the thought has come all the way to fruition. To
reiterate:

- `Orchestrator` orchestrates.
- `OrchestratorHelper` renders.
- `Project` serializes.
- `InMemoryProject` constructs Entities and coordinates communication among
  them.

It's possible that `InMemoryProject` will become `Orchestrator` v2 in the sense
that it becomes the dumping ground for all new responsibilities. The "and" in
the description above is a hint. But I know how to decompose it if that does
happen.

# 2023-11-28: Serde

I have a spike solution for serializing dyn-trait things. That is relatively
straightforward. But I don't know how to handle rewiring deserialized structs to
things like PianoRoll and channels. If there were very few such things, then I
could add to the Entity trait, but I'm not sure how many there will be.

Things that need extra stuff:

- `LivePatternSequencer`: needs a ref to `PianoRoll` (which could be a
  per-project global).
- `Drumkit` needs a ref to `Paths` (which could be a per-project global,
  possibly a real global).
- `ControlTrip` needs a ref to `ControlRouter` (which could be a per-project
  global, possibly per-track).

Options:

1. Design things not to need anything at construction. Not practical.
2. Trait expands to allow Project to provide common items to everyone as they're
   constructed. Default does nothing. Concerns about scaling.
3. Trait defines a single extra method to set up a channel. I think this is the
   same as #2, but more complicated. Adds the ability to continue to communicate
   with the struct later on, but I don't know if I actually need that.

Continuing to think about #3, what would that channel look like?

- Give me a ref to `PianoRoll`. Presumably whoever is listening on the other end
  knows who I am, so if we're ever multi-project, we can return the right one.
- Give me a ref to `Paths`. This feels like a plain old global function.
- Give me a ref to `ControlRouter`. I think this is identical to the `PianoRoll`
  case.
- I just changed something that the system might need to know about.
- (assuming bidirectional) Something in the system just changed; here it is.

# 2023-12-05: Followup to prior Serde topic

The current approach is to design the serialization format in a way that
facilitiates reconstitution via `Orchestrates` methods. It ends up working
because we have access to the destination `DawProject`/`Orchestrator`, which
have access to all the one-off needs of various entities, and if we unite all
the entity parameter types in a single enum, then we know each concrete type
when we want to instantiate them, so we know how to provide the one-offs to
them.

I don't know how to handle very large "one-offs" like all the sequenced patterns
that `PianoRoll` and `LivePatternSequencer` care about. It is easy on the
deserialization side because we can populate the sequencer when we create the
sequencer. For `PianoRoll`, serialization is also easy because we keep the
concrete instance around, and can ask it to produce the `DiskProject` list. But
for `LivePatternSequencer`, it's not easy because the current design constructs
it and then erases its type, so we can't easily talk to it when saving.

Options:

1. Add `Entity::as_sequences()` and add the serialization there. This adds extra
   vtable cost, but that's not a big deal when we'll likely have fewer than
   1,000 entities in a large project. **Problem**: the `Sequences` trait has an
   associated type, `MU`. Either we bite the bullet and let `MU` infect Entity,
   or we create a separate trait altogether for serialization of sequencing
   stuff (which might itself need `MU` all over again).
2. Extend the current `SequencerInput` channel concept to add a
   `SequencerEvent`, and then add a request and response for serialized data.
   **Problem**: At first glance this seems tricky; I'm really trying to do an
   RPC, and sending on a channel, spinning the `work()` event loop, and hoping
   for a response feels like a really flaky way to do it, especially at a
   degenerate time like saving (when we really shouldn't be spinning the event
   loop).
3. Move sequencer storage out of `LivePatternSequencer` and let the sequencer
   subscribe to it. But if we're going to do that, is there benefit to
   `LivePatternSequencer` being an Entity at all?
4. Give up on `DiskProject` owning the arrangement data, and just make it into
   `LivePatternSequencer`'s serialization data. This fits the best with the
   current design, but it makes me sad that the actual notes of the arrangement
   won't be first-class citizens in `DiskProject`. **Problem**: the `Params`
   concept (derived mini-struct that represents the serializable subset of any
   Entity) is having some growing pains. It's not as smart as Serde, and its
   only advantage over Serde is a little more control over struct construction
   (providing more context-sensitive defaults for things like `PianoRoll`, for
   example).

This seems to boil down to how smart we want `DawProject` to be. Is it just a
container of generic things? Or is it super-smart? If it's dumb, then #4 is the
right choice. If it's smart, then #3 is better. #1 is in-between (it's smart
enough to know to ask for help from a vaguely generic entity), and #2 is a poor
implementation of #1.

I'm drawn toward #3 because it doesn't rule out #4. I can make a basic sequencer
data structure in `DawProject`, and then either let entities operate on it, or
build smarter entities in the future that don't need it at all. I'll try that.

# 2023-12-11: Followup again

Experiment results from a prototype of Option #3: `SequenceRepository` lives in
`DawProject` and contains all the project's sequences. It tries to efficiently
answer queries about sequences. Like `PianoRoll`, multiple things have `RwLock`s
on it.

`ThinSequencer` is `SequenceRepository`'s partner. It doesn't maintain any
persistent state of its own, but it knows what to do with `SequenceRepository`'s
data during playback.

`ThinSequencer` needs to know when the data in `SequenceRepository` changes. I
tried a low-tech solution: `SequenceRepository` keeps a counter, and every time
its data changes, it increments the counter. Others (i.e., `ThinSequencer`) keep
their own copies of that counter, and when it no longer equals the main counter,
they know something has changed. `PianoRoll` introduces the same requirement --
it owns data whose changes `SequenceRepository` cares about -- but since I wrote
that part of the code on a different day, I needed to invent yet another way of
handling change detection: the Ensnare app notices that a GUI change operation
happened when drawing `PianoRoll`, so it notifies `SequenceRepository` that it
should touch its counter, and this causes `ThinSequencer` to do the right thing.
TODO: if refactoring to simplify makes sense, do it.

Next - implement `From<&Vec<(...)>)>> for SequenceRepository` and get
persistence working.

# 2023-12-12: Content vs. performance

Insight: the arrangements keep wanting to come back to the project file
because... they're the project!

The project file contains the musical content -- the MIDI notes (or a suitably
abstract representation of them like patterns and pattern uids), the tempo, the
time signature, the automation lanes, etc. It also contains the entity
parameters. **The entities don't contain the music. They perform the music**.

A sequencer is something that uses the music data (it translates them from
project data to MIDI data). Same as instruments and effects (instruments respond
to MIDI events, and effects respond to instrument output).

A sequencer doesn't need to be a user-choosable component. It's an inherent part
of the DAW.

A `Project` could contain composition, orchestration, and automation sections.
The composition is the music. The orchestration is a specification of how to
assign the music to real instruments. Automation is how the knobs and dials turn
during the performance. As a trial balloon, I'll say that `Project` implements
three traits that allow manipulation of the three areas, and `Composer`,
`Orchestrator`, and `Automator` do the work, but generally the data lives inside
`Project`. There might also be a fourth section called `Visualizer` that handles
the GUI stuff, but I think I'm getting ahead of myself. (Where does Entity Store
belong? Is it properly part of Orchestrator?)

Many of these have Uids. It's cumbersome to mention them here, so I don't.

## Composition elements

Composition owns the musical content -- the song as an abstract thing that could
be played with any instruments.

- **Globals**: Time signature, tempo.
- **Track**: A grouping mechanism.
- **MIDI sequence**: A vector of timed notes.
- **Arrangement**: A vector of timed MIDI sequences that are associated with a
  track. *Might also have a MIDI channel.*

## Orchestration elements

Orchestration decides which instruments/effects are in charge of playing the
composition.

- **Entity**: a set of parameters that reconstitute a configured instrument or
  effect.
- **Entities**: a vector of (`Entity`, track).

## Automation elements

Automation changes knobs and dials as the song progresses.

- **Lane**, **Controller**, etc.: Something that occasionally emits Control
  events.
- **ControlTrip**: A kind of Controller; a vector of timed magnitudes with path
  specifications. For example: at time 0.0, mode is (snap, 0.0). At time 1.0,
  value is (linear, 0.5). At time 2.0, value is (logarithmic, 0.75).
- **Controllables**: things that accept Controller output.
- **Control Links**: A vector (uid, index) indicating that Entity #Uid's
  Parameter #Index should follow the specified Controller.

# 2023-12-12: controller/instrument/effect

The division of entity into controller/instrument/effect might be illusory.
Everyone *might* want to do work on the MusicalTime scale. Everyone *might*
respond to MIDI events. Everyone *might* process another component's audio
output. Everyone *might* produce audio. I've known this for a while; it actually
isn't all that important because we already let anyone declare that they're any
combination of the three.

# 2023-12-29: Entity2

This has been a big task punctuated by holidays. Rather than the Entity trait
being a gateway to individual traits like IsEffect's TransformAudio, Entity2
requires that everyone implement everything. Default null implementations reduce
the boilerplate.

Core things now need to know how to serde themselves. Once I figured out that
the musical content should be serialized separately from the entity using it,
that uncorked a bunch of blockage. Now entities can be free to serialize
themselves, which mostly makes the Params concept obsolete, which removes the
need for EntityParams tying Params to Entities and satisfying the EntityWrapper
trait bound.

Composition is still bedeviling me. It's most likely a YAGNI situation. The
problem is that I want both of the following:

1. An app using the ensnare crate can implement more entities, either by adding
   them to the app, or by including more crates that implement entities.
2. An entity doesn't have to implement functionality that is associated with
   specific apps or frameworks such as egui.

The approach for #1 is to define all the traits in the core crate, and then a
factory that allows entity registration. This is pretty simple; it's what I have
today.

The approach for #2 depends on the exact problem definition.

1. Entities know about all traits. Default implementations are provided. This
   would probably mean that every crate needs to depend on every dependency
   (egui for example).
2. Entities know only about the core traits, and somewhere in between the entity
   and the app, a new trait is defined and perhaps implemented. This means
   entities depend only on the basics.

# 2024-01-20: crate organization

- **core**: structures that everyone needs, and building blocks like envelope
generators and modulators. NO EGUI. NO ENTITY CONCEPTS.
- **entity**: structures that define and operate on entities. Project, parts,
  etc., go here. NO EGUI (but if you want EntityBounds without egui, then you
  have to make all these things generic!).
- **egui**: reusable egui widgets. This crate doesn't (and can't) know about
  specific entities.
- **core-egui**: widgets that display building blocks.
- **entity-egui**: widgets that draw entity infrastructure.
- **entities**: "batteries included" instruments. FM synth, reverb, etc. NO
  EGUI.
- **entities-egui**: widgets that display instruments, as well as Entity
  wrappers for them.
- **toys**: a collection of simple instruments as Entities, to show how to make
  a separate crate.
- **services**: interfaces to external things.
- **proc-macros**: proc-macros.

# 2024-01-21: crate organization part 2

There is a little voice in my head that keeps asking "why are you going through
all this trouble to separate core from entity, and egui from not-egui?" I'm
starting to accept that this voice is smarter than I am. But in the interests of
recording all my relevant thoughts, even the bad ones, here's my justification.

**Why keep Entity separate**: I didn't want to require everyone using my core
crate to also take the baggage of the Entity system. Responses: (1) you'll be
lucky if anyone uses the crate at all; (2) you can #cfg[]-guard that stuff if
you want; (3) if you really think the core things are useful and reusable, then
you should make a PJRC-like audio crate that really knows nothing about Ensnare
concepts, and then use it at arm's length, rather than still requiring everyone
to use ensnare-core types.

**Why keep egui separate**: likewise, I didn't want to pull in an egui
dependency that maybe nobody else needed. Responses #1 and #2 are valid here as
well.

# 2024-01-22: crate naming and more thoughts on dividing lines

"element" is a good substitute for "building block." I was thinking of
"primitive" for a while, but that's an ugly name.

What's an element, what's an entity, what's entity infrastructure?

- An **element** is something that more than one entity is composed of. An
  oscillator or envelope is an element because many instruments use it. Elements
  don't know about Entity Uids. There is no programmatic definition of an element.
- An **entity** implements the Entity trait. The Ensnare DAW won't refer to
  concrete Entity types in its code (except for the factory initializer).
- **Entity infrastructure** handles all the relationships among Entities. Some
  parts of entity infrastructure might look like Entities (the built-in
  sequencer is a good example), but the DAW will refer to them concretely, and
  the user typically won't have the option to mix and match them like Entities.

Core crate ingredients:
- All elements.
  - **core**
- All entity infrastructure.
  - **entity**
- The egui widgets that elements and infra need, but maybe behind a feature
  and/or in an `egui` module. (maybe refactor reusable widgets into a
  non-ensnare crate)
  - **egui**
  - **core-egui**
  - **entity-egui**
  
Entities crate ingredients:
- All "batteries included" entities.
  - **entities**
  - All their egui widgets.
  - **entities-egui**

Still separate crates:
  - **toys**
  - **services**
  - **proc-macros**

# 2024-03-05: signal generators

Today I looked at unifying Envelope and ControlTrip/SignalPath, reasoning that
they both look like envelopes. Differences I identified:

1. Envelope operates on a sample granularity, while CT/SP operates on
   MusicalTime ranges. These are both just time divisions, but they're not
   identical.
2. Envelope emits a value per sample, while CT/SP emits ControlEvents only upon
   changes. This sounds a lot like #1, but the difference is during a flat step,
   Envelope keeps emitting values, but CT/SP emits just one at the start of the
   step.
3. Envelope is mostly event-driven (note on/off), while CT/SP just starts,
   plays, and stops.

So I'm less optimistic about my original hope of creating a single engine that
both would use. Instead, there might be a single data structure representing the
sequences of steps, and each will implement its own code for that data. But not
all is lost! I think that a single widget can allow editing of that structure,
as long as it's smart enough to respect policies like ADSR for envelopes.

# 2024-03-14: egui InnerResponse<R>

Insight: I have developed an idiom of my custom Widgets returning an Option<T>
where T = an enum of a type belonging to each Widget. It's not awful, but it
requires the caller to allocate the action variable beforehand. I did this
because the Widget trait's single `ui()` method returns only a Response. I
wonder whether it'd be better to develop a custom method that returns
InnerResponse<T>. I suppose we'd lose the ability to `ui.add()` the Widget (in
fact it wouldn't be a `Widget` anymore), but if they're returning custom
actions, they weren't useful in the truly generic `Widget` sense anyway.
