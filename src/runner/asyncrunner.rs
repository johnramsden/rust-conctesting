use super::{BLOCK_SIZE, CHUNKS};
use crate::disk::Disk;
use crate::runner::threaded::ThreadedRunner;
use crate::runner::Runner;
use std::collections::VecDeque;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};

const READER_THREADS: usize = 14;
const WRITER_THREADS: usize = 14;

pub struct AsyncRunner {}

impl AsyncRunner {
    pub fn new() -> Self {
        AsyncRunner {}
    }
}

struct Query {
    addr: u64,
    payload: Vec<u8>,
}

impl Runner for AsyncRunner {
    fn run(&self, disk: Arc<Disk>) -> Result<(), Box<dyn Error>> {
        let disk_sz = disk.get_size();
        if disk_sz < (BLOCK_SIZE * CHUNKS) {
            return Err("Disk size is too small".into());
        }

        let runtime = Builder::new_multi_thread()
            .worker_threads(32)
            .thread_name("AsyncRunner")
            .build()
            .unwrap();

        let mut readers = VecDeque::new();

        runtime.block_on(async {
            for i in 0..1000 {
                let (tx, rx): (Sender<Query>, Receiver<Query>) = oneshot::channel();
                tokio::spawn({
                    let disk = Arc::clone(&disk);
                    async move {
                        let bufvec = AsyncRunner::get_buffer(i as u32);
                        let addr: u64 = i * BLOCK_SIZE;
                        let q = Query {
                            addr: addr,
                            payload: bufvec,
                        };
                        println!("[W.{}] Writing {} bytes at address {:#x}", i, BLOCK_SIZE, q.addr);
                        if let Err(e) = disk.write(q.payload.as_slice(), q.addr) {
                            eprintln!("Write failed: {:?}", e);
                        }
                        tx.send(q)
                    }
                });

                let handle = tokio::spawn({
                    let disk = Arc::clone(&disk);
                    async move {
                        match rx.await {
                            Ok(q) => {
                                let data_read: &mut [u8] = &mut [0u8; BLOCK_SIZE as usize];
                                if let Err(e) = disk.read(data_read, q.addr) {
                                    eprintln!("Read failed: {:?}", e);
                                }
                                assert_eq!(q.payload.as_slice(), data_read);
                                println!("[R.{}] Read data from {:#x}", i, q.addr);
                            }
                            Err(_) => eprintln!("Writer dropped before sending"),
                        }
                    }
                });

                readers.push_back(handle);
            }

            for h in readers {
                h.await.unwrap();
            }
        });

        Ok(())
    }
}
