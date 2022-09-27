use std::result;
use std::sync::mpsc;
use std::time;

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = mpsc::channel();
    let sender = Sender {
        pos: 0,
        senders: vec![tx],
    };
    let receiver = Receiver { inner: rx };

    (sender, receiver)
}

#[derive(Debug)]
pub struct Receiver<T> {
    inner: mpsc::Receiver<T>,
}

pub struct IntoIter<T> {
    rx: Receiver<T>,
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> result::Result<T, mpsc::RecvError> {
        self.inner.recv()
    }

    pub fn recv_timeout(
        &self,
        timeout: time::Duration,
    ) -> result::Result<T, mpsc::RecvTimeoutError> {
        self.inner.recv_timeout(timeout)
    }

    pub fn try_recv(&self) -> result::Result<T, mpsc::TryRecvError> {
        self.inner.try_recv()
    }
}

impl<T> IntoIterator for Receiver<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        IntoIter { rx: self }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self.rx.recv() {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }
}

pub struct Sender<T> {
    senders: Vec<mpsc::Sender<T>>,
    pos: usize,
}

impl<T> Sender<T> {
    pub fn send(&mut self, mut t: T) -> result::Result<(), mpsc::SendError<T>> {
        self.pos += 1;

        while !self.senders.is_empty() {
            if self.pos == self.senders.len() {
                self.pos = 0;
            }

            t = match self.senders[self.pos].send(t) {
                Ok(_) => return Ok(()),
                Err(mpsc::SendError(t)) => t,
            };

            self.senders.remove(self.pos);
        }

        Err(mpsc::SendError(t))
    }

    pub fn new_receiver(&mut self) -> Receiver<T> {
        let (tx, rx) = mpsc::channel();
        self.senders.push(tx);

        Receiver { inner: rx }
    }
}
