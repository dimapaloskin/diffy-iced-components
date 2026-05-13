pub(crate) mod engine;

use std::ops::RangeInclusive;

use iced::advanced::graphics::text;

use crate::document::Document;
use crate::layout::LayoutConfig;
use crate::layout_cache::CosmicBufferPayload;
use crate::padding::GutterPadding;
use crate::projection::LayoutProjection;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GutterConfig {
  pub enabled: bool,
  pub padding: GutterPadding,
  pub separator_width: f32,
}

impl Default for GutterConfig {
  fn default() -> Self {
    Self {
      enabled: true,
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
pub(crate) struct GutterWidthKey {
  pub(crate) source_line_count: usize,
  pub(crate) layout_font: iced::Font,
  pub(crate) font_size_bits: u32,
  pub(crate) padding_bits: u64,
  pub(crate) separator_width_bits: u32,
  pub(crate) font_system_version: text::Version,
}

impl GutterWidthKey {
  pub(crate) fn from_measure_request(
    request: &GutterMeasureRequest<'_>,
    font_system_version: text::Version,
  ) -> Self {
    Self {
      source_line_count: request.document.source_line_count(),
      layout_font: request.layout_config.font,
      font_size_bits: request.layout_config.font_size.to_bits(),
      padding_bits: request.gutter_config.padding.to_bits(),
      separator_width_bits: request.gutter_config.separator_width.to_bits(),
      font_system_version,
    }
  }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GutterRenderKey {
  pub(crate) width_key: GutterWidthKey,
  pub(crate) render_label_width_bits: u32,
  pub(crate) line_height_bits: u32,
  pub(crate) rows_signature: GutterRowsSignature,
}

impl GutterRenderKey {
  pub(crate) fn for_render_request(
    request: &GutterRenderRequest<'_>,
    width_key: GutterWidthKey,
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
      width_key,
      render_label_width_bits: render_label_width.to_bits(),
      line_height_bits: request.layout_config.line_height.to_bits(),
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
  pub(crate) fn from_projection(projection: &LayoutProjection) -> Option<Self> {
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
  pub(crate) key: GutterRenderKey,
  pub(crate) payload: CosmicBufferPayload,
  pub(crate) first_row_viewport_y: f32,
}

pub(crate) struct GutterCacheEntry {
  pub(crate) width_key: GutterWidthKey,
  pub(crate) metrics: GutterMetrics,
  pub(crate) render_artifact: Option<GutterRenderArtifact>,
}

pub(crate) struct GutterMeasureRequest<'a> {
  pub(crate) document: &'a Document,
  pub(crate) layout_config: LayoutConfig,
  pub(crate) gutter_config: GutterConfig,
}

pub(crate) struct GutterRenderRequest<'a> {
  pub(crate) layout_config: LayoutConfig,
  pub(crate) metrics: GutterMetrics,
  pub(crate) gutter_size: iced::Size,
  pub(crate) projection: &'a LayoutProjection,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::font_lock;
  use crate::projection::RowSnapshot;

  fn projection(rows: &[(usize, usize, f32)]) -> LayoutProjection {
    LayoutProjection {
      visible_text_size: iced::Size::new(80.0, rows.len() as f32 * 24.0),
      visible_rows: rows
        .iter()
        .map(
          |&(source_line_index, wrap_row_index, viewport_y)| RowSnapshot {
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

  fn width_key_for(
    document: &Document,
    layout_config: LayoutConfig,
    gutter_config: GutterConfig,
  ) -> GutterWidthKey {
    let request = GutterMeasureRequest {
      document,
      layout_config,
      gutter_config,
    };

    GutterWidthKey::from_measure_request(&request, font_lock::font_system_version())
  }

  #[test]
  fn gutter_width_key_tracks_padding_and_metrics() {
    let document = Document::new("one\ntwo\nthree");
    let layout_config = LayoutConfig::default();
    let gutter_config = GutterConfig::default();

    assert_ne!(
      width_key_for(&document, layout_config, gutter_config),
      width_key_for(
        &document,
        layout_config,
        GutterConfig {
          padding: GutterPadding::new((
            gutter_config.padding.horizontal.left + 4.0,
            gutter_config.padding.horizontal.right,
          )),
          ..gutter_config
        },
      )
    );

    assert_ne!(
      width_key_for(&document, layout_config, gutter_config),
      width_key_for(
        &document,
        layout_config,
        GutterConfig {
          padding: GutterPadding::new((
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
    let layout_config = LayoutConfig::default();
    let gutter_config = GutterConfig::default();
    let metrics = GutterMetrics::line_numbers(24.0, &gutter_config);
    let width_key = width_key_for(&document, layout_config, gutter_config);

    let projection_a = projection(&[(10, 0, 0.0), (11, 0, 24.0), (12, 0, 48.0)]);
    let projection_b = projection(&[(10, 0, -3.5), (11, 0, 20.5), (12, 0, 44.5)]);

    let request_a = GutterRenderRequest {
      layout_config,
      metrics,
      gutter_size: iced::Size::new(metrics.requested_width, 96.0),
      projection: &projection_a,
    };

    let request_b = GutterRenderRequest {
      layout_config,
      metrics,
      gutter_size: iced::Size::new(metrics.requested_width, 96.0),
      projection: &projection_b,
    };

    assert_eq!(
      GutterRenderKey::for_render_request(&request_a, width_key),
      GutterRenderKey::for_render_request(&request_b, width_key)
    );
  }
}
