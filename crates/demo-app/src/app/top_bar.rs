use iced::widget::{column, row, space, text};
use iced::{Element, Fill};

use iced_code_view::WrapMode;

use diffy_ui::widgets::button::{Mode as ButtonMode, Variant as ButtonVariant, button};

use diffy_ui::{Icon, Sizing, Theme};

use crate::files;

use super::theme_switcher::theme_switcher;
use super::{AppMessage, ThemeMode, selected_variant};

pub(super) struct TopBar {
  pub selected_file: usize,
  pub line_count: usize,
  pub line_numbers: bool,
  pub wrap_mode: WrapMode,
  pub theme_override: Option<ThemeMode>,
}

pub(super) fn top_bar(state: TopBar) -> Element<'static, AppMessage, Theme> {
  let settings_row = row![
    code_settings_buttons(&state),
    theme_switcher(state.theme_override)
  ]
  .align_y(iced::Center)
  .spacing(16);

  let status = format!(
    "file: {}, lines: {}",
    files::FILES[state.selected_file].0,
    state.line_count
  );

  column![
    row![
      file_buttons(state.selected_file),
      space::horizontal(),
      settings_row
    ]
    .align_y(iced::Center)
    .spacing(8)
    .width(Fill),
    text(status).font(iced::Font::MONOSPACE).size(16.0),
  ]
  .spacing(10.0)
  .into()
}

fn file_buttons(selected_file: usize) -> Element<'static, AppMessage, Theme> {
  let file_buttons = files::FILES.iter().enumerate().map(|(index, (name, _))| {
    let selected = index == selected_file;

    let mode = if selected {
      ButtonMode::Fill
    } else {
      ButtonMode::Light
    };
    button(*name)
      .variant(selected_variant(selected))
      .mode(mode)
      .leading_icon(Icon::FileBraces)
      .on_press(AppMessage::SelectFile(index))
      .into()
  });

  row(file_buttons).spacing(8).into()
}

fn code_settings_buttons(state: &TopBar) -> Element<'static, AppMessage, Theme> {
  let gutter_button_variant = selected_variant(state.line_numbers);

  let wrap_button_variant = match state.wrap_mode {
    WrapMode::NoWrap => ButtonVariant::Neutral,
    WrapMode::SoftWrap => ButtonVariant::Primary,
  };

  let gutter_button = button(Icon::ListOrdered)
    .size(Sizing::Sm)
    .variant(gutter_button_variant)
    .fill()
    .on_press(AppMessage::ToggleLineNumbers);

  let wrap_mode_button = button(Icon::WrapText)
    .size(Sizing::Sm)
    .variant(wrap_button_variant)
    .fill()
    .on_press(AppMessage::ToggleWrapMode);

  row![gutter_button, wrap_mode_button].spacing(8).into()
}
