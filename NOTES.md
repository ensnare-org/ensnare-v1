# Code smells

- Aim for components to have just the required eframe/egui imports, and for the
  bulk of the functionality to live in widgets. I've changed the Displays trait
  to qualify rather than use, so the ideal component won't import anything.
