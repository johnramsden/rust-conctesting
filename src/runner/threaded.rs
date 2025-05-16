use std::collections::VecDeque;
use std::error::Error;
use std::sync::{Arc, Mutex};
use crate::disk::Disk;
use crate::runner::Runner;

use std::thread;
use crossbeam_channel::{unbounded, Receiver, Sender};
use crate::runner::sequential::SequentialRunner;
use super::{BLOCK_SIZE, CHUNKS};

const READER_THREADS: usize = 14;
const WRITER_THREADS: usize = 14;

pub struct ThreadedRunner {}

impl ThreadedRunner {
    pub fn new() -> Self { ThreadedRunner {} }
}

struct Query {
    addr: u64,
    payload: Vec<u8>,
}

impl Runner for ThreadedRunner {
    fn run(&self, disk: Arc<Mutex<Disk>>) -> Result<(), Box<dyn Error>> {
        let disk_sz = disk.lock().unwrap().get_size();
        if disk_sz < (BLOCK_SIZE * CHUNKS) {
            return Err("Disk size is too small".into());
        }
        
        let mut deque: VecDeque<Query> = VecDeque::with_capacity(CHUNKS as usize);
        
        for i in 0..CHUNKS {
            let addr: u64 = i * BLOCK_SIZE;
            println!("Writing {} bytes to chunk {} at address {:#x}", BLOCK_SIZE, i, addr);
            let bufvec = ThreadedRunner::get_buffer(i as u32);
            deque.push_back(Query {
                addr: addr,
                payload: bufvec
            });
        }

        let workload = Arc::new(Mutex::new(deque));
        
        let (sender, receiver): (Sender<Query>, Receiver<Query>) = unbounded();
        
        let mut writer_threads = Vec::new();
        
        for i in 0..WRITER_THREADS {
            writer_threads.push(thread::spawn({
                let sender = sender.clone();
                let workload = Arc::clone(&workload);
                let disk = Arc::clone(&disk);
                move || {
                    loop {
                        let val = {
                            let mut v = workload.lock().unwrap();
                            v.pop_front()
                        };

                        match val {
                            Some(val) => {
                                println!("[W.{}] Writing {} bytes at address {:#x}", i, BLOCK_SIZE, val.addr);
                                let mut d = disk.lock().unwrap();
                                if let Err(e) = d.write(val.payload.as_slice(), val.addr) {
                                    eprintln!("Write failed: {:?}", e);
                                }
                                sender.send(val).unwrap()
                            }
                            None => break, // no more work — exit thread
                        }
                    }
                }
            }));
        }

        let mut reader_threads = Vec::new();

        let toread = Arc::new(Mutex::new(CHUNKS));
        
        for i in 0..READER_THREADS {
            reader_threads.push(thread::spawn({
                let receiver = receiver.clone();
                let disk = Arc::clone(&disk);
                let toread = Arc::clone(&toread);
                move || {
                    loop {
                        let q = {
                            let mut left = toread.lock().unwrap();
                            if *left == 0 {
                                break;
                            }
                            *left -= 1;
                            receiver.recv().unwrap()
                        };
                        
                        let mut d = disk.lock().unwrap();
                        let data_read: &mut [u8] = &mut [0u8; BLOCK_SIZE as usize];
                        if let Err(e) = d.read(data_read, q.addr) {
                            eprintln!("Read failed: {:?}", e);
                        }
                        assert_eq!(q.payload.as_slice(), data_read);
                        println!("[R.{}] Read data from {:#x}", i, q.addr);
                    }
                }
            }));
        }

        for handle in writer_threads {
            handle.join().unwrap();
        }
        for handle in reader_threads {
            handle.join().unwrap();
        }

        Ok(())
    }
}