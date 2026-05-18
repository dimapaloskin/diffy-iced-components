use iced::keyboard::{self, key};
use iced::widget::{column, container, operation, row, space, text};
use iced::{Element, Length, Subscription, Task, alignment};

use crate::ui::{Icon, Sizing, Theme, button, button::Variant};

use iced_code_view::{
  CodeViewController, CodeViewInsets, CodeViewMessage, CodeViewStyle, Document, GutterConfig,
  GutterInsets, WrapMode,
};

use crate::files;

#[derive(Debug, Clone)]
pub enum AppMessage {
  CodeView(CodeViewMessage),
  SelectFile(usize),
  TabPressed { shift: bool },
  UseSystemTheme,
  UseLightTheme,
  UseDarkTheme,
  ToggleLineNumbers,
  ToggleWrapMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThemeMode {
  Light,
  Dark,
}

impl ThemeMode {
  fn into_theme(self) -> Theme {
    match self {
      Self::Light => Theme::light(),
      Self::Dark => Theme::dark(),
    }
  }
}

pub struct App {
  code_view: CodeViewController,
  selected_file: usize,
  theme_override: Option<ThemeMode>,
}

impl App {
  pub fn new() -> (Self, Task<AppMessage>) {
    let gutter_config = GutterConfig {
      separator_width: 2.0,
      insets: GutterInsets::new(8.0),
      ..Default::default()
    };

    let style = CodeViewStyle {
      gutter: iced_code_view::GutterStyle {
        background_color: iced::Color::TRANSPARENT,
        separator_color: iced::Color::from_rgb(0.35, 0.35, 0.35),
        ..Default::default()
      },
      background_color: iced::Color::from_rgb8(0x18, 0x19, 0x1b),
      ..Default::default()
    };

    let code_view = CodeViewController::new(Document::new(files::read_demo_file(0)))
      .with_border_radius(iced::border::radius(10.0))
      .with_font_size(13.0)
      .with_line_height(20.0)
      .with_gutter_config(gutter_config)
      .with_style(style)
      .with_wrap_mode(iced_code_view::WrapMode::SoftWrap)
      .with_insets(CodeViewInsets::new(0.0, 8.0));

    (
      Self {
        code_view,
        selected_file: 0,
        theme_override: None,
      },
      Task::none(),
    )
  }

  pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
    match message {
      AppMessage::CodeView(message) => self.code_view.update(message).map(AppMessage::CodeView),
      AppMessage::SelectFile(index) => {
        self.selected_file = index;
        self
          .code_view
          .set_document(Document::new(files::read_demo_file(index)));
        Task::none()
      }
      AppMessage::TabPressed { shift } => {
        if shift {
          operation::focus_previous()
        } else {
          operation::focus_next()
        }
      }
      AppMessage::UseSystemTheme => {
        self.theme_override = None;
        Task::none()
      }
      AppMessage::UseLightTheme => {
        self.theme_override = Some(ThemeMode::Light);
        Task::none()
      }
      AppMessage::UseDarkTheme => {
        self.theme_override = Some(ThemeMode::Dark);
        Task::none()
      }
      AppMessage::ToggleLineNumbers => {
        let mut gutter_config = self.code_view.gutter_config();
        gutter_config.content.line_numbers = !gutter_config.content.line_numbers;
        self.code_view.set_gutter_config(gutter_config);
        Task::none()
      }
      AppMessage::ToggleWrapMode => {
        let wrap_mode = match self.code_view.wrap_mode() {
          WrapMode::NoWrap => WrapMode::SoftWrap,
          WrapMode::SoftWrap => WrapMode::NoWrap,
        };

        self.code_view.set_wrap_mode(wrap_mode);
        Task::none()
      }
    }
  }

  pub fn theme(&self) -> Option<Theme> {
    self.theme_override.map(ThemeMode::into_theme)
  }

  pub fn subscription(&self) -> Subscription<AppMessage> {
    keyboard::listen().filter_map(|event| match event {
      keyboard::Event::KeyPressed {
        key: keyboard::Key::Named(key::Named::Tab),
        modifiers,
        ..
      } => Some(AppMessage::TabPressed {
        shift: modifiers.shift(),
      }),
      _ => None,
    })
  }

  pub fn view(&self) -> Element<'_, AppMessage, Theme> {
    let button_size = Sizing::Sm;

    let file_buttons =
      files::FILES
        .iter()
        .enumerate()
        .fold(row![].spacing(8), |row, (index, (name, _))| {
          let button_variant = if index == self.selected_file {
            Variant::Primary
          } else {
            Variant::Secondary
          };

          row.push(
            button(*name)
              .size(button_size)
              .leading_icon(Icon::File)
              .content_y_offset(-0.5)
              .variant(button_variant)
              .on_press(AppMessage::SelectFile(index)),
          )
        });

    let gutter_button_variant = if self.code_view.gutter_config().content.line_numbers {
      Variant::Primary
    } else {
      Variant::Secondary
    };

    let wrap_button_variant = match self.code_view.wrap_mode() {
      WrapMode::NoWrap => Variant::Secondary,
      WrapMode::SoftWrap => Variant::Primary,
    };

    let system_theme_button = button(Icon::Monitor)
      .size(button_size)
      .variant(selected_variant(self.theme_override.is_none()))
      .on_press(AppMessage::UseSystemTheme);

    let light_theme_button = button(Icon::Sun)
      .size(button_size)
      .variant(selected_variant(
        self.theme_override == Some(ThemeMode::Light),
      ))
      .on_press(AppMessage::UseLightTheme);

    let dark_theme_button = button(Icon::Moon)
      .size(button_size)
      .variant(selected_variant(
        self.theme_override == Some(ThemeMode::Dark),
      ))
      .on_press(AppMessage::UseDarkTheme);

    let gutter_button = button(Icon::ListOrdered)
      .size(button_size)
      .variant(gutter_button_variant)
      .on_press(AppMessage::ToggleLineNumbers);

    let wrap_mode_button = button(Icon::WrapText)
      .size(button_size)
      .variant(wrap_button_variant)
      .on_press(AppMessage::ToggleWrapMode);

    let theme_buttons = row![system_theme_button, light_theme_button, dark_theme_button].spacing(8);
    let settings_buttons = row![gutter_button, wrap_mode_button].spacing(8);
    let settings_row = row![settings_buttons, theme_buttons].spacing(16);

    let status = format!(
      "file: {}, lines: {}",
      files::FILES[self.selected_file].0,
      self.code_view.line_count()
    );

    container(
      column![
        row![file_buttons, space::horizontal(), settings_row],
        text(status).font(iced::Font::MONOSPACE).size(16.0),
        self.code_view.view().map(AppMessage::CodeView),
      ]
      .spacing(10.0),
    )
    .padding(10.0)
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center)
    .into()
  }
}

fn selected_variant(selected: bool) -> Variant {
  if selected {
    Variant::Primary
  } else {
    Variant::Secondary
  }
}
