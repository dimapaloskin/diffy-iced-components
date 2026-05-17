use std::time::Duration;

use iced::Task;

type ElapsedMessage<T> = fn(DebounceToken) -> T;

pub struct Debouncer<T, Message> {
  delay: Duration,
  generation: u64,
  first_request: bool,
  pending: Option<Pending<T>>,
  active_timer: Option<ActiveTimer>,
  elapsed_message: ElapsedMessage<Message>,
}

struct Pending<T> {
  token: DebounceToken,
  value: T,
}

struct ActiveTimer {
  token: DebounceToken,
  handle: iced::task::Handle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DebounceToken {
  generation: u64,
}

#[must_use]
pub enum DebounceAction<T, Message> {
  StartNow(T),
  Wait(iced::Task<Message>),
}

impl<T, Message> Debouncer<T, Message> {
  pub fn new(delay: Duration, elapsed_message: ElapsedMessage<Message>) -> Self {
    Self {
      delay,
      generation: 0,
      first_request: false,
      pending: None,
      active_timer: None,
      elapsed_message,
    }
  }

  pub fn request(&mut self, value: T) -> DebounceAction<T, Message>
  where
    Message: Send + 'static,
  {
    if !self.first_request {
      self.first_request = true;
      return DebounceAction::StartNow(value);
    }

    let token = self.next_token();
    self.pending = Some(Pending { token, value });

    DebounceAction::Wait(self.schedule_timer(token))
  }

  pub fn reset(&mut self) {
    self.abort_timer();
    self.pending = None;
    self.first_request = false;
  }

  pub fn elapsed(&mut self, token: DebounceToken) -> Option<T> {
    self.clear_timer_if_current(token);

    let pending = self.pending.take()?;
    if pending.token == token {
      Some(pending.value)
    } else {
      self.pending = Some(pending);
      None
    }
  }

  fn schedule_timer(&mut self, token: DebounceToken) -> iced::Task<Message>
  where
    Message: Send + 'static,
  {
    self.abort_timer();

    let delay = self.delay;
    let elapsed_message = self.elapsed_message;

    let task = Task::perform(
      async move {
        tokio::time::sleep(delay).await;
        token
      },
      elapsed_message,
    );

    let (task, handle) = task.abortable();
    self.active_timer = Some(ActiveTimer { token, handle });

    task
  }

  fn clear_timer_if_current(&mut self, token: DebounceToken) {
    if self
      .active_timer
      .as_ref()
      .is_some_and(|active| active.token == token)
    {
      self.active_timer = None;
    }
  }

  fn next_token(&mut self) -> DebounceToken {
    self.generation = self.generation.wrapping_add(1);
    DebounceToken {
      generation: self.generation,
    }
  }

  fn abort_timer(&mut self) {
    if let Some(active_timer) = self.active_timer.take() {
      active_timer.handle.abort();
    }
  }
}

impl<T, Message> Drop for Debouncer<T, Message> {
  fn drop(&mut self) {
    self.abort_timer();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Clone, PartialEq, Eq)]
  enum TestMessage {
    Elapsed(DebounceToken),
  }

  fn elapsed_message(token: DebounceToken) -> TestMessage {
    TestMessage::Elapsed(token)
  }

  fn debouncer() -> Debouncer<i32, TestMessage> {
    Debouncer::new(Duration::from_millis(120), elapsed_message)
  }

  #[test]
  fn first_request_starts_immediately() {
    let mut debouncer = debouncer();

    match debouncer.request(1) {
      DebounceAction::StartNow(value) => assert_eq!(value, 1),
      DebounceAction::Wait(_) => panic!("first request should start immediately"),
    }
  }

  #[test]
  fn second_request_waits() {
    let mut debouncer = debouncer();

    let _ = debouncer.request(1);

    match debouncer.request(2) {
      DebounceAction::StartNow(_) => panic!("second request should wait"),
      DebounceAction::Wait(_task) => {}
    }
  }

  #[test]
  fn latest_delayed_request_wins() {
    let mut debouncer = debouncer();

    let _ = debouncer.request(1);
    let _ = debouncer.request(2);
    let stale_token = debouncer.pending.as_ref().unwrap().token;

    let _ = debouncer.request(3);
    let latest_token = debouncer.pending.as_ref().unwrap().token;

    assert_eq!(debouncer.elapsed(stale_token), None);
    assert_eq!(debouncer.elapsed(latest_token), Some(3));
  }

  #[test]
  fn stale_token_does_not_clear_latest_pending_value() {
    let mut debouncer = debouncer();

    let _ = debouncer.request(1);
    let _ = debouncer.request(2);
    let stale_token = debouncer.pending.as_ref().unwrap().token;

    let _ = debouncer.request(3);
    let latest_token = debouncer.pending.as_ref().unwrap().token;

    assert_eq!(debouncer.elapsed(stale_token), None);

    let pending = debouncer.pending.as_ref().unwrap();
    assert_eq!(pending.token, latest_token);
    assert_eq!(pending.value, 3);
  }

  #[test]
  fn reset_cycle_makes_next_request_immediate() {
    let mut debouncer = debouncer();

    let _ = debouncer.request(1);
    let _ = debouncer.request(2);

    debouncer.reset();

    match debouncer.request(3) {
      DebounceAction::StartNow(value) => assert_eq!(value, 3),
      DebounceAction::Wait(_) => panic!("request after reset should start immediately"),
    }
  }

  #[test]
  fn reset_cycle_invalidates_pending_timer_token() {
    let mut debouncer = debouncer();

    let _ = debouncer.request(1);
    let _ = debouncer.request(2);
    let token = debouncer.pending.as_ref().unwrap().token;

    debouncer.reset();

    assert_eq!(debouncer.elapsed(token), None);
  }

  #[test]
  fn reset_cycle_aborts_active_timer() {
    let mut debouncer = debouncer();

    let _ = debouncer.request(1);
    let _ = debouncer.request(2);

    let handle = debouncer.active_timer.as_ref().unwrap().handle.clone();

    debouncer.reset();

    assert!(handle.is_aborted());
  }

  #[test]
  fn drop_aborts_active_timer() {
    let handle = {
      let mut debouncer = debouncer();

      let _ = debouncer.request(1);
      let _ = debouncer.request(2);

      debouncer.active_timer.as_ref().unwrap().handle.clone()
    };

    assert!(handle.is_aborted());
  }
}
