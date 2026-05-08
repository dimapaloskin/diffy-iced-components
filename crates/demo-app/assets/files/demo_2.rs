use std::sync::Arc;

use iced::Element;
use iced::Length;
use iced::advanced::graphics::text;
use iced::advanced::graphics::text::Renderer as TextRendererTrait;
use iced::advanced::widget;
use iced::advanced::{layout, renderer::Renderer as RendererTrait, widget::Widget};

use crate::document::CodeDocument;
use crate::layout::LayoutKey;
use crate::layout::LayoutRequest;
use crate::layout::WrapMode;
use crate::layout_engine;
use crate::policies::TabDisplayPolicy;
use crate::scroll::ScrollExtent;
use crate::state::CodeViewState;
use crate::viewport::Viewport;

pub struct CodeView {
  document: CodeDocument,
  width: Length,
  height: Length,
  font: iced::Font,
  font_size: f32,
  line_height: f32,
  wrap_mode: WrapMode,
  tab_display_policy: TabDisplayPolicy,
  padding: iced::padding::Padding,
  border_radius: iced::border::Radius,
}

impl CodeView {
  pub fn new(document: CodeDocument) -> Self {
    Self {
      document,
      width: Length::Fill,
      height: Length::Fill,
      font: iced::Font::MONOSPACE,
      font_size: 16.0,
      line_height: 24.0,
      wrap_mode: WrapMode::default(),
      tab_display_policy: TabDisplayPolicy::default(),
      padding: iced::padding::Padding::default(),
      border_radius: iced::border::Radius::default(),
    }
  }

  pub fn border_radius(mut self, border_radius: iced::border::Radius) -> Self {
    self.border_radius = border_radius;
    self
  }

  pub fn width(mut self, width: Length) -> Self {
    self.width = width;
    self
  }

  pub fn height(mut self, height: Length) -> Self {
    self.height = height;
    self
  }

  pub fn font(mut self, font: iced::Font) -> Self {
    self.font = font;
    self
  }

  pub fn font_size(mut self, font_size: f32) -> Self {
    self.font_size = font_size;
    self
  }

  pub fn line_height(mut self, line_height: f32) -> Self {
    self.line_height = line_height;
    self
  }

  pub fn padding(mut self, padding: iced::padding::Padding) -> Self {
    self.padding = padding;
    self
  }

  pub fn wrap_mode(mut self, wrap_mode: WrapMode) -> Self {
    self.wrap_mode = wrap_mode;
    self
  }

  pub fn tab_display_policy(mut self, tab_display_policy: TabDisplayPolicy) -> Self {
    self.tab_display_policy = tab_display_policy;
    self
  }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for CodeView
where
  Renderer: RendererTrait + TextRendererTrait,
{
  fn size(&self) -> iced::Size<iced::Length> {
    iced::Size::new(self.width, self.height)
  }

  fn tag(&self) -> iced::advanced::widget::tree::Tag {
    widget::tree::Tag::of::<CodeViewState>()
  }

  fn state(&self) -> widget::tree::State {
    widget::tree::State::new(CodeViewState::default())
  }

  fn update(
    &mut self,
    tree: &mut widget::Tree,
    event: &iced::Event,
    layout: layout::Layout<'_>,
    cursor: iced::advanced::mouse::Cursor,
    _renderer: &Renderer,
    shell: &mut iced::advanced::Shell<'_, Message>,
    _viewport: &iced::Rectangle,
  ) {
    use iced::mouse::{Event as MouseEvent, ScrollDelta};

    // Handle wheel over the whole CodeView area, including padding.
    // If text-only behavior is needed later, narrow this to the content bounds
    if !cursor.is_over(layout.bounds()) {
      return;
    }

    let iced::Event::Mouse(MouseEvent::WheelScrolled { delta }) = event else {
      return;
    };

    // Stop wheel events at CodeView, so scrolling does not chain to a parent at edges.
    // For web-like scroll chaining, move this inside the `state.viewport.scroll_offset != old` check
    shell.capture_event();

    let delta = match delta {
      ScrollDelta::Pixels { x, y } => [*x, *y],
      ScrollDelta::Lines { x, y } => {
        let step = self.line_height * 3.0;
        [*x * step, *y * step]
      }
    };

    let state = tree.state.downcast_mut::<CodeViewState>();

    let old = state.viewport.scroll_offset;

    let candidate = iced::Vector::new(old.x - delta[0], old.y - delta[1]);
    state.viewport.scroll_offset = state
      .scroll_extent
      .clamp_offset(candidate, state.viewport.content_bounds.size());

    if state.viewport.scroll_offset != old {
      if let Some(entry) = &mut state.layout_entry {
        layout_engine::sync_scroll(entry, state.viewport.scroll_offset);
      }

      shell.request_redraw();
    }
  }

  fn layout(
    &mut self,
    tree: &mut iced::advanced::widget::Tree,
    _renderer: &Renderer,
    limits: &iced::advanced::layout::Limits,
  ) -> layout::Node {
    let state = tree.state.downcast_mut::<CodeViewState>();
    let resolved_size = limits.resolve(self.width, self.height, iced::Size::ZERO);
    let viewport = Viewport::new(resolved_size, self.padding, state.viewport.scroll_offset);
    let scroll_extent =
      ScrollExtent::for_document(&self.document, self.wrap_mode, self.line_height);
    let scroll_offset =
      scroll_extent.clamp_offset(viewport.scroll_offset, viewport.content_bounds.size());
    let viewport = viewport.with_scroll_offset(scroll_offset);
    let previous = state.layout_entry.take();

    let layout_request = LayoutRequest {
      document: &self.document,
      content_size: viewport.content_bounds.size(),
      scroll_offset: viewport.scroll_offset,
      font: self.font,
      font_size: self.font_size,
      line_height: self.line_height,
      wrap_mode: self.wrap_mode,
      tab_policy: self.tab_display_policy,
    };

    let key = LayoutKey::from_request(&layout_request);

    let needs_rebuild = previous.as_ref().is_none_or(|p| p.key != key);
    let needs_scroll_sync = previous
      .as_ref()
      .is_some_and(|p| p.prepared_scroll_offset != viewport.scroll_offset);

    state.layout_entry = if needs_rebuild || needs_scroll_sync {
      Some(layout_engine::rebuild_layout(layout_request, previous))
    } else {
      previous
    };

    state.scroll_extent = scroll_extent;
    state.viewport = viewport;

    layout::Node::new(resolved_size)
  }

  fn draw(
    &self,
    tree: &iced::advanced::widget::Tree,
    renderer: &mut Renderer,
    _theme: &Theme,
    _style: &iced::advanced::renderer::Style,
    layout: iced::advanced::Layout<'_>,
    _cursor: iced::advanced::mouse::Cursor,
    viewport: &iced::Rectangle,
  ) {
    use iced::advanced::renderer::Quad;

    let state = tree.state.downcast_ref::<CodeViewState>();

    let quad = Quad {
      bounds: layout.bounds(),
      border: iced::Border {
        color: iced::Color::TRANSPARENT,
        width: 0.0,
        radius: self.border_radius,
      },
      ..Quad::default()
    };

    renderer.fill_quad(quad, iced::Color::BLACK);

    let bounds = layout.bounds();
    let content_bounds = state.viewport.absolute_content_bounds(bounds);
    if let (Some(entry), Some(clip_bounds)) =
      (&state.layout_entry, content_bounds.intersection(viewport))
    {
      let position = iced::Point::new(
        content_bounds.x - state.viewport.scroll_offset.x,
        content_bounds.y,
      );

      renderer.fill_raw(text::Raw {
        buffer: Arc::downgrade(entry.payload.buffer()),
        position,
        color: iced::Color::WHITE,
        clip_bounds,
      });
    }
  }
}

impl<'a, Message, Theme, Renderer> From<CodeView> for Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Theme: 'a,
  Renderer: 'a + RendererTrait + TextRendererTrait,
{
  fn from(code_view: CodeView) -> Self {
    Self::new(code_view)
  }
}

use iced::advanced::graphics::text::{self, cosmic_text};

use crate::layout::{LayoutKey, LayoutRequest};
use crate::state::{CosmicLayoutPayload, LayoutCacheEntry, LayoutSnapshot, VisualLineSnapshot};

pub(crate) fn rebuild_layout(
  request: LayoutRequest,
  previous: Option<LayoutCacheEntry>,
) -> LayoutCacheEntry {
  let mut font_system = text::font_system()
    .write()
    .expect("iced shared font system lock should not be poisoned");

  let needs_set_text = previous
    .as_ref()
    .is_none_or(|entry| entry.key.text_revision != request.document.id());

  let needs_update_attrs = previous.as_ref().is_some_and(|entry| {
    entry.key.text_revision == request.document.id() && entry.key.font != request.font
  });

  let raw_font_system = font_system.raw();
  let metrics = cosmic_text::Metrics::new(request.font_size, request.line_height);

  let mut payload = previous.map(|entry| entry.payload).unwrap_or_else(|| {
    let buffer = cosmic_text::Buffer::new(raw_font_system, metrics);

    CosmicLayoutPayload::new(buffer)
  });

  let buffer = payload.buffer_mut();

  buffer.set_wrap(request.wrap_mode.to_cosmic());
  buffer.set_metrics_and_size(
    metrics,
    Some(request.content_size.width),
    Some(request.content_size.height),
  );
  buffer.set_tab_width(request.tab_policy.spaces_per_tab().into());

  let attrs = text::to_attributes(request.font);

  if needs_set_text {
    buffer.set_text(
      request.document.text(),
      &attrs,
      cosmic_text::Shaping::Advanced,
      None,
    );
  } else if needs_update_attrs {
    update_plain_attrs(buffer, &attrs);
  }

  let snapshot = sync_buffer_scroll_and_snapshot(buffer, raw_font_system, request.scroll_offset);

  let key = LayoutKey::from_request(&request);

  LayoutCacheEntry {
    key,
    snapshot,
    payload,
    prepared_scroll_offset: request.scroll_offset,
  }
}

pub(crate) fn sync_scroll(entry: &mut LayoutCacheEntry, scroll_offset: iced::Vector) {
  if entry.prepared_scroll_offset == scroll_offset {
    return;
  }

  let mut font_system = text::font_system()
    .write()
    .expect("iced shared font system lock should not be poisoned");

  let raw_font_system = font_system.raw();
  let buffer = entry.payload.buffer_mut();

  entry.snapshot = sync_buffer_scroll_and_snapshot(buffer, raw_font_system, scroll_offset);
  entry.prepared_scroll_offset = scroll_offset;
}

fn sync_buffer_scroll_and_snapshot(
  buffer: &mut cosmic_text::Buffer,
  font_system: &mut cosmic_text::FontSystem,
  scroll_offset: iced::Vector,
) -> LayoutSnapshot {
  // `layout_runs()` accounts for `Scroll::vertical` when choosing visible lines,
  // but glyph `x` positions stay relative to the start of each line.
  // Keep horizontal at zero here and apply `scroll_offset.x` in draw translation.
  buffer.set_scroll(cosmic_text::Scroll::new(0, scroll_offset.y, 0.0));
  buffer.shape_until_scroll(font_system, false);

  snapshot_from_buffer(buffer)
}

fn snapshot_from_buffer(buffer: &cosmic_text::Buffer) -> LayoutSnapshot {
  let mut text_width: f32 = 0.0;
  let mut text_height: f32 = 0.0;
  let mut visual_lines = Vec::new();

  for run in buffer.layout_runs() {
    text_width = text_width.max(run.line_w);
    text_height += run.line_height;

    visual_lines.push(VisualLineSnapshot {
      source_line_index: run.line_i,
      y: run.line_top,
      height: run.line_height,
      width: run.line_w,
    });
  }

  LayoutSnapshot {
    text_size: iced::Size::new(text_width, text_height),
    visual_lines,
  }
}

fn update_plain_attrs(buffer: &mut cosmic_text::Buffer, attrs: &cosmic_text::Attrs<'_>) {
  for line in &mut buffer.lines {
    line.set_attrs_list(cosmic_text::AttrsList::new(attrs));
  }
}

use std::sync::atomic::{AtomicU64, Ordering};

use iced::{Element, Task};

use crate::code_view::CodeView;
use crate::document::CodeDocument;

pub struct CodeViewController {
  document: CodeDocument,
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
  DocumentOpened {
    session_id: u64,
    document_id: u64,
    source_line_count: usize,
  },
}

#[derive(Debug, Clone)]
struct OpenedDocumentState {
  document_id: u64,
  source_line_count: usize,
}

impl CodeViewController {
  pub fn new(document: CodeDocument) -> Self {
    Self {
      document,
      padding: iced::padding::Padding::default(),
      border_radius: iced::border::Radius::default(),
      session_id: next_session_id(),
      opened_document: None,
    }
  }

  pub fn border_radius(mut self, border_radius: iced::border::Radius) -> Self {
    self.border_radius = border_radius;
    self
  }

  pub fn padding(mut self, padding: iced::padding::Padding) -> Self {
    self.padding = padding;
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

  pub fn update(&mut self, message: CodeViewMessage) -> Task<CodeViewMessage> {
    match message.result {
      JobResult::DocumentOpened {
        session_id,
        document_id,
        source_line_count,
      } => {
        if session_id == self.session_id && document_id == self.document.id() {
          self.opened_document = Some(OpenedDocumentState {
            document_id,
            source_line_count,
          })
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
      .border_radius(self.border_radius)
      .padding(self.padding)
      .into()
  }
}

fn next_session_id() -> u64 {
  NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed)
}

fn open_document_task(session_id: u64, document: CodeDocument) -> Task<CodeViewMessage> {
  Task::done(CodeViewMessage {
    result: JobResult::DocumentOpened {
      session_id,
      document_id: document.id(),
      source_line_count: document.source_line_count(),
    },
  })
}

use std::sync::Arc;

use iced::advanced::graphics::text::cosmic_text;

use crate::layout::LayoutKey;
use crate::scroll::ScrollExtent;
use crate::viewport::Viewport;

#[derive(Default)]
pub(crate) struct CodeViewState {
  pub(crate) layout_entry: Option<LayoutCacheEntry>,
  pub(crate) viewport: Viewport,
  pub(crate) scroll_extent: ScrollExtent,
}

pub(crate) struct LayoutCacheEntry {
  pub(crate) key: LayoutKey,
  pub(crate) snapshot: LayoutSnapshot,
  pub(crate) payload: CosmicLayoutPayload,
  // The real scroll offset lives in Viewport.
  // This is just the offset already applied to this buffer/snapshot.
  pub(crate) prepared_scroll_offset: iced::Vector,
}

pub(crate) struct LayoutSnapshot {
  pub(crate) text_size: iced::Size,
  pub(crate) visual_lines: Vec<VisualLineSnapshot>,
}

pub(crate) struct VisualLineSnapshot {
  pub(crate) source_line_index: usize,
  pub(crate) y: f32,
  pub(crate) height: f32,
  pub(crate) width: f32,
}

pub(crate) struct CosmicLayoutPayload {
  buffer: Arc<cosmic_text::Buffer>,
}

impl CosmicLayoutPayload {
  pub(crate) fn new(buffer: cosmic_text::Buffer) -> Self {
    Self {
      buffer: Arc::new(buffer),
    }
  }

  pub(crate) fn buffer(&self) -> &Arc<cosmic_text::Buffer> {
    &self.buffer
  }

  pub(crate) fn buffer_mut(&mut self) -> &mut cosmic_text::Buffer {
    Arc::make_mut(&mut self.buffer)
  }
}
