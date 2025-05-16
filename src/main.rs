use std::sync::{Arc, Mutex};
use conc::disk;
use conc::runner::{Runner};
use conc::runner::asyncrunner::AsyncRunner;
use conc::runner::sequential::SequentialRunner;
use conc::runner::threaded::ThreadedRunner;

use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let disk = disk::Disk::new("/dev/zd32")?;
    let wrapped_disk = Arc::new(disk);

    let seqt = Instant::now();
    
    let seq_runner = SequentialRunner::new();
    seq_runner.run(Arc::clone(&wrapped_disk))?;
    
    let seqt = seqt.elapsed();

    let thrt = Instant::now();
    
    let threaded_runner = ThreadedRunner::new();
    threaded_runner.run(Arc::clone(&wrapped_disk))?;
    
    let thrt = thrt.elapsed();

    let asynct = Instant::now();
    
    let async_runner = AsyncRunner::new();
    async_runner.run(Arc::clone(&wrapped_disk))?;
    
    let asynct = asynct.elapsed();
    
    println!("Sequential: {:?}", seqt);
    println!("Threaded: {:?}", thrt);
    println!("Async: {:?}", asynct);
    
    Ok(())
}
