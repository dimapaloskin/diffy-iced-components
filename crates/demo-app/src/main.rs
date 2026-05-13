use iced::widget::{button, column, container, row, space, text};
use iced::{Element, Length, Task, alignment};

use iced_code_view::{
  CodeViewController, CodeViewMessage, CodeViewPadding, CodeViewStyle, Document, GutterConfig,
  GutterPadding,
};

const FILES: &[(&str, &str)] = &[
  ("demo_1.rs", "./assets/files/demo_1.rs"),
  ("demo_2.rs", "./assets/files/demo_2.rs"),
  ("demo_3.txt", "./assets/files/demo_3.txt"),
];

fn read_demo_file(index: usize) -> String {
  let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FILES[index].1);

  std::fs::read_to_string(path).expect("demo file should be readable")
}

fn main() -> iced::Result {
  iced::application(App::new, App::update, App::view).run()
}

#[derive(Debug, Clone)]
enum AppMessage {
  CodeView(CodeViewMessage),
  SelectFile(usize),
  ToggleLineNumbers,
}

struct App {
  code_view: CodeViewController,
  selected_file: usize,
}

impl App {
  fn new() -> (Self, Task<AppMessage>) {
    let gutter_config = GutterConfig {
      separator_width: 2.0,
      padding: GutterPadding::new(8.0),
      ..Default::default()
    };

    let style = CodeViewStyle {
      gutter: iced_code_view::GutterStyle {
        background_color: iced::Color::TRANSPARENT,
        separator_color: iced::Color::from_rgb(0.35, 0.35, 0.35),
        ..Default::default()
      },
      background_color: iced::Color::from_rgb(0.129, 0.141, 0.165),
      ..Default::default()
    };

    let code_view = CodeViewController::new(Document::new(read_demo_file(0)))
      .with_border_radius(iced::border::radius(12.0))
      .with_font_size(15.0)
      .with_gutter_config(gutter_config)
      .with_style(style)
      .with_padding(CodeViewPadding::new(0.0, 8.0));

    (
      Self {
        code_view,
        selected_file: 0,
      },
      Task::none(),
    )
  }

  fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
    match message {
      AppMessage::CodeView(message) => self.code_view.update(message).map(AppMessage::CodeView),
      AppMessage::SelectFile(index) => {
        self.selected_file = index;
        self
          .code_view
          .set_document(Document::new(read_demo_file(index)));
        Task::none()
      }
      AppMessage::ToggleLineNumbers => {
        let mut gutter_config = self.code_view.gutter_config();
        gutter_config.content.line_numbers = !gutter_config.content.line_numbers;
        self.code_view.set_gutter_config(gutter_config);
        Task::none()
      }
    }
  }

  fn view(&self) -> Element<'_, AppMessage> {
    let file_buttons = FILES
      .iter()
      .enumerate()
      .fold(row![].spacing(8), |row, (index, (name, _))| {
        row.push(button(*name).on_press(AppMessage::SelectFile(index)))
      });

    let gutter_label = if self.code_view.gutter_config().content.line_numbers {
      "line numbers:  on"
    } else {
      "line numbers: off"
    };

    let gutter_button = button(gutter_label).on_press(AppMessage::ToggleLineNumbers);

    let status = format!(
      "file: {}, lines: {}",
      FILES[self.selected_file].0,
      self.code_view.line_count()
    );

    container(
      column![
        text(status).font(iced::Font::MONOSPACE),
        row![file_buttons, space::horizontal(), gutter_button],
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
