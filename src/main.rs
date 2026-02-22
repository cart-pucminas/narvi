use risky_sim::{
    hart::extensions::Extensions,
    hart::Hart,
    util::rounding_modes::*,
};

use serde::Serialize;

fn main() {
    let ext = Extensions {m:true, a: false, c:false, f:true, d:false};
    let mut hart = Hart::from_extensions(&ext, 0xFFFF);
    let yaml = serde_yaml::to_string(&hart).unwrap();
    let json = serde_json::to_string(&hart).unwrap();
    println!("Yaml:\n {}", yaml);
    println!("\nJSON:\n {}", json);
}
