use memory::*;

fn main() {
    let l1 = CacheLevel::new(64, 8, 64);
    println!("{l1:?}");
}
