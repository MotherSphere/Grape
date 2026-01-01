#![allow(dead_code)]

use crate::player::NowPlaying;
use crate::ui::message::{PlaylistMessage, UiMessage};
use crate::ui::style;
use iced::font::Weight;
use iced::theme::{Button, Container, TextInput};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PlaylistPanel {
    name: String,
    name_draft: String,
    items: Vec<NowPlaying>,
}

impl PlaylistPanel {
    pub fn new(
        name: impl Into<String>,
        name_draft: impl Into<String>,
        items: Vec<NowPlaying>,
    ) -> Self {
        Self {
            name: name.into(),
            name_draft: name_draft.into(),
            items,
        }
    }

    pub fn view(&self) -> Element<'static, UiMessage> {
        let header = row![
            text("Playlist")
                .size(16)
                .font(style::font_propo(Weight::Semibold))
                .style(style::text_primary()),
            text(format!("{} tracks", self.items.len()))
                .size(12)
                .font(style::font_propo(Weight::Light))
                .style(style::text_muted())
        ]
        .spacing(8)
        .align_items(Alignment::Center);

        let name_label = text(self.name.clone())
            .size(14)
            .font(style::font_propo(Weight::Medium))
            .style(style::text_primary());

        let name_input = text_input("Playlist name", &self.name_draft)
            .style(TextInput::Custom(Box::new(style::SearchInput)))
            .on_input(|value| UiMessage::Playlist(PlaylistMessage::NameChanged(value)));
        let create_button = button(
            text("Create")
                .size(12)
                .font(style::font_propo(Weight::Medium)),
        )
        .style(Button::Custom(Box::new(style::ButtonStyle(
            style::ButtonKind::Icon,
        ))))
        .on_press(UiMessage::Playlist(PlaylistMessage::Create));
        let name_row = row![name_input, create_button]
            .spacing(8)
            .align_items(Alignment::Center);

        let list_items = if self.items.is_empty() {
            vec![
                text("No tracks yet")
                    .size(12)
                    .font(style::font_propo(Weight::Light))
                    .style(style::text_muted())
                    .into(),
            ]
        } else {
            self.items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let number = text(format!("{:>2}", index + 1))
                        .size(12)
                        .font(style::font_mono(Weight::Medium))
                        .style(style::text_muted());
                    let title = text(item.title.clone())
                        .size(14)
                        .font(style::font_propo(Weight::Medium))
                        .style(style::text_primary());
                    let artist = text(item.artist.clone())
                        .size(12)
                        .font(style::font_propo(Weight::Light))
                        .style(style::text_muted());
                    let details = column![title, artist]
                        .spacing(2)
                        .width(Length::Fill)
                        .align_items(Alignment::Start);
                    let duration = text(format_duration(Duration::from_secs(
                        item.duration_secs as u64,
                    )))
                    .size(12)
                    .font(style::font_mono(Weight::Medium))
                    .style(style::text_muted());
                    let remove_button =
                        button(text("✕").size(12).font(style::font_propo(Weight::Medium)))
                            .style(Button::Custom(Box::new(style::ButtonStyle(
                                style::ButtonKind::Icon,
                            ))))
                            .on_press(UiMessage::Playlist(PlaylistMessage::RemoveTrack(index)));
                    row![number, details, duration, remove_button]
                        .spacing(8)
                        .align_items(Alignment::Center)
                        .width(Length::Fill)
                        .into()
                })
                .collect::<Vec<Element<UiMessage>>>()
        };

        let list = column(list_items)
            .spacing(8)
            .width(Length::Fill)
            .align_items(Alignment::Start);
        let scrollable_list = scrollable(list).height(Length::Fill);

        let content = column![header, name_label, name_row, scrollable_list]
            .spacing(12)
            .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .style(Container::Custom(Box::new(style::SurfaceStyle(
                style::Surface::Panel,
            ))))
            .into()
    }
}

fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    let minutes = total / 60;
    let seconds = total % 60;
    format!("{minutes}:{seconds:02}")
}
