use opencl::wrapper::block::{OpenclBlock, OpenclCommonOperation};
use opencl_collections::config::DEFAULT_DEVICE_INDEX;
use opencl_collections::dictionary::config::DictSrc;
use std::time::{Duration, Instant};
use std::{fs, thread};
// TODO ...

const SECOND_SLEEP: u64 = 10;

fn main() {
    let mut dict_src: DictSrc<i32> = DictSrc::new();

    dict_src.add(64, 256, 32);
    dict_src.add(32, 32, 16);

    println!();

    let program_source = dict_src.build();
    // println!("{program_source}");
    fs::write("./tmp/dict_src.cl", &program_source).unwrap();

    println!("start compile cl");
    let now = Instant::now();
    let ocl_block = OpenclBlock::new(DEFAULT_DEVICE_INDEX, &program_source).unwrap();
    println!("ocl_block {}", ocl_block.get_id());
    ocl_block.initialize_memory().unwrap();
    println!("{} seg compile cl", now.elapsed().as_secs());

    thread::sleep(Duration::from_secs(SECOND_SLEEP));
}
