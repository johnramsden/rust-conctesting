use super::{BLOCK_SIZE, CHUNKS};
use crate::disk::Disk;
use crate::runner::Runner;
use std::collections::VecDeque;
use std::error::Error;
use std::sync::{Arc};
use tokio::runtime::Builder;
use tokio::sync::oneshot;
use tokio::sync::oneshot::{Receiver, Sender};

use futures::{stream::FuturesUnordered, StreamExt, future::join_all};
pub struct AsyncRunner {}

impl AsyncRunner {
    pub fn new() -> Self {
        AsyncRunner {}
    }
}
async fn write_query(disk: Arc<Disk>, i: u64) -> Result<Query, Box<dyn Error + Send + Sync>>{
    Ok(Query {
        addr: 0,
        payload: vec![],
    })
}
async fn read_query(disk: Arc<Disk>, i: u64) {
    unimplemented!();
}

#[derive(Debug)]
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

        runtime.block_on(async {
            let mut tasks = FuturesUnordered::new();

            for i in 0..CHUNKS {
                tasks.push(tokio::spawn(write_query(disk.clone(), i)));
            }

            let mut processing_tasks = FuturesUnordered::new();

            while let Some(result) = tasks.next().await {
                processing_tasks.push(tokio::spawn(async move {
                    println!("[Event] {:?}", result);
                    // Do something that must finish
                }));
            }

            join_all(processing_tasks).await;
        });

        Ok(())
    }
}
