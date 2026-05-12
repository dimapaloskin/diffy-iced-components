use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use iced::{Element, Length, Task};

use crate::background;
use crate::code_view::{CodeView, CodeViewInputs};
use crate::document::Document;
use crate::gutter::GutterConfig;
use crate::layout::{LayoutConfig, WrapMode};
use crate::measurement::{MeasurementKey, MeasurementRequest, MeasurementResult, measure_document};
use crate::padding::CodeViewPadding;
use crate::policies::TabDisplayPolicy;
use crate::style::CodeViewStyle;

pub struct CodeViewController {
  document: Document,
  width: Length,
  height: Length,
  layout_config: LayoutConfig,
  padding: CodeViewPadding,
  border_radius: iced::border::Radius,
  session_id: u64,
  gutter_config: GutterConfig,
  style: CodeViewStyle,

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
  MeasureRequested {
    request: MeasurementRequest,
  },
  MeasurementFinished {
    session_id: u64,
    result: MeasurementResult,
  },
}

impl CodeViewController {
  pub fn new(document: Document) -> Self {
    Self {
      document,
      width: Length::Fill,
      height: Length::Fill,
      layout_config: LayoutConfig::default(),
      padding: CodeViewPadding::default(),
      border_radius: iced::border::Radius::default(),
      session_id: next_session_id(),
      gutter_config: GutterConfig::default(),
      style: CodeViewStyle::default(),

      measurement_result: None,
      active_measurement: None,
    }
  }

  pub fn with_width(mut self, width: Length) -> Self {
    self.width = width;
    self
  }

  pub fn with_height(mut self, height: Length) -> Self {
    self.height = height;
    self
  }

  pub fn with_border_radius(mut self, border_radius: iced::border::Radius) -> Self {
    self.border_radius = border_radius;
    self
  }

  pub fn with_padding(mut self, padding: CodeViewPadding) -> Self {
    self.padding = padding;
    self
  }

  pub fn with_font(mut self, font: iced::Font) -> Self {
    self.layout_config.font = font;
    self
  }

  pub fn with_font_size(mut self, font_size: f32) -> Self {
    self.layout_config.font_size = font_size;
    self
  }

  pub fn with_line_height(mut self, line_height: f32) -> Self {
    self.layout_config.line_height = line_height;
    self
  }

  pub fn with_wrap_mode(mut self, wrap_mode: WrapMode) -> Self {
    self.layout_config.wrap_mode = wrap_mode;
    self
  }

  pub fn with_tab_display_policy(mut self, tab_display_policy: TabDisplayPolicy) -> Self {
    self.layout_config.tab_display_policy = tab_display_policy;
    self
  }

  pub fn with_gutter_config(mut self, gutter_config: GutterConfig) -> Self {
    self.gutter_config = gutter_config;
    self
  }

  pub fn with_style(mut self, style: CodeViewStyle) -> Self {
    self.style = style;
    self
  }

  pub fn gutter_config(&self) -> GutterConfig {
    self.gutter_config
  }

  pub fn set_gutter_config(&mut self, gutter_config: GutterConfig) {
    self.gutter_config = gutter_config;
  }

  pub fn style(&self) -> CodeViewStyle {
    self.style
  }

  pub fn set_style(&mut self, style: CodeViewStyle) {
    self.style = style;
  }
}

impl CodeViewController {
  pub fn set_document(&mut self, document: Document) {
    self.cancel_active_measurement();

    self.document = document;
    self.session_id = next_session_id();
    self.measurement_result = None;
  }

  pub fn source_line_count(&self) -> usize {
    self.document.source_line_count()
  }

  fn cancel_active_measurement(&mut self) {
    if let Some(active) = self.active_measurement.take() {
      active.cancel();
    }
  }

  fn on_measure_requested(&mut self, request: MeasurementRequest) -> Task<CodeViewMessage> {
    if request.key.document_revision != self.document.revision() {
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

    if session_id != self.session_id || result.key.document_revision != self.document.revision() {
      return Task::none();
    }

    self.active_measurement = None;
    self.measurement_result = Some(result);

    Task::none()
  }

  pub fn update(&mut self, message: CodeViewMessage) -> Task<CodeViewMessage> {
    match message.event {
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
      gutter_config: self.gutter_config,
      style: self.style,
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

fn measure_document_task(
  session_id: u64,
  request: MeasurementRequest,
  cancel: Arc<AtomicBool>,
) -> Task<CodeViewMessage> {
  background::spawn_optional(move || {
    measure_document(request, &cancel)
      .map(|result| CodeViewMessage::measurement_finished(session_id, result))
  })
}
