use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use iced::{Element, Length, Task};

use diffy_iced_runtime::debounce::{DebounceAction, DebounceToken, Debouncer};

use crate::background;
use crate::code_view::{CodeView, CodeViewInputs};
use crate::document::Document;
use crate::gutter::GutterConfig;
use crate::insets::CodeViewInsets;
use crate::measurement::{
  MeasurementKey, MeasurementKind, MeasurementRequest, MeasurementResult, measure_document,
};
use crate::policies::TabDisplayPolicy;
use crate::style::CodeViewStyle;
use crate::text_layout::{TextLayoutConfig, WrapMode};

pub struct CodeViewController {
  document: Document,
  width: Length,
  height: Length,
  text_layout_config: TextLayoutConfig,
  insets: CodeViewInsets,
  border_radius: iced::border::Radius,
  session_id: u64,
  gutter_config: GutterConfig,
  style: CodeViewStyle,

  measurement_result: Option<MeasurementResult>,
  active_measurement: Option<ActiveMeasurementJob>,
  soft_wrap_measurement: Debouncer<MeasurementRequest, CodeViewMessage>,
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
  fn measurement_requested(request: MeasurementRequest) -> Self {
    Self {
      event: CodeViewEvent::RequestMeasurement { request },
    }
  }

  fn measurement_finished(session_id: u64, result: MeasurementResult) -> Self {
    Self {
      event: CodeViewEvent::FinishMeasurement { session_id, result },
    }
  }

  fn measurement_debounce_elapsed(token: DebounceToken) -> Self {
    Self {
      event: CodeViewEvent::MeasurementDebounceElapsed { token },
    }
  }
}

#[derive(Debug, Clone)]
enum CodeViewEvent {
  RequestMeasurement {
    request: MeasurementRequest,
  },
  FinishMeasurement {
    session_id: u64,
    result: MeasurementResult,
  },
  MeasurementDebounceElapsed {
    token: DebounceToken,
  },
}

impl CodeViewController {
  pub fn new(document: Document) -> Self {
    Self {
      document,
      width: Length::Fill,
      height: Length::Fill,
      text_layout_config: TextLayoutConfig::default(),
      insets: CodeViewInsets::default(),
      border_radius: iced::border::Radius::default(),
      session_id: next_session_id(),
      gutter_config: GutterConfig::default(),
      style: CodeViewStyle::default(),

      measurement_result: None,
      active_measurement: None,
      soft_wrap_measurement: Debouncer::new(
        Duration::from_millis(100),
        CodeViewMessage::measurement_debounce_elapsed,
      ),
    }
  }

  pub fn with_width(mut self, width: Length) -> Self {
    self.width = width;
    self
  }

  pub fn set_width(&mut self, width: Length) {
    if self.width == width {
      return;
    }

    self.width = width;
  }

  pub fn width(&self) -> Length {
    self.width
  }

  pub fn with_height(mut self, height: Length) -> Self {
    self.height = height;
    self
  }

  pub fn set_height(&mut self, height: Length) {
    if self.height == height {
      return;
    }

    self.height = height;
  }

  pub fn height(&self) -> Length {
    self.height
  }

  pub fn with_border_radius(mut self, border_radius: iced::border::Radius) -> Self {
    self.border_radius = border_radius;
    self
  }

  pub fn set_border_radius(&mut self, border_radius: iced::border::Radius) {
    if self.border_radius == border_radius {
      return;
    }

    self.border_radius = border_radius;
  }

  pub fn border_radius(&self) -> iced::border::Radius {
    self.border_radius
  }

  pub fn with_insets(mut self, insets: CodeViewInsets) -> Self {
    self.insets = insets;
    self
  }

  pub fn set_insets(&mut self, insets: CodeViewInsets) {
    if self.insets == insets {
      return;
    }

    self.insets = insets;
  }

  pub fn insets(&self) -> CodeViewInsets {
    self.insets
  }

  pub fn with_font(mut self, font: iced::Font) -> Self {
    self.text_layout_config.font = font;
    self
  }

  pub fn set_font(&mut self, font: iced::Font) {
    if self.text_layout_config.font == font {
      return;
    }

    self.text_layout_config.font = font;
    self.soft_wrap_measurement.reset();
  }

  pub fn font(&self) -> iced::Font {
    self.text_layout_config.font
  }

  pub fn with_font_size(mut self, font_size: f32) -> Self {
    self.text_layout_config.font_size = font_size;
    self
  }

  pub fn set_font_size(&mut self, font_size: f32) {
    if self.text_layout_config.font_size == font_size {
      return;
    }

    self.text_layout_config.font_size = font_size;
    self.soft_wrap_measurement.reset();
  }

  pub fn font_size(&self) -> f32 {
    self.text_layout_config.font_size
  }

  pub fn with_line_height(mut self, line_height: f32) -> Self {
    self.text_layout_config.line_height = line_height;
    self
  }

  pub fn set_line_height(&mut self, line_height: f32) {
    if self.text_layout_config.line_height == line_height {
      return;
    }

    self.text_layout_config.line_height = line_height;
    self.soft_wrap_measurement.reset();
  }

  pub fn line_height(&self) -> f32 {
    self.text_layout_config.line_height
  }

  pub fn with_wrap_mode(mut self, wrap_mode: WrapMode) -> Self {
    self.text_layout_config.wrap_mode = wrap_mode;
    self
  }

  pub fn set_wrap_mode(&mut self, wrap_mode: WrapMode) {
    if self.text_layout_config.wrap_mode == wrap_mode {
      return;
    }

    self.text_layout_config.wrap_mode = wrap_mode;
    self.soft_wrap_measurement.reset();
  }

  pub fn wrap_mode(&self) -> WrapMode {
    self.text_layout_config.wrap_mode
  }

  pub fn with_tab_display_policy(mut self, tab_display_policy: TabDisplayPolicy) -> Self {
    self.text_layout_config.tab_display_policy = tab_display_policy;
    self
  }

  pub fn set_tab_display_policy(&mut self, tab_display_policy: TabDisplayPolicy) {
    if self.text_layout_config.tab_display_policy == tab_display_policy {
      return;
    }

    self.text_layout_config.tab_display_policy = tab_display_policy;
    self.soft_wrap_measurement.reset();
  }

  pub fn tab_display_policy(&self) -> TabDisplayPolicy {
    self.text_layout_config.tab_display_policy
  }

  pub fn with_gutter_config(mut self, gutter_config: GutterConfig) -> Self {
    self.gutter_config = gutter_config;
    self
  }

  pub fn set_gutter_config(&mut self, gutter_config: GutterConfig) {
    if self.gutter_config == gutter_config {
      return;
    }

    self.gutter_config = gutter_config;
  }

  pub fn gutter_config(&self) -> GutterConfig {
    self.gutter_config
  }

  pub fn with_style(mut self, style: CodeViewStyle) -> Self {
    self.style = style;
    self
  }

  pub fn set_style(&mut self, style: CodeViewStyle) {
    if self.style == style {
      return;
    }

    self.style = style;
  }

  pub fn style(&self) -> CodeViewStyle {
    self.style
  }
}

impl CodeViewController {
  pub fn set_document(&mut self, document: Document) {
    self.cancel_active_measurement();

    self.measurement_result = None;
    self.soft_wrap_measurement.reset();

    self.document = document;
    self.session_id = next_session_id();
  }

  pub fn line_count(&self) -> usize {
    self.document.line_count()
  }

  fn cancel_active_measurement(&mut self) {
    if let Some(active) = self.active_measurement.take() {
      active.cancel();
    }
  }

  fn on_measurement_requested(&mut self, request: MeasurementRequest) -> Task<CodeViewMessage> {
    if request.key.document_revision != self.document.revision() {
      return Task::none();
    }

    if !request.key.kind.needs_background_worker() {
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

    match request.key.kind {
      MeasurementKind::NoWrapHorizontalExtent => self.start_measurement(request),
      MeasurementKind::SoftWrapLineHeights { .. } => self.schedule_soft_wrap_measurement(request),
    }
  }

  fn start_measurement(&mut self, request: MeasurementRequest) -> Task<CodeViewMessage> {
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

  fn schedule_soft_wrap_measurement(
    &mut self,
    request: MeasurementRequest,
  ) -> Task<CodeViewMessage> {
    match self.soft_wrap_measurement.request(request) {
      DebounceAction::StartNow(request) => self.start_measurement(request),
      DebounceAction::Wait(task) => {
        self.cancel_active_measurement();
        task
      }
    }
  }

  fn on_measurement_debounce_elapsed(&mut self, token: DebounceToken) -> Task<CodeViewMessage> {
    let Some(request) = self.soft_wrap_measurement.elapsed(token) else {
      return Task::none();
    };

    if request.key.document_revision != self.document.revision() {
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

    self.start_measurement(request)
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
      CodeViewEvent::RequestMeasurement { request } => self.on_measurement_requested(request),
      CodeViewEvent::FinishMeasurement { session_id, result } => {
        self.on_measurement_finished(session_id, result)
      }
      CodeViewEvent::MeasurementDebounceElapsed { token } => {
        self.on_measurement_debounce_elapsed(token)
      }
    }
  }

  fn widget_inputs(&self) -> CodeViewInputs<'_> {
    CodeViewInputs {
      document: &self.document,
      width: self.width,
      height: self.height,
      text_layout_config: self.text_layout_config,
      insets: self.insets,
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
    CodeView::new(self.widget_inputs(), CodeViewMessage::measurement_requested).into()
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
