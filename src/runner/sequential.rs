use std::error::Error;
use std::sync::{Arc, Mutex};
use crate::disk::Disk;
use crate::runner::Runner;

use super::{BLOCK_SIZE, CHUNKS};

pub struct SequentialRunner {}

impl SequentialRunner {
    pub fn new() -> Self { SequentialRunner {} }
}

impl Runner for SequentialRunner {    
    fn run(&self, disk: Arc<Disk>) -> Result<(), Box<dyn Error>> {
       
        let disk_sz = disk.get_size();
        if disk_sz < (BLOCK_SIZE * CHUNKS) {
            return Err("Disk size is too small".into());
        }

        for i in 0..CHUNKS {
            let addr: u64 = i * BLOCK_SIZE;
            println!("Writing {} bytes to chunk {} at address {:#x}", BLOCK_SIZE, i, addr);
            let bufvec = SequentialRunner::get_buffer(i as u32);
            let buf = bufvec.as_slice();
            disk.write(buf, addr)?;
            let data_read: &mut [u8] = &mut [0u8; BLOCK_SIZE as usize];
            disk.read(data_read, addr)?;
            assert_eq!(buf, data_read);
            println!("Read data from {:#x}", addr);
        }

        Ok(())
    }
}
