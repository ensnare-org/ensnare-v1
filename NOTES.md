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
