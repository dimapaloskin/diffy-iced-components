use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Task, alignment, padding};

use iced_code_view::{CodeDocument, CodeViewController, CodeViewMessage};

const FILES: &[(&str, &str)] = &[
  ("demo_1.rs", include_str!("../assets/files/demo_1.rs")),
  ("demo_2.rs", include_str!("../assets/files/demo_2.rs")),
  ("demo_3.txt", include_str!("../assets/files/demo_3.txt")),
];

fn main() -> iced::Result {
  iced::application(App::new, App::update, App::view).run()
}

#[derive(Debug, Clone)]
enum AppMessage {
  CodeView(CodeViewMessage),
  SelectFile(usize),
}

struct App {
  code_view: CodeViewController,
  selected_file: usize,
}

impl App {
  fn new() -> (Self, Task<AppMessage>) {
    let code_view = CodeViewController::new(CodeDocument::new(FILES[0].1))
      .border_radius(iced::border::radius(12.0))
      .padding(padding::horizontal(10.0));

    let task = code_view.start().map(AppMessage::CodeView);

    (
      Self {
        code_view,
        selected_file: 0,
      },
      task,
    )
  }

  fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
    match message {
      AppMessage::CodeView(message) => self.code_view.update(message).map(AppMessage::CodeView),
      AppMessage::SelectFile(index) => {
        self.selected_file = index;
        self
          .code_view
          .set_document(CodeDocument::new(FILES[index].1))
          .map(AppMessage::CodeView)
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

    let status = match self.code_view.opened_document_source_line_count() {
      Some(lines) => format!("Opened: {}, lines: {}", FILES[self.selected_file].0, lines),
      None => "loading".into(),
    };

    container(column![text(status), file_buttons, self.code_view.view(),].spacing(10.0))
      .padding(10.0)
      .width(Length::Fill)
      .height(Length::Fill)
      .align_x(alignment::Horizontal::Center)
      .align_y(alignment::Vertical::Center)
      .into()
  }
}
