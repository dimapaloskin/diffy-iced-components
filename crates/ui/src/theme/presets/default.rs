use crate::theme::tokens::*;
use crate::theme::{ColorScheme, FamilyTone, ThemeTokens};

pub fn tokens(scheme: ColorScheme) -> ThemeTokens {
  match scheme {
    ColorScheme::Dark => default_dark(),
    ColorScheme::Light => default_light(),
  }
}

const FOCUS_TOKENS: FocusTokens = FocusTokens {
  width: 2.0,
  offset: 0.0,
};

fn scrollbar_tokens(thumb: iced::Color, track: iced::Color) -> ScrollbarTokens {
  ScrollbarTokens {
    idle: ScrollbarStateTokens {
      thumb: with_alpha(thumb, 0.38),
      track: iced::Color::TRANSPARENT,
    },
    hovered: ScrollbarStateTokens {
      thumb: with_alpha(thumb, 0.56),
      track: with_alpha(track, 0.18),
    },
    pressed: ScrollbarStateTokens {
      thumb: with_alpha(thumb, 0.72),
      track: with_alpha(track, 0.24),
    },
  }
}

fn with_alpha(color: iced::Color, alpha: f32) -> iced::Color {
  iced::Color {
    a: alpha.clamp(0.0, 1.0),
    ..color
  }
}

fn default_light() -> ThemeTokens {
  ThemeTokens {
    name: "diffy-default-light".into(),
    scheme: ColorScheme::Light,
    border: BorderTokens { width: 1.0 },
    colors: ColorTokens {
      background: iced::Color::from_rgb8(0xff, 0xff, 0xff),
      muted_background: iced::Color::from_rgb8(0xf4, 0xf4, 0xf5),
      foreground: iced::Color::from_rgb8(0x09, 0x09, 0x0b),
      muted_foreground: iced::Color::from_rgb8(0x71, 0x71, 0x7a),

      surface: iced::Color::from_rgb8(0xf4, 0xf4, 0xf5).into(),
      surface_hovered: iced::Color::from_rgb8(0xe4, 0xe4, 0xe7).into(),
      surface_pressed: iced::Color::from_rgb8(0xd4, 0xd4, 0xd8).into(),

      border: iced::Color::from_rgb8(0xe4, 0xe4, 0xe7),

      primary: FamilyTone {
        background: iced::Color::from_rgb8(0x18, 0x18, 0x1b).into(),
        hovered: iced::Color::from_rgb8(0x27, 0x27, 0x2a).into(),
        pressed: iced::Color::from_rgb8(0x3f, 0x3f, 0x46).into(),
        foreground: iced::Color::from_rgb8(0xfa, 0xfa, 0xfa),
        muted_foreground: iced::Color::from_rgb8(0x52, 0x52, 0x5b),
        accent: iced::Color::from_rgb8(0x18, 0x18, 0x1b),
        border: iced::Color::from_rgb8(0x18, 0x18, 0x1b),
      },
      neutral: FamilyTone {
        background: iced::Color::from_rgb8(0xf4, 0xf4, 0xf5).into(),
        hovered: iced::Color::from_rgb8(0xe4, 0xe4, 0xe7).into(),
        pressed: iced::Color::from_rgb8(0xd4, 0xd4, 0xd8).into(),
        foreground: iced::Color::from_rgb8(0x18, 0x18, 0x1b),
        muted_foreground: iced::Color::from_rgb8(0x71, 0x71, 0x7a),
        accent: iced::Color::from_rgb8(0x18, 0x18, 0x1b),
        border: iced::Color::from_rgb8(0xe4, 0xe4, 0xe7),
      },
      danger: FamilyTone {
        background: iced::Color::from_rgb8(0xdc, 0x26, 0x26).into(),
        hovered: iced::Color::from_rgb8(0xb9, 0x1c, 0x1c).into(),
        pressed: iced::Color::from_rgb8(0x99, 0x1b, 0x1b).into(),
        foreground: iced::Color::from_rgb8(0xfe, 0xf2, 0xf2),
        muted_foreground: iced::Color::from_rgb8(0xdc, 0x26, 0x26),
        accent: iced::Color::from_rgb8(0xdc, 0x26, 0x26),
        border: iced::Color::from_rgb8(0xdc, 0x26, 0x26),
      },

      focus_ring: iced::Color::from_rgba8(0x73, 0x73, 0x73, 0.5),
    },
    focus: FOCUS_TOKENS,
    radius: RadiusTokens {
      none: iced::border::radius(0.0),
      control: iced::border::radius(10.0),
      surface: iced::border::radius(14.0),
      pill: iced::border::radius(999.0),
      scrollbar: iced::border::radius(999.0),
    },
    shadows: ShadowTokens {
      none: iced::Shadow::default(),
      control: iced::Shadow::default(),
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
      surface: iced::Shadow {
        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.12),
        offset: iced::Vector::new(0.0, 2.0),
        blur_radius: 10.0,
      },
      pill: iced::Shadow::default(),
    },
    scrollbar: scrollbar_tokens(
      iced::Color::from_rgb8(0x71, 0x71, 0x7a),
      iced::Color::from_rgb8(0xf4, 0xf4, 0xf5),
    ),
  }
}

fn default_dark() -> ThemeTokens {
  ThemeTokens {
    name: "diffy-default-dark".into(),
    scheme: ColorScheme::Dark,
    border: BorderTokens { width: 1.0 },
    colors: ColorTokens {
      background: iced::Color::from_rgb8(0x09, 0x09, 0x09),
      muted_background: iced::Color::from_rgb8(0x18, 0x18, 0x1b),
      foreground: iced::Color::from_rgb8(0xfa, 0xfa, 0xfa),
      muted_foreground: iced::Color::from_rgb8(0xa1, 0xa1, 0xaa),

      surface: iced::Color::from_rgb8(0x18, 0x18, 0x1b).into(),
      surface_hovered: iced::Color::from_rgb8(0x27, 0x27, 0x2a).into(),
      surface_pressed: iced::Color::from_rgb8(0x3f, 0x3f, 0x46).into(),

      border: iced::Color::from_rgb8(0x27, 0x27, 0x2a),

      primary: FamilyTone {
        background: iced::Color::from_rgb8(0xfa, 0xfa, 0xfa).into(),
        hovered: iced::Color::from_rgb8(0xe4, 0xe4, 0xe7).into(),
        pressed: iced::Color::from_rgb8(0xd4, 0xd4, 0xd8).into(),
        foreground: iced::Color::from_rgb8(0x09, 0x09, 0x0b),
        muted_foreground: iced::Color::from_rgb8(0xd4, 0xd4, 0xd8),
        accent: iced::Color::from_rgb8(0xfa, 0xfa, 0xfa),
        border: iced::Color::from_rgb8(0xfa, 0xfa, 0xfa),
      },
      neutral: FamilyTone {
        background: iced::Color::from_rgb8(0x18, 0x18, 0x1b).into(),
        hovered: iced::Color::from_rgb8(0x27, 0x27, 0x2a).into(),
        pressed: iced::Color::from_rgb8(0x3f, 0x3f, 0x46).into(),
        foreground: iced::Color::from_rgb8(0xfa, 0xfa, 0xfa),
        muted_foreground: iced::Color::from_rgb8(0xa1, 0xa1, 0xaa),
        accent: iced::Color::from_rgb8(0xfa, 0xfa, 0xfa),
        border: iced::Color::from_rgb8(0x27, 0x27, 0x2a),
      },
      danger: FamilyTone {
        background: iced::Color::from_rgb8(0x7f, 0x1d, 0x1d).into(),
        hovered: iced::Color::from_rgb8(0x99, 0x1b, 0x1b).into(),
        pressed: iced::Color::from_rgb8(0x65, 0x17, 0x17).into(),
        foreground: iced::Color::from_rgb8(0xfe, 0xf2, 0xf2),
        muted_foreground: iced::Color::from_rgb8(0xf8, 0x71, 0x71),
        accent: iced::Color::from_rgb8(0xef, 0x44, 0x44),
        border: iced::Color::from_rgb8(0x99, 0x1b, 0x1b),
      },

      focus_ring: iced::Color::from_rgba8(0x73, 0x73, 0x73, 0.5),
    },
    focus: FOCUS_TOKENS,
    radius: RadiusTokens {
      none: iced::border::radius(0.0),
      control: iced::border::radius(10.0),
      surface: iced::border::radius(14.0),
      pill: iced::border::radius(999.0),
      scrollbar: iced::border::radius(999.0),
    },
    shadows: ShadowTokens {
      none: iced::Shadow::default(),
      control: iced::Shadow::default(),
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
      surface: iced::Shadow {
        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.28),
        offset: iced::Vector::new(0.0, 2.0),
        blur_radius: 8.0,
      },
      pill: iced::Shadow::default(),
    },
    scrollbar: scrollbar_tokens(
      iced::Color::from_rgb8(0xa1, 0xa1, 0xaa),
      iced::Color::from_rgb8(0x18, 0x18, 0x1b),
    ),
  }
}
