use memory::*;

fn main() {

    let mut l1 = CacheLevel::new(64, 2, 64);
    l1.insert(0, vec![25], false);
    l1.insert(63, vec![35], false);
    l1.insert(64, vec![55], false);

    let x = match l1.read_level(0, 4) {
        CacheReturn::Hit(value) => value,
        _ => vec![]
    };

    println!("{:?}", x);
    println!("{:?}", l1.data[0]);
    println!("{:?}", l1);
}
