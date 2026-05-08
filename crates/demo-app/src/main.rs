use iced::widget::{column, container, text};
use iced::{Element, Length, alignment, padding};

use iced_code_view::{CodeDocument, CodeView};

const DEMO_TEXT: &str = r#"Hello there!
How are you?

fn main() -> iced::Result {
  iced::application(App::new, App::update, App::view).run()
}

struct App {
  document: CodeDocument,
}

impl App {
  fn new() -> Self {
    Self {
      document: CodeDocument::new(DEMO_TEXT),
    }
  }

  fn update(&mut self, _: ()) {}
  fn view(&self) -> Element<'_, ()> {
    container(
      column![
        text("Hello there"),
        CodeView::new(self.document.clone())
          .border_radius(iced::border::radius(12.0)).border_radius(iced::border::radius(12.0)).border_radius(iced::border::radius(12.0))
          .padding(padding::all(10.0))
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

fn main() -> iced::Result {
  iced::application(App::new, App::update, App::view).run()
}

struct App {
  document: CodeDocument,
}

impl App {
  fn new() -> Self {
    Self {
      document: CodeDocument::new(DEMO_TEXT),
    }
  }

  fn update(&mut self, _: ()) {}
  fn view(&self) -> Element<'_, ()> {
    container(
      column![
        text("Hello there"),
        CodeView::new(self.document.clone())
          .border_radius(iced::border::radius(12.0))
          .padding(padding::all(10.0))
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
"#;

fn main() -> iced::Result {
  iced::application(App::new, App::update, App::view).run()
}

struct App {
  document: CodeDocument,
}

impl App {
  fn new() -> Self {
    Self {
      document: CodeDocument::new(DEMO_TEXT),
    }
  }

  fn update(&mut self, _: ()) {}
  fn view(&self) -> Element<'_, ()> {
    container(
      column![
        text("Hello there"),
        CodeView::new(self.document.clone())
          .border_radius(iced::border::radius(12.0))
          .padding(padding::all(10.0))
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
