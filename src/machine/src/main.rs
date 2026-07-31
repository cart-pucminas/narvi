use machine::{Machine, MachineConfig};
use memory::Ram;
use std::{
    env,
    fs,
    fs::File,
    io::prelude::*
};

fn main() {
    let config = MachineConfig::default();

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("Expected a file path");
    }
    
    let _machine = {
        let assembly = fs::read(&args[1]).expect("Could not read file");
        Machine::new(&config, assembly)
    };

    let yaml = serde_yaml::to_string(&config).unwrap();
    {
        let mut f1 = File::create("config.yaml").expect("Could not open f1");
        f1.write_all(yaml.as_bytes()).unwrap();
    }
    // println!("narvi-cli: initialized {} hart(s)", machine.harts().len());
}
