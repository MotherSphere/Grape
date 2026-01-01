use crate::ui::message::{PlaybackMessage, UiMessage};
use crate::ui::state::{PlaybackState, RepeatMode};
use iced::widget::{button, column, container, progress_bar, row, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub struct PlayerBar {
    artwork: String,
    title: String,
    artist: String,
    playback: PlaybackState,
    volume: u8,
    queue_active: bool,
}

impl PlayerBar {
    pub fn new(title: impl Into<String>, artist: impl Into<String>) -> Self {
        Self {
            artwork: "██".to_string(),
            title: title.into(),
            artist: artist.into(),
            playback: PlaybackState::default(),
            volume: 70,
            queue_active: false,
        }
    }

    pub fn with_artwork(mut self, artwork: impl Into<String>) -> Self {
        self.artwork = artwork.into();
        self
    }

    pub fn with_playback(mut self, playback: PlaybackState) -> Self {
        self.playback = playback;
        self
    }

    pub fn with_volume(mut self, volume: u8) -> Self {
        self.volume = volume.min(100);
        self
    }

    pub fn with_queue(mut self, queue_active: bool) -> Self {
        self.queue_active = queue_active;
        self
    }

    pub fn toggle_play_pause_message(&self) -> UiMessage {
        UiMessage::Playback(PlaybackMessage::TogglePlayPause)
    }

    pub fn next_track_message(&self) -> UiMessage {
        UiMessage::Playback(PlaybackMessage::NextTrack)
    }

    pub fn previous_track_message(&self) -> UiMessage {
        UiMessage::Playback(PlaybackMessage::PreviousTrack)
    }

    pub fn toggle_shuffle_message(&self) -> UiMessage {
        UiMessage::Playback(PlaybackMessage::ToggleShuffle)
    }

    pub fn cycle_repeat_message(&self) -> UiMessage {
        UiMessage::Playback(PlaybackMessage::CycleRepeat)
    }

    pub fn view(self) -> Element<'static, UiMessage> {
        let PlayerBar {
            artwork,
            title,
            artist,
            playback,
            volume,
            queue_active,
        } = self;
        let left = row![
            text(format!("[{}]", artwork)).size(18),
            column![text(title).size(16), text(artist).size(12)]
                .spacing(4)
                .align_items(Alignment::Start)
        ]
        .spacing(8)
        .align_items(Alignment::Center)
        .width(Length::FillPortion(3));

        let controls = row![
            button(text(shuffle_icon(playback.shuffle)))
                .on_press(UiMessage::Playback(PlaybackMessage::ToggleShuffle)),
            button(text("⏮")).on_press(UiMessage::Playback(PlaybackMessage::PreviousTrack)),
            button(text(play_pause_icon(playback.is_playing)))
                .on_press(UiMessage::Playback(PlaybackMessage::TogglePlayPause)),
            button(text("⏭")).on_press(UiMessage::Playback(PlaybackMessage::NextTrack)),
            button(text(repeat_icon(playback.repeat)))
                .on_press(UiMessage::Playback(PlaybackMessage::CycleRepeat)),
        ]
        .spacing(8)
        .align_items(Alignment::Center)
        .width(Length::FillPortion(4));

        let elapsed = format_duration(playback.position);
        let duration = format_duration(playback.duration);
        let progress = progress_bar(0.0..=1.0, progress_ratio(playback.position, playback.duration))
            .width(Length::Fill);
        let progress_row = row![text(elapsed), progress, text(duration)]
            .spacing(8)
            .align_items(Alignment::Center)
            .width(Length::Fill);
        let audio_icons = row![text(volume_icon(volume)), text(queue_icon(queue_active))]
            .spacing(8)
            .align_items(Alignment::Center);
        let right = column![progress_row, audio_icons]
            .spacing(6)
            .align_items(Alignment::End)
            .width(Length::FillPortion(5));

        let content = row![left, controls, right]
            .spacing(20)
            .align_items(Alignment::Center)
            .width(Length::Fill);

        container(content)
            .padding(12)
            .width(Length::Fill)
            .into()
    }

    pub fn render(&self) -> String {
        let left = format!("[{}] {} — {}", self.artwork, self.title, self.artist);
        let controls = format!(
            "{} ⏮ {} ⏭ {}",
            shuffle_icon(self.playback.shuffle),
            play_pause_icon(self.playback.is_playing),
            repeat_icon(self.playback.repeat),
        );
        let elapsed = format_duration(self.playback.position);
        let duration = format_duration(self.playback.duration);
        let bar = build_progress_bar(self.playback.position, self.playback.duration, 24);
        let audio_icons = format!("{} {}", volume_icon(self.volume), queue_icon(self.queue_active));

        vec![
            left,
            controls,
            format!("{} {} {}   {}", elapsed, bar, duration, audio_icons),
        ]
        .join("\n")
    }
}

fn shuffle_icon(active: bool) -> &'static str {
    if active {
        "🔀"
    } else {
        "↔"
    }
}

fn play_pause_icon(is_playing: bool) -> &'static str {
    if is_playing {
        "⏸"
    } else {
        "▶"
    }
}

fn repeat_icon(mode: RepeatMode) -> &'static str {
    match mode {
        RepeatMode::Off => "🔁",
        RepeatMode::One => "🔂",
        RepeatMode::All => "🔁",
    }
}

fn volume_icon(volume: u8) -> &'static str {
    match volume {
        0 => "🔇",
        1..=33 => "🔈",
        34..=66 => "🔉",
        _ => "🔊",
    }
}

fn queue_icon(active: bool) -> &'static str {
    if active {
        "📄"
    } else {
        "📃"
    }
}

fn progress_ratio(position: std::time::Duration, duration: std::time::Duration) -> f32 {
    let total = duration.as_secs_f32();
    if total <= 0.0 {
        return 0.0;
    }
    let current = position.as_secs_f32().min(total);
    (current / total).clamp(0.0, 1.0)
}

fn build_progress_bar(position: std::time::Duration, duration: std::time::Duration, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let total = duration.as_secs_f32();
    let current = position.as_secs_f32().min(total);
    let ratio = if total > 0.0 { current / total } else { 0.0 };
    let filled = ((ratio * width as f32).round() as usize).min(width);
    let empty = width.saturating_sub(filled);
    format!("{}{}", "━".repeat(filled), "─".repeat(empty))
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{}:{:02}", minutes, seconds)
}
