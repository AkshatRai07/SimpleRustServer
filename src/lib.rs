use std::{
    fmt,
    sync::{Arc, Mutex, mpsc},
    thread,
};

/// Custom error type for ThreadPool operations.
#[derive(Debug)]
pub enum PoolError {
    CreationError(String),
    SendError(String),
}

impl fmt::Display for PoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PoolError::CreationError(msg) => write!(f, "Pool Creation Error: {msg}"),
            PoolError::SendError(msg) => write!(f, "Job Dispatch Error: {msg}"),
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

/// A group of spawned threads that are waiting and ready to handle tasks.
///
/// This manages a collection of `Worker` instances and uses a channel to
/// dispatch closures to those workers.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Errors
    ///
    /// This function will return a `PoolError::CreationError` if the size is 0.
    pub fn build(size: usize) -> Result<ThreadPool, PoolError> {
        if size == 0 {
            return Err(PoolError::CreationError("Pool size must be greater than zero".into()));
        }

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool {
            workers,
            sender: Some(sender),
        })
    }

    /// Sends a closure to the pool for execution.
    ///
    /// # Errors
    ///
    /// Returns `PoolError::SendError` if the receiving side of the channel has been closed.
    pub fn execute<F>(&self, f: F) -> Result<(), PoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender
            .as_ref()
            .ok_or_else(|| PoolError::SendError("ThreadPool sender is missing".into()))?
            .send(job)
            .map_err(|e| PoolError::SendError(e.to_string()))
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in self.workers.drain(..) {
            if let Some(thread) = worker.thread {
                thread.join().expect("Thread failed to join");
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            
            let message = receiver.lock().expect("Mutex poisoned").recv();

            match message {
                Ok(job) => job(),
                Err(_) => break,
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_pool_creation() {
        let pool = ThreadPool::build(4);
        assert!(pool.is_ok());
    }

    #[test]
    fn test_zero_size_pool_fails() {
        let pool = ThreadPool::build(0);
        assert!(pool.is_err());
    }

    #[test]
    fn test_execution() {
        let pool = ThreadPool::build(2).unwrap();
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..10 {
            let c = Arc::clone(&counter);
            pool.execute(move || {
                c.fetch_add(1, Ordering::SeqCst);
            }).unwrap();
        }

        drop(pool);
        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }
}