use iced::advanced::text::Renderer as TextRenderer;

pub use diffy_lucide_icons::Icon;

pub const FONT: iced::Font = iced::Font::new(diffy_lucide_icons::FONT_NAME);

pub fn text<'a, Theme, Renderer>(icon: Icon) -> iced::widget::Text<'a, Theme, Renderer>
where
  Theme: iced::widget::text::Catalog + 'a,
  Renderer: TextRenderer + 'a,
  <Renderer as TextRenderer>::Font: From<iced::Font>,
{
  iced::widget::text(icon.glyph())
    .font(FONT)
    .shaping(iced::widget::text::Shaping::Basic)
    .align_y(iced::alignment::Vertical::Center)
}
