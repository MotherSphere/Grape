use crate::ui::message::UiMessage;
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

pub struct PlaylistView;

impl PlaylistView {
    pub fn new() -> Self {
        Self
    }

    pub fn view(theme: style::ThemeTokens) -> Element<'static, UiMessage> {
        let header = row![
            text("Playlist")
                .size(theme.size(24))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            button(
                text("✕")
                    .size(theme.size(16))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .on_press(UiMessage::ClosePlaylist)
        ]
        .align_y(Alignment::Center)
        .spacing(12);

        let body = column![
            text("Votre playlist apparaîtra ici.")
                .size(theme.size(14))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_muted(theme))
        ]
        .spacing(8);

        let panel = container(column![header, body].spacing(16))
            .padding(24)
            .width(Length::FillPortion(2))
            .style(move |_| style::surface_style(theme, style::Surface::Panel));

        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(move |_| style::surface_style(theme, style::Surface::AppBackground))
            .into()
    }
}
