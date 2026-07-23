use memory::*;

fn main() {

    let mut l1 = CacheLevel::new(4, 2, 4, CachePolicy::LRU);

    l1.new_block(0, vec![25]);
    l1.new_block(1, vec![35]);
    l1.new_block(2, vec![55]);

    l1.print();
    let x : Vec<u8> = l1.read(0, 4).into();
    println!("x: {:?}", x);
}
