use crate::ui::message::UiMessage;
use crate::ui::state::{SelectionState, Track};
use crate::ui::style;
use iced::theme::{Button, Container};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub struct SongsPanel {
    album: String,
    artist: String,
    tracks: Vec<Track>,
    selected_track_id: Option<usize>,
    scroll_offset: usize,
    viewport_size: usize,
}

impl SongsPanel {
    pub fn new(album: impl Into<String>, artist: impl Into<String>, tracks: Vec<Track>) -> Self {
        Self {
            album: album.into(),
            artist: artist.into(),
            tracks,
            selected_track_id: None,
            scroll_offset: 0,
            viewport_size: 8,
        }
    }

    pub fn with_selection(mut self, selected_track_id: Option<usize>) -> Self {
        self.selected_track_id = selected_track_id;
        self
    }

    pub fn with_scroll(mut self, scroll_offset: usize, viewport_size: usize) -> Self {
        self.scroll_offset = scroll_offset.min(self.tracks.len());
        self.viewport_size = viewport_size.max(1);
        self
    }

    pub fn message_for_track(&self, track_id: usize) -> Option<UiMessage> {
        self.tracks
            .iter()
            .find(|track| track.id == track_id)
            .cloned()
            .map(UiMessage::SelectTrack)
    }

    pub fn view(&self, selection: &SelectionState) -> Element<'static, UiMessage> {
        let selected_id = selection.selected_track.as_ref().map(|track| track.id);
        let header = row![
            text(format!("{} Songs", self.tracks.len()))
                .size(16)
                .style(style::text_primary()),
            text("By album").size(12).style(style::text_muted())
        ]
        .spacing(8)
        .align_items(Alignment::Center);
        let album_info = column![
            text(self.album.clone())
                .size(18)
                .style(style::text_primary()),
            text(self.artist.clone())
                .size(12)
                .style(style::text_muted())
        ]
        .spacing(4)
        .align_items(Alignment::Start);
        let list_items = self
            .tracks
            .iter()
            .enumerate()
            .map(|(index, track)| {
                let is_selected = Some(track.id) == selected_id;
                let number = track.track_number.unwrap_or((index + 1) as u32).to_string();
                let number_label = text(number).size(12).style(style::text_muted());
                let title = text(track.title.clone())
                    .size(14)
                    .style(style::text_primary());
                let artist = text(track.artist.clone())
                    .size(12)
                    .style(style::text_muted());
                let details = column![title, artist]
                    .spacing(2)
                    .width(Length::Fill)
                    .align_items(Alignment::Start);
                let duration = text(format_duration(track.duration))
                    .size(12)
                    .style(style::text_muted());
                let row_content = row![number_label, details, duration]
                    .spacing(12)
                    .align_items(Alignment::Center)
                    .width(Length::Fill);

                button(row_content)
                    .style(Button::Custom(Box::new(style::ButtonStyle(
                        style::ButtonKind::ListItem {
                            selected: is_selected,
                        },
                    ))))
                    .on_press(UiMessage::SelectTrack(track.clone()))
                    .width(Length::Fill)
                    .into()
            })
            .collect::<Vec<Element<UiMessage>>>();
        let list = column(list_items)
            .spacing(8)
            .width(Length::Fill)
            .align_items(Alignment::Start);
        let scrollable_list = scrollable(list).height(Length::Fill);
        let content = column![header, album_info, scrollable_list]
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

    pub fn render(&self) -> String {
        let header = format!(
            "{} Songs · {} — {}",
            self.tracks.len(),
            self.album,
            self.artist
        );
        let visible = self
            .tracks
            .iter()
            .skip(self.scroll_offset)
            .take(self.viewport_size)
            .collect::<Vec<_>>();
        let max_title_len = self
            .tracks
            .iter()
            .map(|track| track.title.len())
            .max()
            .unwrap_or(0)
            .max(12);
        let title_width = max_title_len + 4;
        let duration_width = self
            .tracks
            .iter()
            .map(|track| format_duration(track.duration).len())
            .max()
            .unwrap_or(0)
            .max(4);
        let mut lines = Vec::with_capacity(visible.len() * 2 + 1);
        lines.push(header);

        for (row, track) in visible.into_iter().enumerate() {
            let is_selected = Some(track.id) == self.selected_track_id;
            let number = track
                .track_number
                .map(|value| format!("{:>2}", value))
                .unwrap_or_else(|| format!("{:>2}", self.scroll_offset + row + 1));
            let duration = format_duration(track.duration);
            let title = if is_selected {
                format!("> {} <", track.title)
            } else {
                track.title.clone()
            };
            let artist = if is_selected {
                format!("> {} <", track.artist)
            } else {
                track.artist.clone()
            };
            lines.push(format!(
                "{}. {:<title_width$} {:>duration_width$}",
                number,
                title,
                duration,
                title_width = title_width,
                duration_width = duration_width
            ));
            lines.push(format!("    {}", artist));
        }

        lines.join("\n")
    }
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{}:{:02}", minutes, seconds)
}
