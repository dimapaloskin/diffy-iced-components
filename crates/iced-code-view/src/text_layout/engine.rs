use iced::advanced::graphics::text::{self, cosmic_text};

use crate::cosmic_buffer::CosmicBufferPayload;
use crate::font_lock;
use crate::source_line::SourceLineHeights;
use crate::text_layout::VisibleTextLayout;
use crate::text_layout::VisibleTextProjection;
use crate::text_layout::{TextLayoutKey, TextLayoutRequest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BufferUpdateKind {
  FirstBuild,
  TextChanged,
  FontRuntimeChanged,
  FontChanged,
  GeometryOnly,
}

impl BufferUpdateKind {
  fn classify(prev_key: Option<TextLayoutKey>, new_key: TextLayoutKey) -> Self {
    let Some(prev_key) = prev_key else {
      return Self::FirstBuild;
    };

    if prev_key.document_revision != new_key.document_revision {
      return Self::TextChanged;
    }

    if prev_key.font_system_version != new_key.font_system_version {
      return Self::FontRuntimeChanged;
    }

    if prev_key.font != new_key.font {
      return Self::FontChanged;
    }

    Self::GeometryOnly
  }

  fn requires_set_text(self) -> bool {
    matches!(
      self,
      // Font changes could only update attrs.
      // For now reset text too because it is rare and avoids stale font caches.
      Self::FirstBuild | Self::TextChanged | Self::FontRuntimeChanged | Self::FontChanged
    )
  }
}

pub(crate) fn rebuild_visible_layout(
  request: TextLayoutRequest,
  key: TextLayoutKey,
  prev: Option<VisibleTextLayout>,
) -> VisibleTextLayout {
  let mut font_system = font_lock::foreground_font_system_write();
  let raw_fs = font_system.raw();

  let prev_key = prev.as_ref().map(|entry| entry.key);
  let update_kind = BufferUpdateKind::classify(prev_key, key);

  let metrics = metrics_from_request(&request);
  let mut payload = take_or_create_payload(prev, raw_fs, metrics);

  let buffer = payload.buffer_mut();

  sync_buffer_config(buffer, &request, metrics);
  if update_kind.requires_set_text() {
    sync_buffer_text(buffer, &request);
  }

  let projection = sync_buffer_scroll_and_projection(buffer, raw_fs, &request);

  VisibleTextLayout {
    key,
    projection,
    payload,
    prepared_content_height_bits: request.content_size.height.to_bits(),
    prepared_document_scroll_y_bits: request.scroll_offset.y.to_bits(),
  }
}

pub(crate) fn sync_visible_viewport(
  entry: &mut VisibleTextLayout,
  request: &TextLayoutRequest<'_>,
) {
  let content_height_bits = request.content_size.height.to_bits();
  let document_scroll_y_bits = request.scroll_offset.y.to_bits();
  let height_changed = entry.prepared_content_height_bits != content_height_bits;
  let scroll_changed = entry.prepared_document_scroll_y_bits != document_scroll_y_bits;

  if !height_changed && !scroll_changed {
    return;
  }

  let mut font_system = font_lock::foreground_font_system_write();

  let raw_fs = font_system.raw();
  let buffer = entry.payload.buffer_mut();

  if height_changed {
    sync_buffer_config(buffer, request, metrics_from_request(request));
  }

  entry.projection = sync_buffer_scroll_and_projection(buffer, raw_fs, request);
  entry.prepared_content_height_bits = content_height_bits;
  entry.prepared_document_scroll_y_bits = document_scroll_y_bits;
}

fn metrics_from_request(request: &TextLayoutRequest<'_>) -> cosmic_text::Metrics {
  cosmic_text::Metrics::new(request.config.font_size, request.config.line_height)
}

fn take_or_create_payload(
  prev: Option<VisibleTextLayout>,
  font_system: &mut cosmic_text::FontSystem,
  metrics: cosmic_text::Metrics,
) -> CosmicBufferPayload {
  prev.map(|entry| entry.payload).unwrap_or_else(|| {
    let buffer = cosmic_text::Buffer::new(font_system, metrics);

    CosmicBufferPayload::new(buffer)
  })
}

fn sync_buffer_config(
  buffer: &mut cosmic_text::Buffer,
  request: &TextLayoutRequest<'_>,
  metrics: cosmic_text::Metrics,
) {
  buffer.set_wrap(request.config.wrap_mode.to_cosmic());
  buffer.set_metrics_and_size(
    metrics,
    Some(request.content_size.width),
    Some(request.content_size.height),
  );
  buffer.set_tab_width(request.config.tab_display_policy.spaces_per_tab().into());
}

fn sync_buffer_text(buffer: &mut cosmic_text::Buffer, request: &TextLayoutRequest<'_>) {
  let attrs = text::to_attributes(request.config.font);

  buffer.set_text(
    request.document.text(),
    &attrs,
    cosmic_text::Shaping::Advanced,
    None,
  );
}

fn sync_buffer_scroll_and_projection(
  buffer: &mut cosmic_text::Buffer,
  font_system: &mut cosmic_text::FontSystem,
  request: &TextLayoutRequest<'_>,
) -> VisibleTextProjection {
  let source_line_heights = SourceLineHeights::for_request(request);
  let source_offset = source_line_heights.resolve_document_y(request.scroll_offset.y);

  buffer.set_scroll(source_offset.to_cosmic_scroll());
  buffer.shape_until_scroll(font_system, false);

  VisibleTextProjection::build(buffer, &source_line_heights, request.content_size)
}
