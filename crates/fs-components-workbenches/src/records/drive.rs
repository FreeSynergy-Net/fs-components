//! The thread that asks the provider, and the way it waits for an answer.
//!
//! Pattern: Active object (the worker owns the non-`Send` provider, built inside its own thread; the workbenches only read the
//! shared [`Snapshot`]) + Strategy (`Driver`: how a future is run; the real capnp client brings its own, the default parks the
//! thread). Failure is never silent: a failed call leaves the last snapshot and sets [`Status::Stale`].

use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, JoinHandle, Thread};
use std::time::Duration;

use super::judge::Judge;
use super::mirror::RecordsRead;
use super::shape::RecordsBinding;
use super::snapshot::{Outcome, Snapshot, Status, refresh};

/// How a future of the worker is run to its end.
pub type Driver = for<'a> fn(Pin<Box<dyn Future<Output = Outcome> + 'a>>) -> Outcome;

struct Unpark(Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// The default driver: poll, and park the thread until woken. Enough for a provider that needs no reactor of its own.
pub fn block_on_std(future: Pin<Box<dyn Future<Output = Outcome> + '_>>) -> Outcome {
    let mut future = future;
    let waker = Waker::from(Arc::new(Unpark(thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(v) = future.as_mut().poll(&mut cx) {
            return v;
        }
        thread::park();
    }
}

#[derive(Debug)]
pub(super) struct Shared {
    pub snapshot: Snapshot,
    pub status: Status,
    /// Counts up with every new snapshot or status, so a reader can tell that it changed.
    pub version: u64,
    /// A snapshot has arrived at least once.
    pub loaded: bool,
}

pub(super) type Cell = Arc<Mutex<Shared>>;

pub(super) fn lock(cell: &Cell) -> MutexGuard<'_, Shared> {
    cell.lock().unwrap_or_else(PoisonError::into_inner)
}

enum Msg {
    Poll,
    Reload,
    Stop,
}

/// The worker: its handle keeps the thread alive; dropping the handle stops it.
pub struct Worker {
    pub(super) cell: Cell,
    tx: Sender<Msg>,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Start the thread. `factory` builds the provider INSIDE the thread (it need not be `Send`); every `every` it asks whether the
    /// generation changed (`None` = only on request); the first reading starts at once.
    pub fn spawn<R, F>(
        factory: F,
        binding: RecordsBinding,
        judge: Arc<dyn Judge>,
        every: Option<Duration>,
        driver: Driver,
    ) -> Self
    where
        R: RecordsRead + 'static,
        F: FnOnce() -> R + Send + 'static,
    {
        let cell: Cell = Arc::new(Mutex::new(Shared {
            snapshot: Snapshot::default(),
            status: Status::Loading,
            version: 0,
            loaded: false,
        }));
        let (tx, rx) = channel();
        let shared = Arc::clone(&cell);
        let thread =
            thread::spawn(move || run(&factory(), &binding, &*judge, every, driver, &rx, &shared));
        Self {
            cell,
            tx,
            thread: Some(thread),
        }
    }

    /// Read again now, even when the generation is the same.
    pub fn reload(&self) {
        let _ = self.tx.send(Msg::Reload);
    }

    /// Ask whether the generation changed; read again if so.
    pub fn poll(&self) {
        let _ = self.tx.send(Msg::Poll);
    }

    /// Read the snapshot in place, without copying it.
    pub(super) fn read<T>(&self, f: impl FnOnce(&Snapshot) -> T) -> T {
        f(&lock(&self.cell).snapshot)
    }

    /// Whether a snapshot has arrived at least once.
    pub(super) fn loaded(&self) -> bool {
        lock(&self.cell).loaded
    }

    /// The snapshot as it is now.
    pub fn snapshot(&self) -> Snapshot {
        lock(&self.cell).snapshot.clone()
    }

    /// How the worker stands.
    pub fn status(&self) -> Status {
        lock(&self.cell).status.clone()
    }

    /// Changes whenever the snapshot or the status changed.
    pub fn version(&self) -> u64 {
        lock(&self.cell).version
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        let _ = self.tx.send(Msg::Stop);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn run(
    records: &dyn RecordsRead,
    binding: &RecordsBinding,
    judge: &dyn Judge,
    every: Option<Duration>,
    driver: Driver,
    rx: &Receiver<Msg>,
    cell: &Cell,
) {
    step(records, binding, judge, driver, cell, false);
    loop {
        let msg = match every {
            Some(d) => match rx.recv_timeout(d) {
                Ok(m) => m,
                Err(RecvTimeoutError::Timeout) => Msg::Poll,
                Err(RecvTimeoutError::Disconnected) => return,
            },
            None => match rx.recv() {
                Ok(m) => m,
                Err(_) => return,
            },
        };
        match msg {
            Msg::Stop => return,
            Msg::Reload => step(records, binding, judge, driver, cell, true),
            Msg::Poll => step(records, binding, judge, driver, cell, false),
        }
    }
}

fn step(
    records: &dyn RecordsRead,
    binding: &RecordsBinding,
    judge: &dyn Judge,
    driver: Driver,
    cell: &Cell,
    force: bool,
) {
    let known = {
        let g = lock(cell);
        (!force && g.loaded).then_some(g.snapshot.generation)
    };
    let outcome = driver(Box::pin(refresh(records, binding, judge, known)));
    let mut g = lock(cell);
    match outcome {
        Outcome::Fresh(s) => {
            g.snapshot = s;
            g.loaded = true;
            g.status = Status::Ready;
        }
        Outcome::Unchanged => g.status = Status::Ready,
        Outcome::Failed(f) => g.status = Status::Stale(f),
    }
    g.version += 1;
}
