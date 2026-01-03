use crate::playlist::Playlist;
use crate::ui::message::UiMessage;
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

pub struct PlaylistView;

impl PlaylistView {
    pub fn new() -> Self {
        Self
    }

    pub fn view<'a>(
        theme: style::ThemeTokens,
        playlist: Option<&'a Playlist>,
    ) -> Element<'a, UiMessage> {
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

        let body = match playlist {
            Some(playlist) if !playlist.items.is_empty() => {
                let mut rows: Vec<Element<'a, UiMessage>> = Vec::new();
                for (index, item) in playlist.items.iter().enumerate() {
                    let index_label = text(format!("{:02}", index + 1))
                        .size(theme.size(12))
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_muted(theme));
                    let title = text(item.title.clone())
                        .size(theme.size(14))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_primary(theme));
                    let subtitle = text(format!("{} — {}", item.artist, item.album))
                        .size(theme.size(12))
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_muted(theme));
                    let track = column![title, subtitle].spacing(2);
                    rows.push(row![index_label, track].spacing(12).into());
                }
                scrollable(column(rows).spacing(12)).into()
            }
            _ => column![
                text("Votre playlist apparaîtra ici.")
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_muted(theme))
            ]
            .spacing(8)
            .into(),
        };

        let panel = container(column![header, body].spacing(16))
            .padding(24)
            .width(Length::FillPortion(2))
            .style(move |_| style::surface_style(theme, style::Surface::Panel));

        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| style::surface_style(theme, style::Surface::AppBackground))
            .into()
    }
}
