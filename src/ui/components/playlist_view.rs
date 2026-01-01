use crate::ui::message::UiMessage;
use crate::ui::style;
use iced::font::Weight;
use iced::theme::{Button, Container};
use iced::widget::{button, column, container, row, text};
use iced::{Alignment, Element, Length};

pub struct PlaylistView;

impl PlaylistView {
    pub fn new() -> Self {
        Self
    }

    pub fn view(&self) -> Element<'_, UiMessage> {
        let header = row![
            text("Playlist")
                .size(24)
                .font(style::font_propo(Weight::Semibold))
                .style(style::text_primary()),
            button(
                text("✕")
                    .size(16)
                    .font(style::font_propo(Weight::Medium))
                    .style(style::text_primary()),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle(
                style::ButtonKind::Icon,
            ))))
            .on_press(UiMessage::ClosePlaylist)
        ]
        .align_items(Alignment::Center)
        .spacing(12);

        let body = column![
            text("Votre playlist apparaîtra ici.")
                .size(14)
                .font(style::font_propo(Weight::Medium))
                .style(style::text_muted())
        ]
        .spacing(8);

        let panel = container(column![header, body].spacing(16))
            .padding(24)
            .width(Length::FillPortion(2))
            .style(Container::Custom(Box::new(style::SurfaceStyle(
                style::Surface::Panel,
            ))));

        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(Container::Custom(Box::new(style::SurfaceStyle(
                style::Surface::AppBackground,
            ))))
            .into()
    }
}
