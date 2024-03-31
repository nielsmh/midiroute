use alsa;
use alsa::seq;
use std::env;

mod config;

fn main() {
    let args: Vec<_> = env::args().collect();

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

    println!("opening sequencer");
    let my_client = seq::Seq::open(None, Some(alsa::Direction::Capture), false)
        .expect("wanted to open the sequencer");
    for client in seq::ClientIter::new(&my_client) {
        for port in seq::PortIter::new(&my_client, client.get_client()) {
            if !port.get_type().contains(seq::PortType::HARDWARE) { continue }
            let addr_str = format!("{}:{}", port.get_client(), port.get_port());
            let flags: Vec<_> = port.get_capability().iter()
                .filter_map(|cap| match cap {
                    seq::PortCap::READ  => Some("R"),
                    seq::PortCap::WRITE => Some("W"),
                    _ => None
                })
                .collect();
            println!("port {} '{}' is type {:?} with caps {:?}", addr_str, port.get_name().unwrap_or("?"), port.get_type(), flags);
        }
    }
}
