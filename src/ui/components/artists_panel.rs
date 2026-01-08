#![allow(dead_code)]

use crate::ui::message::UiMessage;
use crate::ui::state::{Artist, SelectionState};
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub struct ArtistsPanel {
    total_count: usize,
    artists: Vec<Artist>,
    selected_artist_id: Option<usize>,
    scroll_offset: usize,
    viewport_size: usize,
}

impl ArtistsPanel {
    pub fn new(artists: Vec<Artist>) -> Self {
        let total_count = artists.len();
        Self {
            total_count,
            artists,
            selected_artist_id: None,
            scroll_offset: 0,
            viewport_size: 8,
        }
    }

    pub fn with_selection(mut self, selected_artist_id: Option<usize>) -> Self {
        self.selected_artist_id = selected_artist_id;
        self
    }

    pub fn with_scroll(mut self, scroll_offset: usize, viewport_size: usize) -> Self {
        self.scroll_offset = scroll_offset.min(self.artists.len());
        self.viewport_size = viewport_size.max(1);
        self
    }

    pub fn message_for_artist(&self, artist_id: usize) -> Option<UiMessage> {
        self.artists
            .iter()
            .find(|artist| artist.id == artist_id)
            .cloned()
            .map(UiMessage::SelectArtist)
    }

    pub fn view(
        &self,
        selection: &SelectionState,
        focused: bool,
        theme: style::ThemeTokens,
    ) -> Element<'static, UiMessage> {
        let selected_id = selection.selected_artist.as_ref().map(|artist| artist.id);
        let header = row![
            text(format!("{} Song artists", self.total_count))
                .size(theme.size(16))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            text("A–Z")
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Light))
                .style(move |_| style::text_style_muted(theme))
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        let list_content: Element<'static, UiMessage> = if self.artists.is_empty() {
            let empty = column![
                text("Aucun artiste disponible")
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
                text("Modifiez les filtres pour afficher plus de résultats.")
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Light))
                    .style(move |_| style::text_style_muted(theme)),
            ]
            .spacing(6)
            .align_x(Alignment::Center);
            container(empty)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        } else {
            let list_items = self
                .artists
                .iter()
                .map(|artist| {
                    let is_selected = Some(artist.id) == selected_id;
                    let avatar = container(
                        text(artist.name.chars().next().unwrap_or('?').to_string())
                            .size(theme.size_accessible(12))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_primary(theme)),
                    )
                    .width(Length::Fixed(24.0))
                    .height(Length::Fixed(24.0))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(move |_| style::surface_style(theme, style::Surface::Avatar));
                    let label = text(artist.name.clone())
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_primary(theme))
                        .size(theme.size(14));
                    let row_content = row![avatar, label]
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .width(Length::Fill);
                    button(row_content)
                        .style(move |_, status| {
                            style::button_style(
                                theme,
                                style::ButtonKind::ListItem {
                                    selected: is_selected,
                                    focused: focused && is_selected,
                                },
                                status,
                            )
                        })
                        .on_press(UiMessage::SelectArtist(artist.clone()))
                        .width(Length::Fill)
                        .into()
                })
                .collect::<Vec<Element<UiMessage>>>();
            let list = column(list_items)
                .spacing(6)
                .width(Length::Fill)
                .align_x(Alignment::Start);
            scrollable(list).height(Length::Fill).into()
        };
        let index_items = ('A'..='Z')
            .map(|letter| {
                text(letter.to_string())
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Light))
                    .style(move |_| style::text_style_muted(theme))
                    .into()
            })
            .collect::<Vec<Element<UiMessage>>>();
        let index = column(index_items).spacing(4).align_x(Alignment::Center);
        let body = row![list_content, index].spacing(12).height(Length::Fill);
        let content = column![header, body].spacing(12).height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .style(move |_| style::surface_style(theme, style::Surface::Sidebar))
            .into()
    }

    pub fn render(&self) -> String {
        let header = format!("{} Song artists", self.total_count);
        let visible = self
            .artists
            .iter()
            .skip(self.scroll_offset)
            .take(self.viewport_size)
            .collect::<Vec<_>>();
        let index_letters: Vec<char> = ('A'..='Z').collect();
        let mut lines = Vec::with_capacity(visible.len() + 1);
        lines.push(header);

        for (row, artist) in visible.into_iter().enumerate() {
            let is_selected = Some(artist.id) == self.selected_artist_id;
            let name = if is_selected {
                format!("> {} <", artist.name)
            } else {
                artist.name.clone()
            };
            let index = index_letters
                .get(row)
                .map(|letter| letter.to_string())
                .unwrap_or_else(|| " ".to_string());
            lines.push(format!("{:<24} {}", name, index));
        }

        lines.join("\n")
    }
}
