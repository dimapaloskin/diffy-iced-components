use crate::theme::Theme;

impl iced::widget::text::Catalog for Theme {
  type Class<'a> = iced::widget::text::StyleFn<'a, Self>;

  fn default<'a>() -> Self::Class<'a> {
    Box::new(|_theme| iced::widget::text::Style { color: None })
  }

  fn style(&self, class: &Self::Class<'_>) -> iced::widget::text::Style {
    class(self)
  }
}

impl iced::widget::container::Catalog for Theme {
  type Class<'a> = iced::widget::container::StyleFn<'a, Self>;

  fn default<'a>() -> Self::Class<'a> {
    Box::new(|_theme| iced::widget::container::Style::default())
  }

  fn style(&self, class: &Self::Class<'_>) -> iced::widget::container::Style {
    class(self)
  }
}
