use super::{BLOCK_SIZE, CHUNKS};
use crate::disk::Disk;
use crate::runner::Runner;
use std::collections::VecDeque;
use std::error::Error;
use std::sync::Arc;
use tokio::runtime::Builder;
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};

use crate::runner::greenrunner::GreenRunner;
use futures::{future::join_all, stream::FuturesUnordered, StreamExt};

#[derive(Debug)]
struct Query {
    addr: u64,
    payload: Vec<u8>,
    id: u64,
}

pub struct AsyncRunner {}

impl AsyncRunner {
    pub fn new() -> Self {
        AsyncRunner {}
    }
}
async fn write_query(disk: Arc<Disk>, i: u64) -> Result<Query, Box<dyn Error + Send + Sync>> {
    let bufvec = GreenRunner::get_buffer(i as u32);
    let addr: u64 = i * BLOCK_SIZE;
    let q = Query {
        addr: addr,
        payload: bufvec,
        id: i,
    };
    println!(
        "[W.{}] Writing {} bytes at address {:#x}",
        i, BLOCK_SIZE, q.addr
    );
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
    Ok(q)
}
async fn read_query(disk: Arc<Disk>, query: Query) -> Result<(), Box<dyn Error + Send + Sync>> {
    let read_result = tokio::task::spawn_blocking({
        let mut buf = [0u8; BLOCK_SIZE as usize];
        move || disk.read(&mut buf, query.addr).map(|_| buf)
    })
    .await
    .unwrap();

    match read_result {
        Ok(read_buf) => {
            assert_eq!(query.payload.as_slice(), read_buf.as_slice());
            println!("[R.{}] Read data from {:#x}", query.id, query.addr);
        }
        Err(e) => eprintln!("Read failed: {:?}", e),
    }

    Ok(())
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

        runtime.block_on(async {
            let mut tasks = FuturesUnordered::new();

            for i in 0..CHUNKS {
                tasks.push(tokio::spawn(write_query(disk.clone(), i)));
            }

            let mut processing_tasks = FuturesUnordered::new();

            while let Some(result) = tasks.next().await {
                let res = result.unwrap();
                processing_tasks.push(tokio::spawn(read_query(disk.clone(), res.unwrap())));
            }

            join_all(processing_tasks).await;
        });

        Ok(())
    }
}
