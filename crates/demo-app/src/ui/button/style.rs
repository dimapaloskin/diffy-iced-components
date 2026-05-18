use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, Default)]
pub enum Variant {
  #[default]
  Primary,
  Secondary,
  Outline,
  Ghost,
  Danger,
  Link,
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

pub(super) type StyleFn<'a> = dyn Fn(&Theme, Status) -> Style + 'a;

pub(super) enum Class<'a> {
  Variant(Variant),
  Custom(Box<StyleFn<'a>>),
}

pub(super) fn resolve(theme: &Theme, class: &Class<'_>, status: Status) -> Style {
  match class {
    Class::Variant(Variant::Primary) => primary(theme, status),
    Class::Variant(Variant::Secondary) => secondary(theme, status),
    Class::Variant(Variant::Outline) => outline(theme, status),
    Class::Variant(Variant::Ghost) => ghost(theme, status),
    Class::Variant(Variant::Danger) => danger(theme, status),
    Class::Variant(Variant::Link) => link(theme, status),
    Class::Custom(style) => style(theme, status),
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

pub fn primary(theme: &Theme, status: Status) -> Style {
  let colors = theme.colors;
  let shadows = theme.shadows;

  match status {
    Status::Active => button_style(
      theme,
      Some(colors.primary),
      colors.primary_foreground,
      colors.primary,
      0.0,
      shadows.none,
    ),
    Status::Hovered => button_style(
      theme,
      Some(colors.primary_hovered),
      colors.primary_foreground,
      colors.primary_hovered,
      0.0,
      shadows.control_hovered,
    ),
    Status::Pressed => button_style(
      theme,
      Some(colors.primary_pressed),
      colors.primary_foreground,
      colors.primary_pressed,
      0.0,
      shadows.control_pressed,
    ),
    Status::Focused => focused(
      theme,
      button_style(
        theme,
        Some(colors.primary),
        colors.primary_foreground,
        colors.primary,
        0.0,
        shadows.control_hovered,
      ),
    ),
    Status::Disabled => button_style(
      theme,
      Some(colors.surface),
      colors.muted_foreground,
      colors.border,
      0.0,
      shadows.none,
    ),
  }
}

pub fn secondary(theme: &Theme, status: Status) -> Style {
  let colors = theme.colors;
  let shadows = theme.shadows;

  match status {
    Status::Active => button_style(
      theme,
      Some(colors.surface),
      colors.foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Status::Hovered => button_style(
      theme,
      Some(colors.surface_hovered),
      colors.foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.control_hovered,
    ),
    Status::Pressed => button_style(
      theme,
      Some(colors.surface_pressed),
      colors.foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.control_pressed,
    ),
    Status::Focused => focused(
      theme,
      button_style(
        theme,
        Some(colors.surface),
        colors.foreground,
        iced::Color::TRANSPARENT,
        0.0,
        shadows.control_hovered,
      ),
    ),
    Status::Disabled => button_style(
      theme,
      Some(colors.surface),
      colors.muted_foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
  }
}

pub fn outline(theme: &Theme, status: Status) -> Style {
  let colors = theme.colors;
  let shadows = theme.shadows;

  match status {
    Status::Active => button_style(
      theme,
      None,
      colors.foreground,
      colors.border,
      theme.border_width,
      shadows.none,
    ),
    Status::Hovered => button_style(
      theme,
      Some(colors.surface_hovered),
      colors.foreground,
      colors.border_strong,
      theme.border_width,
      shadows.control_hovered,
    ),
    Status::Pressed => button_style(
      theme,
      Some(colors.surface_pressed),
      colors.foreground,
      colors.border,
      theme.border_width,
      shadows.control_pressed,
    ),
    Status::Focused => focused(
      theme,
      button_style(
        theme,
        None,
        colors.foreground,
        colors.border_strong,
        theme.border_width,
        shadows.control_hovered,
      ),
    ),
    Status::Disabled => button_style(
      theme,
      None,
      colors.muted_foreground,
      colors.border,
      theme.border_width,
      shadows.none,
    ),
  }
}

pub fn ghost(theme: &Theme, status: Status) -> Style {
  let colors = theme.colors;
  let shadows = theme.shadows;

  match status {
    Status::Active => button_style(
      theme,
      None,
      colors.foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Status::Hovered => button_style(
      theme,
      Some(colors.surface_hovered),
      colors.foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Status::Pressed => button_style(
      theme,
      Some(colors.surface_pressed),
      colors.foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Status::Focused => focused(
      theme,
      button_style(
        theme,
        Some(colors.surface_hovered),
        colors.foreground,
        iced::Color::TRANSPARENT,
        0.0,
        shadows.none,
      ),
    ),
    Status::Disabled => button_style(
      theme,
      None,
      colors.muted_foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
  }
}

pub fn danger(theme: &Theme, status: Status) -> Style {
  let colors = theme.colors;
  let shadows = theme.shadows;

  match status {
    Status::Active => button_style(
      theme,
      Some(colors.danger),
      colors.danger_foreground,
      colors.danger,
      0.0,
      shadows.none,
    ),
    Status::Hovered => button_style(
      theme,
      Some(colors.danger_hovered),
      colors.danger_foreground,
      colors.danger_hovered,
      0.0,
      shadows.control_hovered,
    ),
    Status::Pressed => button_style(
      theme,
      Some(colors.danger_pressed),
      colors.danger_foreground,
      colors.danger_pressed,
      0.0,
      shadows.control_pressed,
    ),
    Status::Focused => focused(
      theme,
      button_style(
        theme,
        Some(colors.danger),
        colors.danger_foreground,
        colors.danger,
        0.0,
        shadows.control_hovered,
      ),
    ),
    Status::Disabled => button_style(
      theme,
      Some(colors.surface),
      colors.muted_foreground,
      colors.border,
      0.0,
      shadows.none,
    ),
  }
}

pub fn link(theme: &Theme, status: Status) -> Style {
  const UNDERLINE: Underline = Underline {
    width: 1.5,
    offset: 3.0,
  };

  let colors = theme.colors;
  let shadows = theme.shadows;
  let mut style = match status {
    Status::Active => button_style(
      theme,
      None,
      colors.foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Status::Hovered => button_style(
      theme,
      None,
      colors.foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Status::Pressed => button_style(
      theme,
      None,
      colors.foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
    Status::Focused => focused(
      theme,
      button_style(
        theme,
        None,
        colors.foreground,
        iced::Color::TRANSPARENT,
        0.0,
        shadows.none,
      ),
    ),
    Status::Disabled => button_style(
      theme,
      None,
      colors.muted_foreground,
      iced::Color::TRANSPARENT,
      0.0,
      shadows.none,
    ),
  };

  if matches!(status, Status::Hovered | Status::Pressed | Status::Focused) {
    style.underline = Some(UNDERLINE);
  }

  style
}
