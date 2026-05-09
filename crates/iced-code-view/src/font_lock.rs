use std::sync::RwLockWriteGuard;
use std::sync::atomic::{AtomicBool, Ordering};

use iced::advanced::graphics::text::{self, FontSystem, font_system};

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
