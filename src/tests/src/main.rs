use memory::*;

fn main() {

    let mut l1 = CacheLevel::new(4, 2, 4, CachePolicy::LRU);
    l1.print();
    println!("==============");

    l1.insert(0, vec![25], false);
    l1.print();
    println!("==============");
    l1.insert(1, vec![35], false);
    l1.print();
    println!("==============");
    l1.insert(2, vec![55], false);

    let x : Vec<u8> = l1.read_level(0, 4).into();

    l1.print();
    println!("x: {:?}", x);
}
