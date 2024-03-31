Octopus-style intelligent MIDI router for Linux/ALSA.

## Goals for the project
- Transfer input MIDI messages to zero, one or multiple output ports.
- Different output routes for different input ports.
- Enable/disable routes based on received messages (SysEx).
  - For example, select between GS and XG output synths depending on a received system reset message.
- Modify the MIDI stream to "polyfill" messages between different feature sets.
  - In specific, initially emulating All Notes Off for the Roland RA-50.
  - Or dropping realtime messages from synths with built-in sequencers.

The intended runtime environment is single-board computers running Linux,
with a suffuciently large number of (hardware) MIDI ports attached,
to function as an appliance that rarely needs to be reconfigured.


## Questions nobody asked

### Why not just use ALSA's built-in MIDI port connections (patchbay)?

Yes they will offer better latency, if the goal is just to have a set of static routes.
However, they aren't capable of intercepting messages and filtering them,
or changing the routing depending on them.

### ALSA port and client numbering is unstable. How to handle that?

Unfortunately, right now it seems the most reliable way to handle port/client
numbering when MIDI devices are added or removed, is to reboot the machine
entirely, to force a re-init of the numbering.
As long as the physical attachment point for a device doesn't change, it should
get the same hardware id and sequencer client id every time the kernel boots.
