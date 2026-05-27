use iced::time::Instant;

use crate::scroll::ScrollChange;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct Effect {
  pub(super) capture_event: bool,
  pub(super) request_redraw: bool,
  pub(super) invalidate_layout: bool,
  pub(super) request_redraw_at: Option<Instant>,
}

impl Effect {
  pub(super) const fn none() -> Self {
    Self {
      capture_event: false,
      request_redraw: false,
      invalidate_layout: false,
      request_redraw_at: None,
    }
  }

  // TODO: consider builder API as soon as needed
  pub(super) const fn capture_event() -> Self {
    Self {
      capture_event: true,
      request_redraw: false,
      invalidate_layout: false,
      request_redraw_at: None,
    }
  }

  pub(super) fn merge(&mut self, other: Self) {
    self.capture_event |= other.capture_event;
    self.request_redraw |= other.request_redraw;
    self.invalidate_layout |= other.invalidate_layout;
    self.request_redraw_at = match (self.request_redraw_at, other.request_redraw_at) {
      (Some(current), Some(next)) => Some(current.min(next)),
      (None, Some(next)) => Some(next),
      (current, None) => current,
    };
  }

  pub(super) fn from_scroll_change(change: Option<ScrollChange>) -> Self {
    match change {
      Some(ScrollChange::RequiresLayout) => Self {
        invalidate_layout: true,
        request_redraw: true,
        capture_event: false,
        request_redraw_at: None,
      },
      Some(ScrollChange::RedrawOnly) => Self {
        request_redraw: true,
        ..Self::none()
      },
      None => Self::none(),
    }
  }

  pub(super) fn apply<Message>(self, shell: &mut iced::advanced::Shell<'_, Message>) {
    if self.capture_event {
      shell.capture_event();
    }

    if self.invalidate_layout {
      shell.invalidate_layout();
    }

    if self.request_redraw {
      shell.request_redraw();
    }

    if let Some(at) = self.request_redraw_at {
      shell.request_redraw_at(at);
    }
  }
}
