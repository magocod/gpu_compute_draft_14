use opencl::wrapper::block::{OpenclBlock, OpenclCommonOperation};
use opencl_collections::config::DEFAULT_DEVICE_INDEX;
use opencl_collections::map::config::MapSrc;
use opencl_collections::utils::MB;
use std::time::{Duration, Instant};
use std::{fs, thread};

const SECOND_SLEEP: u64 = 10;

fn main() {
    let mut map_src: MapSrc<i32> = MapSrc::new(1);
    map_src.add(256, 256);
    map_src.add(512, 256);
    // 50mb -> 200mb (ClType::I32)
    map_src.add(MB * 50, 1);

    let program_source = map_src.build();

    fs::write("./tmp/map_src.cl", &program_source).unwrap();

    // println!("{:#?}", config);
    println!();

    println!("start compile cl");
    let now = Instant::now();
    let ocl_block = OpenclBlock::new(DEFAULT_DEVICE_INDEX, &program_source).unwrap();
    println!("ocl_block {}", ocl_block.get_id());
    ocl_block.initialize_memory().unwrap();
    println!("{} seg compile cl", now.elapsed().as_secs());

    println!();

    let summary = map_src.summary();
    println!("{:#?}", summary);

    println!("{} seg compile cl", now.elapsed().as_secs());

    thread::sleep(Duration::from_secs(SECOND_SLEEP));
}
