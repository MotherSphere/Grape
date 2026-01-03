use crate::config::{
    self, AccentColor, AccessibleTextSize, AudioOutputDevice, AudioStabilityMode, CloseBehavior,
    EqPreset, InterfaceDensity, InterfaceLanguage, MissingDeviceBehavior, StartupScreen,
    SubtitleSize, TextScale, ThemeMode, TimeFormat, UpdateChannel, VolumeLevel,
};
use crate::library::Catalog;
use crate::player::{NowPlaying, PlaybackState as PlayerPlaybackState, Player};
use crate::playlist::{PlaybackQueue, PlaylistManager};
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
use iced::widget::{button, column, container, row, scrollable, slider, text, text_input};
use iced::{
    Alignment, Color, Element, Length, Padding, Settings, Subscription, Task, Theme, event,
    keyboard, mouse, window,
};
use std::time::Duration;
use tracing::{error, info};

pub struct GrapeApp {
    catalog: Catalog,
    player: Option<Player>,
    playlists: PlaylistManager,
    playback_queue: PlaybackQueue,
    ui: UiState,
}

impl GrapeApp {
    pub fn run(catalog: Catalog) -> iced::Result {
        let settings = Self::apply_font_settings(Settings::default());
        iced::application(move || Self::new(catalog.clone()), Self::update, Self::view)
            .settings(settings)
            .title(Self::title)
            .subscription(Self::subscription)
            .theme(Self::theme)
            .run()
    }

    pub fn run_with(catalog: Catalog, settings: Settings) -> iced::Result {
        let settings = Self::apply_font_settings(settings);
        iced::application(move || Self::new(catalog.clone()), Self::update, Self::view)
            .settings(settings)
            .title(Self::title)
            .subscription(Self::subscription)
            .theme(Self::theme)
            .run()
    }

    fn apply_font_settings(mut settings: Settings) -> Settings {
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
                .style(move |_| style::text_style_primary(theme)),
        )
        .padding([6, 10])
        .style(move |_| style::surface_style(theme, style::Surface::Avatar));
        let logo = row![
            logo_mark,
            text("Grape")
                .size(theme.size(20))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme))
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        let logo_button = button(logo)
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .padding([2, 6])
            .on_press(UiMessage::ToggleLogoMenu);
        let menu_button = |label, message| {
            button(
                text(label)
                    .size(theme.size(13))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::ListItem { selected: false },
                    status,
                )
            })
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
        .style(move |_| style::surface_style(theme, style::Surface::Panel));
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
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::Tab {
                        selected: self.ui.active_tab == ActiveTab::Artists,
                    },
                    status,
                )
            })
            .on_press(UiMessage::TabSelected(ActiveTab::Artists)),
            button(
                text(self.tab_label(ActiveTab::Genres, "Genres"))
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::Tab {
                        selected: self.ui.active_tab == ActiveTab::Genres,
                    },
                    status,
                )
            })
            .on_press(UiMessage::TabSelected(ActiveTab::Genres)),
            button(
                text(self.tab_label(ActiveTab::Albums, "Albums"))
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::Tab {
                        selected: self.ui.active_tab == ActiveTab::Albums,
                    },
                    status,
                )
            })
            .on_press(UiMessage::TabSelected(ActiveTab::Albums)),
            button(
                text(self.tab_label(ActiveTab::Folders, "Folders"))
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::Tab {
                        selected: self.ui.active_tab == ActiveTab::Folders,
                    },
                    status,
                )
            })
            .on_press(UiMessage::TabSelected(ActiveTab::Folders)),
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        let search_input = text_input("Search...", &self.ui.search.query)
            .style(move |_, status| style::text_input_style(theme, status))
            .on_input(|value| UiMessage::Search(SearchMessage::QueryChanged(value)));
        let search = row![
            search_input,
            button(
                text("≡")
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14))
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .on_press(UiMessage::ToggleLogoMenu),
            button(
                text("—")
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14))
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .on_press(UiMessage::WindowMinimize),
            button(
                text("")
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14))
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .on_press(UiMessage::WindowToggleMaximize),
            button(
                text("✕")
                    .font(style::font_propo(Weight::Medium))
                    .size(theme.size(14))
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Icon, status))
            .on_press(UiMessage::WindowClose)
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let layout = row![
            container(logo_widget).width(Length::Shrink),
            container(tabs).width(Length::Fill).center_x(Length::Fill),
            container(search).width(Length::Shrink)
        ]
        .spacing(24)
        .align_y(Alignment::Center);

        container(layout)
            .padding([10, 16])
            .width(Length::Fill)
            .style(move |_| style::surface_style(theme, style::Surface::TopBar))
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
        let now_playing = Self::now_playing_from_ui_track(track);
        self.playlists.add(now_playing.clone());
        let (queue_items, queue_index) = self.queue_from_track_selection(track);
        self.playback_queue.set_queue(queue_items);
        self.playback_queue.set_index(queue_index);
        let Some(player) = &mut self.player else {
            return;
        };
        if let Err(err) = player.load(&track.path) {
            error!(error = %err, path = %track.path.display(), "Failed to load track");
            return;
        }
        player.play();
    }

    fn now_playing_from_ui_track(track: &UiTrack) -> crate::player::NowPlaying {
        crate::player::NowPlaying {
            artist: track.artist.clone(),
            album: track.album.clone(),
            title: track.title.clone(),
            duration_secs: u32::try_from(track.duration.as_secs()).unwrap_or(u32::MAX),
            path: track.path.clone(),
        }
    }

    fn ui_track_from_now_playing(&self, now_playing: &NowPlaying) -> UiTrack {
        for artist in &self.catalog.artists {
            for album in &artist.albums {
                for (id, track) in album.tracks.iter().enumerate() {
                    if track.path == now_playing.path {
                        return UiTrack {
                            id,
                            title: track.title.clone(),
                            album: album.title.clone(),
                            artist: artist.name.clone(),
                            track_number: Some(track.number as u32),
                            duration: Duration::from_secs(track.duration_secs as u64),
                            path: track.path.clone(),
                            cover_path: album.cover.as_ref().map(|cover| cover.cached_path.clone()),
                        };
                    }
                }
            }
        }
        UiTrack {
            id: 0,
            title: now_playing.title.clone(),
            album: now_playing.album.clone(),
            artist: now_playing.artist.clone(),
            track_number: None,
            duration: Duration::from_secs(now_playing.duration_secs as u64),
            path: now_playing.path.clone(),
            cover_path: None,
        }
    }

    fn queue_from_track_selection(&self, track: &UiTrack) -> (Vec<NowPlaying>, usize) {
        if let Some(selected_album) = self.ui.selection.selected_album.as_ref() {
            if let Some((artist, album)) = self.album_entry_by_id(selected_album.id) {
                let mut items = Vec::with_capacity(album.tracks.len());
                let mut selected_index = 0;
                for (index, album_track) in album.tracks.iter().enumerate() {
                    if album_track.path == track.path {
                        selected_index = index;
                    }
                    items.push(NowPlaying {
                        artist: artist.name.clone(),
                        album: album.title.clone(),
                        title: album_track.title.clone(),
                        duration_secs: album_track.duration_secs,
                        path: album_track.path.clone(),
                    });
                }
                if !items.is_empty() {
                    return (items, selected_index);
                }
            }
        }
        (vec![Self::now_playing_from_ui_track(track)], 0)
    }

    fn load_from_queue(&mut self, now_playing: Option<NowPlaying>) {
        let Some(player) = &mut self.player else {
            return;
        };
        let Some(now_playing) = now_playing else {
            return;
        };
        if let Err(err) = player.load(&now_playing.path) {
            error!(error = %err, path = %now_playing.path.display(), "Failed to load track");
            return;
        }
        player.play();
        self.ui.selection.selected_track = Some(self.ui_track_from_now_playing(&now_playing));
    }

    fn handle_playback_message(&mut self, message: &PlaybackMessage) {
        match message {
            PlaybackMessage::TogglePlayPause => {
                let Some(player) = &mut self.player else {
                    return;
                };
                match player.state() {
                    PlayerPlaybackState::Playing => player.pause(),
                    PlayerPlaybackState::Paused | PlayerPlaybackState::Stopped => player.play(),
                }
            }
            PlaybackMessage::NextTrack => {
                let next_track = self.playback_queue.next();
                self.load_from_queue(next_track);
            }
            PlaybackMessage::PreviousTrack => {
                let previous_track = self.playback_queue.previous();
                self.load_from_queue(previous_track);
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
        let section_padding = Padding {
            top: 4.0,
            right: 12.0,
            bottom: 0.0,
            left: 12.0,
        };
        let header = row![
            text("Préférences")
                .size(theme.size(22))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            button(
                text("Fermer")
                    .size(theme.size(13))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::ListItem { selected: false },
                    status,
                )
            })
            .padding([6, 10])
            .on_press(UiMessage::ClosePreferences)
        ]
        .align_y(Alignment::Center)
        .spacing(12);

        let menu_button = |tab: PreferencesTab, label: &'static str| {
            button(
                text(label)
                    .size(theme.size(14))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::ListItem {
                        selected: self.ui.preferences_tab == tab,
                    },
                    status,
                )
            })
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

        let section_header = |label: &'static str, expanded: bool, message: UiMessage| {
            let chevron = if expanded { "▾" } else { "▸" };
            button(
                row![
                    text(label)
                        .size(theme.size(14))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_primary(theme)),
                    text(chevron)
                        .size(theme.size(14))
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_muted(theme)),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::ListItem { selected: expanded },
                    status,
                )
            })
            .padding([8, 12])
            .width(Length::Fill)
            .on_press(message)
        };
        let section_hint = |label: &'static str| {
            text(label)
                .size(theme.size(12))
                .font(style::font_propo(Weight::Light))
                .style(move |_| style::text_style_muted(theme))
        };
        let setting_label = |title: &'static str, subtitle: &'static str| {
            column![
                text(title)
                    .size(theme.size(13))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
                text(subtitle)
                    .size(theme.size(12))
                    .font(style::font_propo(Weight::Light))
                    .style(move |_| style::text_style_muted(theme)),
            ]
            .spacing(2)
            .width(Length::Fill)
        };
        let option_button = |selected: bool, label: &'static str, message: UiMessage| {
            button(
                text(label)
                    .size(theme.size(12))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| {
                style::button_style(theme, style::ButtonKind::Tab { selected }, status)
            })
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
        fn controls<'a>(content: Element<'a, UiMessage>) -> Element<'a, UiMessage> {
            container(content)
                .width(Length::FillPortion(2))
                .center_x(Length::Fill)
                .padding(Padding {
                    top: 0.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 24.0,
                })
                .into()
        }
        let action_button = |label: &'static str, message: UiMessage| {
            button(
                text(label)
                    .size(theme.size(12))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| style::button_style(theme, style::ButtonKind::Control, status))
            .padding([6, 10])
            .on_press(message)
        };

        let library_input = text_input("Dossier de bibliothèque", &self.ui.settings.library_folder)
            .style(move |_, status| style::text_input_style(theme, status))
            .on_input(UiMessage::LibraryFolderChanged);
        let cache_input = text_input("Emplacement du cache", &self.ui.settings.cache_path)
            .style(move |_, status| style::text_input_style(theme, status))
            .on_input(UiMessage::CachePathChanged);

        let startup_content = || {
            column![
                section_hint("Gérez l'ouverture de Grape et la restauration des sessions."),
                row![
                    setting_label(
                        "Lancer Grape au démarrage du système",
                        "Active l'application dès l'ouverture de votre session."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.launch_at_startup,
                            UiMessage::SetLaunchAtStartup(true),
                            UiMessage::SetLaunchAtStartup(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Restaurer la dernière session",
                        "Lecture, file d'attente et écran affiché."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.restore_last_session,
                            UiMessage::SetRestoreLastSession(true),
                            UiMessage::SetRestoreLastSession(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Ouvrir sur", "Choisissez l'écran par défaut."),
                    controls(
                        column![
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
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Comportement à la fermeture",
                        "Choisissez l'action à la fermeture."
                    ),
                    controls(
                        row![
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
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let language_content = || {
            column![
                section_hint("Personnalisez l'interface et le format horaire."),
                row![
                    setting_label("Langue de l'interface", "Synchronisée avec le système."),
                    controls(
                        row![
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
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Format horaire", "Format utilisé dans l'application."),
                    controls(
                        row![
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
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let updates_content = || {
            column![
                section_hint("Gérez la vérification et le canal des mises à jour."),
                row![
                    setting_label(
                        "Vérifier automatiquement les mises à jour",
                        "Vérifie les nouvelles versions au lancement."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.auto_check_updates,
                            UiMessage::SetAutoCheckUpdates(true),
                            UiMessage::SetAutoCheckUpdates(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Canal", "Choisissez la stabilité des versions."),
                    controls(
                        row![
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
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Télécharger et installer automatiquement",
                        "Installe les mises à jour en arrière-plan."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.auto_install_updates,
                            UiMessage::SetAutoInstallUpdates(true),
                            UiMessage::SetAutoInstallUpdates(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let privacy_content = || {
            column![
                section_hint("Choisissez les données partagées avec Grape."),
                row![
                    setting_label(
                        "Envoyer des rapports d'erreurs",
                        "Permet d'améliorer la stabilité."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.send_error_reports,
                            UiMessage::SetSendErrorReports(true),
                            UiMessage::SetSendErrorReports(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Envoyer des statistiques anonymes d'utilisation",
                        "Aide à comprendre l'usage de Grape."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.send_usage_stats,
                            UiMessage::SetSendUsageStats(true),
                            UiMessage::SetSendUsageStats(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Effacer l'historique local", "Supprime les traces locales."),
                    controls(action_button("Effacer", UiMessage::ClearHistory).into()),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
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
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Scanner automatiquement au lancement",
                        "Met à jour la bibliothèque au démarrage."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.auto_scan_on_launch,
                            UiMessage::SetAutoScanOnLaunch(true),
                            UiMessage::SetAutoScanOnLaunch(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
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
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let notifications_content = || {
            column![
                section_hint("Gérez les alertes système."),
                row![
                    setting_label(
                        "Activer les notifications système",
                        "Autorise l'affichage des notifications."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.notifications_enabled,
                            UiMessage::SetNotificationsEnabled(true),
                            UiMessage::SetNotificationsEnabled(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Afficher “Now Playing” lors des changements de piste",
                        "Notification à chaque changement de lecture."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.now_playing_notifications,
                            UiMessage::SetNowPlayingNotifications(true),
                            UiMessage::SetNowPlayingNotifications(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let performance_content = || {
            column![
                section_hint("Ajustez les options de performance."),
                row![
                    setting_label(
                        "Accélération matérielle",
                        "Utilise le GPU pour les animations."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.hardware_acceleration,
                            UiMessage::SetHardwareAcceleration(true),
                            UiMessage::SetHardwareAcceleration(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Limiter l'utilisation CPU pendant la lecture",
                        "Réduit la charge pendant la musique."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.limit_cpu_during_playback,
                            UiMessage::SetLimitCpuDuringPlayback(true),
                            UiMessage::SetLimitCpuDuringPlayback(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let advanced_content = || {
            column![
                section_hint("Outils pour diagnostiquer et réinitialiser."),
                row![
                    setting_label("Ouvrir le dossier de logs", "Accès aux journaux."),
                    controls(action_button("Ouvrir", UiMessage::OpenLogsFolder).into()),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Réindexer la bibliothèque", "Reconstruit l'index local."),
                    controls(action_button("Réindexer", UiMessage::ReindexLibrary).into()),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Réinitialiser les préférences",
                        "Restaure les valeurs par défaut."
                    ),
                    controls(action_button("Réinitialiser", UiMessage::ResetPreferences).into()),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let general_panel = scrollable(
            column![
                column![
                    text("Paramètres généraux")
                        .size(theme.size(16))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_primary(theme)),
                    text("Les préférences sont enregistrées automatiquement.")
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme))
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

        let accent_color_value = |accent: AccentColor| match accent {
            AccentColor::Blue => Color::from_rgb8(0x3d, 0x7c, 0xff),
            AccentColor::Violet => Color::from_rgb8(0xa0, 0x6c, 0xff),
            AccentColor::Green => Color::from_rgb8(0x2f, 0xd0, 0x8c),
            AccentColor::Amber => Color::from_rgb8(0xf2, 0xb3, 0x47),
        };
        let accent_button = |accent: AccentColor| {
            let selected = self.ui.settings.accent_color == accent;
            button(
                row![
                    text("●")
                        .size(theme.size(14))
                        .style(move |_| style::text_style(accent_color_value(accent))),
                    text(accent.label())
                        .size(theme.size(12))
                        .font(style::font_propo(Weight::Medium))
                        .style(move |_| style::text_style_primary(theme)),
                ]
                .spacing(6)
                .align_y(Alignment::Center),
            )
            .style(move |_, status| {
                style::button_style(theme, style::ButtonKind::Tab { selected }, status)
            })
            .padding([6, 10])
            .on_press(UiMessage::SetAccentColor(accent))
        };
        let typography_group = || {
            column![
                row![
                    setting_label(
                        "Taille de police UI",
                        "Ajustez la taille des textes pour améliorer la lisibilité."
                    ),
                    controls(
                        column![
                            slider(
                                0.0..=2.0,
                                self.ui.settings.text_scale.slider_value(),
                                |value| UiMessage::SetTextScale(TextScale::from_slider_value(
                                    value
                                )),
                            ),
                            text(self.ui.settings.text_scale.label())
                                .size(theme.size(12))
                                .font(style::font_propo(Weight::Light))
                                .style(move |_| style::text_style_muted(theme)),
                        ]
                        .spacing(6)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Densité d'interface",
                        "Choisissez l'espacement des éléments."
                    ),
                    controls(
                        row![
                            option_button(
                                self.ui.settings.interface_density == InterfaceDensity::Compact,
                                InterfaceDensity::Compact.label(),
                                UiMessage::SetInterfaceDensity(InterfaceDensity::Compact),
                            ),
                            option_button(
                                self.ui.settings.interface_density == InterfaceDensity::Comfort,
                                InterfaceDensity::Comfort.label(),
                                UiMessage::SetInterfaceDensity(InterfaceDensity::Comfort),
                            ),
                            option_button(
                                self.ui.settings.interface_density == InterfaceDensity::Large,
                                InterfaceDensity::Large.label(),
                                UiMessage::SetInterfaceDensity(InterfaceDensity::Large),
                            ),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let vision_group = || {
            column![
                row![
                    setting_label("Augmenter le contraste", "Renforce les contrastes UI."),
                    controls(
                        toggle_row(
                            self.ui.settings.increase_contrast,
                            UiMessage::SetIncreaseContrast(true),
                            UiMessage::SetIncreaseContrast(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Réduire la transparence",
                        "Diminue les effets translucides."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.reduce_transparency,
                            UiMessage::SetReduceTransparency(true),
                            UiMessage::SetReduceTransparency(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Taille de texte accessible",
                        "Ajustez les textes d'aide et contenus."
                    ),
                    controls(
                        column![
                            slider(
                                0.0..=2.0,
                                self.ui.settings.accessible_text_size.slider_value(),
                                |value| UiMessage::SetAccessibleTextSize(
                                    AccessibleTextSize::from_slider_value(value)
                                ),
                            ),
                            text(self.ui.settings.accessible_text_size.label())
                                .size(theme.size(12))
                                .font(style::font_propo(Weight::Light))
                                .style(move |_| style::text_style_muted(theme)),
                        ]
                        .spacing(6)
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let movement_group = || {
            column![
                row![
                    setting_label(
                        "Réduire les animations",
                        "Limite les animations décoratives."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.reduce_animations,
                            UiMessage::SetReduceAnimations(true),
                            UiMessage::SetReduceAnimations(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Réduire les transitions",
                        "Simplifie les transitions d'écran."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.reduce_transitions,
                            UiMessage::SetReduceTransitions(true),
                            UiMessage::SetReduceTransitions(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let audio_subtitles_group = || {
            column![
                row![
                    setting_label(
                        "Sous-titres par défaut",
                        "Active automatiquement les sous-titres."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.subtitles_enabled,
                            UiMessage::SetSubtitlesEnabled(true),
                            UiMessage::SetSubtitlesEnabled(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Taille des sous-titres", "Ajustez la taille affichée."),
                    controls(
                        column![
                            slider(
                                0.0..=2.0,
                                self.ui.settings.subtitle_size.slider_value(),
                                |value| {
                                    UiMessage::SetSubtitleSize(SubtitleSize::from_slider_value(
                                        value,
                                    ))
                                },
                            ),
                            text(self.ui.settings.subtitle_size.label())
                                .size(theme.size(12))
                                .font(style::font_propo(Weight::Light))
                                .style(move |_| style::text_style_muted(theme)),
                        ]
                        .spacing(6)
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let navigation_group = || {
            column![
                row![
                    setting_label(
                        "Surligner le focus clavier",
                        "Met en avant l'élément actif."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.highlight_keyboard_focus,
                            UiMessage::SetHighlightKeyboardFocus(true),
                            UiMessage::SetHighlightKeyboardFocus(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Activer raccourcis avancés",
                        "Débloque les raccourcis experts."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.enable_advanced_shortcuts,
                            UiMessage::SetAdvancedShortcuts(true),
                            UiMessage::SetAdvancedShortcuts(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let playback_group = || {
            let playback_speed = self.ui.settings.default_playback_speed as f32 / 10.0;
            column![
                row![
                    setting_label("Vitesse de lecture par défaut", "Appliquée aux médias."),
                    controls(
                        column![
                            slider(0.5..=2.0, playback_speed, |value| {
                                UiMessage::SetDefaultPlaybackSpeed((value * 10.0).round() as u8)
                            }),
                            text(format!("{:.1}x", playback_speed))
                                .size(theme.size(12))
                                .font(style::font_propo(Weight::Light))
                                .style(move |_| style::text_style_muted(theme)),
                        ]
                        .spacing(6)
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Pause auto sur perte de focus",
                        "Met en pause si l'app perd le focus."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.pause_on_focus_loss,
                            UiMessage::SetPauseOnFocusLoss(true),
                            UiMessage::SetPauseOnFocusLoss(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let appearance_theme_content = || {
            column![
                row![
                    option_button(
                        self.ui.settings.theme_mode == ThemeMode::Dark,
                        ThemeMode::Dark.label(),
                        UiMessage::SetThemeMode(ThemeMode::Dark),
                    ),
                    option_button(
                        self.ui.settings.theme_mode == ThemeMode::Light,
                        ThemeMode::Light.label(),
                        UiMessage::SetThemeMode(ThemeMode::Light),
                    ),
                    option_button(
                        self.ui.settings.theme_mode == ThemeMode::System,
                        ThemeMode::System.label(),
                        UiMessage::SetThemeMode(ThemeMode::System),
                    ),
                ]
                .spacing(8),
                row![
                    setting_label(
                        "Suivre le thème du système",
                        "Synchronise automatiquement clair / sombre."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.follow_system_theme,
                            UiMessage::SetFollowSystemTheme(true),
                            UiMessage::SetFollowSystemTheme(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let appearance_accents_content = || {
            column![
                row![
                    accent_button(AccentColor::Blue),
                    accent_button(AccentColor::Violet),
                    accent_button(AccentColor::Green),
                    accent_button(AccentColor::Amber),
                ]
                .spacing(8),
                row![
                    setting_label(
                        "Accent automatique selon le fond",
                        "Adapte automatiquement l'accent aux arrière-plans."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.accent_auto,
                            UiMessage::SetAccentAuto(true),
                            UiMessage::SetAccentAuto(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let appearance_effects_content = || {
            column![
                row![
                    setting_label("Transparence / Flou", "Applique des effets de profondeur."),
                    controls(
                        toggle_row(
                            self.ui.settings.transparency_blur,
                            UiMessage::SetTransparencyBlur(true),
                            UiMessage::SetTransparencyBlur(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Animations d'interface",
                        "Active les transitions et micro-interactions."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.ui_animations,
                            UiMessage::SetUiAnimations(true),
                            UiMessage::SetUiAnimations(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let appearance_preview_content = || {
            column![
                container(
                    column![
                        text("Carte de prévisualisation")
                            .size(theme.size(13))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_primary(theme)),
                        text(format!(
                            "Thème : {} · Accent : {} · Densité : {}",
                            self.ui.settings.theme_mode.label(),
                            self.ui.settings.accent_color.label(),
                            self.ui.settings.interface_density.label()
                        ))
                        .size(theme.size(12))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme)),
                        text(format!(
                            "Texte : {} · Effets : {} · Animations : {}",
                            self.ui.settings.text_scale.label(),
                            if self.ui.settings.transparency_blur {
                                "Activés"
                            } else {
                                "Désactivés"
                            },
                            if self.ui.settings.ui_animations {
                                "Activées"
                            } else {
                                "Désactivées"
                            }
                        ))
                        .size(theme.size(12))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme)),
                    ]
                    .spacing(4),
                )
                .padding(12)
                .width(Length::Fill)
                .style(move |_| style::surface_style(theme, style::Surface::Panel)),
            ]
            .spacing(12)
            .padding(section_padding)
        };

        let appearance_panel = scrollable(
            column![
                column![
                    text("Paramètres d'apparence")
                        .size(theme.size(16))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_primary(theme)),
                    text("Ajustez le thème, les accents et les effets visuels.")
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme))
                ]
                .spacing(6),
                section_header(
                    "Thème",
                    self.ui.preferences_sections.appearance_theme,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AppearanceTheme),
                ),
                if self.ui.preferences_sections.appearance_theme {
                    appearance_theme_content()
                } else {
                    column![]
                },
                section_header(
                    "Couleurs & accents",
                    self.ui.preferences_sections.appearance_accents,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AppearanceAccents),
                ),
                if self.ui.preferences_sections.appearance_accents {
                    appearance_accents_content()
                } else {
                    column![]
                },
                section_header(
                    "Typographie",
                    self.ui.preferences_sections.appearance_typography,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AppearanceTypography),
                ),
                if self.ui.preferences_sections.appearance_typography {
                    typography_group()
                } else {
                    column![]
                },
                section_header(
                    "Arrière-plans & effets",
                    self.ui.preferences_sections.appearance_effects,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AppearanceEffects),
                ),
                if self.ui.preferences_sections.appearance_effects {
                    appearance_effects_content()
                } else {
                    column![]
                },
                section_header(
                    "Aperçu",
                    self.ui.preferences_sections.appearance_preview,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AppearancePreview),
                ),
                if self.ui.preferences_sections.appearance_preview {
                    appearance_preview_content()
                } else {
                    column![]
                },
            ]
            .spacing(16),
        )
        .height(Length::Fill);

        let accessibility_panel = scrollable(
            column![
                column![
                    text("Paramètres d'accessibilité")
                        .size(theme.size(16))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_primary(theme)),
                    text("Facilitez la lecture, la navigation et la lecture média.")
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme))
                ]
                .spacing(6),
                section_header(
                    "Vision",
                    self.ui.preferences_sections.accessibility_vision,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AccessibilityVision),
                ),
                if self.ui.preferences_sections.accessibility_vision {
                    vision_group()
                } else {
                    column![]
                },
                section_header(
                    "Mouvement",
                    self.ui.preferences_sections.accessibility_movement,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AccessibilityMovement),
                ),
                if self.ui.preferences_sections.accessibility_movement {
                    movement_group()
                } else {
                    column![]
                },
                section_header(
                    "Audio & sous-titres",
                    self.ui.preferences_sections.accessibility_audio_subtitles,
                    UiMessage::TogglePreferencesSection(
                        PreferencesSection::AccessibilityAudioSubtitles
                    ),
                ),
                if self.ui.preferences_sections.accessibility_audio_subtitles {
                    audio_subtitles_group()
                } else {
                    column![]
                },
                section_header(
                    "Navigation & interaction",
                    self.ui.preferences_sections.accessibility_navigation,
                    UiMessage::TogglePreferencesSection(
                        PreferencesSection::AccessibilityNavigation
                    ),
                ),
                if self.ui.preferences_sections.accessibility_navigation {
                    navigation_group()
                } else {
                    column![]
                },
                section_header(
                    "Lecture",
                    self.ui.preferences_sections.accessibility_playback,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AccessibilityPlayback),
                ),
                if self.ui.preferences_sections.accessibility_playback {
                    playback_group()
                } else {
                    column![]
                },
            ]
            .spacing(16),
        )
        .height(Length::Fill);

        let volume_value = self.ui.settings.default_volume as f32;
        let crossfade_value = self.ui.settings.crossfade_seconds as f32;
        let audio_output_content = || {
            column![
                section_hint("Choisissez la sortie audio principale."),
                row![
                    setting_label("Périphérique de sortie", "Sélectionnez la sortie active."),
                    controls(
                        row![
                            option_button(
                                self.ui.settings.output_device == AudioOutputDevice::System,
                                AudioOutputDevice::System.label(),
                                UiMessage::SetAudioOutputDevice(AudioOutputDevice::System)
                            ),
                            option_button(
                                self.ui.settings.output_device == AudioOutputDevice::UsbHeadset,
                                AudioOutputDevice::UsbHeadset.label(),
                                UiMessage::SetAudioOutputDevice(AudioOutputDevice::UsbHeadset)
                            ),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Si le périphérique disparaît",
                        "Détermine la reprise automatique."
                    ),
                    controls(
                        row![
                            option_button(
                                self.ui.settings.missing_device_behavior
                                    == MissingDeviceBehavior::SwitchToSystem,
                                MissingDeviceBehavior::SwitchToSystem.label(),
                                UiMessage::SetMissingDeviceBehavior(
                                    MissingDeviceBehavior::SwitchToSystem
                                )
                            ),
                            option_button(
                                self.ui.settings.missing_device_behavior
                                    == MissingDeviceBehavior::PausePlayback,
                                MissingDeviceBehavior::PausePlayback.label(),
                                UiMessage::SetMissingDeviceBehavior(
                                    MissingDeviceBehavior::PausePlayback
                                )
                            ),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };
        let audio_playback_content = || {
            column![
                section_hint("Gérez la transition entre les morceaux."),
                row![
                    setting_label(
                        "Lecture sans blanc (Gapless)",
                        "Supprime les silences entre les pistes."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.gapless_playback,
                            UiMessage::SetGaplessPlayback(true),
                            UiMessage::SetGaplessPlayback(false)
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label(
                        "Fondu enchaîné (Crossfade)",
                        "Durée du fondu entre les morceaux."
                    ),
                    controls(
                        column![
                            slider(0.0..=12.0, crossfade_value, |value| {
                                UiMessage::SetCrossfadeSeconds(value.round().clamp(0.0, 12.0) as u8)
                            }),
                            text(format!("{} s", self.ui.settings.crossfade_seconds))
                                .size(theme.size(12))
                                .font(style::font_propo(Weight::Medium))
                                .style(move |_| style::text_style_muted(theme))
                        ]
                        .spacing(6)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Automix", "Mixe automatiquement les transitions."),
                    controls(
                        toggle_row(
                            self.ui.settings.automix_enabled,
                            UiMessage::SetAutomixEnabled(true),
                            UiMessage::SetAutomixEnabled(false)
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };
        let audio_volume_content = || {
            column![
                section_hint("Ajustez la dynamique et le volume global."),
                row![
                    setting_label("Normaliser le volume", "Harmonise les niveaux sonores."),
                    controls(
                        toggle_row(
                            self.ui.settings.normalize_volume,
                            UiMessage::SetNormalizeVolume(true),
                            UiMessage::SetNormalizeVolume(false)
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Niveau", "Profil de volume préféré."),
                    controls(
                        row![
                            option_button(
                                self.ui.settings.volume_level == VolumeLevel::Quiet,
                                VolumeLevel::Quiet.label(),
                                UiMessage::SetVolumeLevel(VolumeLevel::Quiet)
                            ),
                            option_button(
                                self.ui.settings.volume_level == VolumeLevel::Normal,
                                VolumeLevel::Normal.label(),
                                UiMessage::SetVolumeLevel(VolumeLevel::Normal)
                            ),
                            option_button(
                                self.ui.settings.volume_level == VolumeLevel::Loud,
                                VolumeLevel::Loud.label(),
                                UiMessage::SetVolumeLevel(VolumeLevel::Loud)
                            ),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Volume par défaut", "Volume global de l'application."),
                    controls(
                        column![
                            slider(0.0..=100.0, volume_value, |value| {
                                UiMessage::SetDefaultVolume(value.round().clamp(0.0, 100.0) as u8)
                            }),
                            text(format!("{} %", self.ui.settings.default_volume))
                                .size(theme.size(13))
                                .font(style::font_propo(Weight::Medium))
                                .style(move |_| style::text_style_muted(theme))
                        ]
                        .spacing(6)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };
        let audio_equalizer_content = || {
            column![
                section_hint("Sculptez le rendu audio avec un preset."),
                row![
                    setting_label("Activer l'égaliseur", "Active les réglages EQ."),
                    controls(
                        toggle_row(
                            self.ui.settings.eq_enabled,
                            UiMessage::SetEqEnabled(true),
                            UiMessage::SetEqEnabled(false)
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Preset", "Sélectionnez un profil."),
                    controls(
                        column![
                            row![
                                option_button(
                                    self.ui.settings.eq_preset == EqPreset::Flat,
                                    EqPreset::Flat.label(),
                                    UiMessage::SetEqPreset(EqPreset::Flat)
                                ),
                                option_button(
                                    self.ui.settings.eq_preset == EqPreset::Bass,
                                    EqPreset::Bass.label(),
                                    UiMessage::SetEqPreset(EqPreset::Bass)
                                ),
                                option_button(
                                    self.ui.settings.eq_preset == EqPreset::Treble,
                                    EqPreset::Treble.label(),
                                    UiMessage::SetEqPreset(EqPreset::Treble)
                                ),
                            ]
                            .spacing(8),
                            row![
                                option_button(
                                    self.ui.settings.eq_preset == EqPreset::Vocal,
                                    EqPreset::Vocal.label(),
                                    UiMessage::SetEqPreset(EqPreset::Vocal)
                                ),
                                option_button(
                                    self.ui.settings.eq_preset == EqPreset::Custom,
                                    EqPreset::Custom.label(),
                                    UiMessage::SetEqPreset(EqPreset::Custom)
                                ),
                            ]
                            .spacing(8),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Réinitialiser EQ", "Retourne aux réglages par défaut."),
                    controls(action_button("Réinitialiser", UiMessage::ResetEq).into()),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };
        let audio_advanced_content = || {
            column![
                section_hint("Options avancées pour la stabilité audio."),
                row![
                    setting_label("Mode de stabilité audio", "Ajuste la latence et le buffer."),
                    controls(
                        row![
                            option_button(
                                self.ui.settings.audio_stability_mode == AudioStabilityMode::Auto,
                                AudioStabilityMode::Auto.label(),
                                UiMessage::SetAudioStabilityMode(AudioStabilityMode::Auto)
                            ),
                            option_button(
                                self.ui.settings.audio_stability_mode == AudioStabilityMode::Stable,
                                AudioStabilityMode::Stable.label(),
                                UiMessage::SetAudioStabilityMode(AudioStabilityMode::Stable)
                            ),
                            option_button(
                                self.ui.settings.audio_stability_mode
                                    == AudioStabilityMode::LowLatency,
                                AudioStabilityMode::LowLatency.label(),
                                UiMessage::SetAudioStabilityMode(AudioStabilityMode::LowLatency)
                            ),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Réinitialiser le moteur audio", "Recharge la sortie audio."),
                    controls(action_button("Réinitialiser", UiMessage::ResetAudioEngine).into()),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Logs audio (debug)", "Active la journalisation audio."),
                    controls(
                        toggle_row(
                            self.ui.settings.audio_debug_logs,
                            UiMessage::SetAudioDebugLogs(true),
                            UiMessage::SetAudioDebugLogs(false)
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
            ]
            .spacing(12)
            .padding(section_padding)
        };
        let audio_panel = scrollable(
            column![
                column![
                    text("Paramètres audio")
                        .size(theme.size(16))
                        .font(style::font_propo(Weight::Semibold))
                        .style(move |_| style::text_style_primary(theme)),
                    text("Personnalisez la sortie et la lecture audio.")
                        .size(theme.size(13))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme))
                ]
                .spacing(6),
                section_header(
                    "Sortie audio",
                    self.ui.preferences_sections.audio_output,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AudioOutput),
                ),
                if self.ui.preferences_sections.audio_output {
                    audio_output_content()
                } else {
                    column![]
                },
                section_header(
                    "Lecture",
                    self.ui.preferences_sections.audio_playback,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AudioPlayback),
                ),
                if self.ui.preferences_sections.audio_playback {
                    audio_playback_content()
                } else {
                    column![]
                },
                section_header(
                    "Niveau sonore",
                    self.ui.preferences_sections.audio_volume,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AudioVolume),
                ),
                if self.ui.preferences_sections.audio_volume {
                    audio_volume_content()
                } else {
                    column![]
                },
                section_header(
                    "Égaliseur",
                    self.ui.preferences_sections.audio_equalizer,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AudioEqualizer),
                ),
                if self.ui.preferences_sections.audio_equalizer {
                    audio_equalizer_content()
                } else {
                    column![]
                },
                section_header(
                    "Avancé / Dépannage",
                    self.ui.preferences_sections.audio_advanced,
                    UiMessage::TogglePreferencesSection(PreferencesSection::AudioAdvanced),
                ),
                if self.ui.preferences_sections.audio_advanced {
                    audio_advanced_content()
                } else {
                    column![]
                },
            ]
            .spacing(16),
        )
        .height(Length::Fill);

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
                .style(move |_| style::surface_style(theme, style::Surface::Sidebar)),
            container(content_panel)
                .padding(20)
                .width(Length::Fill)
                .style(move |_| style::surface_style(theme, style::Surface::Panel))
        ]
        .spacing(16)
        .height(Length::Fill);

        column![header, body]
            .spacing(16)
            .height(Length::Fill)
            .into()
    }
}

impl GrapeApp {
    fn new(catalog: Catalog) -> Self {
        let player = match Player::new() {
            Ok(player) => Some(player),
            Err(err) => {
                error!(error = %err, "Failed to initialize audio player");
                None
            }
        };
        let settings = config::load_settings();
        Self {
            catalog,
            player,
            playlists: PlaylistManager::new_default(),
            playback_queue: PlaybackQueue::default(),
            ui: UiState::new(settings),
        }
    }

    fn title(&self) -> String {
        "Grape".to_string()
    }

    fn update(&mut self, message: UiMessage) -> Task<UiMessage> {
        let should_persist = matches!(
            message,
            UiMessage::SetThemeMode(_)
                | UiMessage::SetFollowSystemTheme(_)
                | UiMessage::SetAccentColor(_)
                | UiMessage::SetAccentAuto(_)
                | UiMessage::SetTextScale(_)
                | UiMessage::SetInterfaceDensity(_)
                | UiMessage::SetTransparencyBlur(_)
                | UiMessage::SetUiAnimations(_)
                | UiMessage::SetIncreaseContrast(_)
                | UiMessage::SetReduceTransparency(_)
                | UiMessage::SetAccessibleTextSize(_)
                | UiMessage::SetReduceAnimations(_)
                | UiMessage::SetReduceTransitions(_)
                | UiMessage::SetSubtitlesEnabled(_)
                | UiMessage::SetSubtitleSize(_)
                | UiMessage::SetHighlightKeyboardFocus(_)
                | UiMessage::SetAdvancedShortcuts(_)
                | UiMessage::SetDefaultPlaybackSpeed(_)
                | UiMessage::SetPauseOnFocusLoss(_)
                | UiMessage::SetDefaultVolume(_)
                | UiMessage::SetAudioOutputDevice(_)
                | UiMessage::SetMissingDeviceBehavior(_)
                | UiMessage::SetGaplessPlayback(_)
                | UiMessage::SetCrossfadeSeconds(_)
                | UiMessage::SetAutomixEnabled(_)
                | UiMessage::SetNormalizeVolume(_)
                | UiMessage::SetVolumeLevel(_)
                | UiMessage::SetEqEnabled(_)
                | UiMessage::SetEqPreset(_)
                | UiMessage::ResetEq
                | UiMessage::SetAudioStabilityMode(_)
                | UiMessage::SetAudioDebugLogs(_)
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
        let mut task = Task::none();
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
            UiMessage::WindowMinimize => {
                task = window::oldest().then(|id| {
                    if let Some(id) = id {
                        window::minimize(id, true)
                    } else {
                        Task::none()
                    }
                });
            }
            UiMessage::WindowToggleMaximize => {
                task = window::oldest().then(|id| {
                    if let Some(id) = id {
                        window::toggle_maximize(id)
                    } else {
                        Task::none()
                    }
                });
            }
            UiMessage::WindowClose => {
                task = window::oldest().then(|id| {
                    if let Some(id) = id {
                        window::close(id)
                    } else {
                        Task::none()
                    }
                });
            }
            UiMessage::PickLibraryFolder => {
                task = Task::perform(
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
            UiMessage::ResetAudioEngine => {
                info!("Audio engine reset requested");
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
        task
    }

    fn view(&self) -> Element<'_, UiMessage> {
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
            .style(move |_| style::surface_style(theme, style::Surface::AppBackground))
            .into()
    }

    fn theme(&self) -> Theme {
        match self.ui.settings.theme_mode {
            ThemeMode::Dark => Theme::Dark,
            ThemeMode::Light => Theme::Light,
            ThemeMode::System => Theme::Dark,
        }
    }

    fn subscription(&self) -> Subscription<UiMessage> {
        let mut subscriptions = Vec::new();

        if self.ui.menu_open {
            subscriptions.push(event::listen_with(|event, status, _| match event {
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
            subscriptions.push(event::listen_with(|event, _status, _| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
                {
                    Some(UiMessage::ClosePlaylist)
                }
                _ => None,
            }));
        }

        if self.ui.preferences_open {
            subscriptions.push(event::listen_with(|event, _status, _| match event {
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
