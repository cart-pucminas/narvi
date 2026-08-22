use std::{
    env,
    fs,
    fs::File,
    io::prelude::*
};

use core::error::Error;

use narvi_core::{
    serialization::{
        MachineConfig
    }
};

use engine::Engine;

fn main() -> Result<(), Box<dyn Error>> {
    let config = MachineConfig::default();

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("Expected a file path");
    }
    
    let mut engine = {
        let assembly = fs::read(&args[1]).expect("Could not read file");
        println!("{assembly:X?}");
        Engine::build_from_config(&config, assembly)
    };

    let mut is_running = true;

    while is_running {
        is_running = engine.update();
    }

    println!("{:?}", engine.get_journal());

    let yaml = serde_yaml::to_string(&config).unwrap();
    {
        let mut f1 = File::create("config.yaml").expect("Could not open f1");
        f1.write_all(yaml.as_bytes()).unwrap();
        Ok(())
    }
}
