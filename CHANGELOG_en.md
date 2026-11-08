# Changelog

## [Unreleased]

## [v1.0.0-beta.8] - 2026-11-08

### Added

- [Editor] Added a History window that lists every edit, marks where you last saved, and lets you jump back to any point
- [Editor] Pending Holds, events, and Curve Note Tracks now highlight their timeline and show the available actions and hotkeys at the bottom
- [Editor] Curve Note Tracks can now be copied, cut, and pasted, and their start and end notes come along with them

### Changed

- [Core] Bumped the chart format to version `7`, which gives every object in a chart a persistent ID
- [Core] Events of the same kind on the same line can no longer overlap or have zero length. Events that break this rule are adjusted automatically when a chart is read or imported from the official or RPE format
- [Converter] Phichain charts in older formats are now upgraded automatically when used as input
- [Editor] Redesigned the event timeline
  - Events now show a curve of how their value changes
  - The line's current X, Y, rotation, opacity, and speed are shown on the playhead
  - Values are more compact, rounded to at most two decimal places, and no longer overlap each other
- [Editor] Timelines now label whole beats only, and hovering highlights the beat line under the cursor and shows its full beat
- [Editor] Edits and placements that would make events overlap or have zero length are no longer allowed
- [Editor] Renaming lines and editing Curve Note Track properties can now be undone and redone
- [Editor] Only one Hold, event, or Curve Note Track can be pending at a time. Starting another one, or switching the timeline to a different line, cancels the pending one and tells you why
- [Editor] While something is pending, undoing, redoing, or jumping to a point in the History window cancels it first
- [Editor] When placing an event, the hotkey you finish with now decides its type: Place Transition Event creates a transition event, and Place Constant Event creates a constant event
- [Editor] New BPM points are now added at the playhead and inherit the current BPM
- [Editor] In the BPM List, a BPM point's beat can no longer be moved past its neighbors
- [Editor] New lines now default to an opacity of 255 (fully opaque) instead of 0 (fully transparent)

### Performance

- [Core] Improved playback performance on charts with many events and notes
- [Editor] Improved timeline performance on charts with many notes and events

### Fixed

- [Editor] Fixed being able to create a Curve Note Track whose start and end are the same note
- [Editor] Fixed a Hold briefly appearing too long when you start placing it
- [Editor] Fixed a possible crash when pressing `Esc` to cancel a pending Hold or event
- [Editor] Fixed translucent elements on the timeline (pending objects, the selection box, and grid lines) rendering incorrectly
- [Editor] Fixed Curve Note Tracks not coming back when undoing the deletion of their start or end note
