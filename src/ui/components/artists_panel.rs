use crate::ui::state::Artist;

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
