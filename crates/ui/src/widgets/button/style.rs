use derive_setters::Setters;

use crate::theme::{FamilyTone, Theme};

pub type StyleFn<'a> = Box<dyn Fn(&Theme, Status, Style) -> Style + 'a>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
  Primary,
  #[default]
  Neutral,
  Danger,
}

impl Variant {
  pub(super) fn tone(self, theme: &Theme) -> &FamilyTone {
    match self {
      Variant::Primary => &theme.tokens().colors.primary,
      Variant::Neutral => &theme.tokens().colors.neutral,
      Variant::Danger => &theme.tokens().colors.danger,
    }
  }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Fill,
  Light,
  Outline,
  Ghost,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Status {
  #[default]
  Active,
  Hovered,
  Pressed,
  Disabled,
}

#[derive(Debug, Clone, Copy)]
pub struct FocusRing {
  pub color: iced::Color,
  pub width: f32,
  pub offset: f32,
}

#[derive(Debug, Setters, Clone, Copy)]
pub struct Style {
  pub background: Option<iced::Background>,
  pub text_color: iced::Color,
  pub border: iced::Border,
  pub shadow: iced::Shadow,
  pub focus_ring: Option<FocusRing>,
}

impl Style {
  fn fill(theme: &Theme, tone: &FamilyTone, status: Status) -> Self {
    let shadows = theme.shadows();

    let (background, shadow) = match status {
      Status::Active => (tone.background, shadows.control),
      Status::Hovered => (tone.hovered, shadows.control_hovered),
      Status::Pressed => (tone.pressed, shadows.control_pressed),
      Status::Disabled => unreachable!("handled before fill style resolution"),
    };

    Style {
      background: Some(background),
      text_color: tone.foreground,
      border: iced::Border {
        radius: theme.radius().control,
        width: 0.0,
        color: tone.border,
      },
      shadow,
      focus_ring: None,
    }
  }

  fn light(theme: &Theme, tone: &FamilyTone, status: Status) -> Self {
    let neutral = &theme.colors().neutral;
    let shadows = theme.shadows();

    let (background, shadow) = match status {
      Status::Active => (neutral.background, shadows.control),
      Status::Hovered => (neutral.hovered, shadows.control_hovered),
      Status::Pressed => (neutral.pressed, shadows.control_pressed),
      Status::Disabled => unreachable!("handled before light style resolution"),
    };

    Style {
      background: Some(background),
      text_color: tone.muted_foreground,
      border: iced::Border {
        radius: theme.radius().control,
        width: 0.0,
        color: iced::Color::TRANSPARENT,
      },
      shadow,
      focus_ring: None,
    }
  }

  fn outline(theme: &Theme, tone: &FamilyTone, status: Status) -> Self {
    let neutral = &theme.colors().neutral;
    let shadows = theme.shadows();

    let (background, shadow) = match status {
      Status::Active => (None, shadows.control),
      Status::Hovered => (Some(neutral.hovered), shadows.control_hovered),
      Status::Pressed => (Some(neutral.pressed), shadows.control_pressed),
      Status::Disabled => unreachable!("handled before outline style  resolution"),
    };

    Style {
      background,
      text_color: tone.accent,
      border: iced::Border {
        radius: theme.radius().control,
        width: theme.border().width,
        color: tone.border,
      },
      shadow,
      focus_ring: None,
    }
  }

  fn ghost(theme: &Theme, tone: &FamilyTone, status: Status) -> Self {
    let neutral = &theme.colors().neutral;

    let background = match status {
      Status::Active => None,
      Status::Hovered => Some(neutral.hovered),
      Status::Pressed => Some(neutral.pressed),
      Status::Disabled => unreachable!("handled before ghost style resolution"),
    };

    Style {
      background,
      text_color: tone.accent,
      border: iced::Border {
        radius: theme.radius().control,
        width: 0.0,
        color: iced::Color::TRANSPARENT,
      },
      shadow: theme.shadows().none,
      focus_ring: None,
    }
  }

  fn disabled(theme: &Theme, mode: Mode) -> Self {
    let colors = theme.colors();

    let (background, border_width, border_color) = match mode {
      Mode::Fill | Mode::Light => (
        Some(colors.neutral.background),
        0.0,
        iced::Color::TRANSPARENT,
      ),
      Mode::Outline => (None, theme.border().width, colors.border),
      Mode::Ghost => (None, 0.0, iced::Color::TRANSPARENT),
    };

    Style {
      background,
      text_color: colors.muted_foreground,
      border: iced::Border {
        radius: theme.radius().control,
        width: border_width,
        color: border_color,
      },
      shadow: theme.shadows().none,
      focus_ring: None,
    }
  }

  pub(super) fn focused(mut self, theme: &Theme) -> Self {
    let focus = theme.focus();

    self.focus_ring = Some(FocusRing {
      color: theme.colors().focus_ring,
      width: focus.width,
      offset: focus.offset,
    });

    self
  }

  fn mode(theme: &Theme, tone: &FamilyTone, mode: Mode, status: Status) -> Self {
    match mode {
      Mode::Fill => Self::fill(theme, tone, status),
      Mode::Light => Self::light(theme, tone, status),
      Mode::Outline => Self::outline(theme, tone, status),
      Mode::Ghost => Self::ghost(theme, tone, status),
    }
  }

  pub fn resolve(theme: &Theme, variant: Variant, mode: Mode, status: Status) -> Style {
    let tone = variant.tone(theme);

    match status {
      Status::Disabled => Style::disabled(theme, mode),
      Status::Active | Status::Hovered | Status::Pressed => Style::mode(theme, tone, mode, status),
    }
  }
}
