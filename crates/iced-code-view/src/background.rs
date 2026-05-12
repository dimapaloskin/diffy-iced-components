use std::thread;

use iced::Task;
use iced::futures::channel::mpsc as async_mpsc;

pub(crate) fn spawn_optional<T>(job: impl FnOnce() -> Option<T> + Send + 'static) -> Task<T>
where
  T: Send + 'static,
{
  let (mut tx, rx) = async_mpsc::channel(1);

  thread::spawn(move || {
    if let Some(message) = job() {
      let _ = tx.try_send(message);
    }
  });

  Task::stream(rx)
}
