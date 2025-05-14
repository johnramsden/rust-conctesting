use std::error::Error;
use crate::disk::Disk;

const BLOCK_SIZE: u64 = 4096;
const CHUNKS: u64 = 1000;

pub struct SequentialRunner {}

impl SequentialRunner {
    pub fn new() -> Self {
        SequentialRunner {}
    }
}

pub trait Runner {
    fn run(&self, disk: &mut Disk) -> Result<(), Box<dyn Error>>;

    fn get_buffer(pattern: u32) -> Vec<u8> {
        let repeats = 4096 / std::mem::size_of::<u32>();
        let mut buffer = Vec::with_capacity(4096);

        for _ in 0..repeats {
            buffer.extend_from_slice(&pattern.to_ne_bytes());
        }

        assert_eq!(buffer.len(), 4096);

        buffer
    }
}

impl Runner for SequentialRunner {    
    fn run(&self, disk: &mut Disk) -> Result<(), Box<dyn Error>> {
        let disk_sz = disk.get_size();
        if disk_sz < (BLOCK_SIZE * CHUNKS) as u64 {
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