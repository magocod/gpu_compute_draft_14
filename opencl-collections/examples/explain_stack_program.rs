use opencl::wrapper::block::{OpenclBlock, OpenclCommonOperation};
use opencl_collections::config::DEFAULT_DEVICE_INDEX;
use opencl_collections::stack::config::StackSrc;
use std::time::{Duration, Instant};
use std::{fs, thread};
// TODO ...

const SECOND_SLEEP: u64 = 10;

fn main() {
    let mut stack_src = StackSrc::new();
    stack_src.add(256);
    stack_src.add(512);

    println!("{:#?}", stack_src);

    let program_source = stack_src.build();
    // println!("{program_source}");
    fs::write("./tmp/stack_src.cl", &program_source).unwrap();

    println!("start compile cl");
    let now = Instant::now();
    let ocl_block = OpenclBlock::new(DEFAULT_DEVICE_INDEX, &program_source).unwrap();
    println!("ocl_block {}", ocl_block.get_id());
    ocl_block.initialize_memory().unwrap();
    println!("{} seg compile cl", now.elapsed().as_secs());

    thread::sleep(Duration::from_secs(SECOND_SLEEP));
}
