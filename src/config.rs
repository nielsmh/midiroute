pub struct Device {
    pub name: String,
    pub input: Option<String>,
    pub output: Option<String>,
    pub features: Vec<String>,
    pub input_filters: Vec<String>,
    pub output_filters: Vec<String>,
}

pub struct Route {
    pub enabled: bool,
    pub source: String,
    pub sink: String,
    pub filters: Vec<String>,
}

pub struct Config {
    pub filename: String,
    pub devices: Vec<Device>,
    pub routes: Vec<Route>,
}

fn get_stringopt_from_value(value: &json::JsonValue) -> Option<String> {
    value.as_str().map(|s| s.to_string())
}
fn get_stringvec_from_value(value: &json::JsonValue) -> Vec<String> {
    value.members()
        .filter_map(|f| f.as_str().map(|s| s.to_string()))
        .collect()
}

impl Config {
    pub fn read(filename: &str) -> Result<Config, String> {
        use std::fs;
        use std::io;

        let cfgtext = match fs::read_to_string(filename) {
            io::Result::Ok(text) => text,
            io::Result::Err(err) => return Err(err.to_string()),
        };
        let cfg = match json::parse(&cfgtext) {
            Ok(obj) => obj,
            Err(err) => return Err(err.to_string()),
        };

        if !cfg["devices"].is_array() {
            return Err("invalid or missing 'devices' section in config".to_string());
        }
        let devices: Vec<Device> = cfg["devices"].members()
            .filter_map(|dev|
                dev["name"].as_str().map(|name|
                    Device {
                        name: name.to_string(),
                        input: get_stringopt_from_value(&dev["input"]),
                        output: get_stringopt_from_value(&dev["output"]),
                        features: get_stringvec_from_value(&dev["features"]),
                        input_filters: get_stringvec_from_value(&dev["input_filters"]),
                        output_filters: get_stringvec_from_value(&dev["output_filters"]),
                    }
                )
            )
            .collect();
        
        if !cfg["routes"].is_array() {
            return Err("invalid or missing 'routes' section in config".to_string());
        }
        let routes: Vec<Route> = cfg["routes"].members()
            .filter_map(|route| {
                let enabled = route["enabled"].as_bool()?;
                let source = route["source"].as_str()?.to_string();
                let sink = route["sink"].as_str()?.to_string();
                let filters = get_stringvec_from_value(&route["filters"]);

                if !devices.iter().any(|dev| dev.name == source) { return None; }
                if !devices.iter().any(|dev| dev.name == sink) { return None; }

                Some(Route {
                    enabled: enabled,
                    source: source,
                    sink: sink,
                    filters: filters,
                })
            })
            .collect();
    
        Ok(Config {
            filename: String::from(filename),
            devices: devices,
            routes: routes,
        })
    }

    pub fn get_device(&self, devname: &str) -> Option<&Device> {
        self.devices.iter().find(|dev| dev.name == devname)
    }
}
