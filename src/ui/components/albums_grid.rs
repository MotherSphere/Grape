use crate::ui::message::UiMessage;
use crate::ui::state::Album;
use crate::ui::style;
use iced::font::Weight;
use iced::theme::{Button, Container};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub struct AlbumsGrid {
    sort_label: String,
    albums: Vec<Album>,
    selected_album_id: Option<usize>,
    columns: usize,
    scroll_offset: usize,
    viewport_rows: usize,
}

impl AlbumsGrid {
    pub fn new(albums: Vec<Album>) -> Self {
        Self {
            sort_label: "A–Z".to_string(),
            albums,
            selected_album_id: None,
            columns: 3,
            scroll_offset: 0,
            viewport_rows: 3,
        }
    }

    pub fn with_sort_label(mut self, sort_label: impl Into<String>) -> Self {
        self.sort_label = sort_label.into();
        self
    }

    pub fn with_selection(mut self, selected_album_id: Option<usize>) -> Self {
        self.selected_album_id = selected_album_id;
        self
    }

    pub fn with_layout(
        mut self,
        columns: usize,
        scroll_offset: usize,
        viewport_rows: usize,
    ) -> Self {
        self.columns = columns.max(1);
        self.scroll_offset = scroll_offset;
        self.viewport_rows = viewport_rows.max(1);
        self
    }

    pub fn message_for_album(&self, album_id: usize) -> Option<UiMessage> {
        self.albums
            .iter()
            .find(|album| album.id == album_id)
            .cloned()
            .map(UiMessage::SelectAlbum)
    }

    pub fn view(self) -> Element<'static, UiMessage> {
        let header = row![
            text(format!("{} Albums", self.albums.len()))
                .size(16)
                .font(style::font_propo(Weight::Semibold))
                .style(style::text_primary()),
            text(format!("{} ", self.sort_label))
                .size(12)
                .font(style::font_propo(Weight::Light))
                .style(style::text_muted())
        ]
        .spacing(8)
        .align_items(Alignment::Center);
        let rows = self
            .albums
            .chunks(self.columns)
            .map(|chunk| {
                let cells = chunk
                    .iter()
                    .map(|album| {
                        let is_selected = Some(album.id) == self.selected_album_id;
                        let cover = container(
                            text("♪")
                                .size(26)
                                .font(style::font_propo(Weight::Medium))
                                .style(style::text_muted()),
                        )
                        .width(Length::Fixed(120.0))
                        .height(Length::Fixed(120.0))
                        .center_x()
                        .center_y()
                        .style(Container::Custom(Box::new(
                            style::SurfaceStyle(style::Surface::AlbumCover),
                        )));

                        let title = text(album.title.clone())
                            .size(14)
                            .font(style::font_propo(Weight::Medium))
                            .style(style::text_primary());
                        let artist = text(album.artist.clone())
                            .size(12)
                            .font(style::font_propo(Weight::Light))
                            .style(style::text_muted());
                        let card = column![cover, title, artist]
                            .spacing(6)
                            .align_items(Alignment::Center)
                            .width(Length::Fill);

                        button(card)
                            .style(Button::Custom(Box::new(style::ButtonStyle(
                                style::ButtonKind::AlbumCard {
                                    selected: is_selected,
                                },
                            ))))
                            .on_press(UiMessage::SelectAlbum(album.clone()))
                            .width(Length::FillPortion(1))
                            .into()
                    })
                    .collect::<Vec<Element<UiMessage>>>();

                row(cells)
                    .spacing(16)
                    .align_items(Alignment::Start)
                    .width(Length::Fill)
                    .into()
            })
            .collect::<Vec<Element<UiMessage>>>();
        let grid = column(rows)
            .spacing(20)
            .width(Length::Fill)
            .align_items(Alignment::Start);
        let content = column![header, grid]
            .spacing(12)
            .width(Length::Fill)
            .align_items(Alignment::Start);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .style(Container::Custom(Box::new(style::SurfaceStyle(
                style::Surface::Panel,
            ))))
            .into()
    }

    pub fn render(&self) -> String {
        let cover_width = 6usize;
        let cover_height = 3usize;
        let longest_label = self
            .albums
            .iter()
            .map(|album| album.title.len().max(album.artist.len()))
            .max()
            .unwrap_or(0);
        let cell_width = cover_width.max(longest_label).max(10);
        let cell_height = cover_height + 2;
        let total_rows = self.albums.len().div_ceil(self.columns);
        let scroll_offset = self.scroll_offset.min(total_rows.saturating_sub(1));

        let mut lines = Vec::new();
        lines.push(format!("Tri: {}", self.sort_label));

        let rows = self.albums.chunks(self.columns).collect::<Vec<_>>();
        let visible_rows = rows.iter().skip(scroll_offset).take(self.viewport_rows);

        for row in visible_rows {
            let cells = row
                .iter()
                .map(|album| self.build_cell(album, cover_width, cover_height))
                .collect::<Vec<_>>();

            for line_idx in 0..cell_height {
                let mut line = String::new();
                for (col, cell) in cells.iter().enumerate() {
                    let content = cell.get(line_idx).map(String::as_str).unwrap_or("");
                    line.push_str(&format!("{:<width$}", content, width = cell_width));
                    if col + 1 < self.columns {
                        line.push_str("  ");
                    }
                }
                lines.push(line.trim_end().to_string());
            }
        }

        lines.join("\n")
    }

    fn build_cell(&self, album: &Album, cover_width: usize, cover_height: usize) -> Vec<String> {
        let is_selected = Some(album.id) == self.selected_album_id;
        let cover_char = if is_selected { '▓' } else { '█' };
        let cover_line = cover_char.to_string().repeat(cover_width);
        let mut cell = Vec::with_capacity(cover_height + 2);

        for _ in 0..cover_height {
            cell.push(cover_line.clone());
        }

        let title = if is_selected {
            format!("> {} <", album.title)
        } else {
            album.title.clone()
        };
        let artist = if is_selected {
            format!("> {} <", album.artist)
        } else {
            album.artist.clone()
        };

        cell.push(title);
        cell.push(artist);
        cell
    }
}
