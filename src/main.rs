use crate::runner::Runner;

mod runner;
mod disk;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut disk = disk::Disk::new("/dev/zd32")?;
    let seq_runner = runner::SequentialRunner::new();
    seq_runner.run(&mut disk)
}
