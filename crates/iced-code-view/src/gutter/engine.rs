use std::fmt::Write as _;

use iced::advanced::graphics::text::{self, cosmic_text};

use super::{
  GutterMetrics, GutterMetricsKey, GutterMetricsRequest, GutterRenderArtifact,
  GutterRenderArtifactKey, GutterRenderArtifactRequest, MeasuredGutter,
};
use crate::cosmic_buffer::CosmicBufferPayload;
use crate::font_lock;
use crate::text_layout::VisibleTextProjection;

pub(crate) fn ensure_measured_gutter(
  request: GutterMetricsRequest<'_>,
  prev: Option<MeasuredGutter>,
) -> (GutterMetrics, Option<MeasuredGutter>) {
  if !request.gutter_config.has_visible_content() {
    return (GutterMetrics::disabled(), None);
  }

  let metrics_key =
    GutterMetricsKey::from_metrics_request(&request, font_lock::font_system_version());

  if let Some(prev) = prev
    && prev.metrics_key == metrics_key
  {
    return (prev.metrics, Some(prev));
  }

  let metrics = measure_line_number_metrics(&request);

  let measured = MeasuredGutter {
    metrics_key,
    metrics,
    render_artifact: None,
  };

  (metrics, Some(measured))
}

fn measure_line_number_metrics(request: &GutterMetricsRequest<'_>) -> GutterMetrics {
  let mut font_system = font_lock::foreground_font_system_write();
  let raw_fs = font_system.raw();

  let metrics = cosmic_text::Metrics::new(
    request.text_layout_config.font_size,
    request.text_layout_config.line_height,
  );

  let attrs = text::to_attributes(request.text_layout_config.font);
  let mut buffer = cosmic_text::Buffer::new_empty(metrics);

  buffer.set_wrap(cosmic_text::Wrap::None);

  // Digits can have different widths in proportional fonts.
  // Measure same-length digit samples and reserve the widest one.
  let label_width = widest_line_number_samples(request.document.line_count())
    .map(|sample| measure_sample_width(&mut buffer, raw_fs, &attrs, &sample))
    .fold(0.0, f32::max);

  GutterMetrics::line_numbers(label_width, &request.gutter_config)
}

fn measure_sample_width(
  buffer: &mut cosmic_text::Buffer,
  font_system: &mut cosmic_text::FontSystem,
  attrs: &cosmic_text::Attrs<'_>,
  text: &str,
) -> f32 {
  buffer.set_text(text, attrs, cosmic_text::Shaping::Basic, None);

  buffer
    .line_layout(font_system, 0)
    .expect("sample buffer contains exactly one line")
    .iter()
    .fold(0.0_f32, |max_width, line| max_width.max(line.w))
}

pub(crate) fn rebuild_render_artifact(
  request: GutterRenderArtifactRequest<'_>,
  key: GutterRenderArtifactKey,
  prev: Option<GutterRenderArtifact>,
) -> GutterRenderArtifact {
  let mut font_system = font_lock::foreground_font_system_write();
  let raw_fs = font_system.raw();

  let metrics = cosmic_text::Metrics::new(
    request.text_layout_config.font_size,
    request.text_layout_config.line_height,
  );

  let mut payload = prev.map(|artifact| artifact.payload).unwrap_or_else(|| {
    let buffer = cosmic_text::Buffer::new_empty(metrics);
    CosmicBufferPayload::new(buffer)
  });

  let buffer = payload.buffer_mut();
  let render_label_width = request
    .metrics
    .render_label_width(request.gutter_size.width);
  let buffer_height =
    request.projection.visible_rows.len() as f32 * request.text_layout_config.line_height;
  let attrs = text::to_attributes(request.text_layout_config.font);
  let labels = build_visible_line_number_text(request.projection);

  buffer.set_wrap(cosmic_text::Wrap::None);
  buffer.set_metrics_and_size(metrics, Some(render_label_width), Some(buffer_height));
  buffer.set_scroll(cosmic_text::Scroll::default());
  buffer.set_text(
    &labels,
    &attrs,
    cosmic_text::Shaping::Basic,
    Some(cosmic_text::Align::Right),
  );
  buffer.shape_until_scroll(raw_fs, false);

  // Gutter position comes from the code scroll.
  // Its own buffer scroll stays zero.
  debug_assert_eq!(buffer.scroll(), cosmic_text::Scroll::default());

  GutterRenderArtifact {
    key,
    payload,
    first_row_viewport_y: first_row_viewport_y(request.projection),
  }
}

pub(crate) fn sync_render_artifact_origin(
  artifact: &mut GutterRenderArtifact,
  projection: &VisibleTextProjection,
) {
  artifact.first_row_viewport_y = first_row_viewport_y(projection);
}

fn first_row_viewport_y(projection: &VisibleTextProjection) -> f32 {
  projection
    .visible_rows
    .first()
    .map(|row| row.viewport_y)
    .unwrap_or(0.0)
}

fn build_visible_line_number_text(projection: &VisibleTextProjection) -> String {
  let mut text = String::new();

  for (index, row) in projection.visible_rows.iter().enumerate() {
    if index > 0 {
      text.push('\n');
    }

    if row.wrap_row_index == 0 {
      let _ = write!(text, "{}", row.source_line_index + 1);
    }
  }

  text
}

fn widest_line_number_samples(source_line_count: usize) -> impl Iterator<Item = String> {
  let digit_count = source_line_count.max(1).to_string().len();

  ('0'..='9').map(move |digit| std::iter::repeat_n(digit, digit_count).collect())
}
