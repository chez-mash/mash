//! Advisory file locks that are released explicitly rather than by closing.
//!
//! On Unix an `flock` lock belongs to the open file description, not to one
//! descriptor. When any thread starts a child process, the child holds a copy
//! of every descriptor until it execs, close-on-exec ones included. A lock
//! "released" by dropping its `File` therefore stays held until that child
//! execs, and an immediate relock by this or another process reports busy.
//! Unlocking releases the lock for the whole description however many copies
//! are still open, so every lock taken here is unlocked before its file
//! closes. Use these helpers instead of `File::try_lock` and friends.

use std::fs::{File, TryLockError};
use std::io;

/// A file that can carry an advisory lock.
pub trait LockFile {
    fn lock_file(&self) -> &File;
}

impl LockFile for File {
    fn lock_file(&self) -> &File {
        self
    }
}

impl LockFile for tempfile::NamedTempFile {
    fn lock_file(&self) -> &File {
        self.as_file()
    }
}

/// An advisory lock held on `F` until the guard drops.
#[derive(Debug)]
pub struct FileLock<F: LockFile = File> {
    file: F,
}

impl<F: LockFile> FileLock<F> {
    /// Take an exclusive lock without waiting: `None` when another holder
    /// keeps any lock on the file.
    pub fn try_exclusive(file: F) -> io::Result<Option<Self>> {
        settle(file.lock_file().try_lock()).map(|locked| locked.then_some(Self { file }))
    }

    /// Take a shared lock without waiting: `None` when another holder keeps
    /// an exclusive lock on the file.
    pub fn try_shared(file: F) -> io::Result<Option<Self>> {
        settle(file.lock_file().try_lock_shared()).map(|locked| locked.then_some(Self { file }))
    }

    /// Wait for an exclusive lock.
    pub fn exclusive(file: F) -> io::Result<Self> {
        file.lock_file().lock()?;
        Ok(Self { file })
    }

    /// Wait for a shared lock.
    pub fn shared(file: F) -> io::Result<Self> {
        file.lock_file().lock_shared()?;
        Ok(Self { file })
    }

    /// The locked file.
    pub fn file(&self) -> &F {
        &self.file
    }
}

impl<F: LockFile> Drop for FileLock<F> {
    fn drop(&mut self) {
        // Closing alone may leave the lock held; see the module comment. An
        // unlock failure leaves closing as the only release there is.
        let _ = self.file.lock_file().unlock();
    }
}

/// Whether another holder keeps any lock on `file`. The probe's own lock, if
/// it gets one, is released before this returns.
pub fn held(file: &File) -> io::Result<bool> {
    probe(file, file.try_lock())
}

/// Whether another holder keeps an exclusive lock on `file`. The probe's own
/// shared lock, if it gets one, is released before this returns.
pub fn held_exclusive(file: &File) -> io::Result<bool> {
    probe(file, file.try_lock_shared())
}

fn probe(file: &File, attempt: Result<(), TryLockError>) -> io::Result<bool> {
    let locked = settle(attempt)?;
    if locked {
        file.unlock()?;
    }
    Ok(!locked)
}

/// `true` when the attempt locked, `false` when another holder blocked it.
fn settle(attempt: Result<(), TryLockError>) -> io::Result<bool> {
    match attempt {
        Ok(()) => Ok(true),
        Err(TryLockError::WouldBlock) => Ok(false),
        Err(TryLockError::Error(error)) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(path: &std::path::Path) -> File {
        File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path)
            .unwrap()
    }

    #[test]
    fn a_probe_leaves_the_lock_free() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("probe.lock");
        assert!(!held(&open(&path)).unwrap());
        assert!(!held_exclusive(&open(&path)).unwrap());
        let shared = FileLock::try_shared(open(&path)).unwrap().unwrap();
        assert!(held(&open(&path)).unwrap());
        assert!(!held_exclusive(&open(&path)).unwrap());
        assert!(FileLock::try_exclusive(open(&path)).unwrap().is_none());
        drop(shared);
        let exclusive = FileLock::try_exclusive(open(&path)).unwrap().unwrap();
        assert!(held_exclusive(&open(&path)).unwrap());
        drop(exclusive);
        assert!(FileLock::try_exclusive(open(&path)).unwrap().is_some());
    }

    /// Children started by other threads copy the lock's descriptor until
    /// they exec. A dropped guard must still free the lock at once.
    #[cfg(unix)]
    #[test]
    fn a_dropped_lock_is_free_while_other_threads_start_children() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::time::{Duration, Instant};
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("relock.lock");
        let stop = Arc::new(AtomicBool::new(false));
        let spawners: Vec<_> = (0..4)
            .map(|_| {
                let stop = Arc::clone(&stop);
                std::thread::spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        let _ = std::process::Command::new("true").status();
                    }
                })
            })
            .collect();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut busy = 0;
        let mut attempts = 0;
        while attempts < 5_000 && Instant::now() < deadline {
            attempts += 1;
            match FileLock::try_exclusive(open(&path)).unwrap() {
                Some(lock) => drop(lock),
                None => busy += 1,
            }
        }
        stop.store(true, Ordering::Relaxed);
        for spawner in spawners {
            spawner.join().unwrap();
        }
        assert_eq!(
            busy, 0,
            "{busy} of {attempts} relocks found the lock still held"
        );
    }
}
