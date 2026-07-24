use memory::*;

fn main() {

    let l1 = CacheLevel::new(4, 2, 4, CachePolicy::LRU);
    let l2 = CacheLevel::new(4, 2, 4, CachePolicy::LRU);
    
    let mut ram = Ram::with_size(1024);
    let _ = ram.write_32(0, 55);

    let levels = vec![l1, l2];
    let mut cache = CacheController::new(levels, ram);

    let _ = cache.read_32(0);

    cache.print();
}
