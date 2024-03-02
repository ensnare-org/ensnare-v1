# MiniDAW example

## Using

1. Download the latest release from
   [https://github.com/ensnare-org/ensnare/releases]. If you're on Windows or
   Mac OS, look for "windows" or "apple" in the filename. If you're on Linux,
   you probably want the `amd64` `.deb`.
2. On Windows or Mac OS, unzip the archive. On Linux, install using
   `sudo apt install ~/Downloads/wherever-you-put-the.deb`.
3. Launch the "Ensnare MiniDAW" app. On Windows and Mac OS, that means running
   the executable `minidaw` in the unzipped directory. On Linux, the Ensnare app
   should appear in your desktop's application menu.
4. You should see a DAW at this point!

It's too early to document the DAW GUI because it's changing quickly, but here
are some things to try.

- Drag one of the instruments (FM Synth, Subtractive Synth, Sampler, Drumkit) to
  the bottom of the first track to add it to that track.
- If you have a MIDI keyboard attached to your PC, you should be able to pick it
  in Settings as a MIDI In (you might have to restart the app after plugging the
  keyboard in). If you don't have a MIDI keyboard, your computer keyboard is a
  virtual MIDI keyboard. The keys A-K are white keys, and the row above has the
  black keys. Use the left and right arrows to change octaves.
- Drag an effect to the track with your instrument to change the sound.
- Click any effect or instrument on a track to edit its parameters. Some are
  missing their editors -- sorry!
- Right-click any effect or instrument on a track to remove it.
- Create patterns in the Composer tab. Hold down the Control/Command key and
  drag/scroll to pan/zoom.
- To arrange a pattern, drag its icon from the Composer to a track.
- Drag an arranged pattern to move it.
- To duplicate an arranged pattern, select it and press Control-D (or Command-D
  on a Mac), or shift-drag it to the new position.
- To delete an arranged pattern, select it and press the Delete key.
- To save your project, press the Save button. Open works, too.
- Export your creation via the Export to WAV button and send it to your friends!

Other things being worked on now:

- Only the subtractive synth has patches/presets. You can only load them; you
  can't save new ones..
- Automation. If you right-click on a track's title bar, you can switch to the
  automation view. (You can drag the path to some of the controls in the
  instrument detail view, and that will establish a link letting the path
  automate the control.) You can't edit the automation tracks yet; they're
  randomly generated. Try dragging an automation path to the FmSynth's pan
  slider!

[File a GitHub issue](https://github.com/ensnare-org/ensnare/issues) to help
prioritize work!
