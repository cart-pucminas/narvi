use machine::{Machine, MachineConfig};

fn main() {
    let config = MachineConfig::default();
    let machine = Machine::new(&config);

    println!("narvi-cli: initialized {} hart(s)", machine.harts().len());
}
