use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, Default)]
pub enum Variant {
  #[default]
  Primary,
  Neutral,
  Danger,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Mode {
  #[default]
  Fill,
  Light,
  Outline,
  Ghost,
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
  pub background: Option<iced::Background>,
  pub text_color: iced::Color,
  pub border: iced::Border,
  pub shadow: iced::Shadow,
  pub focus_ring: Option<FocusRing>,
  pub underline: Option<Underline>,
}

#[derive(Debug, Clone, Copy)]
pub struct FocusRing {
  pub color: iced::Color,
  pub width: f32,
  pub offset: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Underline {
  pub width: f32,
  pub offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
  Active,
  Hovered,
  Pressed,
  Focused,
  Disabled,
}

pub(super) type StyleFn<'a> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

pub(super) enum Class<'a> {
  BuiltIn { variant: Variant, mode: Mode },
  Custom(StyleFn<'a>),
}

impl<'a> Class<'a> {
  pub(super) fn built_in(variant: Variant, mode: Mode) -> Self {
    Self::BuiltIn { variant, mode }
  }

  pub(super) fn set_variant(&mut self, variant: Variant) {
    match self {
      Self::BuiltIn {
        variant: current, ..
      } => *current = variant,
      Self::Custom(_) => *self = Self::built_in(variant, Mode::default()),
    }
  }

  pub(super) fn set_mode(&mut self, mode: Mode) {
    match self {
      Self::BuiltIn { mode: current, .. } => *current = mode,
      Self::Custom(_) => *self = Self::built_in(Variant::default(), mode),
    }
  }
}

pub(super) fn resolve(theme: &Theme, class: &Class<'_>, status: Status) -> Style {
  match class {
    Class::BuiltIn { variant, mode } => built_in(theme, *variant, *mode, status),
    Class::Custom(style) => style(theme, status),
  }
}

fn built_in(theme: &Theme, variant: Variant, mode: Mode, status: Status) -> Style {
  match status {
    Status::Disabled => disabled(theme, mode),
    Status::Focused => focused(theme, focus_base(theme, variant, mode)),
    Status::Active | Status::Hovered | Status::Pressed => mode_style(theme, variant, mode, status),
  }
}

fn mode_style(theme: &Theme, variant: Variant, mode: Mode, status: Status) -> Style {
  match mode {
    Mode::Fill => fill(theme, variant, status),
    Mode::Light => light(theme, variant, status),
    Mode::Outline => outline(theme, variant, status),
    Mode::Ghost => ghost(theme, variant, status),
  }
}

fn fill(theme: &Theme, variant: Variant, status: Status) -> Style {
  let colors = theme.colors;
  let shadows = theme.shadows;

  let (background, hovered, pressed, foreground) = match variant {
    Variant::Primary => (
      colors.primary,
      colors.primary_hovered,
      colors.primary_pressed,
      colors.primary_foreground,
    ),
    Variant::Neutral => (
      colors.surface,
      colors.surface_hovered,
      colors.surface_pressed,
      colors.foreground,
    ),
    Variant::Danger => (
      colors.danger,
      colors.danger_hovered,
      colors.danger_pressed,
      colors.danger_foreground,
    ),
  };

  match status {
    Status::Active => button_style(
      theme,
      Some(background),
      foreground,
      background,
      0.0,
      shadows.none,
    ),
    Status::Hovered => button_style(
      theme,
      Some(hovered),
      foreground,
      hovered,
      0.0,
      shadows.control_hovered,
    ),
    Status::Pressed => button_style(
      theme,
      Some(pressed),
      foreground,
      pressed,
      0.0,
      shadows.control_pressed,
    ),
    Status::Focused | Status::Disabled => unreachable!("handled before mode style resolution"),
  }
}

fn light(theme: &Theme, variant: Variant, status: Status) -> Style {
  let colors = theme.colors;
  let shadows = theme.shadows;
  let text_color = semantic_text(theme, variant);

  match status {
    Status::Active => button_style(
      theme,
      Some(colors.surface),
      text_color,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Status::Hovered => button_style(
      theme,
      Some(colors.surface_hovered),
      text_color,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.control_hovered,
    ),
    Status::Pressed => button_style(
      theme,
      Some(colors.surface_pressed),
      text_color,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.control_pressed,
    ),
    Status::Focused | Status::Disabled => unreachable!("handled before mode style resolution"),
  }
}

fn outline(theme: &Theme, variant: Variant, status: Status) -> Style {
  let shadows = theme.shadows;
  let text_color = semantic_text(theme, variant);
  let (border, _, _) = semantic_border(theme, variant);
  let tint = semantic_tint(theme, variant);

  match status {
    Status::Active => button_style(
      theme,
      None,
      text_color,
      border,
      theme.border_width,
      shadows.none,
    ),
    Status::Hovered => button_style(
      theme,
      Some(tint.scale_alpha(0.10)),
      text_color,
      tint.scale_alpha(0.10),
      theme.border_width,
      shadows.control_hovered,
    ),
    Status::Pressed => button_style(
      theme,
      Some(tint.scale_alpha(0.16)),
      text_color,
      tint.scale_alpha(0.16),
      theme.border_width,
      shadows.control_pressed,
    ),
    Status::Focused | Status::Disabled => unreachable!("handled before mode style resolution"),
  }
}

fn ghost(theme: &Theme, variant: Variant, status: Status) -> Style {
  let colors = theme.colors;
  let shadows = theme.shadows;
  let text_color = semantic_text(theme, variant);

  match status {
    Status::Active => button_style(
      theme,
      None,
      text_color,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Status::Hovered => button_style(
      theme,
      Some(colors.surface_hovered),
      text_color,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Status::Pressed => button_style(
      theme,
      Some(colors.surface_pressed),
      text_color,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Status::Focused | Status::Disabled => unreachable!("handled before mode style resolution"),
  }
}

fn focus_base(theme: &Theme, variant: Variant, mode: Mode) -> Style {
  let status = match mode {
    Mode::Ghost => Status::Hovered,
    Mode::Fill | Mode::Light | Mode::Outline => Status::Active,
  };

  let style = mode_style(theme, variant, mode, status);

  match mode {
    Mode::Fill | Mode::Light | Mode::Outline => with_shadow(style, theme.shadows.control_hovered),
    Mode::Ghost => style,
  }
}

fn disabled(theme: &Theme, mode: Mode) -> Style {
  let colors = theme.colors;
  let shadows = theme.shadows;

  match mode {
    Mode::Fill | Mode::Light => button_style(
      theme,
      Some(colors.surface),
      colors.muted_foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Mode::Outline => button_style(
      theme,
      None,
      colors.muted_foreground,
      colors.border,
      theme.border_width,
      shadows.none,
    ),
    Mode::Ghost => button_style(
      theme,
      None,
      colors.muted_foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
  }
}

fn semantic_text(theme: &Theme, variant: Variant) -> iced::Color {
  match variant {
    Variant::Primary | Variant::Neutral => theme.colors.foreground,
    Variant::Danger => theme.colors.danger_text,
  }
}

fn semantic_border(theme: &Theme, variant: Variant) -> (iced::Color, iced::Color, iced::Color) {
  match variant {
    Variant::Primary => (
      theme.colors.primary,
      theme.colors.primary_hovered,
      theme.colors.primary_pressed,
    ),
    Variant::Neutral => (
      theme.colors.border,
      theme.colors.border_strong,
      theme.colors.border,
    ),
    Variant::Danger => (
      theme.colors.danger,
      theme.colors.danger_hovered,
      theme.colors.danger_pressed,
    ),
  }
}

fn semantic_tint(theme: &Theme, variant: Variant) -> iced::Color {
  match variant {
    Variant::Primary => theme.colors.primary,
    Variant::Neutral => theme.colors.foreground,
    Variant::Danger => theme.colors.danger,
  }
}

fn button_style(
  theme: &Theme,
  background: Option<iced::Color>,
  text_color: iced::Color,
  border_color: iced::Color,
  border_width: f32,
  shadow: iced::Shadow,
) -> Style {
  Style {
    background: background.map(iced::Background::Color),
    text_color,
    border: iced::Border {
      radius: theme.radius,
      width: border_width,
      color: border_color,
    },
    shadow,
    focus_ring: None,
    underline: None,
  }
}

fn focused(theme: &Theme, mut style: Style) -> Style {
  style.focus_ring = Some(FocusRing {
    color: theme.colors.focus_ring,
    width: theme.focus_ring_width,
    offset: theme.focus_ring_offset,
  });
  style
}

fn with_shadow(mut style: Style, shadow: iced::Shadow) -> Style {
  style.shadow = shadow;
  style
}
