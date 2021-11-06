use std::sync::mpsc::{channel, RecvError, SendError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle};

pub struct FixedThreadPool {
    threads: Option<Vec<JoinHandle<()>>>,
    queue_sender: Sender<Message>,
}

impl FixedThreadPool {
    pub fn new(n: usize) -> Self {
        if n == 0 {
            panic!("thread size must be > 0");
        }
        let (queue_sender, queue_receiver): (Sender<Message>, _) = channel();
        let queue_receiver = Arc::new(Mutex::new(queue_receiver));

        let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(n);
        for _ in 0..n {
            let receiver = queue_receiver.clone();
            threads.push(spawn(move || loop {
                let received = receiver.lock().unwrap().recv();
                if let Ok(msg) = received {
                    match msg {
                        Message::Task(task) => {
                            task();
                        }
                        Message::Terminate => {
                            break;
                        }
                    }
                }
            }))
        }
        Self {
            threads: Some(threads),
            queue_sender,
        }
    }

    pub fn execute<T>(&self, f: T) -> Result<()>
    where
        T: FnOnce() + 'static + Send,
    {
        self.queue_sender.send(Message::Task(Box::new(f)))?;
        Ok(())
    }
}

impl Drop for FixedThreadPool {
    fn drop(&mut self) {
        if let Some(threads) = self.threads.take() {
            for _ in 0..threads.len() {
                self.queue_sender.send(Message::Terminate);
            }
            for t in threads {
                t.join().unwrap();
            }
        }
    }
}

type Task = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    Task(Task),
    Terminate,
}

#[derive(Debug)]
pub enum ThreadPoolError {
    SendTaskError(SendError<Message>),
}

impl From<SendError<Message>> for ThreadPoolError {
    fn from(e: SendError<Message>) -> Self {
        ThreadPoolError::SendTaskError(e)
    }
}

type Result<O> = std::result::Result<O, ThreadPoolError>;
