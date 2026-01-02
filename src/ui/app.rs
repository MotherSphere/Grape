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
    SortOption, Track as UiTrack, UiState,
};
use crate::ui::style;
use iced::font::Weight;
use iced::theme::{Button, Container, TextInput};
use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Application, Command, Element, Length, Settings, event, keyboard, mouse};
use iced::{Subscription, Theme};
use std::time::Duration;
use tracing::error;

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
        let logo_mark = container(
            text("G")
                .size(18)
                .font(style::font_propo(Weight::Bold))
                .style(style::text_primary()),
        )
        .padding([6, 10])
        .style(Container::Custom(Box::new(style::SurfaceStyle(
            style::Surface::Avatar,
        ))));
        let logo = row![
            logo_mark,
            text("Grape")
                .size(20)
                .font(style::font_propo(Weight::Semibold))
                .style(style::text_primary())
        ]
        .spacing(8)
        .align_items(Alignment::Center);
        let logo_button = button(logo)
            .style(Button::Custom(Box::new(style::ButtonStyle(
                style::ButtonKind::Icon,
            ))))
            .padding([2, 6])
            .on_press(UiMessage::ToggleLogoMenu);
        let menu_button = |label, message| {
            button(
                text(label)
                    .size(13)
                    .font(style::font_propo(Weight::Medium))
                    .style(style::text_primary()),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle(
                style::ButtonKind::ListItem { selected: false },
            ))))
            .padding([4, 8])
            .on_press(message)
        };
        let logo_menu = container(
            column![
                menu_button("Bibliothèque", UiMessage::CloseMenu),
                menu_button("Playlist", UiMessage::OpenPlaylist),
                menu_button("Préférences", UiMessage::CloseMenu),
            ]
            .spacing(6),
        )
        .padding([8, 12])
        .style(Container::Custom(Box::new(style::SurfaceStyle(
            style::Surface::Panel,
        ))));
        let logo_widget: Element<'_, UiMessage> = if self.ui.menu_open {
            AnchoredOverlay::new(logo_button, logo_menu).into()
        } else {
            logo_button.into()
        };
        let tabs = row![
            button(
                text(self.tab_label(ActiveTab::Artists, "Artists"))
                    .font(style::font_propo(Weight::Medium)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle(
                style::ButtonKind::Tab {
                    selected: self.ui.active_tab == ActiveTab::Artists,
                },
            ))))
            .on_press(UiMessage::TabSelected(ActiveTab::Artists)),
            button(
                text(self.tab_label(ActiveTab::Genres, "Genres"))
                    .font(style::font_propo(Weight::Medium)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle(
                style::ButtonKind::Tab {
                    selected: self.ui.active_tab == ActiveTab::Genres,
                },
            ))))
            .on_press(UiMessage::TabSelected(ActiveTab::Genres)),
            button(
                text(self.tab_label(ActiveTab::Albums, "Albums"))
                    .font(style::font_propo(Weight::Medium)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle(
                style::ButtonKind::Tab {
                    selected: self.ui.active_tab == ActiveTab::Albums,
                },
            ))))
            .on_press(UiMessage::TabSelected(ActiveTab::Albums)),
            button(
                text(self.tab_label(ActiveTab::Folders, "Folders"))
                    .font(style::font_propo(Weight::Medium)),
            )
            .style(Button::Custom(Box::new(style::ButtonStyle(
                style::ButtonKind::Tab {
                    selected: self.ui.active_tab == ActiveTab::Folders,
                },
            ))))
            .on_press(UiMessage::TabSelected(ActiveTab::Folders)),
        ]
        .spacing(12)
        .align_items(Alignment::Center);
        let search_input = text_input("Search...", &self.ui.search.query)
            .style(TextInput::Custom(Box::new(style::SearchInput)))
            .on_input(|value| UiMessage::Search(SearchMessage::QueryChanged(value)));
        let search = row![
            search_input,
            button(text("≡").font(style::font_propo(Weight::Medium))).style(Button::Custom(
                Box::new(style::ButtonStyle(style::ButtonKind::Icon))
            ),),
            button(text("—").font(style::font_propo(Weight::Medium))).style(Button::Custom(
                Box::new(style::ButtonStyle(style::ButtonKind::Icon))
            ),),
            button(text("▢").font(style::font_propo(Weight::Medium))).style(Button::Custom(
                Box::new(style::ButtonStyle(style::ButtonKind::Icon))
            ),),
            button(text("✕").font(style::font_propo(Weight::Medium))).style(Button::Custom(
                Box::new(style::ButtonStyle(style::ButtonKind::Icon))
            ),)
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
            .style(Container::Custom(Box::new(style::SurfaceStyle(
                style::Surface::TopBar,
            ))))
            .into()
    }

    fn artists_panel(&self) -> Element<'_, UiMessage> {
        let selected_id = self
            .ui
            .selection
            .selected_artist
            .as_ref()
            .map(|artist| artist.id);
        let artists = self.filtered_artists_from_catalog();
        let panel = ArtistsPanel::new(artists).with_selection(selected_id);
        panel.view(&self.ui.selection)
    }

    fn albums_panel(&self) -> Element<'_, UiMessage> {
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
            .view();

        container(grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn songs_panel(&self) -> Element<'_, UiMessage> {
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
        panel.view(&self.ui.selection)
    }

    fn genres_panel(&self) -> Element<'_, UiMessage> {
        let selected_id = self
            .ui
            .selection
            .selected_genre
            .as_ref()
            .map(|genre| genre.id);
        let genres = self.filtered_genres_from_catalog();
        let panel = GenresPanel::new(genres).with_selection(selected_id);
        panel.view()
    }

    fn folders_panel(&self) -> Element<'_, UiMessage> {
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
            .view()
    }

    fn player_bar(&self) -> Element<'_, UiMessage> {
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
            .with_volume(72)
            .with_queue(false)
            .view()
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
        PlaylistView::view()
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
        (
            Self {
                catalog: flags,
                player,
                ui: UiState::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Grape".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
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
            _ => {}
        }
        self.ui.update(message);
        self.sync_playback_state();
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        if self.ui.playlist_open {
            return self.playlist_view();
        }

        let content = match self.ui.active_tab {
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
        .height(Length::Fill);

        let layout = column![self.top_bar(), content, self.player_bar()]
            .spacing(16)
            .padding(16)
            .height(Length::Fill);

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(Container::Custom(Box::new(style::SurfaceStyle(
                style::Surface::AppBackground,
            ))))
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
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

        Subscription::batch(subscriptions)
    }
}
