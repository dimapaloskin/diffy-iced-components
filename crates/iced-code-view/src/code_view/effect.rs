use crate::scroll::ScrollChange;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct Effect {
  pub(super) capture_event: bool,
  pub(super) request_redraw: bool,
  pub(super) invalidate_layout: bool,
}

impl Effect {
  pub(super) const fn none() -> Self {
    Self {
      capture_event: false,
      request_redraw: false,
      invalidate_layout: false,
    }
  }

  // TODO: consider builder API as soon as needed
  pub(super) const fn capture_event() -> Self {
    Self {
      capture_event: true,
      request_redraw: false,
      invalidate_layout: false,
    }
  }

  pub(super) fn merge(&mut self, other: Self) {
    self.capture_event |= other.capture_event;
    self.request_redraw |= other.request_redraw;
    self.invalidate_layout |= other.invalidate_layout;
  }

  pub(super) fn from_scroll_change(change: Option<ScrollChange>) -> Self {
    match change {
      Some(ScrollChange::RequiresLayout) => Self {
        invalidate_layout: true,
        request_redraw: true,
        capture_event: false,
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
  }
}
