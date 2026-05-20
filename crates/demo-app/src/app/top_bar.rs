use iced::widget::{column, row, space, text};
use iced::{Element, Length};
use iced_code_view::WrapMode;

use crate::files;
use diffy_ui::widgets::button::{Mode, Variant, button};
use diffy_ui::{Icon, Sizing, Theme};

use super::{AppMessage, ThemeMode};

pub(super) struct TopBar {
  pub selected_file: usize,
  pub line_count: usize,
  pub line_numbers: bool,
  pub wrap_mode: WrapMode,
  pub theme_override: Option<ThemeMode>,
}

pub(super) fn top_bar(state: TopBar) -> Element<'static, AppMessage, Theme> {
  let button_size = Sizing::Sm;

  let file_buttons =
    files::FILES
      .iter()
      .enumerate()
      .fold(row![].spacing(8), |row, (index, (name, _))| {
        let button_variant = selected_variant(index == state.selected_file);
        let button_mode = if index == state.selected_file {
          Mode::Fill
        } else {
          Mode::Light
        };

        row.push(
          button(*name)
            .variant(button_variant)
            .mode(button_mode)
            .leading_icon(Icon::File)
            .on_press(AppMessage::SelectFile(index)),
        )
      });

  let gutter_button_variant = selected_variant(state.line_numbers);

  let wrap_button_variant = match state.wrap_mode {
    WrapMode::NoWrap => Variant::Neutral,
    WrapMode::SoftWrap => Variant::Primary,
  };

  let system_theme_button = button(Icon::Monitor)
    .size(button_size)
    .variant(selected_variant(state.theme_override.is_none()))
    .mode(Mode::Fill)
    .on_press(AppMessage::UseSystemTheme);

  let light_theme_button = button(Icon::Sun)
    .size(button_size)
    .variant(selected_variant(
      state.theme_override == Some(ThemeMode::Light),
    ))
    .mode(Mode::Fill)
    .on_press(AppMessage::UseLightTheme);

  let dark_theme_button = button(Icon::Moon)
    .size(button_size)
    .variant(selected_variant(
      state.theme_override == Some(ThemeMode::Dark),
    ))
    .mode(Mode::Fill)
    .on_press(AppMessage::UseDarkTheme);

  let gutter_button = button(Icon::ListOrdered)
    .size(button_size)
    .variant(gutter_button_variant)
    .mode(Mode::Fill)
    .on_press(AppMessage::ToggleLineNumbers);

  let wrap_mode_button = button(Icon::WrapText)
    .size(button_size)
    .variant(wrap_button_variant)
    .mode(Mode::Fill)
    .on_press(AppMessage::ToggleWrapMode);

  let theme_buttons = row![system_theme_button, light_theme_button, dark_theme_button].spacing(8);
  let settings_buttons = row![gutter_button, wrap_mode_button].spacing(8);
  let settings_row = row![settings_buttons, theme_buttons].spacing(16);

  let status = format!(
    "file: {}, lines: {}",
    files::FILES[state.selected_file].0,
    state.line_count
  );

  column![
    row![file_buttons, space::horizontal(), settings_row]
      .spacing(8)
      .width(Length::Fill),
    text(status).font(iced::Font::MONOSPACE).size(16.0),
  ]
  .spacing(10.0)
  .into()
}

fn selected_variant(selected: bool) -> Variant {
  if selected {
    Variant::Primary
  } else {
    Variant::Neutral
  }
}
