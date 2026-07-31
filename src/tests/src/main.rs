use memory::*;

fn main() {

    let block_size = 4;
    let associativity = 2;
    let n_sets = 1;

    let l1 = CacheLevel::new(block_size, associativity, n_sets, CachePolicy::LRU);
    let l2 = CacheLevel::new(block_size, associativity, n_sets, CachePolicy::LRU);
    let l3 = CacheLevel::new(block_size, associativity, n_sets, CachePolicy::LFU);
    
    let mut ram = Ram::with_size(1024);
    let _ = ram.write_16(0, 300);

    let levels = vec![l1, l2, l3];
    let mut cache = CacheController::new(levels, ram, block_size);

    let _ = cache.read_8(0);
    let _ = cache.write_8(0, 1);
    let _ = cache.write_8(0, 2);
    let _ = cache.write_8(0, 3);
    let _ = cache.write_8(0, 4);

    cache.print();
}
