pub(crate) mod engine;

use std::ops::RangeInclusive;

use iced::advanced::graphics::text;

use crate::cosmic_buffer::CosmicBufferPayload;
use crate::document::Document;
use crate::padding::GutterPadding;
use crate::text_layout::TextLayoutConfig;
use crate::text_layout::VisibleTextProjection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GutterVisibility {
  #[default]
  Visible,
  Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GutterContent {
  pub line_numbers: bool,
}

impl GutterContent {
  pub(crate) fn is_empty(self) -> bool {
    !self.line_numbers
  }
}

impl Default for GutterContent {
  fn default() -> Self {
    Self { line_numbers: true }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GutterConfig {
  pub visibility: GutterVisibility,
  pub content: GutterContent,
  pub padding: GutterPadding,
  pub separator_width: f32,
}

impl GutterConfig {
  pub(crate) fn has_visible_content(self) -> bool {
    self.visibility == GutterVisibility::Visible && !self.content.is_empty()
  }
}

impl Default for GutterConfig {
  fn default() -> Self {
    Self {
      visibility: GutterVisibility::Visible,
      content: GutterContent::default(),
      padding: GutterPadding::default(),
      separator_width: 1.0,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GutterMetrics {
  pub(crate) enabled: bool,
  pub(crate) requested_width: f32,
  pub(crate) padding: GutterPadding,
  pub(crate) separator_width: f32,
}

impl GutterMetrics {
  pub(crate) fn disabled() -> Self {
    Self {
      enabled: false,
      requested_width: 0.0,
      padding: GutterPadding::ZERO,
      separator_width: 0.0,
    }
  }

  pub(crate) fn line_numbers(label_width: f32, gutter_config: &GutterConfig) -> Self {
    Self {
      enabled: true,
      requested_width: gutter_config.padding.horizontal.left
        + label_width
        + gutter_config.padding.horizontal.right
        + gutter_config.separator_width,
      padding: gutter_config.padding,
      separator_width: gutter_config.separator_width,
    }
  }

  pub(crate) fn visible_separator_width(&self, gutter_width: f32) -> f32 {
    if !self.enabled || gutter_width <= 0.0 {
      return 0.0;
    }

    self.separator_width.min(gutter_width)
  }

  pub(crate) fn render_label_width(&self, gutter_width: f32) -> f32 {
    let separator_width = self.visible_separator_width(gutter_width);
    (gutter_width - self.padding.horizontal.left - self.padding.horizontal.right - separator_width)
      .max(0.0)
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GutterMetricsKey {
  pub(crate) source_line_count: usize,
  pub(crate) text_font: iced::Font,
  pub(crate) font_size_bits: u32,
  pub(crate) padding_bits: u64,
  pub(crate) separator_width_bits: u32,
  pub(crate) font_system_version: text::Version,
}

impl GutterMetricsKey {
  pub(crate) fn from_metrics_request(
    request: &GutterMetricsRequest<'_>,
    font_system_version: text::Version,
  ) -> Self {
    Self {
      source_line_count: request.document.line_count(),
      text_font: request.text_layout_config.font,
      font_size_bits: request.text_layout_config.font_size.to_bits(),
      padding_bits: request.gutter_config.padding.to_bits(),
      separator_width_bits: request.gutter_config.separator_width.to_bits(),
      font_system_version,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GutterRenderArtifactKey {
  pub(crate) metrics_key: GutterMetricsKey,
  pub(crate) render_label_width_bits: u32,
  pub(crate) line_height_bits: u32,
  pub(crate) rows_signature: GutterRowsSignature,
}

impl GutterRenderArtifactKey {
  pub(crate) fn for_request(
    request: &GutterRenderArtifactRequest<'_>,
    metrics_key: GutterMetricsKey,
  ) -> Option<Self> {
    if !request.metrics.enabled || request.gutter_size.height <= 0.0 {
      return None;
    }

    let render_label_width = request
      .metrics
      .render_label_width(request.gutter_size.width);

    if render_label_width <= 0.0 {
      return None;
    }

    Some(Self {
      metrics_key,
      render_label_width_bits: render_label_width.to_bits(),
      line_height_bits: request.text_layout_config.line_height.to_bits(),
      rows_signature: GutterRowsSignature::from_projection(request.projection)?,
    })
  }
}

// NoWrap can use compact ranges.
// SoftWrap will need a sequence/hash signature of visible row identities.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GutterRowsSignature {
  pub(crate) source_lines: RangeInclusive<usize>,
  pub(crate) wrap_rows: RangeInclusive<usize>,
  pub(crate) row_count: usize,
}

impl GutterRowsSignature {
  pub(crate) fn from_projection(projection: &VisibleTextProjection) -> Option<Self> {
    let first = projection.visible_rows.first()?;
    let last = projection
      .visible_rows
      .last()
      .expect("first visible row exists");

    Some(Self {
      source_lines: first.source_line_index..=last.source_line_index,
      wrap_rows: first.wrap_row_index..=last.wrap_row_index,
      row_count: projection.visible_rows.len(),
    })
  }
}

pub(crate) struct GutterRenderArtifact {
  pub(crate) key: GutterRenderArtifactKey,
  pub(crate) payload: CosmicBufferPayload,
  pub(crate) first_row_viewport_y: f32,
}

pub(crate) struct MeasuredGutter {
  pub(crate) metrics_key: GutterMetricsKey,
  pub(crate) metrics: GutterMetrics,
  pub(crate) render_artifact: Option<GutterRenderArtifact>,
}

pub(crate) struct GutterMetricsRequest<'a> {
  pub(crate) document: &'a Document,
  pub(crate) text_layout_config: TextLayoutConfig,
  pub(crate) gutter_config: GutterConfig,
}

pub(crate) struct GutterRenderArtifactRequest<'a> {
  pub(crate) text_layout_config: TextLayoutConfig,
  pub(crate) metrics: GutterMetrics,
  pub(crate) gutter_size: iced::Size,
  pub(crate) projection: &'a VisibleTextProjection,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::font_lock;
  use crate::padding::HorizontalPadding;
  use crate::text_layout::projection::VisibleTextRow;

  fn projection(rows: &[(usize, usize, f32)]) -> VisibleTextProjection {
    VisibleTextProjection {
      visible_text_size: iced::Size::new(80.0, rows.len() as f32 * 24.0),
      visible_rows: rows
        .iter()
        .map(
          |&(source_line_index, wrap_row_index, viewport_y)| VisibleTextRow {
            source_line_index,
            wrap_row_index,
            y_inside_source_line: wrap_row_index as f32 * 24.0,
            document_y: source_line_index as f32 * 24.0 + wrap_row_index as f32 * 24.0,
            viewport_y,
            height: 24.0,
            width: 80.0,
          },
        )
        .collect(),
    }
  }

  fn metrics_key_for(
    document: &Document,
    text_layout_config: TextLayoutConfig,
    gutter_config: GutterConfig,
  ) -> GutterMetricsKey {
    let request = GutterMetricsRequest {
      document,
      text_layout_config,
      gutter_config,
    };

    GutterMetricsKey::from_metrics_request(&request, font_lock::font_system_version())
  }

  #[test]
  fn gutter_metrics_key_tracks_padding_and_metrics() {
    let document = Document::new("one\ntwo\nthree");
    let text_layout_config = TextLayoutConfig::default();
    let gutter_config = GutterConfig::default();

    assert_ne!(
      metrics_key_for(&document, text_layout_config, gutter_config),
      metrics_key_for(
        &document,
        text_layout_config,
        GutterConfig {
          padding: GutterPadding::new(HorizontalPadding::new(
            gutter_config.padding.horizontal.left + 4.0,
            gutter_config.padding.horizontal.right,
          )),
          ..gutter_config
        },
      )
    );

    assert_ne!(
      metrics_key_for(&document, text_layout_config, gutter_config),
      metrics_key_for(
        &document,
        text_layout_config,
        GutterConfig {
          padding: GutterPadding::new(HorizontalPadding::new(
            gutter_config.padding.horizontal.left,
            gutter_config.padding.horizontal.right + 4.0,
          )),
          ..gutter_config
        },
      )
    );
  }

  #[test]
  fn gutter_rows_signature_uses_visible_no_wrap_range_and_count() {
    let projection = projection(&[(10, 0, 0.0), (11, 0, 24.0), (12, 0, 48.0)]);

    let signature = GutterRowsSignature::from_projection(&projection).expect("visible rows exist");

    assert_eq!(signature.source_lines, 10..=12);
    assert_eq!(signature.wrap_rows, 0..=0);
    assert_eq!(signature.row_count, 3);
  }

  #[test]
  fn gutter_render_key_ignores_fractional_row_origin() {
    let document = Document::new("one\ntwo\nthree");
    let text_layout_config = TextLayoutConfig::default();
    let gutter_config = GutterConfig::default();
    let metrics = GutterMetrics::line_numbers(24.0, &gutter_config);
    let metrics_key = metrics_key_for(&document, text_layout_config, gutter_config);

    let projection_a = projection(&[(10, 0, 0.0), (11, 0, 24.0), (12, 0, 48.0)]);
    let projection_b = projection(&[(10, 0, -3.5), (11, 0, 20.5), (12, 0, 44.5)]);

    let request_a = GutterRenderArtifactRequest {
      text_layout_config,
      metrics,
      gutter_size: iced::Size::new(metrics.requested_width, 96.0),
      projection: &projection_a,
    };

    let request_b = GutterRenderArtifactRequest {
      text_layout_config,
      metrics,
      gutter_size: iced::Size::new(metrics.requested_width, 96.0),
      projection: &projection_b,
    };

    assert_eq!(
      GutterRenderArtifactKey::for_request(&request_a, metrics_key),
      GutterRenderArtifactKey::for_request(&request_b, metrics_key)
    );
  }
}
