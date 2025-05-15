use std::sync::{Arc, Mutex};
use conc::disk;
use conc::runner::{Runner};
use conc::runner::sequential::SequentialRunner;
use conc::runner::threaded::ThreadedRunner;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut disk = disk::Disk::new("/dev/zd32")?;
    let wrapped_disk = Arc::new(Mutex::new(disk));
    let seq_runner = SequentialRunner::new();
    seq_runner.run(Arc::clone(&wrapped_disk))?;
    let threaded_runner = ThreadedRunner::new();
    threaded_runner.run(Arc::clone(&wrapped_disk))
}
