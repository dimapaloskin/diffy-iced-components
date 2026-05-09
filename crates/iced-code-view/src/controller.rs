use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

use iced::futures::channel::mpsc as async_mpsc;
use iced::{Element, Length, Task};

use crate::code_view::CodeView;
use crate::document::CodeDocument;
use crate::layout::LayoutConfig;
use crate::{TabDisplayPolicy, WrapMode};

pub struct CodeViewController {
  document: CodeDocument,
  width: Length,
  height: Length,
  layout_config: LayoutConfig,
  padding: iced::padding::Padding,
  border_radius: iced::border::Radius,
  session_id: u64,
  opened_document: Option<OpenedDocumentState>,
}

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct CodeViewMessage {
  result: JobResult,
}

#[derive(Debug, Clone)]
enum JobResult {
  DocumentOpened { key: OpenedDocumentKey },
}

#[derive(Debug, Clone)]
struct OpenedDocumentState;

#[derive(Debug, Clone, PartialEq, Eq)]
struct OpenedDocumentKey {
  session_id: u64,
  document_id: u64,
}

impl OpenedDocumentKey {
  fn new(session_id: u64, document_id: u64) -> Self {
    Self {
      session_id,
      document_id,
    }
  }
}

impl CodeViewController {
  pub fn new(document: CodeDocument) -> Self {
    Self {
      document,
      width: Length::Fill,
      height: Length::Fill,
      layout_config: LayoutConfig::default(),
      padding: iced::padding::Padding::default(),
      border_radius: iced::border::Radius::default(),
      session_id: next_session_id(),
      opened_document: None,
    }
  }

  pub fn width(mut self, width: Length) -> Self {
    self.width = width;
    self
  }

  pub fn height(mut self, height: Length) -> Self {
    self.height = height;
    self
  }

  pub fn border_radius(mut self, border_radius: iced::border::Radius) -> Self {
    self.border_radius = border_radius;
    self
  }

  pub fn padding(mut self, padding: iced::padding::Padding) -> Self {
    self.padding = padding;
    self
  }

  pub fn font(mut self, font: iced::Font) -> Self {
    self.layout_config.font = font;
    self
  }

  pub fn font_size(mut self, font_size: f32) -> Self {
    self.layout_config.font_size = font_size;
    self
  }

  pub fn line_height(mut self, line_height: f32) -> Self {
    self.layout_config.line_height = line_height;
    self
  }

  pub fn wrap_mode(mut self, wrap_mode: WrapMode) -> Self {
    self.layout_config.wrap_mode = wrap_mode;
    self
  }

  pub fn tab_display_policy(mut self, tab_display_policy: TabDisplayPolicy) -> Self {
    self.layout_config.tab_display_policy = tab_display_policy;
    self
  }
}

impl CodeViewController {
  pub fn start(&self) -> Task<CodeViewMessage> {
    open_document_task(self.session_id, self.document.clone())
  }

  pub fn set_document(&mut self, document: CodeDocument) -> Task<CodeViewMessage> {
    self.document = document;
    self.session_id = next_session_id();
    self.opened_document = None;
    self.start()
  }

  pub fn document_id(&self) -> u64 {
    self.document.id()
  }

  pub fn source_line_count(&self) -> usize {
    self.document.source_line_count()
  }

  pub fn is_opened(&self) -> bool {
    self.opened_document.is_some()
  }

  pub fn update(&mut self, message: CodeViewMessage) -> Task<CodeViewMessage> {
    match message.result {
      JobResult::DocumentOpened { key } => {
        let current_key = OpenedDocumentKey::new(self.session_id, self.document.id());

        if current_key == key {
          self.opened_document = Some(OpenedDocumentState)
        }
      }
    }

    Task::none()
  }

  pub fn view<'a, Message, Theme, Renderer>(&self) -> Element<'a, Message, Theme, Renderer>
  where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + iced::advanced::renderer::Renderer + iced::advanced::graphics::text::Renderer,
  {
    CodeView::new(self.document.clone())
      .layout_config(self.layout_config)
      .width(self.width)
      .height(self.height)
      .border_radius(self.border_radius)
      .padding(self.padding)
      .into()
  }
}

fn next_session_id() -> u64 {
  NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed)
}

fn open_document_task(session_id: u64, document: CodeDocument) -> Task<CodeViewMessage> {
  background_job(move || {
    let key = OpenedDocumentKey::new(session_id, document.id());

    CodeViewMessage {
      result: JobResult::DocumentOpened { key },
    }
  })
}

fn background_job<T>(job: impl FnOnce() -> T + Send + 'static) -> Task<T>
where
  T: Send + 'static,
{
  let (mut tx, rx) = async_mpsc::channel(1);

  thread::spawn(move || {
    let _ = tx.try_send(job());
  });

  Task::stream(rx)
}
