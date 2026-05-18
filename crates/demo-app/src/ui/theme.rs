#[derive(Debug, Clone)]
pub struct Theme {
  iced: iced::Theme,
  pub colors: Colors,
  pub shadows: Shadows,
  pub radius: iced::border::Radius,
  pub border_width: f32,
  pub focus_ring_width: f32,
  pub focus_ring_offset: f32,
}

impl Theme {
  pub fn light() -> Self {
    Self {
      iced: iced::Theme::Light,
      colors: Colors::light(),
      shadows: Shadows::light(),
      radius: iced::border::radius(10.0),
      border_width: 1.0,
      focus_ring_width: 2.0,
      focus_ring_offset: 2.0,
    }
  }

  pub fn dark() -> Self {
    Self {
      iced: iced::Theme::Dark,
      colors: Colors::dark(),
      shadows: Shadows::dark(),
      radius: iced::border::radius(10.0),
      border_width: 1.0,
      focus_ring_width: 2.0,
      focus_ring_offset: 2.0,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Shadows {
  pub none: iced::Shadow,
  pub control_hovered: iced::Shadow,
  pub control_pressed: iced::Shadow,
}

impl Shadows {
  fn light() -> Self {
    Self {
      none: iced::Shadow::default(),
      control_hovered: iced::Shadow {
        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.20),
        offset: iced::Vector::new(0.0, 1.0),
        blur_radius: 5.0,
      },
      control_pressed: iced::Shadow {
        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.10),
        offset: iced::Vector::new(0.0, 0.5),
        blur_radius: 3.0,
      },
    }
  }

  fn dark() -> Self {
    Self {
      none: iced::Shadow::default(),
      control_hovered: iced::Shadow {
        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.35),
        offset: iced::Vector::new(0.0, 1.0),
        blur_radius: 3.0,
      },
      control_pressed: iced::Shadow {
        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.20),
        offset: iced::Vector::new(0.0, 0.5),
        blur_radius: 1.0,
      },
    }
  }
}

impl iced::theme::Base for Theme {
  fn default(preference: iced::theme::Mode) -> Self {
    match preference {
      iced::theme::Mode::Dark => Self::dark(),
      _ => Self::light(),
    }
  }

  fn mode(&self) -> iced::theme::Mode {
    <iced::Theme as iced::theme::Base>::mode(&self.iced)
  }

  fn base(&self) -> iced::theme::Style {
    iced::theme::Style {
      background_color: self.colors.background,
      text_color: self.colors.foreground,
    }
  }

  fn seed(&self) -> Option<iced::theme::palette::Seed> {
    <iced::Theme as iced::theme::Base>::seed(&self.iced)
  }

  fn name(&self) -> &str {
    match self.mode() {
      iced::theme::Mode::Dark => "diffy-demo-dark",
      _ => "diffy-demo-light",
    }
  }
}

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
  type Class<'a> = <iced::Theme as iced::widget::container::Catalog>::Class<'a>;

  fn default<'a>() -> Self::Class<'a> {
    <iced::Theme as iced::widget::container::Catalog>::default()
  }

  fn style(&self, class: &Self::Class<'_>) -> iced::widget::container::Style {
    <iced::Theme as iced::widget::container::Catalog>::style(&self.iced, class)
  }
}

impl iced::widget::button::Catalog for Theme {
  type Class<'a> = <iced::Theme as iced::widget::button::Catalog>::Class<'a>;

  fn default<'a>() -> Self::Class<'a> {
    <iced::Theme as iced::widget::button::Catalog>::default()
  }

  fn style(
    &self,
    class: &Self::Class<'_>,
    status: iced::widget::button::Status,
  ) -> iced::widget::button::Style {
    <iced::Theme as iced::widget::button::Catalog>::style(&self.iced, class, status)
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Colors {
  pub background: iced::Color,
  pub foreground: iced::Color,
  pub muted_foreground: iced::Color,

  pub surface: iced::Color,
  pub surface_hovered: iced::Color,
  pub surface_pressed: iced::Color,

  pub border: iced::Color,
  pub border_strong: iced::Color,
  pub focus_ring: iced::Color,

  pub primary: iced::Color,
  pub primary_hovered: iced::Color,
  pub primary_pressed: iced::Color,
  pub primary_foreground: iced::Color,

  pub danger: iced::Color,
  pub danger_hovered: iced::Color,
  pub danger_pressed: iced::Color,
  pub danger_foreground: iced::Color,
}

impl Colors {
  fn light() -> Self {
    Self {
      background: iced::Color::from_rgb8(0xff, 0xff, 0xff),
      foreground: iced::Color::from_rgb8(0x09, 0x09, 0x0b),
      muted_foreground: iced::Color::from_rgb8(0x71, 0x71, 0x7a),

      surface: iced::Color::from_rgb8(0xf4, 0xf4, 0xf5),
      surface_hovered: iced::Color::from_rgb8(0xe4, 0xe4, 0xe7),
      surface_pressed: iced::Color::from_rgb8(0xd4, 0xd4, 0xd8),

      border: iced::Color::from_rgb8(0xe4, 0xe4, 0xe7),
      border_strong: iced::Color::from_rgb8(0xd4, 0xd4, 0xd8),
      focus_ring: iced::Color::from_rgb8(0x18, 0x18, 0x1b),

      primary: iced::Color::from_rgb8(0x18, 0x18, 0x1b),
      primary_hovered: iced::Color::from_rgb8(0x27, 0x27, 0x2a),
      primary_pressed: iced::Color::from_rgb8(0x3f, 0x3f, 0x46),
      primary_foreground: iced::Color::from_rgb8(0xfa, 0xfa, 0xfa),

      danger: iced::Color::from_rgb8(0xdc, 0x26, 0x26),
      danger_hovered: iced::Color::from_rgb8(0xb9, 0x1c, 0x1c),
      danger_pressed: iced::Color::from_rgb8(0x99, 0x1b, 0x1b),
      danger_foreground: iced::Color::from_rgb8(0xfe, 0xf2, 0xf2),
    }
  }

  fn dark() -> Self {
    Self {
      background: iced::Color::from_rgb8(0x09, 0x09, 0x09),
      foreground: iced::Color::from_rgb8(0xfa, 0xfa, 0xfa),
      muted_foreground: iced::Color::from_rgb8(0xa1, 0xa1, 0xaa),

      surface: iced::Color::from_rgb8(0x18, 0x18, 0x1b),
      surface_hovered: iced::Color::from_rgb8(0x27, 0x27, 0x2a),
      surface_pressed: iced::Color::from_rgb8(0x3f, 0x3f, 0x46),

      border: iced::Color::from_rgb8(0x27, 0x27, 0x2a),
      border_strong: iced::Color::from_rgb8(0x52, 0x52, 0x5b),
      focus_ring: iced::Color::from_rgb8(0xfa, 0xfa, 0xfa),

      primary: iced::Color::from_rgb8(0xfa, 0xfa, 0xfa),
      primary_hovered: iced::Color::from_rgb8(0xe4, 0xe4, 0xe7),
      primary_pressed: iced::Color::from_rgb8(0xd4, 0xd4, 0xd8),
      primary_foreground: iced::Color::from_rgb8(0x09, 0x09, 0x0b),

      danger: iced::Color::from_rgb8(0x7f, 0x1d, 0x1d),
      danger_hovered: iced::Color::from_rgb8(0x99, 0x1b, 0x1b),
      danger_pressed: iced::Color::from_rgb8(0x65, 0x17, 0x17),
      danger_foreground: iced::Color::from_rgb8(0xfe, 0xf2, 0xf2),
    }
  }
}
