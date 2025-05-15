use std::error::Error;
use crate::disk::Disk;
use crate::runner::Runner;

use super::{BLOCK_SIZE, CHUNKS};

pub struct ThreadedRunner {}

impl ThreadedRunner {
    pub fn new() -> Self { ThreadedRunner {} }
}

impl Runner for ThreadedRunner {
    fn run(&self, disk: &mut Disk) -> Result<(), Box<dyn Error>> {
        let disk_sz = disk.get_size();
        if disk_sz < (BLOCK_SIZE * CHUNKS) {
            return Err("Disk size is too small".into());
        }




        Ok(())
    }
}