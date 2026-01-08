use crate::playlist::PlaybackQueue;
use crate::ui::message::UiMessage;
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

pub struct QueueView;

impl QueueView {
    pub fn new() -> Self {
        Self
    }

    pub fn view<'a>(
        theme: style::ThemeTokens,
        playback_queue: &'a PlaybackQueue,
        play_from_queue: bool,
    ) -> Element<'a, UiMessage> {
        let play_from_queue_label = if play_from_queue {
            "Lecture depuis queue : activée"
        } else {
            "Lecture depuis queue : désactivée"
        };
        let header = row![
            text("Queue")
                .size(theme.size(24))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            button(
                text(play_from_queue_label)
                    .size(theme.size(13))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
            .padding([6, 10])
            .on_press(UiMessage::TogglePlayFromQueue),
            button(
                text("Vider")
                    .size(theme.size(13))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
            .padding([6, 10])
            .on_press(UiMessage::ClearQueue),
            button(
                text("✕")
                    .size(theme.size(16))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .on_press(UiMessage::CloseQueue)
        ]
        .align_y(Alignment::Center)
        .spacing(12);

        let body: Element<'a, UiMessage> = if playback_queue.is_empty() {
            column![
                text("La queue est vide.")
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_muted(theme))
            ]
            .into()
        } else {
            let current_index = playback_queue.index();
            let total_items = playback_queue.items().len();
            let rows = playback_queue
                .items()
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let index_label = if index == current_index { "▶" } else { "•" };
                    let index_text = text(index_label)
                        .size(theme.size_accessible(12))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_muted(theme));
                    let title = text(item.title.clone())
                        .size(theme.size(14))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_primary(theme));
                    let subtitle = text(format!("{} — {}", item.artist, item.album))
                        .size(theme.size_accessible(12))
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_muted(theme));
                    let track = column![title, subtitle].spacing(2);

                    let mut move_up = button(
                        text("↑")
                            .size(theme.size_accessible(12))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_muted(theme)),
                    )
                    .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                    .padding([2, 6]);
                    if index > 0 {
                        move_up = move_up.on_press(UiMessage::MoveQueueItemUp(index));
                    }
                    let mut move_down = button(
                        text("↓")
                            .size(theme.size_accessible(12))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_muted(theme)),
                    )
                    .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                    .padding([2, 6]);
                    if index + 1 < total_items {
                        move_down = move_down.on_press(UiMessage::MoveQueueItemDown(index));
                    }
                    let remove = button(
                        text("✕")
                            .size(theme.size_accessible(12))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_muted(theme)),
                    )
                    .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                    .padding([2, 6])
                    .on_press(UiMessage::RemoveQueueItem(index));
                    let actions = row![move_up, move_down, remove].spacing(4);

                    row![index_text, track, actions]
                        .align_y(Alignment::Center)
                        .spacing(12)
                        .into()
                })
                .collect::<Vec<Element<'a, UiMessage>>>();

            scrollable(column(rows).spacing(12)).into()
        };

        let panel = container(column![header, body].spacing(16))
            .padding(24)
            .width(Length::Fill)
            .style(move |_| style::surface_style(theme, style::Surface::Panel));

        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
