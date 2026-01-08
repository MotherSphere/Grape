use crate::playlist::PlaylistManager;
use crate::ui::message::UiMessage;
use crate::ui::state::SelectionState;
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Element, Length};

pub struct PlaylistView;

impl PlaylistView {
    pub fn new() -> Self {
        Self
    }

    pub fn view<'a>(
        theme: style::ThemeTokens,
        playlists: &'a PlaylistManager,
        selection: &'a SelectionState,
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

        let active_playlist = playlists.active();
        let playlist_rows = if playlists.playlists.is_empty() {
            vec![
                text("Aucune playlist enregistrée.")
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_muted(theme))
                    .into(),
            ]
        } else {
            playlists
                .playlists
                .iter()
                .enumerate()
                .map(|(index, playlist)| {
                    let label = text(format!("{} ({})", playlist.name, playlist.items.len()))
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_primary(theme));
                    let selected = index == playlists.active_index;
                    button(label)
                        .style(move |_, status| {
                            style::button_style(
                                theme,
                                style::ButtonKind::ListItem {
                                    selected,
                                    focused: false,
                                },
                                status,
                            )
                        })
                        .padding([6, 10])
                        .width(Length::Fill)
                        .on_press(UiMessage::SelectPlaylist(index))
                        .into()
                })
                .collect::<Vec<Element<'a, UiMessage>>>()
        };

        let playlist_section = column![
            text("Playlists existantes")
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_muted(theme)),
            scrollable(column(playlist_rows).spacing(6)).height(Length::Fixed(120.0))
        ]
        .spacing(8);

        let action_button = |label: &'static str, message: UiMessage| {
            button(
                text(label)
                    .size(theme.size_accessible(12))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
            .padding([6, 10])
            .on_press(message)
        };

        let mut add_track_button = button(
            text("Ajouter piste")
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Medium))
                .style(move |_| style::text_style_primary(theme)),
        )
        .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
        .padding([6, 10]);
        if selection.selected_track.is_some() {
            add_track_button = add_track_button.on_press(UiMessage::AddSelectedTrackToPlaylist);
        }

        let action_row = column![
            text_input("Nom de la playlist", &selection.playlist_name_draft)
                .style(move |_, status| style::text_input_style(theme, status))
                .on_input(UiMessage::PlaylistNameChanged),
            row![
                action_button("Créer", UiMessage::CreatePlaylist),
                action_button("Renommer", UiMessage::RenamePlaylist),
                action_button("Supprimer", UiMessage::DeletePlaylist),
                add_track_button
            ]
            .spacing(8)
        ]
        .spacing(12);

        let body: Element<'a, UiMessage> = match active_playlist {
            Some(playlist) if !playlist.items.is_empty() => {
                let mut rows: Vec<Element<'a, UiMessage>> = Vec::new();
                let total_items = playlist.items.len();
                for (index, item) in playlist.items.iter().enumerate() {
                    let index_label = text(format!("{:02}", index + 1))
                        .size(theme.size_accessible(12))
                        .font(style::font_propo(Weight::Medium))
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
                        move_up = move_up.on_press(UiMessage::MovePlaylistItemUp(index));
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
                        move_down = move_down.on_press(UiMessage::MovePlaylistItemDown(index));
                    }
                    let remove = button(
                        text("✕")
                            .size(theme.size_accessible(12))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_muted(theme)),
                    )
                    .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
                    .padding([2, 6])
                    .on_press(UiMessage::RemovePlaylistItem(index));
                    let actions = row![move_up, move_down, remove].spacing(4);
                    rows.push(
                        row![index_label, track, actions]
                            .align_y(Alignment::Center)
                            .spacing(12)
                            .into(),
                    );
                }
                scrollable(column(rows).spacing(12)).into()
            }
            _ => column![
                text("Les pistes de la playlist apparaîtront ici.")
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_muted(theme))
            ]
            .spacing(8)
            .into(),
        };

        let panel = container(column![header, playlist_section, action_row, body].spacing(16))
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
