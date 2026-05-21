use diffy_ui::widgets::{button, group};
use diffy_ui::{Icon, Sizing, Theme};

use iced::Element;

use super::{AppMessage, ThemeMode, selected_variant};

pub fn theme_switcher(
  current_theme_mode: Option<ThemeMode>,
) -> Element<'static, AppMessage, Theme> {
  let buttons: [(Icon, AppMessage, Option<ThemeMode>); 3] = [
    (Icon::Monitor, AppMessage::UseSystemTheme, None),
    (Icon::Sun, AppMessage::UseLightTheme, Some(ThemeMode::Light)),
    (Icon::Moon, AppMessage::UseDarkTheme, Some(ThemeMode::Dark)),
  ];

  let theme_buttons = buttons.into_iter().map(|(icon, message, mode)| {
    button(icon)
      .fill()
      .variant(selected_variant(mode == current_theme_mode))
      .on_press(message)
  });

  group(theme_buttons)
    .size(Sizing::Sm)
    .frame_width(4.0)
    .into()
}
