use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{RwLockWriteGuard, TryLockError};
use std::thread;
use std::time::{Duration, Instant};

use iced::advanced::graphics::text::{self, FontSystem, cosmic_text, font_system};

// If more types of components start doing heavy font work in background,
// this should probably move into a shared crate/runtime layer.
// For now keep it local and see how the model holds up.

const WORKER_FONT_LOCK_BUDGET: Duration = Duration::from_millis(2);

static FOREGROUND_FONT_LOCK_REQUESTED: AtomicBool = AtomicBool::new(false);

pub(crate) fn foreground_font_lock_requested() -> bool {
  FOREGROUND_FONT_LOCK_REQUESTED.load(Ordering::Acquire)
}

pub(crate) fn font_system_version() -> text::Version {
  font_system()
    .read()
    .expect("iced shared font system lock should not be poisoned")
    .version()
}

pub(crate) fn foreground_font_system_write() -> RwLockWriteGuard<'static, FontSystem> {
  FOREGROUND_FONT_LOCK_REQUESTED.store(true, Ordering::Release);

  let guard = font_system()
    .write()
    .expect("iced shared font system lock should not be poisoned");

  FOREGROUND_FONT_LOCK_REQUESTED.store(false, Ordering::Release);

  guard
}

pub(crate) fn with_worker_font_system<T>(
  cancel: &AtomicBool,
  mut f: impl FnMut(&mut cosmic_text::FontSystem, &WorkerFontLockBudget) -> Option<T>,
) -> Option<T> {
  // Keep the closure bounded. Passive foreground reads like
  // `FontSystem::version()` do not raise the foreground flag and may wait for
  // the current worker write lock to finish.
  loop {
    if cancel.load(Ordering::Relaxed) {
      return None;
    }

    if foreground_font_lock_requested() {
      thread::yield_now();
      continue;
    }

    match font_system().try_write() {
      Ok(mut font_system) => {
        if foreground_font_lock_requested() {
          drop(font_system);
          thread::yield_now();
          continue;
        }

        let budget = WorkerFontLockBudget::start();
        return f(font_system.raw(), &budget);
      }
      Err(TryLockError::WouldBlock) => {
        thread::yield_now();
      }
      Err(TryLockError::Poisoned(_)) => {
        panic!("iced shared font system lock should not be poisoned");
      }
    }
  }
}

pub(crate) struct WorkerFontLockBudget {
  started_at: Instant,
}

impl WorkerFontLockBudget {
  pub(crate) fn start() -> Self {
    Self {
      started_at: Instant::now(),
    }
  }

  pub(crate) fn exhausted(&self) -> bool {
    self.started_at.elapsed() >= WORKER_FONT_LOCK_BUDGET
  }
}
