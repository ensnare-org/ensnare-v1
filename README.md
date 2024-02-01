# Ensnare

A library for generating digital audio, with an example DAW.

Ensnare is currently unstable. The API is changing significantly. No
backward-compatibility guarantee exists.

## Using the DAW

1. Download the latest release from
   [https://github.com/ensnare-org/ensnare/releases]. If you're on Windows or
   Mac OS, look for "windows" or "apple" in the filename. If you're on Linux,
   you probably want the `amd64` `.deb`.
2. On Windows or Mac OS, unzip the archive. On Linux, install using `sudo apt install ~/Downloads/wherever-you-put-the.deb`.
3. Launch the app. On Windows and Mac OS, that means running the executable
   `ensnare-daw` in the unzipped directory. On Linux, the Ensnare app should
   appear in your desktop's application menu.
4. You should see a DAW at this point!

It's too early to document the DAW GUI because it's changing quickly, but here
are some things to try.

- Drag one of the instruments (FM Synth, Welsh Synth, Sampler, Drumkit) to the
  circular icon on the first track to add it to the track.
- If you have a MIDI keyboard attached to your PC, you should be able to pick it
  in Settings as a MIDI In. If you don't have a MIDI keyboard, your computer
  keyboard is a virtual MIDI keyboard. The keys A-K are white keys, and the row
  above has the black keys. Use the left and right arrows to change octaves.
- Drag an effect to the track with your instrument to change the sound.
- Click any effect or instrument on a track to edit its parameters. Some are
  missing their editors -- sorry!
- Right-click any effect or instrument on a track to remove it.
- Click the little piano icon to show the Composer, where you can create
  patterns. Drag the six-dot icon below a pattern to a track to arrange it.
- To duplicate an arranged pattern, select it and press Control-D (or Command-D
  on a Mac).
- To delete an arranged pattern, select it and press the Delete key.
- To save your project, press the Save button. **BEWARE** that the app uses a
  single file (`ensnare-project.json`) for loading and saving, so you can't have
  more than one project right now! Also **BEWARE** that the app won't confirm
  quitting or opening a saved project over your current one, so **expect to lose work**!
- Export your creation via the Export to WAV button. Look for
  `ensnare-project.wav` and send it to your friends!

Other things being worked on now:

- Synth patches. Today, each synth has a default that doesn't sound awful, but
  there is no patch or sample library.
- Automation. This works pretty well in the backend, but there is no GUI for it.

File a GitHub issue to help prioritize more work!

## Using the Ensnare library

Check out the example code (in the standard `examples` directory). As with most
crates, `cargo doc --no-deps --workspace` will generate rustdoc that you can
view locally in your web browser.
