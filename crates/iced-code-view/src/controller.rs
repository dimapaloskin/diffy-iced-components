use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;

use iced::futures::channel::mpsc as async_mpsc;
use iced::{Element, Length, Task};

use crate::code_view::{CodeView, CodeViewInputs};
use crate::document::CodeDocument;
use crate::layout::LayoutConfig;
use crate::measurement::{MeasurementKey, MeasurementRequest, MeasurementResult, measure_document};
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
  measurement_result: Option<MeasurementResult>,
  active_measurement: Option<ActiveMeasurementJob>,
}

struct ActiveMeasurementJob {
  session_id: u64,
  key: MeasurementKey,
  cancel: Arc<AtomicBool>,
}

impl ActiveMeasurementJob {
  fn cancel(self) {
    self.cancel.store(true, Ordering::Relaxed);
  }
}

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct CodeViewMessage {
  event: CodeViewEvent,
}

impl CodeViewMessage {
  fn document_opened(key: OpenedDocumentKey) -> Self {
    Self {
      event: CodeViewEvent::DocumentOpened { key },
    }
  }

  fn measure_requested(request: MeasurementRequest) -> Self {
    Self {
      event: CodeViewEvent::MeasureRequested { request },
    }
  }

  fn measurement_finished(session_id: u64, result: MeasurementResult) -> Self {
    Self {
      event: CodeViewEvent::MeasurementFinished { session_id, result },
    }
  }
}

#[derive(Debug, Clone)]
enum CodeViewEvent {
  DocumentOpened {
    key: OpenedDocumentKey,
  },
  MeasureRequested {
    request: MeasurementRequest,
  },
  MeasurementFinished {
    session_id: u64,
    result: MeasurementResult,
  },
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

      measurement_result: None,
      active_measurement: None,
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
    self.cancel_active_measurement();

    self.document = document;
    self.session_id = next_session_id();
    self.opened_document = None;
    self.measurement_result = None;

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

  fn cancel_active_measurement(&mut self) {
    if let Some(active) = self.active_measurement.take() {
      active.cancel();
    }
  }

  fn on_measure_requested(&mut self, request: MeasurementRequest) -> Task<CodeViewMessage> {
    if request.key.document_id != self.document.id() {
      return Task::none();
    }

    if !request.key.mode.needs_background_worker() {
      return Task::none();
    }

    if self
      .measurement_result
      .as_ref()
      .is_some_and(|result| result.key == request.key)
    {
      return Task::none();
    }

    if self
      .active_measurement
      .as_ref()
      .is_some_and(|active| active.session_id == self.session_id && active.key == request.key)
    {
      return Task::none();
    }

    self.cancel_active_measurement();

    let session_id = self.session_id;
    let key = request.key;
    let cancel = Arc::new(AtomicBool::new(false));

    self.active_measurement = Some(ActiveMeasurementJob {
      session_id,
      key,
      cancel: Arc::clone(&cancel),
    });

    measure_document_task(session_id, request, cancel)
  }

  fn on_measurement_finished(
    &mut self,
    session_id: u64,
    result: MeasurementResult,
  ) -> Task<CodeViewMessage> {
    let Some(active) = self.active_measurement.as_ref() else {
      return Task::none();
    };

    if active.session_id != session_id || active.key != result.key {
      return Task::none();
    }

    if session_id != self.session_id || result.key.document_id != self.document.id() {
      return Task::none();
    }

    self.active_measurement = None;
    self.measurement_result = Some(result);

    Task::none()
  }

  pub fn update(&mut self, message: CodeViewMessage) -> Task<CodeViewMessage> {
    match message.event {
      CodeViewEvent::DocumentOpened { key } => {
        let current_key = OpenedDocumentKey::new(self.session_id, self.document.id());

        if current_key == key {
          self.opened_document = Some(OpenedDocumentState)
        }

        Task::none()
      }
      CodeViewEvent::MeasureRequested { request } => self.on_measure_requested(request),
      CodeViewEvent::MeasurementFinished { session_id, result } => {
        self.on_measurement_finished(session_id, result)
      }
    }
  }

  fn widget_inputs(&self) -> CodeViewInputs<'_> {
    CodeViewInputs {
      document: &self.document,
      width: self.width,
      height: self.height,
      layout_config: self.layout_config,
      padding: self.padding,
      border_radius: self.border_radius,
      measurement_result: self.measurement_result.as_ref(),
    }
  }

  pub fn view<'a, Theme, Renderer>(&'a self) -> Element<'a, CodeViewMessage, Theme, Renderer>
  where
    Theme: 'a,
    Renderer: 'a + iced::advanced::renderer::Renderer + iced::advanced::graphics::text::Renderer,
  {
    CodeView::new(self.widget_inputs(), CodeViewMessage::measure_requested).into()
  }
}

fn next_session_id() -> u64 {
  NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed)
}

fn open_document_task(session_id: u64, document: CodeDocument) -> Task<CodeViewMessage> {
  background_optional_job(move || {
    let key = OpenedDocumentKey::new(session_id, document.id());
    Some(CodeViewMessage::document_opened(key))
  })
}

fn background_optional_job<T>(job: impl FnOnce() -> Option<T> + Send + 'static) -> Task<T>
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

fn measure_document_task(
  session_id: u64,
  request: MeasurementRequest,
  cancel: Arc<AtomicBool>,
) -> Task<CodeViewMessage> {
  background_optional_job(move || {
    measure_document(request, &cancel)
      .map(|result| CodeViewMessage::measurement_finished(session_id, result))
  })
}
