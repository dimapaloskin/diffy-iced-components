use iced::widget::{column, container, text};
use iced::{Element, Length, alignment};

use iced_code_view::CodeView;

fn main() -> iced::Result {
  iced::application(App::default, App::update, App::view).run()
}

#[derive(Default)]
struct App {}

impl App {
  fn update(&mut self, _: ()) {}
  fn view(&self) -> Element<'_, ()> {
    container(
      column![
        text("Hello there"),
        CodeView::new().border_radius(iced::border::radius(15.0))
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
