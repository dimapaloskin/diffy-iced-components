#[derive(Debug, Clone)]
pub struct BorderTokens {
  pub width: f32,
}

#[derive(Debug, Clone)]
pub struct ColorTokens {
  pub background: iced::Color,
  pub muted_background: iced::Color,
  pub foreground: iced::Color,
  pub muted_foreground: iced::Color,

  pub surface: iced::Background,
  pub surface_hovered: iced::Background,
  pub surface_pressed: iced::Background,

  pub border: iced::Color,

  pub primary: FamilyTone,
  pub neutral: FamilyTone,
  pub danger: FamilyTone,

  pub focus_ring: iced::Color,
}

#[derive(Debug, Clone)]
pub struct FamilyTone {
  pub background: iced::Background,
  pub hovered: iced::Background,
  pub pressed: iced::Background,
  pub foreground: iced::Color,
  pub muted_foreground: iced::Color, // TODO: Not sure how to  deal with it. Probably remove
  pub accent: iced::Color,
  pub border: iced::Color,
}

#[derive(Debug, Clone)]
pub struct FocusTokens {
  pub width: f32,
  pub offset: f32,
}

#[derive(Debug, Clone)]
pub struct RadiusTokens {
  pub none: iced::border::Radius,
  pub control: iced::border::Radius,
  pub surface: iced::border::Radius,
  pub pill: iced::border::Radius,
  pub scrollbar: iced::border::Radius,
}

#[derive(Debug, Clone)]
pub struct ShadowTokens {
  pub none: iced::Shadow,

  pub control: iced::Shadow,
  pub control_hovered: iced::Shadow,
  pub control_pressed: iced::Shadow,

  pub surface: iced::Shadow,
  pub pill: iced::Shadow,
}

#[derive(Debug, Clone)]
pub struct ScrollbarStateTokens {
  pub thumb: iced::Color,
  pub track: iced::Color,
}

#[derive(Debug, Clone)]
pub struct ScrollbarTokens {
  pub idle: ScrollbarStateTokens,
  pub hovered: ScrollbarStateTokens,
  pub pressed: ScrollbarStateTokens,
}
