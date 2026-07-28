use memory::*;

fn main() {

    let block_size = 4;

    let l1 = CacheLevel::new(block_size, 2, 4, CachePolicy::LRU);
    let l2 = CacheLevel::new(block_size, 2, 4, CachePolicy::LRU);
    
    let mut ram = Ram::with_size(1024);
    let _ = ram.write_16(0, 300);

    let levels = vec![l1, l2];
    let mut cache = CacheController::new(levels, ram, block_size);

    let _ = cache.read_8(0);
    let _ = cache.write_8(0, 55);

    cache.print();
}
