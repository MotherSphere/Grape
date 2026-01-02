use crate::config::{
    self, CloseBehavior, InterfaceLanguage, StartupScreen, TextScale, ThemeMode, TimeFormat,
    UpdateChannel,
};
use crate::library::Catalog;
use crate::player::{PlaybackState as PlayerPlaybackState, Player};
use crate::ui::components::albums_grid::AlbumsGrid;
use crate::ui::components::anchored_overlay::AnchoredOverlay;
use crate::ui::components::artists_panel::ArtistsPanel;
use crate::ui::components::folders_panel::FoldersPanel;
use crate::ui::components::genres_panel::GenresPanel;
use crate::ui::components::player_bar::PlayerBar;
use crate::ui::components::playlist_view::PlaylistView;
use crate::ui::components::songs_panel::SongsPanel;
use crate::ui::message::{PlaybackMessage, SearchMessage, UiMessage};
use crate::ui::state::{
    ActiveTab, Album as UiAlbum, Artist as UiArtist, Folder as UiFolder, Genre as UiGenre,
    PreferencesSection, PreferencesTab, SortOption, Track as UiTrack, UiState,
};
use crate::ui::style;
use iced::font::Weight;
use iced::theme::{Button, Container, TextInput};
use iced::widget::{button, column, container, row, scrollable, slider, text, text_input};
use iced::{Alignment, Application, Command, Element, Length, Settings, event, keyboard, mouse};
use iced::{Subscription, Theme};
use std::time::Duration;
use tracing::{error, info};

pub struct GrapeApp {
    catalog: Catalog,
    player: Option<Player>,
    ui: UiState,
}

impl GrapeApp {
    pub fn run(catalog: Catalog) -> iced::Result {
        <Self as Application>::run(Self::apply_font_settings(Settings::with_flags(catalog)))
    }

    pub fn run_with(catalog: Catalog, settings: Settings<Catalog>) -> iced::Result {
        let mut settings = settings;
        settings.flags = catalog;
        <Self as Application>::run(Self::apply_font_settings(settings))
    }

    fn apply_font_settings(mut settings: Settings<Catalog>) -> Settings<Catalog> {
        settings.fonts = vec![
            include_bytes!(
                "../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontPropo-Light.ttf"
            )
            .into(),
            include_bytes!(
                "../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontPropo-Regular.ttf"
            )
            .into(),
            include_bytes!(
                "../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontPropo-Medium.ttf"
            )
            .into(),
            include_bytes!(
                "../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontPropo-SemiBold.ttf"
            )
            .into(),
            include_bytes!(
                "../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontPropo-Bold.ttf"
            )
            .into(),
            include_bytes!(
                "../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontMono-Regular.ttf"
            )
            .into(),
            include_bytes!(
                "../../assets/fonts/JetBrainsMonoFont/JetBrainsMonoNerdFontMono-Medium.ttf"
            )
            .into(),
        ];
        settings.default_font = style::font_propo(Weight::Normal);
        settings
    }

    fn tab_label(&self, _tab: ActiveTab, label: &str) -> String {
        label.to_string()
    }

    fn theme_tokens(&self) -> style::ThemeTokens {
        style::ThemeTokens::new(
            self.ui.settings.theme_mode,
            self.ui.settings.text_scale.scale(),
        )
    }

    fn normalized_query(&self) -> Option<String> {
        let query = self.ui.search.query.trim().to_lowercase();
        if query.is_empty() { None } else { Some(query) }
    }

    fn albums_from_catalog(&self) -> Vec<UiAlbum> {
        let mut albums = Vec::new();
        let mut id = 0usize;

        for artist in &self.catalog.artists {
            for album in &artist.albums {
                albums.push(UiAlbum {
                    id,
                    title: album.title.clone(),
                    artist: artist.name.clone(),
                    year: if album.year == 0 {
                        None
                    } else {
                        Some(album.year as u32)
                    },
                    cover_path: album.cover.as_ref().map(|cover| cover.cached_path.clone()),
                });
                id += 1;
            }
        }

        albums
    }

    fn filtered_albums_from_catalog(&self) -> Vec<UiAlbum> {
        let mut albums = self.albums_from_catalog();
        if let Some(query) = self.normalized_query() {
            albums.retain(|album| {
                album.title.to_lowercase().contains(&query)
                    || album.artist.to_lowercase().contains(&query)
            });
        }
        match self.ui.search.sort {
            SortOption::Alphabetical => {
                albums.sort_by(|a, b| {
                    a.title
                        .to_lowercase()
                        .cmp(&b.title.to_lowercase())
                        .then_with(|| a.artist.to_lowercase().cmp(&b.artist.to_lowercase()))
                });
            }
            SortOption::ByAlbum => {
                albums.sort_by(|a, b| {
                    a.artist
                        .to_lowercase()
                        .cmp(&b.artist.to_lowercase())
                        .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
                        .then_with(|| a.year.cmp(&b.year))
                });
            }
        }
        albums
    }

    fn artists_from_catalog(&self) -> Vec<UiArtist> {
        self.catalog
            .artists
            .iter()
            .enumerate()
            .map(|(id, artist)| UiArtist {
                id,
                name: artist.name.clone(),
            })
            .collect()
    }

    fn filtered_artists_from_catalog(&self) -> Vec<UiArtist> {
        let mut artists = self.artists_from_catalog();
        if let Some(query) = self.normalized_query() {
            artists.retain(|artist| artist.name.to_lowercase().contains(&query));
        }
        artists.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        artists
    }

    fn genres_from_catalog(&self) -> Vec<UiGenre> {
        self.catalog
            .genres()
            .into_iter()
            .enumerate()
            .map(|(id, genre)| UiGenre {
                id,
                name: genre.name,
                track_count: genre.track_count,
            })
            .collect()
    }

    fn filtered_genres_from_catalog(&self) -> Vec<UiGenre> {
        let mut genres = self.genres_from_catalog();
        if let Some(query) = self.normalized_query() {
            genres.retain(|genre| genre.name.to_lowercase().contains(&query));
        }
        genres.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        genres
    }

    fn album_entry_by_id(
        &self,
        album_id: usize,
    ) -> Option<(&crate::library::Artist, &crate::library::Album)> {
        let mut id = 0usize;
        for artist in &self.catalog.artists {
            for album in &artist.albums {
                if id == album_id {
                    return Some((artist, album));
                }
                id += 1;
            }
        }
        None
    }

    fn tracks_for_album(
        &self,
        artist: &crate::library::Artist,
        album: &crate::library::Album,
    ) -> Vec<UiTrack> {
        album
            .tracks
            .iter()
            .enumerate()
            .map(|(id, track)| UiTrack {
                id,
                title: track.title.clone(),
                album: album.title.clone(),
                artist: artist.name.clone(),
                track_number: Some(track.number as u32),
                duration: std::time::Duration::from_secs(track.duration_secs as u64),
                path: track.path.clone(),
                cover_path: album.cover.as_ref().map(|cover| cover.cached_path.clone()),
            })
            .collect()
    }

    fn filtered_tracks_for_album(
        &self,
        artist: &crate::library::Artist,
        album: &crate::library::Album,
    ) -> Vec<UiTrack> {
        let mut tracks = self.tracks_for_album(artist, album);
        if let Some(query) = self.normalized_query() {
            tracks.retain(|track| {
                track.title.to_lowercase().contains(&query)
                    || track.artist.to_lowercase().contains(&query)
                    || track.album.to_lowercase().contains(&query)
            });
        }
        match self.ui.search.sort {
            SortOption::Alphabetical => {
                tracks.sort_by(|a, b| {
                    a.title
                        .to_lowercase()
                        .cmp(&b.title.to_lowercase())
                        .then_with(|| a.track_number.cmp(&b.track_number))
                });
            }
            SortOption::ByAlbum => {
                tracks.sort_by(|a, b| {
                    a.track_number
                        .unwrap_or(u32::MAX)
                        .cmp(&b.track_number.unwrap_or(u32::MAX))
                        .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
                });
            }
        }
        tracks
    }

    fn folders_from_catalog(&self) -> Vec<UiFolder> {
        self.catalog
            .folders()
            .into_iter()
            .enumerate()
            .map(|(id, folder)| UiFolder {
                id,
                name: folder.name,
                track_count: folder.track_count,
            })
            .collect()
    }

    fn filtered_folders_from_catalog(&self) -> Vec<UiFolder> {
        let mut folders = self.folders_from_catalog();
        if let Some(query) = self.normalized_query() {
            folders.retain(|folder| folder.name.to_lowercase().contains(&query));
        }
        folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        folders
    }

    fn top_bar(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let logo_mark = container(
            text("G")
                .size(theme.size(18))
                .font(style::font_propo(Weight::Bold))
                .style(style::text_primary(theme)),
        )
        .padding([6, 10])
        .style(Container::Custom(Box::new(style::SurfaceStyle::new(
            style::Surface::Avatar,
            theme,
        ))));
        let logo = row![
            logo_mark,
            text("Grape")
                .size(theme.size(20))
                .font(style::font_propo(Weight::Semibold))
                .style(style::text_primary(theme))
        ]
        .spacing(8)
        .align_items(Alignment::Center);
        let logo_button = button(logo)
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::Icon,
                theme,
            ))))
            .padding([2, 6])
            .on_press(UiMessage::ToggleLogoMenu);
        let menu_button = |label, message| {
            button(
                text(label)
                    .size(theme.size(13))
                    .font(style::font_propo(Weight::Medium))
                    .style(style::text_primary(theme)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::ListItem { selected: false },
                theme,
            ))))
            .padding([4, 8])
            .on_press(message)
        };
        let logo_menu = container(
            column![
                menu_button("Bibliothèque", UiMessage::ShowLibrary),
                menu_button("Playlist", UiMessage::OpenPlaylist),
                menu_button("Préférences", UiMessage::OpenPreferences),
            ]
            .spacing(6),
        )
        .padding([8, 12])
        .style(Container::Custom(Box::new(style::SurfaceStyle::new(
            style::Surface::Panel,
            theme,
        ))));
        let logo_widget: Element<'_, UiMessage> = if self.ui.menu_open {
            AnchoredOverlay::new(logo_button, logo_menu).into()
        } else {
            logo_button.into()
        };
        let tabs = row![
            button(
                text(self.tab_label(ActiveTab::Artists, "Artists"))
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::Tab {
                    selected: self.ui.active_tab == ActiveTab::Artists,
                },
                theme,
            ))))
            .on_press(UiMessage::TabSelected(ActiveTab::Artists)),
            button(
                text(self.tab_label(ActiveTab::Genres, "Genres"))
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::Tab {
                    selected: self.ui.active_tab == ActiveTab::Genres,
                },
                theme,
            ))))
            .on_press(UiMessage::TabSelected(ActiveTab::Genres)),
            button(
                text(self.tab_label(ActiveTab::Albums, "Albums"))
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::Tab {
                    selected: self.ui.active_tab == ActiveTab::Albums,
                },
                theme,
            ))))
            .on_press(UiMessage::TabSelected(ActiveTab::Albums)),
            button(
                text(self.tab_label(ActiveTab::Folders, "Folders"))
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::Tab {
                    selected: self.ui.active_tab == ActiveTab::Folders,
                },
                theme,
            ))))
            .on_press(UiMessage::TabSelected(ActiveTab::Folders)),
        ]
        .spacing(12)
        .align_items(Alignment::Center);
        let search_input = text_input("Search...", &self.ui.search.query)
            .style(TextInput::Custom(Box::new(style::SearchInput::new(theme))))
            .on_input(|value| UiMessage::Search(SearchMessage::QueryChanged(value)));
        let search = row![
            search_input,
            button(
                text("≡")
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14))
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::Icon,
                theme,
            )))),
            button(
                text("—")
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14))
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::Icon,
                theme,
            )))),
            button(
                text("▢")
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14))
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::Icon,
                theme,
            )))),
            button(
                text("✕")
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14))
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::Icon,
                theme,
            ))))
        ]
        .spacing(8)
        .align_items(Alignment::Center);

        let layout = row![
            container(logo_widget).width(Length::Shrink),
            container(tabs).width(Length::Fill).center_x(),
            container(search).width(Length::Shrink)
        ]
        .spacing(24)
        .align_items(Alignment::Center);

        container(layout)
            .padding([10, 16])
            .width(Length::Fill)
            .style(Container::Custom(Box::new(style::SurfaceStyle::new(
                style::Surface::TopBar,
                theme,
            ))))
            .into()
    }

    fn artists_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let selected_id = self
            .ui
            .selection
            .selected_artist
            .as_ref()
            .map(|artist| artist.id);
        let artists = self.filtered_artists_from_catalog();
        let panel = ArtistsPanel::new(artists).with_selection(selected_id);
        panel.view(&self.ui.selection, theme)
    }

    fn albums_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let sort_label = match self.ui.search.sort {
            SortOption::Alphabetical => "A–Z",
            SortOption::ByAlbum => "By album",
        };
        let selected_id = self
            .ui
            .selection
            .selected_album
            .as_ref()
            .map(|album| album.id);
        let albums = self.filtered_albums_from_catalog();
        let grid = AlbumsGrid::new(albums)
            .with_sort_label(sort_label)
            .with_selection(selected_id)
            .view(theme);

        container(grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn songs_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let selected_album = self.ui.selection.selected_album.as_ref().and_then(|album| {
            self.album_entry_by_id(album.id)
                .map(|(artist, entry)| (artist, entry))
        });
        let (album_title, artist_name, tracks) = match selected_album {
            Some((artist, album)) => (
                album.title.clone(),
                artist.name.clone(),
                self.filtered_tracks_for_album(artist, album),
            ),
            None => (
                "Select an album".to_string(),
                "Pick a track to start".to_string(),
                Vec::new(),
            ),
        };
        let selected_id = self
            .ui
            .selection
            .selected_track
            .as_ref()
            .map(|track| track.id);
        let panel = SongsPanel::new(album_title, artist_name, tracks).with_selection(selected_id);
        panel.view(&self.ui.selection, theme)
    }

    fn genres_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let selected_id = self
            .ui
            .selection
            .selected_genre
            .as_ref()
            .map(|genre| genre.id);
        let genres = self.filtered_genres_from_catalog();
        let panel = GenresPanel::new(genres).with_selection(selected_id);
        panel.view(theme)
    }

    fn folders_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let sort_label = match self.ui.search.sort {
            SortOption::Alphabetical => "A–Z",
            SortOption::ByAlbum => "By album",
        };
        let selected_id = self
            .ui
            .selection
            .selected_folder
            .as_ref()
            .map(|folder| folder.id);
        let folders = self.filtered_folders_from_catalog();
        FoldersPanel::new(folders)
            .with_sort_label(sort_label)
            .with_selection(selected_id)
            .view(theme)
    }

    fn player_bar(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let (title, artist, cover_path) = self
            .ui
            .selection
            .selected_track
            .as_ref()
            .map(|track| {
                (
                    track.title.clone(),
                    track.artist.clone(),
                    track.cover_path.clone(),
                )
            })
            .or_else(|| {
                self.ui.selection.selected_album.as_ref().map(|album| {
                    (
                        album.title.clone(),
                        album.artist.clone(),
                        album.cover_path.clone(),
                    )
                })
            })
            .unwrap_or_else(|| {
                (
                    "No track selected".to_string(),
                    "Pick a track to play".to_string(),
                    None,
                )
            });

        PlayerBar::new(title, artist)
            .with_cover(cover_path)
            .with_playback(self.ui.playback)
            .with_volume(self.ui.settings.default_volume)
            .with_queue(false)
            .view(theme)
    }

    fn handle_track_selection(&mut self, track: &UiTrack) {
        let Some(player) = &mut self.player else {
            return;
        };
        if let Err(err) = player.load(&track.path) {
            error!(error = %err, path = %track.path.display(), "Failed to load track");
            return;
        }
        player.play();
    }

    fn handle_playback_message(&mut self, message: &PlaybackMessage) {
        let Some(player) = &mut self.player else {
            return;
        };
        match message {
            PlaybackMessage::TogglePlayPause => match player.state() {
                PlayerPlaybackState::Playing => player.pause(),
                PlayerPlaybackState::Paused | PlayerPlaybackState::Stopped => player.play(),
            },
            PlaybackMessage::NextTrack | PlaybackMessage::PreviousTrack => {
                if let Err(err) = player.seek(Duration::ZERO) {
                    error!(error = %err, "Failed to seek to start");
                } else {
                    player.play();
                }
            }
            PlaybackMessage::ToggleShuffle | PlaybackMessage::CycleRepeat => {}
        }
    }

    fn sync_playback_state(&mut self) {
        let (is_playing, position) = match &self.player {
            Some(player) => (
                matches!(player.state(), PlayerPlaybackState::Playing),
                player.position(),
            ),
            None => (false, Duration::ZERO),
        };
        self.ui.playback.is_playing = is_playing;
        self.ui.playback.position = position;
        self.ui.playback.duration = self
            .ui
            .selection
            .selected_track
            .as_ref()
            .map(|track| track.duration)
            .unwrap_or(Duration::ZERO);
    }

    fn playlist_view(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        PlaylistView::view(theme)
    }

    fn preferences_view(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let header = row![
            text("Préférences")
                .size(theme.size(22))
                .font(style::font_propo(Weight::Semibold))
                .style(style::text_primary(theme)),
            button(
                text("Fermer")
                    .size(theme.size(13))
                    .font(style::font_propo(Weight::Medium))
                    .style(style::text_primary(theme)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::ListItem { selected: false },
                theme,
            ))))
            .padding([6, 10])
            .on_press(UiMessage::ClosePreferences)
        ]
        .align_items(Alignment::Center)
        .spacing(12);

        let menu_button = |tab: PreferencesTab, label: &str| {
            button(
                text(label)
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(style::text_primary(theme)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::ListItem {
                    selected: self.ui.preferences_tab == tab,
                },
                theme,
            ))))
            .padding([6, 10])
            .width(Length::Fill)
            .on_press(UiMessage::PreferencesTabSelected(tab))
        };
        let menu = column![
            menu_button(PreferencesTab::General, "Général"),
            menu_button(PreferencesTab::Appearance, "Apparences"),
            menu_button(PreferencesTab::Accessibility, "Accessibility"),
            menu_button(PreferencesTab::Audio, "Audio"),
        ]
        .spacing(6)
        .width(Length::Fill);

        let section_header = |label: &str, expanded: bool, message: UiMessage| {
            let chevron = if expanded { "▾" } else { "▸" };
            button(
                row![
                    text(label)
                        .size(theme.size(14))
                        .font(style::font_propo(Weight::Semibold))
                        .style(style::text_primary(theme)),
                    text(chevron)
                        .size(theme.size(14))
                        .font(style::font_propo(Weight::Medium))
                        .style(style::text_muted(theme)),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::ListItem { selected: expanded },
                theme,
            ))))
            .padding([8, 12])
            .width(Length::Fill)
            .on_press(message)
        };
        let section_hint = |label: &str| {
            text(label)
                .size(theme.size(12))
                .font(style::font_propo(Weight::Light))
                .style(style::text_muted(theme))
        };
        let setting_label = |title: &str, subtitle: &str| {
            column![
                text(title)
                    .size(theme.size(13))
                    .font(style::font_propo(Weight::Medium))
                    .style(style::text_primary(theme)),
                text(subtitle)
                    .size(theme.size(12))
                    .font(style::font_propo(Weight::Light))
                    .style(style::text_muted(theme)),
            ]
            .spacing(2)
            .width(Length::Fill)
        };
        let option_button = |selected: bool, label: &str, message: UiMessage| {
            button(
                text(label)
                    .size(theme.size(12))
                    .font(style::font_propo(Weight::Medium))
                    .style(style::text_primary(theme)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::Tab { selected },
                theme,
            ))))
            .padding([6, 10])
            .on_press(message)
        };
        let toggle_row = |enabled: bool, on_message: UiMessage, off_message: UiMessage| {
            row![
                option_button(enabled, "Activé", on_message),
                option_button(!enabled, "Désactivé", off_message),
            ]
            .spacing(8)
        };
        let controls = |content: Element<'_, UiMessage>| {
            container(content)
                .width(Length::FillPortion(2))
                .center_x()
                .padding([0, 0, 0, 24])
        };
        let action_button = |label: &str, message: UiMessage| {
            button(
                text(label)
                    .size(theme.size(12))
                    .font(style::font_propo(Weight::Medium))
                    .style(style::text_primary(theme)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle::new(
                style::ButtonKind::Control,
                theme,
            ))))
            .padding([6, 10])
            .on_press(message)
        };

        let library_input = text_input("Dossier de bibliothèque", &self.ui.settings.library_folder)
            .style(TextInput::Custom(Box::new(style::SearchInput::new(theme))))
            .on_input(UiMessage::LibraryFolderChanged);
        let cache_input = text_input("Emplacement du cache", &self.ui.settings.cache_path)
            .style(TextInput::Custom(Box::new(style::SearchInput::new(theme))))
            .on_input(UiMessage::CachePathChanged);

        let startup_content = || {
            column![
                section_hint("Gérez l'ouverture de Grape et la restauration des sessions."),
                row![
                    setting_label(
                        "Lancer Grape au démarrage du système",
                        "Active l'application dès l'ouverture de votre session."
                    ),
                    controls(toggle_row(
                        self.ui.settings.launch_at_startup,
                        UiMessage::SetLaunchAtStartup(true),
                        UiMessage::SetLaunchAtStartup(false),
                    )
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Restaurer la dernière session",
                        "Lecture, file d'attente et écran affiché."
                    ),
                    controls(toggle_row(
                        self.ui.settings.restore_last_session,
                        UiMessage::SetRestoreLastSession(true),
                        UiMessage::SetRestoreLastSession(false),
                    )
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Ouvrir sur", "Choisissez l'écran par défaut."),
                    controls(column![
                        row![
                            option_button(
                                self.ui.settings.open_on == StartupScreen::Home,
                                StartupScreen::Home.label(),
                                UiMessage::SetOpenOn(StartupScreen::Home),
                            ),
                            option_button(
                                self.ui.settings.open_on == StartupScreen::Library,
                                StartupScreen::Library.label(),
                                UiMessage::SetOpenOn(StartupScreen::Library),
                            ),
                        ]
                        .spacing(8),
                        row![
                            option_button(
                                self.ui.settings.open_on == StartupScreen::Playlists,
                                StartupScreen::Playlists.label(),
                                UiMessage::SetOpenOn(StartupScreen::Playlists),
                            ),
                            option_button(
                                self.ui.settings.open_on == StartupScreen::LastScreen,
                                StartupScreen::LastScreen.label(),
                                UiMessage::SetOpenOn(StartupScreen::LastScreen),
                            ),
                        ]
                        .spacing(8),
                    ]
                    .spacing(6)
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Comportement à la fermeture",
                        "Choisissez l'action à la fermeture."
                    ),
                    controls(row![
                        option_button(
                            self.ui.settings.close_behavior == CloseBehavior::Quit,
                            CloseBehavior::Quit.label(),
                            UiMessage::SetCloseBehavior(CloseBehavior::Quit),
                        ),
                        option_button(
                            self.ui.settings.close_behavior == CloseBehavior::MinimizeToTray,
                            CloseBehavior::MinimizeToTray.label(),
                            UiMessage::SetCloseBehavior(CloseBehavior::MinimizeToTray),
                        ),
                    ]
                    .spacing(8)
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding([4, 12, 0, 12])
        };

        let language_content = || {
            column![
                section_hint("Personnalisez l'interface et le format horaire."),
                row![
                    setting_label("Langue de l'interface", "Synchronisée avec le système."),
                    controls(row![
                        option_button(
                            self.ui.settings.interface_language == InterfaceLanguage::System,
                            InterfaceLanguage::System.label(),
                            UiMessage::SetInterfaceLanguage(InterfaceLanguage::System),
                        ),
                        option_button(
                            self.ui.settings.interface_language == InterfaceLanguage::French,
                            InterfaceLanguage::French.label(),
                            UiMessage::SetInterfaceLanguage(InterfaceLanguage::French),
                        ),
                        option_button(
                            self.ui.settings.interface_language == InterfaceLanguage::English,
                            InterfaceLanguage::English.label(),
                            UiMessage::SetInterfaceLanguage(InterfaceLanguage::English),
                        ),
                    ]
                    .spacing(8)
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Format horaire", "Format utilisé dans l'application."),
                    controls(row![
                        option_button(
                            self.ui.settings.time_format == TimeFormat::H24,
                            TimeFormat::H24.label(),
                            UiMessage::SetTimeFormat(TimeFormat::H24),
                        ),
                        option_button(
                            self.ui.settings.time_format == TimeFormat::H12,
                            TimeFormat::H12.label(),
                            UiMessage::SetTimeFormat(TimeFormat::H12),
                        ),
                    ]
                    .spacing(8)
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding([4, 12, 0, 12])
        };

        let updates_content = || {
            column![
                section_hint("Gérez la vérification et le canal des mises à jour."),
                row![
                    setting_label(
                        "Vérifier automatiquement les mises à jour",
                        "Vérifie les nouvelles versions au lancement."
                    ),
                    controls(toggle_row(
                        self.ui.settings.auto_check_updates,
                        UiMessage::SetAutoCheckUpdates(true),
                        UiMessage::SetAutoCheckUpdates(false),
                    )
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Canal", "Choisissez la stabilité des versions."),
                    controls(row![
                        option_button(
                            self.ui.settings.update_channel == UpdateChannel::Stable,
                            UpdateChannel::Stable.label(),
                            UiMessage::SetUpdateChannel(UpdateChannel::Stable),
                        ),
                        option_button(
                            self.ui.settings.update_channel == UpdateChannel::Beta,
                            UpdateChannel::Beta.label(),
                            UiMessage::SetUpdateChannel(UpdateChannel::Beta),
                        ),
                    ]
                    .spacing(8)
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Télécharger et installer automatiquement",
                        "Installe les mises à jour en arrière-plan."
                    ),
                    controls(toggle_row(
                        self.ui.settings.auto_install_updates,
                        UiMessage::SetAutoInstallUpdates(true),
                        UiMessage::SetAutoInstallUpdates(false),
                    )
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding([4, 12, 0, 12])
        };

        let privacy_content = || {
            column![
                section_hint("Choisissez les données partagées avec Grape."),
                row![
                    setting_label(
                        "Envoyer des rapports d'erreurs",
                        "Permet d'améliorer la stabilité."
                    ),
                    controls(toggle_row(
                        self.ui.settings.send_error_reports,
                        UiMessage::SetSendErrorReports(true),
                        UiMessage::SetSendErrorReports(false),
                    )
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Envoyer des statistiques anonymes d'utilisation",
                        "Aide à comprendre l'usage de Grape."
                    ),
                    controls(toggle_row(
                        self.ui.settings.send_usage_stats,
                        UiMessage::SetSendUsageStats(true),
                        UiMessage::SetSendUsageStats(false),
                    )
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Effacer l'historique local", "Supprime les traces locales."),
                    controls(action_button("Effacer", UiMessage::ClearHistory).into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding([4, 12, 0, 12])
        };

        let storage_content = || {
            column![
                section_hint("Gérez l'emplacement de la bibliothèque et du cache."),
                row![
                    setting_label(
                        "Dossier de bibliothèque",
                        "Sélectionnez le dossier principal."
                    ),
                    controls(
                        row![
                            library_input.width(Length::Fill),
                            action_button("Choisir", UiMessage::PickLibraryFolder),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Scanner automatiquement au lancement",
                        "Met à jour la bibliothèque au démarrage."
                    ),
                    controls(toggle_row(
                        self.ui.settings.auto_scan_on_launch,
                        UiMessage::SetAutoScanOnLaunch(true),
                        UiMessage::SetAutoScanOnLaunch(false),
                    )
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Emplacement du cache",
                        "Chemin modifiable pour les fichiers temporaires."
                    ),
                    controls(
                        row![
                            cache_input.width(Length::Fill),
                            action_button("Vider le cache", UiMessage::ClearCache),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding([4, 12, 0, 12])
        };

        let notifications_content = || {
            column![
                section_hint("Gérez les alertes système."),
                row![
                    setting_label(
                        "Activer les notifications système",
                        "Autorise l'affichage des notifications."
                    ),
                    controls(toggle_row(
                        self.ui.settings.notifications_enabled,
                        UiMessage::SetNotificationsEnabled(true),
                        UiMessage::SetNotificationsEnabled(false),
                    )
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Afficher “Now Playing” lors des changements de piste",
                        "Notification à chaque changement de lecture."
                    ),
                    controls(toggle_row(
                        self.ui.settings.now_playing_notifications,
                        UiMessage::SetNowPlayingNotifications(true),
                        UiMessage::SetNowPlayingNotifications(false),
                    )
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding([4, 12, 0, 12])
        };

        let performance_content = || {
            column![
                section_hint("Ajustez les options de performance."),
                row![
                    setting_label(
                        "Accélération matérielle",
                        "Utilise le GPU pour les animations."
                    ),
                    controls(toggle_row(
                        self.ui.settings.hardware_acceleration,
                        UiMessage::SetHardwareAcceleration(true),
                        UiMessage::SetHardwareAcceleration(false),
                    )
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Limiter l'utilisation CPU pendant la lecture",
                        "Réduit la charge pendant la musique."
                    ),
                    controls(toggle_row(
                        self.ui.settings.limit_cpu_during_playback,
                        UiMessage::SetLimitCpuDuringPlayback(true),
                        UiMessage::SetLimitCpuDuringPlayback(false),
                    )
                    .into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding([4, 12, 0, 12])
        };

        let advanced_content = || {
            column![
                section_hint("Outils pour diagnostiquer et réinitialiser."),
                row![
                    setting_label("Ouvrir le dossier de logs", "Accès aux journaux."),
                    controls(action_button("Ouvrir", UiMessage::OpenLogsFolder).into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Réindexer la bibliothèque", "Reconstruit l'index local."),
                    controls(action_button("Réindexer", UiMessage::ReindexLibrary).into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Réinitialiser les préférences",
                        "Restaure les valeurs par défaut."
                    ),
                    controls(action_button("Réinitialiser", UiMessage::ResetPreferences).into()),
                ]
                .align_items(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding([4, 12, 0, 12])
        };

        let general_panel = scrollable(
            column![
                column![
                    text("Paramètres généraux")
                        .size(theme.size(16))
                        .font(style::font_propo(Weight::Semibold))
                        .style(style::text_primary(theme)),
                    text("Les préférences sont enregistrées automatiquement.")
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Light))
                        .style(style::text_muted(theme))
                ]
                .spacing(6),
                section_header(
                    "Démarrage",
                    self.ui.preferences_sections.startup,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Startup),
                ),
                if self.ui.preferences_sections.startup {
                    startup_content()
                } else {
                    column![]
                },
                section_header(
                    "Langue",
                    self.ui.preferences_sections.language,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Language),
                ),
                if self.ui.preferences_sections.language {
                    language_content()
                } else {
                    column![]
                },
                section_header(
                    "Mises à jour",
                    self.ui.preferences_sections.updates,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Updates),
                ),
                if self.ui.preferences_sections.updates {
                    updates_content()
                } else {
                    column![]
                },
                section_header(
                    "Confidentialité",
                    self.ui.preferences_sections.privacy,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Privacy),
                ),
                if self.ui.preferences_sections.privacy {
                    privacy_content()
                } else {
                    column![]
                },
                section_header(
                    "Stockage",
                    self.ui.preferences_sections.storage,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Storage),
                ),
                if self.ui.preferences_sections.storage {
                    storage_content()
                } else {
                    column![]
                },
                section_header(
                    "Notifications",
                    self.ui.preferences_sections.notifications,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Notifications),
                ),
                if self.ui.preferences_sections.notifications {
                    notifications_content()
                } else {
                    column![]
                },
                section_header(
                    "Performance",
                    self.ui.preferences_sections.performance,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Performance),
                ),
                if self.ui.preferences_sections.performance {
                    performance_content()
                } else {
                    column![]
                },
                section_header(
                    "Avancé / Dépannage",
                    self.ui.preferences_sections.advanced,
                    UiMessage::TogglePreferencesSection(PreferencesSection::Advanced),
                ),
                if self.ui.preferences_sections.advanced {
                    advanced_content()
                } else {
                    column![]
                },
            ]
            .spacing(16),
        )
        .height(Length::Fill);

        let appearance_panel = column![
            text("Thème")
                .size(theme.size(16))
                .font(style::font_propo(Weight::Semibold))
                .style(style::text_primary(theme)),
            row![
                button(
                    text("Sombre")
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Medium))
                        .style(style::text_primary(theme)),
                )
                .style(Button::Custom(Box::new(style::ButtonStyle::new(
                    style::ButtonKind::Tab {
                        selected: self.ui.settings.theme_mode == ThemeMode::Dark,
                    },
                    theme,
                ))))
                .padding([6, 10])
                .on_press(UiMessage::SetThemeMode(ThemeMode::Dark)),
                button(
                    text("Clair")
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Medium))
                        .style(style::text_primary(theme)),
                )
                .style(Button::Custom(Box::new(style::ButtonStyle::new(
                    style::ButtonKind::Tab {
                        selected: self.ui.settings.theme_mode == ThemeMode::Light,
                    },
                    theme,
                ))))
                .padding([6, 10])
                .on_press(UiMessage::SetThemeMode(ThemeMode::Light)),
            ]
            .spacing(10),
            text("Les couleurs d'accent et les surfaces s'ajustent automatiquement.")
                .size(theme.size(13))
                .font(style::font_propo(Weight::Light))
                .style(style::text_muted(theme))
        ]
        .spacing(8);

        let accessibility_panel = column![
            text("Taille de texte")
                .size(theme.size(16))
                .font(style::font_propo(Weight::Semibold))
                .style(style::text_primary(theme)),
            row![
                button(
                    text(TextScale::Normal.label())
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Medium))
                        .style(style::text_primary(theme)),
                )
                .style(Button::Custom(Box::new(style::ButtonStyle::new(
                    style::ButtonKind::Tab {
                        selected: self.ui.settings.text_scale == TextScale::Normal,
                    },
                    theme,
                ))))
                .padding([6, 10])
                .on_press(UiMessage::SetTextScale(TextScale::Normal)),
                button(
                    text(TextScale::Large.label())
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Medium))
                        .style(style::text_primary(theme)),
                )
                .style(Button::Custom(Box::new(style::ButtonStyle::new(
                    style::ButtonKind::Tab {
                        selected: self.ui.settings.text_scale == TextScale::Large,
                    },
                    theme,
                ))))
                .padding([6, 10])
                .on_press(UiMessage::SetTextScale(TextScale::Large)),
                button(
                    text(TextScale::ExtraLarge.label())
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Medium))
                        .style(style::text_primary(theme)),
                )
                .style(Button::Custom(Box::new(style::ButtonStyle::new(
                    style::ButtonKind::Tab {
                        selected: self.ui.settings.text_scale == TextScale::ExtraLarge,
                    },
                    theme,
                ))))
                .padding([6, 10])
                .on_press(UiMessage::SetTextScale(TextScale::ExtraLarge)),
            ]
            .spacing(10),
            text("Ajustez la taille de texte pour améliorer la lisibilité.")
                .size(theme.size(13))
                .font(style::font_propo(Weight::Light))
                .style(style::text_muted(theme))
        ]
        .spacing(8);

        let volume_value = self.ui.settings.default_volume as f32;
        let audio_panel = column![
            text("Volume par défaut")
                .size(theme.size(16))
                .font(style::font_propo(Weight::Semibold))
                .style(style::text_primary(theme)),
            slider(0.0..=100.0, volume_value, |value| {
                UiMessage::SetDefaultVolume(value.round().clamp(0.0, 100.0) as u8)
            }),
            text(format!("{} %", self.ui.settings.default_volume))
                .size(theme.size(13))
                .font(style::font_propo(Weight::Medium))
                .style(style::text_muted(theme))
        ]
        .spacing(8);

        let content_panel: Element<'_, UiMessage> = match self.ui.preferences_tab {
            PreferencesTab::General => general_panel.into(),
            PreferencesTab::Appearance => appearance_panel.into(),
            PreferencesTab::Accessibility => accessibility_panel.into(),
            PreferencesTab::Audio => audio_panel.into(),
        };

        let body = row![
            container(menu)
                .padding(16)
                .width(Length::Fixed(200.0))
                .style(Container::Custom(Box::new(style::SurfaceStyle::new(
                    style::Surface::Sidebar,
                    theme,
                )))),
            container(content_panel)
                .padding(20)
                .width(Length::Fill)
                .style(Container::Custom(Box::new(style::SurfaceStyle::new(
                    style::Surface::Panel,
                    theme,
                ))))
        ]
        .spacing(16)
        .height(Length::Fill);

        column![header, body]
            .spacing(16)
            .height(Length::Fill)
            .into()
    }
}

impl Application for GrapeApp {
    type Executor = iced::executor::Default;
    type Message = UiMessage;
    type Theme = Theme;
    type Flags = Catalog;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let player = match Player::new() {
            Ok(player) => Some(player),
            Err(err) => {
                error!(error = %err, "Failed to initialize audio player");
                None
            }
        };
        let settings = config::load_settings();
        (
            Self {
                catalog: flags,
                player,
                ui: UiState::new(settings),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Grape".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        let should_persist = matches!(
            message,
            UiMessage::SetThemeMode(_)
                | UiMessage::SetTextScale(_)
                | UiMessage::SetDefaultVolume(_)
                | UiMessage::SetLaunchAtStartup(_)
                | UiMessage::SetRestoreLastSession(_)
                | UiMessage::SetOpenOn(_)
                | UiMessage::SetCloseBehavior(_)
                | UiMessage::SetInterfaceLanguage(_)
                | UiMessage::SetTimeFormat(_)
                | UiMessage::SetAutoCheckUpdates(_)
                | UiMessage::SetUpdateChannel(_)
                | UiMessage::SetAutoInstallUpdates(_)
                | UiMessage::SetSendErrorReports(_)
                | UiMessage::SetSendUsageStats(_)
                | UiMessage::LibraryFolderChanged(_)
                | UiMessage::LibraryFolderPicked(_)
                | UiMessage::SetAutoScanOnLaunch(_)
                | UiMessage::CachePathChanged(_)
                | UiMessage::SetNotificationsEnabled(_)
                | UiMessage::SetNowPlayingNotifications(_)
                | UiMessage::SetHardwareAcceleration(_)
                | UiMessage::SetLimitCpuDuringPlayback(_)
                | UiMessage::ResetPreferences
        );
        let mut command = Command::none();
        match &message {
            UiMessage::SelectTrack(track) => {
                self.handle_track_selection(track);
            }
            UiMessage::Playback(playback_message) => {
                self.handle_playback_message(playback_message);
            }
            UiMessage::OpenPlaylist => {
                self.ui.playlist_open = true;
            }
            UiMessage::ClosePlaylist => {
                self.ui.playlist_open = false;
            }
            UiMessage::PickLibraryFolder => {
                command = Command::perform(
                    async {
                        rfd::FileDialog::new()
                            .pick_folder()
                            .map(|path| path.display().to_string())
                    },
                    UiMessage::LibraryFolderPicked,
                );
            }
            UiMessage::ClearCache => {
                if let Err(err) = config::clear_cache(&self.ui.settings) {
                    error!(error = %err, "Failed to clear cache");
                } else {
                    info!("Cache cleared");
                }
            }
            UiMessage::ClearHistory => {
                if let Err(err) = config::clear_history() {
                    error!(error = %err, "Failed to clear local history");
                } else {
                    info!("Local history cleared");
                }
            }
            UiMessage::OpenLogsFolder => {
                let path = config::logs_path();
                info!(path = %path.display(), "Open logs folder requested");
            }
            UiMessage::ReindexLibrary => {
                info!("Library reindex requested");
            }
            _ => {}
        }
        self.ui.update(message);
        if should_persist {
            if let Err(err) = config::save_settings(&self.ui.settings) {
                error!(error = %err, "Failed to save preferences");
            }
        }
        self.sync_playback_state();
        command
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let theme = self.theme_tokens();
        if self.ui.playlist_open {
            return self.playlist_view();
        }

        let content = if self.ui.preferences_open {
            self.preferences_view()
        } else {
            match self.ui.active_tab {
                ActiveTab::Artists | ActiveTab::Albums => row![
                    container(self.artists_panel())
                        .width(Length::FillPortion(2))
                        .height(Length::Fill),
                    container(self.albums_panel())
                        .width(Length::FillPortion(5))
                        .height(Length::Fill),
                    container(self.songs_panel())
                        .width(Length::FillPortion(3))
                        .height(Length::Fill),
                ],
                ActiveTab::Genres => row![
                    container(self.genres_panel())
                        .width(Length::FillPortion(2))
                        .height(Length::Fill),
                    container(self.albums_panel())
                        .width(Length::FillPortion(5))
                        .height(Length::Fill),
                    container(self.songs_panel())
                        .width(Length::FillPortion(3))
                        .height(Length::Fill),
                ],
                ActiveTab::Folders => row![
                    container(self.folders_panel())
                        .width(Length::FillPortion(7))
                        .height(Length::Fill),
                    container(self.songs_panel())
                        .width(Length::FillPortion(3))
                        .height(Length::Fill),
                ],
            }
            .spacing(16)
            .height(Length::Fill)
            .into()
        };

        let layout = column![self.top_bar(), content, self.player_bar()]
            .spacing(16)
            .padding(16)
            .height(Length::Fill);

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(Container::Custom(Box::new(style::SurfaceStyle::new(
                style::Surface::AppBackground,
                theme,
            ))))
            .into()
    }

    fn theme(&self) -> Theme {
        match self.ui.settings.theme_mode {
            ThemeMode::Dark => Theme::Dark,
            ThemeMode::Light => Theme::Light,
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subscriptions = Vec::new();

        if self.ui.menu_open {
            subscriptions.push(event::listen_with(|event, status| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
                {
                    Some(UiMessage::CloseMenu)
                }
                event::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    if status == event::Status::Ignored =>
                {
                    Some(UiMessage::CloseMenu)
                }
                _ => None,
            }));
        }

        if self.ui.playlist_open {
            subscriptions.push(event::listen_with(|event, _status| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
                {
                    Some(UiMessage::ClosePlaylist)
                }
                _ => None,
            }));
        }

        if self.ui.preferences_open {
            subscriptions.push(event::listen_with(|event, _status| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
                {
                    Some(UiMessage::ClosePreferences)
                }
                _ => None,
            }));
        }

        Subscription::batch(subscriptions)
    }
}
