use alsa::seq;
use crate::pipeline;


struct IgnoreSense;
impl pipeline::Filter for IgnoreSense {
    fn process(&self, events: &mut Vec<seq::Event>) {
        events.retain(|ev| ev.get_type() != seq::EventType::Sensing)
    }
}


struct IgnoreClock;
impl pipeline::Filter for IgnoreClock {
    fn process(&self, events: &mut Vec<seq::Event>) {
        events.retain(|ev| ev.get_type() != seq::EventType::Clock)
    }
}


pub fn new(name: &str) -> Result<Box<dyn pipeline::Filter>, String> {
    match name {
        "ignore_sense" => Ok(Box::new(IgnoreSense)),
        "ignore_clock" => Ok(Box::new(IgnoreClock)),
        _ => Err(format!("unknown filter: '{}'", name)),
    }
}
