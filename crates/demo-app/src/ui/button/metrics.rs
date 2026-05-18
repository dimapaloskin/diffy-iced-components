use crate::ui::sizing::Sizing;

#[derive(Debug, Clone, Copy)]
pub struct Metrics {
  pub height: f32,
  pub padding: iced::Padding,
  pub font_size: f32,
  pub icon_size: f32,
  pub content_y_offset: f32,
  pub icon_y_offset: f32,
  pub gap: f32,
}

impl From<Sizing> for Metrics {
  fn from(size: Sizing) -> Self {
    match size {
      Sizing::Xs => Self {
        height: 24.0,
        padding: iced::padding::horizontal(8.0).vertical(4.0),
        font_size: 12.0,
        icon_size: 12.0,
        content_y_offset: 0.0,
        icon_y_offset: 0.0,
        gap: 6.0,
      },
      Sizing::Sm => Self {
        height: 28.0,
        padding: iced::padding::horizontal(10.0).vertical(5.0),
        font_size: 14.0,
        icon_size: 14.0,
        content_y_offset: 0.0,
        icon_y_offset: 0.0,
        gap: 6.0,
      },
      Sizing::Md => Self {
        height: 32.0,
        padding: iced::padding::horizontal(12.0).vertical(6.0),
        font_size: 16.0,
        icon_size: 16.0,
        content_y_offset: 0.0,
        icon_y_offset: 0.0,
        gap: 8.0,
      },
      Sizing::Lg => Self {
        height: 40.0,
        padding: iced::padding::horizontal(16.0).vertical(8.0),
        font_size: 20.0,
        icon_size: 18.0,
        content_y_offset: 0.0,
        icon_y_offset: 0.0,
        gap: 10.0,
      },
    }
  }
}
