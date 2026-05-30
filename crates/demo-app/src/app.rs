mod theme_switcher;
mod top_bar;

use iced::keyboard::{self, key};
use iced::widget::{column, container, operation};
use iced::{Element, Fill, Subscription, Task, alignment};

use self::top_bar::{TopBar, top_bar};

use diffy_ui::Theme;
use diffy_ui::widgets::button::Variant as ButtonVariant;

use iced_code_view::{
  Controller, Document, GutterConfig, GutterInsets, Insets, Message, Style, WrapMode,
};

use crate::files;

#[derive(Debug, Clone)]
pub enum AppMessage {
  CodeView(Message),
  SelectFile(usize),
  TabPressed { shift: bool },
  UseSystemTheme,
  UseLightTheme,
  UseDarkTheme,
  ToggleLineNumbers,
  ToggleWrapMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ThemeMode {
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
  code_view: Controller,
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

    let style = Style {
      gutter: iced_code_view::GutterStyle {
        background_color: iced::Color::TRANSPARENT,
        separator_color: iced::Color::from_rgb(0.35, 0.35, 0.35),
        ..Default::default()
      },
      background_color: iced::Color::from_rgb8(0x18, 0x19, 0x1b),
      ..Default::default()
    };

    let code_view = Controller::new(Document::new(files::read_demo_file(0)))
      .with_border_radius(iced::border::radius(10.0))
      .with_font_size(13.0)
      .with_line_height(20.0)
      .with_gutter_config(gutter_config)
      .with_style(style)
      .with_wrap_mode(iced_code_view::WrapMode::SoftWrap)
      .with_insets(Insets::new(0.0, 8.0));

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
    let top_bar = top_bar(TopBar {
      selected_file: self.selected_file,
      line_count: self.code_view.line_count(),
      line_numbers: self.code_view.gutter_config().content.line_numbers,
      wrap_mode: self.code_view.wrap_mode(),
      theme_override: self.theme_override,
    });

    container(column![top_bar, self.code_view.view().map(AppMessage::CodeView)].spacing(10.0))
      .padding(10.0)
      .width(Fill)
      .height(Fill)
      .align_x(alignment::Horizontal::Center)
      .align_y(alignment::Vertical::Center)
      .into()
  }
}

fn selected_variant(selected: bool) -> ButtonVariant {
  if selected {
    ButtonVariant::Primary
  } else {
    ButtonVariant::Neutral
  }
}
