use conc::disk;
use conc::runner::{Runner, SequentialRunner};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut disk = disk::Disk::new("/dev/zd32")?;
    let seq_runner = SequentialRunner::new();
    seq_runner.run(&mut disk)
}
