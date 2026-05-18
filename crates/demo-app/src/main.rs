mod app;
mod files;
mod ui;

use app::App;
use diffy_lucide_icons::LUCIDE_FONT_BYTES;

fn main() -> iced::Result {
  iced::application(App::new, App::update, App::view)
    .theme(App::theme)
    .subscription(App::subscription)
    .font(LUCIDE_FONT_BYTES)
    .antialiasing(true)
    .title("Demo")
    .run()
}
