use crate::ui::message::UiMessage;
use crate::ui::state::Album;

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

    pub fn with_layout(mut self, columns: usize, scroll_offset: usize, viewport_rows: usize) -> Self {
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
        let visible_rows = rows
            .iter()
            .skip(scroll_offset)
            .take(self.viewport_rows);

        for row in visible_rows {
            let cells = row
                .iter()
                .map(|album| self.build_cell(album, cover_width, cover_height))
                .collect::<Vec<_>>();

            for line_idx in 0..cell_height {
                let mut line = String::new();
                for (col, cell) in cells.iter().enumerate() {
                    let content = cell
                        .get(line_idx)
                        .map(String::as_str)
                        .unwrap_or("");
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
