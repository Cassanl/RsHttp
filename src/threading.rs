use std::fmt;
use std::fmt::write;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

type Job = Box<dyn FnOnce() + Send + 'static>;

// #[derive(Debug, Clone)]
// struct PoolCreationError;

// impl fmt::Display for PoolCreationError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "Unable to create thread pool")
//     }
// }

pub struct ThreadPool {
    workers: Vec<Worker>,
    pub tx: Sender<Job>
}

impl ThreadPool {
    // return Result<>
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (tx, rx): (Sender<Job>, Receiver<Job>) = mpsc::channel();
        let guarded_rx = Arc::new(Mutex::new(rx));
        let mut workers: Vec<Worker> = Vec::with_capacity(size);
        for id in 0..size {
            match Worker::new(id, Arc::clone(&guarded_rx)) {
                Some(worker) => workers.push(worker),
                None => panic!()
            }
        }
        ThreadPool { workers, tx }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        let _result = self.tx.send(job);
    }
}

struct Worker {
    id: usize,
    thread: JoinHandle<()>
}

impl Worker {
    fn new(id: usize, rx: Arc<Mutex<Receiver<Job>>>) -> Option<Self> {
        let thread_builder = thread::Builder::new();
        let worker_result = thread_builder.spawn(move || { 
            loop {
                let job = rx
                    .lock()
                    .unwrap()
                    .recv()
                    .unwrap();
                job()
            }
        });
        match worker_result {
            Ok(worker) => Some(Worker { id, thread: worker}),
            Err(_) => None
        }

        // let thread = thread::spawn(move || {
        //     loop {
        //         let job = rx
        //             .lock()
        //             .unwrap()
        //             .recv()
        //             .unwrap();
        //         job()
        //     }
        // });
        // Worker { id, thread }
    } 
}
