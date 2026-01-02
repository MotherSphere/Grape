#![allow(dead_code)]

use crate::ui::message::UiMessage;
use crate::ui::state::Genre;
use crate::ui::style;
use iced::font::Weight;
use iced::theme::{Button, Container};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub struct GenresPanel {
    total_count: usize,
    genres: Vec<Genre>,
    selected_genre_id: Option<usize>,
    scroll_offset: usize,
    viewport_size: usize,
}

impl GenresPanel {
    pub fn new(genres: Vec<Genre>) -> Self {
        let total_count = genres.len();
        Self {
            total_count,
            genres,
            selected_genre_id: None,
            scroll_offset: 0,
            viewport_size: 8,
        }
    }

    pub fn with_selection(mut self, selected_genre_id: Option<usize>) -> Self {
        self.selected_genre_id = selected_genre_id;
        self
    }

    pub fn with_scroll(mut self, scroll_offset: usize, viewport_size: usize) -> Self {
        self.scroll_offset = scroll_offset.min(self.genres.len());
        self.viewport_size = viewport_size.max(1);
        self
    }

    pub fn view(&self, theme: style::ThemeTokens) -> Element<'static, UiMessage> {
        let header = row![
            text(format!("{} Genres", self.total_count))
                .size(theme.size(16))
                .font(style::font_propo(Weight::Semibold))
                .style(style::text_primary(theme)),
            text("A–Z")
                .size(theme.size(12))
                .font(style::font_propo(Weight::Light))
                .style(style::text_muted(theme))
        ]
        .spacing(8)
        .align_items(Alignment::Center);
        let list_items =
            self.genres
                .iter()
                .map(|genre| {
                    let is_selected = Some(genre.id) == self.selected_genre_id;
                    let badge = container(
                        text(genre.name.chars().next().unwrap_or('?').to_string())
                            .size(theme.size(12))
                            .font(style::font_propo(Weight::Medium))
                            .style(style::text_primary(theme)),
                    )
                    .width(Length::Fixed(24.0))
                    .height(Length::Fixed(24.0))
                    .center_x()
                    .center_y()
                    .style(Container::Custom(Box::new(
                        style::SurfaceStyle::new(style::Surface::Avatar, theme),
                    )));
                    let name = text(genre.name.clone())
                        .font(style::font_propo(Weight::Medium))
                        .style(style::text_primary(theme))
                        .size(theme.size(14));
                    let count = text(format!("{} tracks", genre.track_count))
                        .font(style::font_propo(Weight::Light))
                        .style(style::text_muted(theme))
                        .size(theme.size(12));
                    let details = column![name, count]
                        .spacing(2)
                        .align_items(Alignment::Start)
                        .width(Length::Fill);
                    let row_content = row![badge, details]
                        .spacing(10)
                        .align_items(Alignment::Center)
                        .width(Length::Fill);
                    button(row_content)
                        .style(Button::Custom(Box::new(style::ButtonStyle::new(
                            style::ButtonKind::ListItem {
                                selected: is_selected,
                            },
                            theme,
                        ))))
                        .on_press(UiMessage::SelectGenre(genre.clone()))
                        .width(Length::Fill)
                        .into()
                })
                .collect::<Vec<Element<UiMessage>>>();
        let list = column(list_items)
            .spacing(6)
            .width(Length::Fill)
            .align_items(Alignment::Start);
        let scrollable_list = scrollable(list).height(Length::Fill);
        let content = column![header, scrollable_list]
            .spacing(12)
            .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .style(Container::Custom(Box::new(style::SurfaceStyle::new(
                style::Surface::Sidebar,
                theme,
            ))))
            .into()
    }

    pub fn render(&self) -> String {
        let header = format!("{} Genres", self.total_count);
        let visible = self
            .genres
            .iter()
            .skip(self.scroll_offset)
            .take(self.viewport_size)
            .collect::<Vec<_>>();
        let mut lines = Vec::with_capacity(visible.len() + 1);
        lines.push(header);

        for genre in visible {
            let is_selected = Some(genre.id) == self.selected_genre_id;
            let name = if is_selected {
                format!("> {} <", genre.name)
            } else {
                genre.name.clone()
            };
            lines.push(format!("{:<18} {:>3} tracks", name, genre.track_count));
        }

        lines.join("\n")
    }
}
