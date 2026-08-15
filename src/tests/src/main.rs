use memory::*;

fn main() {

    let block_size = 4;
    let associativity = 2;
    let n_sets = 1;

    let mut ram = Ram::new(1024);
    let _ = ram.write_16(0, 300);

    let config = vec![
        CacheLevelConfig::new(associativity, block_size, n_sets, 1, CacheReplacementPolicy::LRU, CacheWritePolicy::WriteThrough),
        CacheLevelConfig::new(associativity, block_size, n_sets, 1, CacheReplacementPolicy::FIFO, CacheWritePolicy::WriteBack),
        CacheLevelConfig::new(associativity, block_size, n_sets, 1, CacheReplacementPolicy::LFU, CacheWritePolicy::WriteBack)
    ];

    let mut cache = CacheController::new(&config).unwrap();

    let _ = cache.read_8(&mut ram, 0);
    let _ = cache.write_8(&mut ram, 0, 5);
    let _ = cache.write_8(&mut ram, 0, 8);
    println!("{:?}", cache.read_8(&mut ram, 0).unwrap());

    cache.print();
}
