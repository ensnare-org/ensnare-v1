# Good practices

- Aim for components to have just the required eframe/egui imports, and for the
  bulk of the functionality to live in widgets. I've changed the Displays trait
  to qualify rather than use, so the ideal component won't import anything.
- Where possible, parameters should be normalized, and then scaled to the needed
  range at the last possible moment. A great example is envelope
  attack/decay/release times. Don't store as seconds; store as a Normal, and
  then make time range a late-binding scalar.
  https://github.com/sowbug/groove/issues/130
- For any given physical parameter, make sure it's correctly represented as
  linear or logarithmic. https://github.com/sowbug/groove/issues/44

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
https://github.com/ensnare-org/ensnare/commit/95db90375b6d31707dc92658aa6f046cb6716b4e

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
