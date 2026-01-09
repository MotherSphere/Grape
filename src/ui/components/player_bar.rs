#![allow(dead_code)]

use crate::ui::message::{PlaybackMessage, UiMessage};
use crate::ui::state::{PlaybackState, RepeatMode};
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{button, column, container, image, progress_bar, row, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone)]
pub struct PlayerBar {
    cover_path: Option<std::path::PathBuf>,
    title: String,
    artist: String,
    playback: PlaybackState,
    volume: u8,
    queue_active: bool,
    queue_message: Option<UiMessage>,
    volume_bar_active: bool,
    volume_message: Option<UiMessage>,
}

impl PlayerBar {
    pub fn new(title: impl Into<String>, artist: impl Into<String>) -> Self {
        Self {
            cover_path: None,
            title: title.into(),
            artist: artist.into(),
            playback: PlaybackState::default(),
            volume: 70,
            queue_active: false,
            queue_message: None,
            volume_bar_active: false,
            volume_message: None,
        }
    }

    pub fn with_cover(mut self, cover_path: Option<std::path::PathBuf>) -> Self {
        self.cover_path = cover_path;
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

    pub fn with_queue_action(mut self, message: Option<UiMessage>) -> Self {
        self.queue_message = message;
        self
    }

    pub fn with_volume_bar(mut self, active: bool) -> Self {
        self.volume_bar_active = active;
        self
    }

    pub fn with_volume_action(mut self, message: Option<UiMessage>) -> Self {
        self.volume_message = message;
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

    pub fn view(self, theme: style::ThemeTokens) -> Element<'static, UiMessage> {
        let PlayerBar {
            cover_path,
            title,
            artist,
            playback,
            volume,
            queue_active,
            queue_message,
            volume_bar_active,
            volume_message,
        } = self;
        let cover_content: Element<UiMessage> = if let Some(path) = cover_path {
            image(image::Handle::from_path(path))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            text("♪")
                .size(theme.size(18))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_muted(theme))
                .into()
        };
        let cover = container(cover_content)
            .width(Length::Fixed(32.0))
            .height(Length::Fixed(32.0))
            .style(move |_| style::surface_style(theme, style::Surface::AlbumCover));
        let left = row![
            cover,
            column![
                text(title)
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
                text(artist)
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Light))
                    .style(move |_| style::text_style_muted(theme))
            ]
            .spacing(4)
            .align_x(Alignment::Start)
        ]
        .spacing(12)
        .align_y(Alignment::Center)
        .width(Length::FillPortion(3));

        let controls = row![
            button(text(shuffle_icon(playback.shuffle)).font(style::font_propo(Weight::Medium)),)
                .style(move |_, status| {
                    style::button_style(theme, style::ButtonKind::Icon, status)
                })
                .on_press(UiMessage::Playback(PlaybackMessage::ToggleShuffle)),
            button(text("\u{f04ae}").font(style::font_propo(Weight::Medium)))
                .style(move |_, status| {
                    style::button_style(theme, style::ButtonKind::Control, status)
                })
                .on_press(UiMessage::Playback(PlaybackMessage::PreviousTrack)),
            button(
                text(play_pause_icon(playback.is_playing)).font(style::font_propo(Weight::Medium)),
            )
            .style(move |_, status| {
                style::button_style(theme, style::ButtonKind::Control, status)
            })
            .on_press(UiMessage::Playback(PlaybackMessage::TogglePlayPause)),
            button(text("\u{f04ad}").font(style::font_propo(Weight::Medium)))
                .style(move |_, status| {
                    style::button_style(theme, style::ButtonKind::Control, status)
                })
                .on_press(UiMessage::Playback(PlaybackMessage::NextTrack)),
            button(text(repeat_icon(playback.repeat)).font(style::font_propo(Weight::Medium)))
                .style(move |_, status| {
                    style::button_style(theme, style::ButtonKind::Icon, status)
                })
                .on_press(UiMessage::Playback(PlaybackMessage::CycleRepeat)),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .width(Length::FillPortion(4));

        let elapsed = format_duration(playback.position);
        let duration = format_duration(playback.duration);
        let progress =
            container(progress_bar(0.0..=1.0, playback.animated_progress)).width(Length::Fill);
        let progress_row = row![
            text(elapsed)
                .size(theme.size_accessible(12))
                .font(style::font_mono(Weight::Medium))
                .style(move |_| style::text_style_muted(theme)),
            progress,
            text(duration)
                .size(theme.size_accessible(12))
                .font(style::font_mono(Weight::Medium))
                .style(move |_| style::text_style_muted(theme))
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Length::Fill);
        let queue_label = text(queue_icon(queue_active))
            .font(style::font_propo(Weight::Medium))
            .style(move |_| {
                if queue_active {
                    style::text_style_primary(theme)
                } else {
                    style::text_style_muted(theme)
                }
            });
        let queue_control: Element<UiMessage> = if let Some(message) = queue_message {
            button(queue_label)
                .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                .on_press(message)
                .into()
        } else {
            queue_label.into()
        };
        let volume_label = text(volume_icon(volume))
            .font(style::font_propo(Weight::Medium))
            .style(move |_| {
                if volume_bar_active {
                    style::text_style_primary(theme)
                } else {
                    style::text_style_muted(theme)
                }
            });
        let volume_control: Element<UiMessage> = if let Some(message) = volume_message {
            button(volume_label)
                .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                .on_press(message)
                .into()
        } else {
            volume_label.into()
        };
        let audio_icons = row![volume_control, queue_control]
            .spacing(8)
            .align_y(Alignment::Center);
        let right = column![progress_row, audio_icons]
            .spacing(6)
            .align_x(Alignment::End)
            .width(Length::FillPortion(5));

        let content = row![left, controls, right]
            .spacing(20)
            .align_y(Alignment::Center)
            .width(Length::Fill);

        container(content)
            .padding([10, 16])
            .width(Length::Fill)
            .style(move |_| style::surface_style(theme, style::Surface::PlayerBar))
            .into()
    }

    pub fn render(&self) -> String {
        let artwork = if self.cover_path.is_some() {
            "🖼"
        } else {
            "♪"
        };
        let left = format!("[{}] {} — {}", artwork, self.title, self.artist);
        let controls = format!(
            "{} ⏮ {} ⏭ {}",
            shuffle_icon(self.playback.shuffle),
            play_pause_icon(self.playback.is_playing),
            repeat_icon(self.playback.repeat),
        );
        let elapsed = format_duration(self.playback.position);
        let duration = format_duration(self.playback.duration);
        let bar = build_progress_bar(self.playback.position, self.playback.duration, 24);
        let audio_icons = format!(
            "{} {}",
            volume_icon(self.volume),
            queue_icon(self.queue_active)
        );

        vec![
            left,
            controls,
            format!("{} {} {}   {}", elapsed, bar, duration, audio_icons),
        ]
        .join("\n")
    }
}

fn shuffle_icon(active: bool) -> &'static str {
    if active { "\u{f049d}" } else { "\u{f049e}" }
}

fn play_pause_icon(is_playing: bool) -> &'static str {
    if is_playing { "\u{f03e4}" } else { "\u{f040a}" }
}

fn repeat_icon(mode: RepeatMode) -> &'static str {
    match mode {
        RepeatMode::Off => "\u{f0457}",
        RepeatMode::One => "\u{f0458}",
        RepeatMode::All => "\u{f0456}",
    }
}

fn volume_icon(volume: u8) -> &'static str {
    match volume {
        0 => "\u{f0581}",
        1..=33 => "\u{f057f}",
        34..=66 => "\u{f0580}",
        _ => "\u{f057e}",
    }
}

fn queue_icon(active: bool) -> &'static str {
    if active { "\u{f0cb8}" } else { "\u{f0cb9}" }
}

fn build_progress_bar(
    position: std::time::Duration,
    duration: std::time::Duration,
    width: usize,
) -> String {
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
