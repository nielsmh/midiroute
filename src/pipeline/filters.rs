use alsa::seq;
use alsa::seq::EventType;
use crate::pipeline;

struct IgnoreSense;
impl pipeline::Filter for IgnoreSense {
    fn process(&mut self, events: &mut Vec<seq::Event>) {
        events.retain(|ev| ev.get_type() != EventType::Sensing)
    }
}


struct IgnoreClock;
impl pipeline::Filter for IgnoreClock {
    fn process(&mut self, events: &mut Vec<seq::Event>) {
        events.retain(|ev| ev.get_type() != EventType::Clock)
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
struct ActiveNote {
    channel: u8,
    note: u8,
}
struct EmulateANO(Vec<ActiveNote>);
impl pipeline::Filter for EmulateANO {
    fn process(&mut self, events: &mut Vec<seq::Event>) {
        let mut new_events = Vec::new();
        let mut injected = false;
        for ev in events.iter_mut() {
            match ev.get_type() {
                EventType::Noteon | EventType::Noteoff => {
                    // keep track of active notes
                    let data: seq::EvNote = ev.get_data().unwrap();
                    let is_off = data.velocity == 0 || ev.get_type() == EventType::Noteoff;
                    let this_note = ActiveNote { channel: data.channel, note: data.note };
                    let existing_index = self.0.iter().position(|an| an == &this_note);
                    if is_off {
                        existing_index.map(|idx| self.0.swap_remove(idx));
                    } else if existing_index == None {
                        self.0.push(this_note);
                    }
                },
                EventType::Controller => {
                    let data: seq::EvCtrl = ev.get_data().unwrap();
                    if data.param == 123 {
                        // controller 123 is All Notes Off (on channel)
                        for an in self.0.iter() {
                            if an.channel == data.channel {
                                let note_off = seq::EvNote {
                                    channel: an.channel,
                                    note: an.note,
                                    velocity: 0,
                                    duration: 0,
                                    off_velocity: 0,
                                };
                                new_events.push(seq::Event::new(EventType::Noteoff, &note_off));
                                injected = true;
                            }
                        }
                        self.0.retain(|an| an.channel != data.channel);
                    }
                }
                _ => (),
            }
            new_events.push(ev.clone());
        }
        if injected {
            // only replace events if new ones were actually injected
            events.clear();
            events.append(&mut new_events);
        }
    }
}


pub fn new(name: &str) -> Result<Box<dyn pipeline::Filter>, String> {
    match name {
        "ignore_sense" => Ok(Box::new(IgnoreSense)),
        "ignore_clock" => Ok(Box::new(IgnoreClock)),
        "emulate_ano" => Ok(Box::new(EmulateANO(Vec::new()))),
        _ => Err(format!("unknown filter: '{}'", name)),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::Filter;

    /// Make a note-on event
    fn make_noteon(channel: u8, note: u8, velocity: u8) -> seq::Event<'static> {
        let d = seq::EvNote {
            channel: channel,
            note: note,
            velocity: velocity,
            duration: 0,
            off_velocity: 0,
        };
        seq::Event::new(seq::EventType::Noteon, &d)
    }
    /// Make a note-off event
    fn make_noteoff(channel: u8, note: u8) -> seq::Event<'static> {
        let d = seq::EvNote {
            channel: channel,
            note: note,
            velocity: 0,
            duration: 0,
            off_velocity: 0,
        };
        seq::Event::new(seq::EventType::Noteoff, &d)
    }
    /// Make a channel control change event
    fn make_chctrl(channel: u8, param: u32, value: i32) -> seq::Event<'static> {
        let d = seq::EvCtrl {
            channel: channel,
            param: param,
            value: value,
        };
        seq::Event::new(seq::EventType::Controller, &d)
    }

    #[test]
    fn test_emulate_ano() {
        let mut eano = EmulateANO(Vec::new());
        let mut evts: Vec<seq::Event>;

        // Initially zero active notes
        assert_eq!(eano.0.len(), 0);

        // One one is played and one note is active
        evts = vec!(make_noteon(0, 10, 100));
        eano.process(&mut evts);
        assert_eq!(eano.0.len(), 1);
        assert_eq!(evts.len(), 1);

        // A second note is played and two notes are active
        evts = vec!(make_noteon(0, 12, 100));
        eano.process(&mut evts);
        assert_eq!(eano.0.len(), 2);
        assert_eq!(evts.len(), 1);

        // The first note is played at zero velocity is a note-off, so back to one note active
        evts = vec!(make_noteon(0, 10, 0));
        eano.process(&mut evts);
        assert_eq!(eano.0.len(), 1);
        assert_eq!(evts.len(), 1);

        // The second note is off, so back to zero active
        evts = vec!(make_noteoff(0, 12));
        eano.process(&mut evts);
        assert_eq!(eano.0.len(), 0);
        assert_eq!(evts.len(), 1);

        // Note off for a note that isn't playing should do nothing
        evts = vec!(make_noteoff(0, 12));
        eano.process(&mut evts);
        assert_eq!(eano.0.len(), 0);
        assert_eq!(evts.len(), 1);

        // Play the same note on two different channels, should have two notes active
        evts = vec!(make_noteon(0, 10, 100), make_noteon(1, 10, 100));
        eano.process(&mut evts);
        assert_eq!(eano.0.len(), 2);
        assert_eq!(evts.len(), 2);
        
        // Play one of the notes again on the same channel, should still have two notes active
        evts = vec!(make_noteon(0, 10, 50));
        eano.process(&mut evts);
        assert_eq!(eano.0.len(), 2);
        assert_eq!(evts.len(), 1);

        // All notes off on channel 1 should stop one note, leave the other active, and add a note-off event to the evts
        evts = vec!(make_chctrl(1, 123, 0));
        eano.process(&mut evts);
        assert_eq!(eano.0.len(), 1);
        assert_eq!(evts.len(), 2);
        assert_eq!(evts[0].get_type(), seq::EventType::Noteoff);

        // Another All notes off on channel 1 should do nothing
        evts = vec!(make_chctrl(1, 123, 0));
        eano.process(&mut evts);
        assert_eq!(eano.0.len(), 1);
        assert_eq!(evts.len(), 1);
    }
}
