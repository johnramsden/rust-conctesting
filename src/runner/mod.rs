pub mod sequential;
pub mod threaded;

use std::error::Error;
use std::sync::{Arc, Mutex};
use super::disk::Disk;

const BLOCK_SIZE: u64 = 4096;
const CHUNKS: u64 = 1000;

pub trait Runner {
    fn run(&self, disk: Arc<Mutex<Disk>>) -> Result<(), Box<dyn Error>>;

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