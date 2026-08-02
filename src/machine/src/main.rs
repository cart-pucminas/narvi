use machine::{Machine, MachineConfig};
use core::error::Error;
use std::{
    env,
    fs,
    fs::File,
    io::prelude::*
};

fn main() -> Result<(), Box<dyn Error>> {
    let config = MachineConfig::default();

    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        panic!("Expected a file path");
    }
    
    let mut machine = {
        let assembly = fs::read(&args[1]).expect("Could not read file");
        println!("{assembly:X?}");
        Machine::new(&config, assembly).expect("Could not construct machine")
    };

    while !machine.done() {
        machine.update()?
    }

    println!("{:?}", machine.get_hart(0));

    print_hex_table(&machine.dump_l1());    

    let yaml = serde_yaml::to_string(&config).unwrap();
    {
        let mut f1 = File::create("config.yaml").expect("Could not open f1");
        f1.write_all(yaml.as_bytes()).unwrap();
        Ok(())
    }
    // println!("narvi-cli: initialized {} hart(s)", machine.harts().len());
}

fn print_hex_table(bytes: &[u8]) {
    const BYTES_PER_ROW: usize = 16;

    for (row_idx, chunk) in bytes.chunks(BYTES_PER_ROW).enumerate() {
        print!("{:08x}  ", row_idx * BYTES_PER_ROW);

        for byte in chunk {
            print!("{:02x} ", byte);
        }

        if chunk.len() < BYTES_PER_ROW {
            let missing_bytes = BYTES_PER_ROW - chunk.len();
            print!("{}", " ".repeat(missing_bytes * 3));
        }

        print!(" |");

        for &byte in chunk {
            if byte.is_ascii_graphic() || byte == b' ' {
                print!("{}", byte as char);
            } else {
                print!(".");
            }
        }

        println!("|");
    }
}
