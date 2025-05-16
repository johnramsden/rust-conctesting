use super::{BLOCK_SIZE, CHUNKS};
use crate::disk::Disk;
use crate::runner::Runner;
use std::collections::VecDeque;
use std::error::Error;
use std::sync::{Arc};
use tokio::runtime::Builder;
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};

pub struct GreenRunner {}

impl GreenRunner {
    pub fn new() -> Self {
        GreenRunner {}
    }
}

struct Query {
    addr: u64,
    payload: Vec<u8>,
}

impl Runner for GreenRunner {
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
                        let bufvec = GreenRunner::get_buffer(i as u32);
                        let addr: u64 = i * BLOCK_SIZE;
                        let q = Query {
                            addr: addr,
                            payload: bufvec,
                        };
                        println!("[W.{}] Writing {} bytes at address {:#x}", i, BLOCK_SIZE, q.addr);
                        if let Err(e) = tokio::task::spawn_blocking({
                            let disk = Arc::clone(&disk);
                            let payload = q.payload.clone();
                            move || disk.write(payload.as_slice(), q.addr)
                        })
                            .await
                            .unwrap()
                        {
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
                                let read_result = tokio::task::spawn_blocking({
                                    let disk = Arc::clone(&disk);
                                    let mut buf = [0u8; BLOCK_SIZE as usize];
                                    move || {
                                        disk.read(&mut buf, q.addr).map(|_| buf)
                                    }
                                })
                                    .await
                                    .unwrap();

                                match read_result {
                                    Ok(read_buf) => {
                                        assert_eq!(q.payload.as_slice(), read_buf.as_slice());
                                        println!("[R.{}] Read data from {:#x}", i, q.addr);
                                    }
                                    Err(e) => eprintln!("Read failed: {:?}", e),
                                }
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
