use machine::{Machine, MachineConfig};
use serde::Serialize;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let config = MachineConfig::default();
    let machine = Machine::new(&config);

    let yaml = serde_yaml::to_string(&config).unwrap();
    {
        let mut f1 = File::create("config.yaml").expect("Could not open f1");
        f1.write_all(yaml.as_bytes()).unwrap();
    }
    // println!("narvi-cli: initialized {} hart(s)", machine.harts().len());
}
