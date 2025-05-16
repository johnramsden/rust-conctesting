use std::fs::File;
use std::{io, thread};
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::fs::FileExt;
use std::os::unix::io::AsRawFd;
use std::time::Duration;
use nix::ioctl_read;
use nix::sys::uio::pwrite;

pub struct Disk {
    handle: File,
    device: String,
    size: u64,
}

ioctl_read!(blkgetsize64, 0x12, 114, u64);

impl Disk {
    pub fn new(device: &str) -> Result<Self, io::Error> {
        let handle = OpenOptions::new()
            .read(true)
            .write(true)
            .open(device)?;
        let fd = handle.as_raw_fd();

        let mut size: u64 = 0;
        unsafe {
            blkgetsize64(fd, &mut size).unwrap();
        }
        
        if size == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "Disk size is 0"));
        }
        
        Ok(Self {
            handle,
            device: device.to_owned(),
            size,
        })
    }
    pub fn write(&self, data: &[u8], offset: u64) -> Result<usize, io::Error> {
        let sz = pwrite(&self.handle, data, offset as i64)
            .map_err(|e| std::io::Error::from_raw_os_error(e as i32));
        thread::sleep(Duration::from_millis(25));
        sz
    }

    pub fn read(&self, data: &mut [u8], offset: u64) -> Result<usize, io::Error> {
        self.handle.read_at(data, offset)
    }
    
    pub fn get_size(&self) -> u64 {
        self.size
    }
}