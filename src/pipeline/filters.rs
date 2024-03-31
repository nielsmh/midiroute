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
