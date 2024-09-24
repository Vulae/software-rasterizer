// TEMP: Testing stuff perhaps

#![allow(unused)]

use std::{
    sync::{atomic::AtomicBool, mpsc, Arc, Mutex},
    thread,
};

use termion::input::TermRead;

#[derive(Debug)]
struct AsyncStdin {
    rx: mpsc::Receiver<termion::event::Event>,
    closed: Arc<Mutex<bool>>,
}

impl AsyncStdin {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<termion::event::Event>();
        let closed = Arc::new(Mutex::new(false));
        let self_ = Self {
            rx,
            closed: Arc::clone(&closed),
        };
        thread::spawn(move || {
            let stdin = std::io::stdin();
            for event in stdin.events() {
                if *closed.lock().unwrap() {
                    break;
                }
                tx.send(event.unwrap());
            }
        });
        self_
    }

    pub fn next(&self) -> Option<termion::event::Event> {
        self.rx.try_recv().ok()
    }
}

impl Drop for AsyncStdin {
    fn drop(&mut self) {
        *self.closed.lock().unwrap() = true;
    }
}
