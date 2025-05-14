use std::error::Error;
use crate::disk::Disk;

pub struct SequentialRunner {}

pub trait Runner {
    fn run(&self, disk: Disk) -> Result<(), Box<dyn Error>>;
}

impl Runner for SequentialRunner {
    fn run(&self, disk: Disk) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}