use iced::advanced::renderer::Quad;
use iced::advanced::renderer::Renderer as RendererTrait;

use super::{Geometry, Status, Style};

pub fn draw<Renderer>(
  renderer: &mut Renderer,
  geometry: &Geometry,
  style: &Style,
  status: Status,
  opacity: f32,
) where
  Renderer: RendererTrait,
{
  let Some(thumb) = geometry.thumb else {
    return;
  };

  let opacity = opacity.clamp(0.0, 1.0);
  if opacity <= 0.0 {
    return;
  }

  let status_style = style.for_status(status);
  let track = with_opacity(status_style.track, opacity);
  let thumb_color = with_opacity(status_style.thumb, opacity);

  if track.a > 0.0 {
    renderer.fill_quad(
      Quad {
        bounds: geometry.track_bounds,
        border: iced::Border {
          radius: style.radius,
          ..iced::Border::default()
        },
        ..Quad::default()
      },
      iced::Background::Color(track),
    );
  }

  if thumb_color.a > 0.0 {
    renderer.fill_quad(
      Quad {
        bounds: thumb.visual_bounds,
        border: iced::Border {
          radius: style.radius,
          ..iced::Border::default()
        },
        ..Quad::default()
      },
      iced::Background::Color(thumb_color),
    );
  }
}

fn with_opacity(color: iced::Color, opacity: f32) -> iced::Color {
  iced::Color {
    a: color.a * opacity,
    ..color
  }
}
