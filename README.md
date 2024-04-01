# Squiddi MIDI

Squid-style intelligent MIDI router for Linux/ALSA:
Multiple sources to multiple sinks, with filtering and switching.

The intended use is to run on a single-board Linux computer with a
sufficiently large number of (hardware) MIDI ports attached, and
function as an appliance to switch, split, and merge MIDI data
between a number of devices.

One example use case is a retrocomputer setup with multiple computers
and multiple MIDI synths. In this setup, all the computers can be
permanently connected to the router, and similarly all the synths
can be permanently connected. The MIDI output from the computers can
then be automatically routed to the most appropriate synth(s) for
the current software running.


## Features
- Transfer input MIDI messages to zero, one or multiple output ports.
- Different output routes for different input ports.
- Modify the MIDI stream to "polyfill" messages between different feature sets.
  - In specific, initially emulating All Notes Off for the Roland RA-50.
  - Or dropping realtime messages from synths with built-in sequencers.

### To-do/planned
- Enable/disable routes based on received messages (SysEx).
  - For example, recognize GM, GS, XG system reset messages and enable routes
    to relevant outputs.
- Live management interface.
  - Implement a custom SysEx protocol to retrieve status and reconfigure.
  - Implement an HTTP adapter for the SysEx protocol, and a web-based UI.


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
