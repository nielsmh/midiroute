use alsa::{
    rawmidi,
    card,
};
use std::env;

mod config;

fn main() {
    let args: Vec<_> = env::args().collect();

    use alsa::Ctl;

    for a in card::Iter::new().map(|a| a.unwrap()) {
        for b in rawmidi::Iter::new(&Ctl::from_card(&a, false).unwrap()).map(|b| b.unwrap()) {
            println!("Rawmidi {:?} (hw:{},{},{}) {} - {}", b.get_stream(), a.get_index(), b.get_device(), b.get_subdevice(),
                 a.get_name().unwrap(), b.get_subdevice_name().unwrap())
        }
    }

    let cfgfn = &args[1];
    match config::Config::read(&cfgfn) {
        Ok(cfg) => {
            println!("yay! successfully read {}", cfg.filename);
            for dev in cfg.devices.iter() {
                println!("device {}:", dev.name);
                if let Some(name) = &dev.input {
                    println!("- input: {}", name);
                }
                if let Some(name) = &dev.output {
                    println!("- output: {}", name);
                }
            }
            for route in cfg.routes.iter() {
                let state = match route.enabled {
                    true => "active",
                    false => "inactive",
                };
                println!("{} route: {} -> {}", state, route.source, route.sink);
            }
        },
        Err(why) => println!("oh no! {}", why),
    };
}
