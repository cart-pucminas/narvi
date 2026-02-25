use narvi::{
    hart::extensions::Extensions,
    hart::Hart,
    util::rounding_modes::*,
};

use std::fs::File;
use std::io::prelude::*;

fn main() {
    let hart = Hart::from_extensions(&Extensions{
        m:true, a: false, c:false, f:true, d:false
        }, 
        0xFFFF);

    let yaml = serde_yaml::to_string(&hart).unwrap();
    {
        let mut f1 = File::create("hart.yaml").expect("Could not open f1");
        f1.write_all(yaml.as_bytes()).unwrap();
    }

    {
        let f1 = File::open("hart.yaml").expect("Could not open f1 (read).");
        let hart2:Hart = serde_yaml::from_reader(f1).unwrap();

        assert_eq!(hart, hart2);
    }

    print!("Hello World");
}
