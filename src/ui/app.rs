use crate::config::{
    self, AccentColor, AccessibleTextSize, AudioOutputDevice, AudioStabilityMode, CloseBehavior,
    DeclarativeAction, EqPreset, InterfaceDensity, InterfaceLanguage, MissingDeviceBehavior,
    StartupScreen, SubtitleSize, TextScale, ThemeMode, TimeFormat, UpdateChannel, VolumeLevel,
};
use crate::library::{self, Catalog};
use crate::player::{
    AudioFallback, AudioOptions, NowPlaying, PlaybackState as PlayerPlaybackState, Player,
};
use crate::playlist::{PlaybackQueue, PlaylistManager};
use crate::ui::components::albums_grid::AlbumsGrid;
use crate::ui::components::anchored_overlay::AnchoredOverlay;
use crate::ui::components::artists_panel::ArtistsPanel;
use crate::ui::components::audio_settings::eq_band_controls;
use crate::ui::components::folders_panel::FoldersPanel;
use crate::ui::components::genres_panel::GenresPanel;
use crate::ui::components::player_bar::PlayerBar;
use crate::ui::components::playlist_view::PlaylistView;
use crate::ui::components::queue_view::QueueView;
use crate::ui::components::songs_panel::SongsPanel;
use crate::ui::message::{LibraryNavigation, PlaybackMessage, SearchMessage, UiMessage};
use crate::ui::state::{
    ActiveTab, Album as UiAlbum, Artist as UiArtist, Folder as UiFolder, Genre as UiGenre,
    LibraryFocus, ListLimits, PreferencesSection, PreferencesTab, ScanStage, ScanStatus,
    SearchFilter, SearchState, SelectionState, SortOption, ThemeCategory, Track as UiTrack,
    UiState, progress_ratio,
};
use crate::ui::style;
use iced::font::Weight;
use iced::widget::{
    button, column, container, image, progress_bar, row, scrollable, slider, text, text_input,
};
use iced::{
    Alignment, Color, Element, Length, Padding, Settings, Subscription, Task, Theme, event,
    keyboard, mouse, time, window,
};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tracing::{error, info, warn};
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

const ALBUMS_GRID_COLUMNS: usize = 3;
const ARTIST_FOCUS_ORDER: [LibraryFocus; 3] = [
    LibraryFocus::Artists,
    LibraryFocus::Albums,
    LibraryFocus::Songs,
];
const GENRE_FOCUS_ORDER: [LibraryFocus; 3] = [
    LibraryFocus::Genres,
    LibraryFocus::Albums,
    LibraryFocus::Songs,
];
const FOLDER_FOCUS_ORDER: [LibraryFocus; 2] = [LibraryFocus::Folders, LibraryFocus::Songs];

pub struct GrapeApp {
    catalog: Catalog,
    player: Option<Player>,
    playlists: PlaylistManager,
    playback_queue: PlaybackQueue,
    ui: UiState,
    cover_preloads: Vec<image::Handle>,
}

impl GrapeApp {
    pub fn run(catalog: Catalog, library_root_override: Option<PathBuf>) -> iced::Result {
        let settings = Self::apply_font_settings(Settings::default());
        iced::application(
            move || Self::new(catalog.clone(), library_root_override.clone()),
            Self::update,
            Self::view,
        )
        .settings(settings)
        .title(Self::title)
        .subscription(Self::subscription)
        .theme(Self::theme)
        .run()
    }

    pub fn run_with(
        catalog: Catalog,
        settings: Settings,
        library_root_override: Option<PathBuf>,
    ) -> iced::Result {
        let settings = Self::apply_font_settings(settings);
        iced::application(
            move || Self::new(catalog.clone(), library_root_override.clone()),
            Self::update,
            Self::view,
        )
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
        style::ThemeTokens::from_settings(&self.ui.settings)
    }

    fn normalize_text(value: &str) -> String {
        value
            .nfkd()
            .filter(|character| !is_combining_mark(*character))
            .collect::<String>()
            .to_lowercase()
    }

    fn normalized_contains(query: &str, value: &str) -> bool {
        Self::normalize_text(value).contains(query)
    }

    fn codec_matches(query: &str, codec: Option<&str>) -> bool {
        codec
            .map(|value| Self::normalized_contains(query, value))
            .unwrap_or(false)
    }

    fn apply_limit<T>(items: Vec<T>, limit: usize) -> (Vec<T>, usize) {
        let total = items.len();
        let limited = items.into_iter().take(limit).collect();
        (limited, total)
    }

    fn move_selection<T: Clone>(
        items: &[T],
        current_id: Option<usize>,
        step: isize,
        id_fn: impl Fn(&T) -> usize,
    ) -> Option<T> {
        if items.is_empty() {
            return None;
        }
        let current_index = current_id
            .and_then(|id| items.iter().position(|item| id_fn(item) == id))
            .unwrap_or(0);
        let next_index = if step >= 0 {
            (current_index + step as usize).min(items.len().saturating_sub(1))
        } else {
            current_index.saturating_sub(step.abs() as usize)
        };
        items.get(next_index).cloned()
    }

    fn focus_order(&self) -> &'static [LibraryFocus] {
        match self.ui.active_tab {
            ActiveTab::Artists | ActiveTab::Albums => &ARTIST_FOCUS_ORDER,
            ActiveTab::Genres => &GENRE_FOCUS_ORDER,
            ActiveTab::Folders => &FOLDER_FOCUS_ORDER,
        }
    }

    fn move_library_focus(&mut self, direction: isize) {
        let order = self.focus_order();
        if order.is_empty() {
            return;
        }
        let current_index = order
            .iter()
            .position(|focus| *focus == self.ui.library_focus)
            .unwrap_or(0);
        let next_index = if direction >= 0 {
            (current_index + direction as usize).min(order.len().saturating_sub(1))
        } else {
            current_index.saturating_sub(direction.abs() as usize)
        };
        self.ui.library_focus = order[next_index];
        self.ensure_focus_selection();
    }

    fn ensure_focus_selection(&mut self) {
        match self.ui.library_focus {
            LibraryFocus::Artists => {
                if self.ui.selection.selected_artist.is_none() {
                    if let Some(artist) = self.filtered_artists_from_catalog().into_iter().next() {
                        self.apply_artist_selection(artist);
                    }
                }
            }
            LibraryFocus::Genres => {
                if self.ui.selection.selected_genre.is_none() {
                    if let Some(genre) = self.filtered_genres_from_catalog().into_iter().next() {
                        self.apply_genre_selection(genre);
                    }
                }
            }
            LibraryFocus::Albums => {
                if self.ui.selection.selected_album.is_none() {
                    if let Some(album) = self.filtered_albums_from_catalog().into_iter().next() {
                        self.apply_album_selection(album);
                    }
                }
            }
            LibraryFocus::Folders => {
                if self.ui.selection.selected_folder.is_none() {
                    if let Some(folder) = self.filtered_folders_from_catalog().into_iter().next() {
                        self.apply_folder_selection(folder);
                    }
                }
            }
            LibraryFocus::Songs => {
                if self.ui.selection.selected_track.is_none() {
                    if let Some(track) = self.current_tracks().into_iter().next() {
                        self.apply_track_selection(track);
                    }
                }
            }
        }
    }

    fn handle_library_navigation(&mut self, navigation: LibraryNavigation) {
        match navigation {
            LibraryNavigation::Left | LibraryNavigation::PreviousPanel => {
                self.move_library_focus(-1);
            }
            LibraryNavigation::Right | LibraryNavigation::NextPanel => {
                self.move_library_focus(1);
            }
            LibraryNavigation::Up => {
                self.move_library_selection(-1);
            }
            LibraryNavigation::Down => {
                self.move_library_selection(1);
            }
        }
    }

    fn move_library_selection(&mut self, step: isize) {
        match self.ui.library_focus {
            LibraryFocus::Artists => {
                let artists = self.filtered_artists_from_catalog();
                let current = self
                    .ui
                    .selection
                    .selected_artist
                    .as_ref()
                    .map(|artist| artist.id);
                if let Some(artist) =
                    Self::move_selection(&artists, current, step, |artist| artist.id)
                {
                    self.apply_artist_selection(artist);
                }
            }
            LibraryFocus::Genres => {
                let genres = self.filtered_genres_from_catalog();
                let current = self
                    .ui
                    .selection
                    .selected_genre
                    .as_ref()
                    .map(|genre| genre.id);
                if let Some(genre) = Self::move_selection(&genres, current, step, |genre| genre.id)
                {
                    self.apply_genre_selection(genre);
                }
            }
            LibraryFocus::Albums => {
                let albums = self.filtered_albums_from_catalog();
                let current = self
                    .ui
                    .selection
                    .selected_album
                    .as_ref()
                    .map(|album| album.id);
                let adjusted_step = step * ALBUMS_GRID_COLUMNS as isize;
                if let Some(album) =
                    Self::move_selection(&albums, current, adjusted_step, |album| album.id)
                {
                    self.apply_album_selection(album);
                }
            }
            LibraryFocus::Folders => {
                let folders = self.filtered_folders_from_catalog();
                let current = self
                    .ui
                    .selection
                    .selected_folder
                    .as_ref()
                    .map(|folder| folder.id);
                if let Some(folder) =
                    Self::move_selection(&folders, current, step, |folder| folder.id)
                {
                    self.apply_folder_selection(folder);
                }
            }
            LibraryFocus::Songs => {
                let tracks = self.current_tracks();
                let current = self
                    .ui
                    .selection
                    .selected_track
                    .as_ref()
                    .map(|track| track.id);
                if let Some(track) = Self::move_selection(&tracks, current, step, |track| track.id)
                {
                    self.apply_track_selection(track);
                }
            }
        }
    }

    fn apply_artist_selection(&mut self, artist: UiArtist) {
        let artist_name = artist.name.clone();
        self.ui.selection.selected_artist = Some(artist);
        self.ui.selection.selected_album = None;
        self.ui.selection.selected_genre = None;
        self.ui.selection.selected_folder = None;
        self.ui.selection.selected_track = None;
        self.ui.library_focus = LibraryFocus::Artists;
        if let Some(album) = self
            .filtered_albums_from_catalog()
            .into_iter()
            .find(|album| album.artist == artist_name)
        {
            self.ui.selection.selected_album = Some(album.clone());
            self.ui.selection.selected_track =
                self.album_entry_by_id(album.id)
                    .and_then(|(artist, entry)| {
                        self.filtered_tracks_for_album(artist, entry)
                            .into_iter()
                            .next()
                    });
        }
        self.refresh_album_metadata_drafts();
    }

    fn apply_album_selection(&mut self, album: UiAlbum) {
        let album_id = album.id;
        self.ui.selection.selected_album = Some(album);
        self.ui.selection.selected_folder = None;
        self.ui.selection.selected_track =
            self.album_entry_by_id(album_id)
                .and_then(|(artist, entry)| {
                    self.filtered_tracks_for_album(artist, entry)
                        .into_iter()
                        .next()
                });
        self.ui.library_focus = LibraryFocus::Songs;
        self.refresh_album_metadata_drafts();
    }

    fn apply_genre_selection(&mut self, genre: UiGenre) {
        self.ui.selection.selected_genre = Some(genre);
        self.ui.selection.selected_folder = None;
        self.ui.selection.selected_track = None;
        self.ui.library_focus = LibraryFocus::Genres;
    }

    fn apply_folder_selection(&mut self, folder: UiFolder) {
        self.ui.selection.selected_folder = Some(folder);
        self.ui.selection.selected_genre = None;
        self.ui.selection.selected_album = None;
        self.ui.library_focus = LibraryFocus::Songs;
        self.ui.selection.selected_track = self
            .ui
            .selection
            .selected_folder
            .as_ref()
            .and_then(|folder| {
                self.folder_entry_by_id(folder.id)
                    .map(|(artist, entry)| self.filtered_tracks_for_album(artist, entry))
            })
            .and_then(|tracks| tracks.into_iter().next());
        self.refresh_album_metadata_drafts();
    }

    fn apply_track_selection(&mut self, track: UiTrack) {
        self.ui.selection.selected_track = Some(track);
        self.ui.library_focus = LibraryFocus::Songs;
    }

    fn activate_selection(&mut self) {
        if let Some(track) = self.ui.selection.selected_track.clone() {
            self.handle_track_selection(&track);
            return;
        }
        if let Some(album) = self.ui.selection.selected_album.clone() {
            if let Some((artist, entry)) = self.album_entry_by_id(album.id) {
                if let Some(track) = self
                    .filtered_tracks_for_album(artist, entry)
                    .into_iter()
                    .next()
                {
                    self.handle_track_selection(&track);
                    self.apply_track_selection(track);
                }
            }
        }
    }

    fn current_tracks(&self) -> Vec<UiTrack> {
        self.ui
            .selection
            .selected_album
            .as_ref()
            .and_then(|album| {
                self.album_entry_by_id(album.id)
                    .map(|(artist, entry)| (artist, entry))
            })
            .or_else(|| {
                self.ui
                    .selection
                    .selected_folder
                    .as_ref()
                    .and_then(|folder| {
                        self.folder_entry_by_id(folder.id)
                            .map(|(artist, entry)| (artist, entry))
                    })
            })
            .map(|(artist, album)| self.filtered_tracks_for_album(artist, album))
            .unwrap_or_default()
    }

    fn year_matches(query: &str, year: Option<u32>) -> bool {
        year.map(|year| year.to_string().contains(query))
            .unwrap_or(false)
    }

    fn duration_matches(query: &str, duration: Duration) -> bool {
        let total_seconds = duration.as_secs();
        let total_minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;
        let mmss = format!("{minutes:02}:{seconds:02}");
        let hmmss = format!("{hours}:{minutes:02}:{seconds:02}");
        [
            total_seconds.to_string(),
            total_minutes.to_string(),
            mmss,
            hmmss,
        ]
        .iter()
        .any(|value| value.contains(query))
    }

    fn normalized_query(&self) -> Option<String> {
        let query = self.ui.search.query.trim();
        if query.is_empty() {
            None
        } else {
            Some(Self::normalize_text(query))
        }
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
                    total_duration: Duration::from_secs(album.total_duration_secs as u64),
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
            let filters = self.ui.search.filters;
            albums.retain(|album| {
                let mut matches = Self::normalized_contains(&query, &album.title)
                    || Self::normalized_contains(&query, &album.artist);
                if filters.genre {
                    matches |= self
                        .album_entry_by_id(album.id)
                        .and_then(|(_, entry)| entry.genre.as_deref())
                        .map(|genre| Self::normalized_contains(&query, genre))
                        .unwrap_or(false);
                }
                if filters.year {
                    matches |= Self::year_matches(&query, album.year);
                }
                if filters.duration {
                    matches |= Self::duration_matches(&query, album.total_duration);
                }
                if filters.codec {
                    matches |= self
                        .album_entry_by_id(album.id)
                        .map(|(_, entry)| {
                            entry
                                .tracks
                                .iter()
                                .any(|track| Self::codec_matches(&query, track.codec.as_deref()))
                        })
                        .unwrap_or(false);
                }
                matches
            });
        }
        if self.ui.active_tab == ActiveTab::Genres {
            if let Some(selected_genre) = self.ui.selection.selected_genre.as_ref() {
                let normalized_genre = Self::normalize_text(selected_genre.name.trim());
                albums.retain(|album| {
                    self.album_entry_by_id(album.id)
                        .and_then(|(_, entry)| entry.genre.as_deref())
                        .map(|genre| Self::normalize_text(genre.trim()) == normalized_genre)
                        .unwrap_or(false)
                });
            }
        }
        match self.ui.search.sort {
            SortOption::Alphabetical => {
                albums.sort_by(|a, b| {
                    Self::normalize_text(&a.title)
                        .cmp(&Self::normalize_text(&b.title))
                        .then_with(|| {
                            Self::normalize_text(&a.artist).cmp(&Self::normalize_text(&b.artist))
                        })
                });
            }
            SortOption::ByAlbum => {
                albums.sort_by(|a, b| {
                    Self::normalize_text(&a.artist)
                        .cmp(&Self::normalize_text(&b.artist))
                        .then_with(|| {
                            Self::normalize_text(&a.title).cmp(&Self::normalize_text(&b.title))
                        })
                        .then_with(|| a.year.cmp(&b.year))
                });
            }
            SortOption::ByYear => {
                albums.sort_by(|a, b| {
                    let year_a = a.year.unwrap_or(u32::MAX);
                    let year_b = b.year.unwrap_or(u32::MAX);
                    year_a
                        .cmp(&year_b)
                        .then_with(|| {
                            Self::normalize_text(&a.title).cmp(&Self::normalize_text(&b.title))
                        })
                        .then_with(|| {
                            Self::normalize_text(&a.artist).cmp(&Self::normalize_text(&b.artist))
                        })
                });
            }
            SortOption::ByDuration => {
                albums.sort_by(|a, b| {
                    a.total_duration
                        .cmp(&b.total_duration)
                        .then_with(|| {
                            Self::normalize_text(&a.title).cmp(&Self::normalize_text(&b.title))
                        })
                        .then_with(|| {
                            Self::normalize_text(&a.artist).cmp(&Self::normalize_text(&b.artist))
                        })
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
            artists.retain(|artist| Self::normalized_contains(&query, &artist.name));
        }
        artists.sort_by(|a, b| Self::normalize_text(&a.name).cmp(&Self::normalize_text(&b.name)));
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
            genres.retain(|genre| Self::normalized_contains(&query, &genre.name));
        }
        genres.sort_by(|a, b| Self::normalize_text(&a.name).cmp(&Self::normalize_text(&b.name)));
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

    fn album_entry_by_id_mut(
        &mut self,
        album_id: usize,
    ) -> Option<&mut crate::library::Album> {
        let mut id = 0usize;
        for artist in &mut self.catalog.artists {
            for album in &mut artist.albums {
                if id == album_id {
                    return Some(album);
                }
                id += 1;
            }
        }
        None
    }

    fn refresh_album_metadata_drafts(&mut self) {
        let draft = self
            .ui
            .selection
            .selected_album
            .as_ref()
            .and_then(|selected| self.album_entry_by_id(selected.id))
            .map(|(_, album)| {
                (
                    album.genre.clone().unwrap_or_default(),
                    if album.year > 0 {
                        album.year.to_string()
                    } else {
                        String::new()
                    },
                )
            })
            .unwrap_or_else(|| (String::new(), String::new()));
        self.ui.album_genre_draft = draft.0;
        self.ui.album_year_draft = draft.1;
    }

    fn folder_entry_by_id(
        &self,
        folder_id: usize,
    ) -> Option<(&crate::library::Artist, &crate::library::Album)> {
        let mut id = 0usize;
        for artist in &self.catalog.artists {
            for album in &artist.albums {
                if album.tracks.is_empty() {
                    continue;
                }
                if id == folder_id {
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
                artist: track
                    .artist
                    .as_deref()
                    .unwrap_or(&artist.name)
                    .to_string(),
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
        let album_year = if album.year == 0 {
            None
        } else {
            Some(album.year as u32)
        };
        if let Some(query) = self.normalized_query() {
            let filters = self.ui.search.filters;
            tracks.retain(|track| {
                let mut matches = Self::normalized_contains(&query, &track.title)
                    || Self::normalized_contains(&query, &track.artist)
                    || Self::normalized_contains(&query, &track.album);
                if filters.genre {
                    matches |= album
                        .genre
                        .as_deref()
                        .map(|genre| Self::normalized_contains(&query, genre))
                        .unwrap_or(false);
                }
                if filters.year {
                    matches |= Self::year_matches(&query, album_year);
                }
                if filters.duration {
                    matches |= Self::duration_matches(&query, track.duration);
                }
                if filters.codec {
                    matches |= album
                        .tracks
                        .get(track.id)
                        .map(|entry| Self::codec_matches(&query, entry.codec.as_deref()))
                        .unwrap_or(false);
                }
                matches
            });
        }
        match self.ui.search.sort {
            SortOption::Alphabetical => {
                tracks.sort_by(|a, b| {
                    Self::normalize_text(&a.title)
                        .cmp(&Self::normalize_text(&b.title))
                        .then_with(|| a.track_number.cmp(&b.track_number))
                });
            }
            SortOption::ByAlbum => {
                tracks.sort_by(|a, b| {
                    a.track_number
                        .unwrap_or(u32::MAX)
                        .cmp(&b.track_number.unwrap_or(u32::MAX))
                        .then_with(|| {
                            Self::normalize_text(&a.title).cmp(&Self::normalize_text(&b.title))
                        })
                });
            }
            SortOption::ByYear => {
                let year_key = album_year.unwrap_or(u32::MAX);
                tracks.sort_by(|a, b| {
                    year_key
                        .cmp(&year_key)
                        .then_with(|| {
                            a.track_number
                                .unwrap_or(u32::MAX)
                                .cmp(&b.track_number.unwrap_or(u32::MAX))
                        })
                        .then_with(|| {
                            Self::normalize_text(&a.title).cmp(&Self::normalize_text(&b.title))
                        })
                });
            }
            SortOption::ByDuration => {
                tracks.sort_by(|a, b| {
                    a.duration
                        .cmp(&b.duration)
                        .then_with(|| {
                            Self::normalize_text(&a.title).cmp(&Self::normalize_text(&b.title))
                        })
                        .then_with(|| a.track_number.cmp(&b.track_number))
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
            let filters = self.ui.search.filters;
            folders.retain(|folder| {
                let mut matches = Self::normalized_contains(&query, &folder.name);
                if filters.genre || filters.year || filters.duration || filters.codec {
                    let Some((_, entry)) = self.folder_entry_by_id(folder.id) else {
                        return matches;
                    };
                    if filters.genre {
                        matches |= entry
                            .genre
                            .as_deref()
                            .map(|genre| Self::normalized_contains(&query, genre))
                            .unwrap_or(false);
                    }
                    if filters.year {
                        let year = if entry.year == 0 {
                            None
                        } else {
                            Some(entry.year as u32)
                        };
                        matches |= Self::year_matches(&query, year);
                    }
                    if filters.duration {
                        let duration = Duration::from_secs(entry.total_duration_secs as u64);
                        matches |= Self::duration_matches(&query, duration);
                    }
                    if filters.codec {
                        matches |= entry
                            .tracks
                            .iter()
                            .any(|track| Self::codec_matches(&query, track.codec.as_deref()));
                    }
                }
                matches
            });
        }
        folders.sort_by(|a, b| Self::normalize_text(&a.name).cmp(&Self::normalize_text(&b.name)));
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
                    style::ButtonKind::ListItem {
                        selected: false,
                        focused: false,
                    },
                    status,
                )
            })
            .padding([4, 8])
            .on_press(message)
        };
        let menu_toggle = |label: &'static str, enabled: bool, filter: SearchFilter| {
            let label = if enabled {
                format!("{label} ✓")
            } else {
                label.to_string()
            };
            button(
                text(label)
                    .size(theme.size(12))
                    .font(style::font_propo(Weight::Medium))
                    .style(move |_| style::text_style_primary(theme)),
            )
            .style(move |_, status| {
                style::button_style(
                    theme,
                    style::ButtonKind::ListItem {
                        selected: enabled,
                        focused: false,
                    },
                    status,
                )
            })
            .padding([4, 8])
            .on_press(UiMessage::Search(SearchMessage::ToggleFilter(filter)))
        };
        let logo_menu = container(
            column![
                menu_button("Bibliothèque", UiMessage::ShowLibrary),
                menu_button("Playlist", UiMessage::OpenPlaylist),
                menu_button("Queue", UiMessage::OpenQueue),
                menu_button("Préférences", UiMessage::OpenPreferences),
                text("Filtres")
                    .size(theme.size(11))
                    .font(style::font_propo(Weight::Light))
                    .style(move |_| style::text_style_muted(theme)),
                menu_toggle("Genre", self.ui.search.filters.genre, SearchFilter::Genre),
                menu_toggle("Année", self.ui.search.filters.year, SearchFilter::Year),
                menu_toggle(
                    "Durée",
                    self.ui.search.filters.duration,
                    SearchFilter::Duration,
                ),
                menu_toggle("Codec", self.ui.search.filters.codec, SearchFilter::Codec),
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

    fn scan_banner(&self) -> Option<Element<'_, UiMessage>> {
        let status = self.ui.scan_status.as_ref()?;
        let theme = self.theme_tokens();
        let stage_label = match status.stage {
            ScanStage::Indexing => "Indexation en cours",
        };
        let progress =
            container(progress_bar(0.0..=1.0, status.progress)).height(Length::Fixed(6.0));
        let content = column![
            text(stage_label)
                .size(theme.size(14))
                .font(style::font_propo(Weight::Semibold))
                .style(move |_| style::text_style_primary(theme)),
            text(format!("Dossier : {}", status.root.display()))
                .size(theme.size_accessible(12))
                .font(style::font_propo(Weight::Light))
                .style(move |_| style::text_style_muted(theme)),
            progress
        ]
        .spacing(6)
        .width(Length::Fill);
        Some(
            container(content)
                .padding(12)
                .width(Length::Fill)
                .style(move |_| style::surface_style(theme, style::Surface::Panel))
                .into(),
        )
    }

    fn artists_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let selected_id = self
            .ui
            .selection
            .selected_artist
            .as_ref()
            .map(|artist| artist.id);
        let (artists, total) = Self::apply_limit(
            self.filtered_artists_from_catalog(),
            self.ui.list_limits.artists,
        );
        let load_more = (total > artists.len()).then_some(UiMessage::LoadMoreArtists);
        let panel = ArtistsPanel::new(artists, total)
            .with_selection(selected_id)
            .with_load_more(load_more);
        panel.view(
            &self.ui.selection,
            self.ui.library_focus == crate::ui::state::LibraryFocus::Artists,
            theme,
        )
    }

    fn albums_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let sort_label = match self.ui.search.sort {
            SortOption::Alphabetical => "A–Z",
            SortOption::ByAlbum => "By album",
            SortOption::ByYear => "By year",
            SortOption::ByDuration => "By duration",
        };
        let selected_id = self
            .ui
            .selection
            .selected_album
            .as_ref()
            .map(|album| album.id);
        let (albums, total) = Self::apply_limit(
            self.filtered_albums_from_catalog(),
            self.ui.list_limits.albums,
        );
        let load_more = (total > albums.len()).then_some(UiMessage::LoadMoreAlbums);
        let grid = AlbumsGrid::new(albums, total)
            .with_sort_label(sort_label)
            .with_selection(selected_id)
            .with_load_more(load_more)
            .view(
                self.ui.library_focus == crate::ui::state::LibraryFocus::Albums,
                theme,
            );

        container(grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn songs_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let selected_album = self
            .ui
            .selection
            .selected_album
            .as_ref()
            .and_then(|album| {
                self.album_entry_by_id(album.id)
                    .map(|(artist, entry)| (artist, entry))
            })
            .or_else(|| {
                self.ui
                    .selection
                    .selected_folder
                    .as_ref()
                    .and_then(|folder| {
                        self.folder_entry_by_id(folder.id)
                            .map(|(artist, entry)| (artist, entry))
                    })
            });
        let (album_title, artist_name, tracks, show_metadata_editor) = match selected_album {
            Some((artist, album)) => (
                album.title.clone(),
                artist.name.clone(),
                self.filtered_tracks_for_album(artist, album),
                true,
            ),
            None => (
                "Select an album".to_string(),
                "Pick a track to start".to_string(),
                Vec::new(),
                false,
            ),
        };
        let (tracks, total) = Self::apply_limit(tracks, self.ui.list_limits.tracks);
        let load_more = (total > tracks.len()).then_some(UiMessage::LoadMoreTracks);
        let selected_id = self
            .ui
            .selection
            .selected_track
            .as_ref()
            .map(|track| track.id);
        let panel = SongsPanel::new(album_title, artist_name, tracks, total)
            .with_selection(selected_id)
            .with_load_more(load_more)
            .with_metadata_editor(
                self.ui.album_genre_draft.clone(),
                self.ui.album_year_draft.clone(),
                show_metadata_editor,
            );
        panel.view(
            &self.ui.selection,
            self.ui.library_focus == crate::ui::state::LibraryFocus::Songs,
            theme,
        )
    }

    fn genres_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let selected_id = self
            .ui
            .selection
            .selected_genre
            .as_ref()
            .map(|genre| genre.id);
        let (genres, total) = Self::apply_limit(
            self.filtered_genres_from_catalog(),
            self.ui.list_limits.genres,
        );
        let load_more = (total > genres.len()).then_some(UiMessage::LoadMoreGenres);
        let panel = GenresPanel::new(genres, total)
            .with_selection(selected_id)
            .with_load_more(load_more);
        panel.view(
            self.ui.library_focus == crate::ui::state::LibraryFocus::Genres,
            theme,
        )
    }

    fn folders_panel(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        let sort_label = match self.ui.search.sort {
            SortOption::Alphabetical => "A–Z",
            SortOption::ByAlbum => "By album",
            SortOption::ByYear => "By year",
            SortOption::ByDuration => "By duration",
        };
        let selected_id = self
            .ui
            .selection
            .selected_folder
            .as_ref()
            .map(|folder| folder.id);
        let (folders, total) = Self::apply_limit(
            self.filtered_folders_from_catalog(),
            self.ui.list_limits.folders,
        );
        let load_more = (total > folders.len()).then_some(UiMessage::LoadMoreFolders);
        FoldersPanel::new(folders, total)
            .with_sort_label(sort_label)
            .with_selection(selected_id)
            .with_load_more(load_more)
            .view(
                self.ui.library_focus == crate::ui::state::LibraryFocus::Folders,
                theme,
            )
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

        let queue_message = if self.ui.queue_open {
            Some(UiMessage::CloseQueue)
        } else {
            Some(UiMessage::OpenQueue)
        };
        PlayerBar::new(title, artist)
            .with_cover(cover_path)
            .with_playback(self.ui.playback)
            .with_volume(self.ui.settings.default_volume)
            .with_queue(self.ui.queue_open)
            .with_queue_action(queue_message)
            .with_inline_volume_bar(self.ui.inline_volume_bar_open)
            .with_inline_volume_toggle(Some(UiMessage::ToggleInlineVolumeBar))
            .view(theme)
    }

    fn handle_track_selection(&mut self, track: &UiTrack) {
        let now_playing = Self::now_playing_from_ui_track(track);
        self.playlist_add(now_playing.clone());
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

    fn playback_queue_from_playlist(playlists: &PlaylistManager) -> PlaybackQueue {
        let mut queue = PlaybackQueue::default();
        let items = playlists
            .active()
            .map(|playlist| playlist.items.clone())
            .unwrap_or_default();
        queue.set_queue(items);
        queue
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
                            artist: track
                                .artist
                                .as_deref()
                                .unwrap_or(&artist.name)
                                .to_string(),
                            track_number: Some(track.number as u32),
                            duration: Duration::from_secs(track.duration_secs as u64),
                            path: track.path.clone(),
                            cover_path: album.cover.as_ref().map(|cover| cover.cached_path.clone()),
                        };
                    }
                }
            }
        }
        let normalized_title = Self::normalize_text(&now_playing.title);
        if !normalized_title.is_empty() {
            let normalized_album = Self::normalize_text(&now_playing.album);
            let normalized_artist = Self::normalize_text(&now_playing.artist);
            for artist in &self.catalog.artists {
                let artist_match = normalized_artist.is_empty()
                    || Self::normalize_text(&artist.name) == normalized_artist;
                if !artist_match {
                    continue;
                }
                for album in &artist.albums {
                    let album_match = normalized_album.is_empty()
                        || Self::normalize_text(&album.title) == normalized_album;
                    if !album_match {
                        continue;
                    }
                    for (id, track) in album.tracks.iter().enumerate() {
                        if Self::normalize_text(&track.title) == normalized_title {
                            return UiTrack {
                                id,
                                title: track.title.clone(),
                                album: album.title.clone(),
                                artist: track
                                    .artist
                                    .as_deref()
                                    .unwrap_or(&artist.name)
                                    .to_string(),
                                track_number: Some(track.number as u32),
                                duration: Duration::from_secs(track.duration_secs as u64),
                                path: track.path.clone(),
                                cover_path: album
                                    .cover
                                    .as_ref()
                                    .map(|cover| cover.cached_path.clone()),
                            };
                        }
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

    fn playlist_add(&mut self, now_playing: NowPlaying) {
        self.playlists.add(now_playing);
        let preferred_index = self
            .playlists
            .active()
            .map(|playlist| playlist.items.len().saturating_sub(1));
        self.refresh_playback_queue(preferred_index);
        self.persist_playlist();
    }

    #[allow(dead_code)]
    fn playlist_remove(&mut self, index: usize) -> Option<NowPlaying> {
        let removed = self.playlists.delete_item(index);
        if removed.is_some() {
            self.refresh_playback_queue(None);
            self.persist_playlist();
        }
        removed
    }

    #[allow(dead_code)]
    fn playlist_reorder(&mut self, from: usize, to: usize) -> bool {
        let changed = self.playlists.move_item(from, to);
        if changed {
            self.refresh_playback_queue(None);
            self.persist_playlist();
        }
        changed
    }

    #[allow(dead_code)]
    fn playlist_clear(&mut self) {
        self.playlists.clear();
        self.refresh_playback_queue(None);
        self.persist_playlist();
    }

    fn playlist_save_order(&mut self) {
        self.refresh_playback_queue(None);
        self.persist_playlist();
    }

    fn refresh_playback_queue(&mut self, preferred_index: Option<usize>) {
        let items = self
            .playlists
            .active()
            .map(|playlist| playlist.items.clone())
            .unwrap_or_default();
        let index = match preferred_index {
            Some(index) => index.min(items.len().saturating_sub(1)),
            None => {
                let current = self.playback_queue.current();
                current
                    .as_ref()
                    .and_then(|now_playing| {
                        items.iter().position(|item| item.path == now_playing.path)
                    })
                    .unwrap_or(0)
            }
        };
        self.playback_queue.set_queue(items);
        self.playback_queue.set_index(index);
    }

    fn persist_playlist(&self) {
        if let Err(err) = self.playlists.save() {
            warn!(error = %err, "Failed to persist playlist");
        }
    }

    fn refresh_cover_preloads(&mut self) {
        let mut handles = Vec::new();
        let albums = self.filtered_albums_from_catalog();
        for album in albums.into_iter().take(self.ui.list_limits.albums) {
            if let Some(path) = album.cover_path {
                handles.push(image::Handle::from_path(path));
            }
        }
        let tracks = self.current_tracks();
        for track in tracks.into_iter().take(self.ui.list_limits.tracks) {
            if let Some(path) = track.cover_path {
                handles.push(image::Handle::from_path(path));
            }
        }
        self.cover_preloads = handles;
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
                if !self.ui.play_from_queue {
                    return;
                }
                let next_track = self.playback_queue.next();
                self.load_from_queue(next_track);
            }
            PlaybackMessage::PreviousTrack => {
                if !self.ui.play_from_queue {
                    return;
                }
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

    fn library_root(&self) -> Option<PathBuf> {
        let root = self.ui.settings.library_folder.trim();
        if root.is_empty() {
            warn!("Library folder is not set; skipping library scan");
            return None;
        }
        Some(PathBuf::from(root))
    }

    fn begin_scan(&mut self, root: PathBuf, use_cache: bool) -> Task<UiMessage> {
        if self.ui.scan_status.is_some() {
            return Task::none();
        }
        let settings = self.ui.settings.clone();
        self.ui.scan_status = Some(ScanStatus::new(root.clone()));
        Task::perform(
            async move {
                let scan_result = if use_cache {
                    library::scan_library(&root, &settings)
                } else {
                    library::scan_library_full(&root, &settings)
                };
                scan_result.map_err(|err| err.to_string())
            },
            UiMessage::LibraryScanCompleted,
        )
    }

    fn begin_scan_from_settings(&mut self, use_cache: bool) -> Task<UiMessage> {
        let Some(root) = self.library_root() else {
            return Task::none();
        };
        self.begin_scan(root, use_cache)
    }

    fn reset_audio_engine(&mut self) {
        info!("Resetting audio engine");
        let options = AudioOptions::from_settings(&self.ui.settings);
        match self.player.as_mut() {
            Some(player) => {
                if let Err(err) = player.reset(options) {
                    error!(error = %err, "Failed to reset audio player");
                    self.player = Player::new_with_settings(&config::UserSettings::default()).ok();
                }
            }
            None => {
                self.player = match Player::new_with_settings(&self.ui.settings) {
                    Ok(player) => Some(player),
                    Err(err) => {
                        error!(error = %err, "Failed to reinitialize audio player");
                        None
                    }
                };
            }
        }
        self.ui.playback = crate::ui::state::PlaybackState::default();
        self.ui.selection = SelectionState::default();
        self.playback_queue = Self::playback_queue_from_playlist(&self.playlists);
    }

    fn open_logs_folder(&self) {
        let path = match config::ensure_logs_dir() {
            Ok(path) => path,
            Err(err) => {
                error!(error = %err, "Failed to ensure logs directory");
                return;
            }
        };
        if let Err(err) = Self::open_path_in_shell(&path) {
            error!(error = %err, path = %path.display(), "Failed to open logs folder");
        } else {
            info!(path = %path.display(), "Opened logs folder");
        }
    }

    fn open_path_in_shell(path: &Path) -> io::Result<()> {
        #[cfg(target_os = "windows")]
        let mut command = {
            let mut command = Command::new("cmd");
            command.args(["/C", "start", ""]);
            command.arg(path);
            command
        };

        #[cfg(target_os = "macos")]
        let mut command = {
            let mut command = Command::new("open");
            command.arg(path);
            command
        };

        #[cfg(all(unix, not(target_os = "macos")))]
        let mut command = {
            let mut command = Command::new("xdg-open");
            command.arg(path);
            command
        };

        let status = command.status()?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("open command failed with status {status}"),
            ))
        }
    }

    fn apply_audio_settings(&mut self) {
        let Some(player) = &mut self.player else {
            return;
        };
        if let Err(err) = player.apply_settings(&self.ui.settings) {
            error!(error = %err, "Failed to apply audio settings");
            return;
        }
        if let Some(fallback) = player.take_last_fallback_notice() {
            self.handle_audio_fallback(fallback);
        }
    }

    fn handle_audio_fallback(&mut self, fallback: AudioFallback) {
        self.ui.audio_notice = Some(fallback.notice());
        Self::apply_audio_fallback_to_settings(&mut self.ui.settings, &fallback);
        if matches!(fallback.behavior, MissingDeviceBehavior::PausePlayback) {
            if let Some(player) = &mut self.player {
                player.pause();
            }
        }
        if let Err(err) = config::save_settings(&self.ui.settings) {
            error!(error = %err, "Failed to persist audio fallback settings");
        }
    }

    fn apply_audio_fallback_to_settings(
        settings: &mut config::UserSettings,
        fallback: &AudioFallback,
    ) {
        let _ = fallback;
        settings.output_device = AudioOutputDevice::System;
        settings.output_sample_rate_hz = None;
    }

    fn handle_declarative_action(&mut self, action: DeclarativeAction) -> Task<UiMessage> {
        match action {
            DeclarativeAction::ReindexLibrary => {
                info!("Library reindex requested");
                return self.begin_scan_from_settings(true);
            }
            DeclarativeAction::ClearCache => {
                info!("Cache clear requested");
                if let Some(root) = self.library_root() {
                    let cache_path = config::library_cache_dir(&self.ui.settings, &root);
                    match config::clear_library_cache(&self.ui.settings, &root) {
                        Ok(()) => {
                            info!(path = %cache_path.display(), "Library cache cleared");
                            return self.begin_scan(root, true);
                        }
                        Err(err) => {
                            error!(error = %err, path = %cache_path.display(), "Failed to clear cache");
                        }
                    }
                }
            }
            DeclarativeAction::ResetAudioEngine => {
                info!("Audio engine reset requested");
                self.reset_audio_engine();
            }
        }
        Task::none()
    }

    fn handle_album_metadata_save(&mut self) {
        let Some(selected_album) = self.ui.selection.selected_album.as_ref() else {
            return;
        };
        let Some(root) = self.library_root() else {
            return;
        };
        let Some((artist, album)) = self.album_entry_by_id(selected_album.id) else {
            return;
        };
        let genre_value = self.ui.album_genre_draft.trim();
        let genre = if genre_value.is_empty() {
            None
        } else {
            Some(genre_value.to_string())
        };
        let year_value = self.ui.album_year_draft.trim();
        let year = if year_value.is_empty() {
            None
        } else {
            match year_value.parse::<u16>() {
                Ok(value) if value > 0 => Some(value),
                Ok(_) => None,
                Err(error) => {
                    warn!(error = %error, "Invalid album year input");
                    return;
                }
            }
        };
        if let Err(error) = library::persist_album_metadata_override(
            &root,
            &artist.name,
            &album.title,
            genre.clone(),
            year,
        ) {
            warn!(error = %error, "Failed to persist album metadata override");
            return;
        }
        if let Some(album) = self.album_entry_by_id_mut(selected_album.id) {
            album.genre = genre.clone();
            album.year = year.unwrap_or(0);
        }
        if let Some(selected_album) = self.ui.selection.selected_album.as_mut() {
            selected_album.year = year.map(|value| value as u32);
        }
        self.refresh_album_metadata_drafts();
    }

    fn playlist_view(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        PlaylistView::view(theme, &self.playlists, &self.ui.selection)
    }

    fn queue_view(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        QueueView::view(theme, &self.playback_queue, self.ui.play_from_queue)
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
                    style::ButtonKind::ListItem {
                        selected: false,
                        focused: false,
                    },
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
                        focused: false,
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
                    style::ButtonKind::ListItem {
                        selected: expanded,
                        focused: false,
                    },
                    status,
                )
            })
            .padding([8, 12])
            .width(Length::Fill)
            .on_press(message)
        };
        let section_hint = |label: &'static str| {
            text(label)
                .size(theme.size_accessible(12))
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
                    .size(theme.size_accessible(12))
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
        let action_controls = |action: DeclarativeAction| -> Element<'_, UiMessage> {
            if self.ui.pending_action == Some(action) {
                row![
                    action_button(
                        action.confirm_label(),
                        UiMessage::ConfirmDeclarativeAction(action),
                    ),
                    action_button("Annuler", UiMessage::CancelDeclarativeAction),
                ]
                .spacing(8)
                .into()
            } else {
                action_button(
                    action.button_label(),
                    UiMessage::RequestDeclarativeAction(action),
                )
                .into()
            }
        };

        let reindex_action = DeclarativeAction::ReindexLibrary;
        let clear_cache_action = DeclarativeAction::ClearCache;
        let reset_audio_action = DeclarativeAction::ResetAudioEngine;

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
                            action_button("Ajouter un dossier", UiMessage::PickLibraryFolder),
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
                    setting_label(clear_cache_action.title(), clear_cache_action.description()),
                    controls(
                        row![
                            cache_input.width(Length::Fill),
                            action_controls(clear_cache_action),
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
                    setting_label(reindex_action.title(), reindex_action.description()),
                    controls(action_controls(reindex_action)),
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
                    setting_label(
                        "Police large",
                        "Active un mode de lecture avec textes agrandis."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.accessibility_large_text,
                            UiMessage::SetAccessibilityLargeText(true),
                            UiMessage::SetAccessibilityLargeText(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
                row![
                    setting_label("Contraste élevé", "Renforce les contrastes UI."),
                    controls(
                        toggle_row(
                            self.ui.settings.accessibility_high_contrast,
                            UiMessage::SetAccessibilityHighContrast(true),
                            UiMessage::SetAccessibilityHighContrast(false),
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
                        "Réduire les mouvements",
                        "Désactive les transitions et animations non essentielles."
                    ),
                    controls(
                        toggle_row(
                            self.ui.settings.accessibility_reduce_motion,
                            UiMessage::SetAccessibilityReduceMotion(true),
                            UiMessage::SetAccessibilityReduceMotion(false),
                        )
                        .into()
                    ),
                ]
                .align_y(Alignment::Center)
                .spacing(12),
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
            let theme_category = |label: &'static str,
                                  expanded: bool,
                                  message: UiMessage,
                                  options: Element<'static, UiMessage>|
             -> Element<'static, UiMessage> {
                let chevron = if expanded { "▾" } else { "▸" };
                let expanded_content: Element<'static, UiMessage> = if expanded {
                    container(
                        row![
                            text("↳")
                                .size(theme.size(12))
                                .font(style::font_propo(Weight::Light))
                                .style(move |_| style::text_style_muted(theme)),
                            options,
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .padding(Padding {
                        top: 0.0,
                        right: 0.0,
                        bottom: 0.0,
                        left: 24.0,
                    })
                    .width(Length::Fill)
                    .into()
                } else {
                    column![].into()
                };
                column![
                    button(
                        row![
                            text(label)
                                .size(theme.size(13))
                                .font(style::font_propo(Weight::Medium))
                                .style(move |_| style::text_style_primary(theme)),
                            text(chevron)
                                .size(theme.size(13))
                                .font(style::font_propo(Weight::Medium))
                                .style(move |_| style::text_style_muted(theme)),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center),
                    )
                    .style(move |_, status| {
                        style::button_style(
                            theme,
                            style::ButtonKind::ListItem {
                                selected: expanded,
                                focused: false,
                            },
                            status,
                        )
                    })
                    .padding([8, 12])
                    .width(Length::Fill)
                    .on_press(message),
                    expanded_content,
                ]
                .spacing(6)
                .into()
            };

            column![
                row![
                    theme_category(
                        "Catppuccin",
                        self.ui.theme_categories.catppuccin,
                        UiMessage::ToggleThemeCategory(ThemeCategory::Catppuccin),
                        row![
                            option_button(
                                self.ui.settings.theme_mode == ThemeMode::Latte,
                                ThemeMode::Latte.label(),
                                UiMessage::SetThemeMode(ThemeMode::Latte),
                            ),
                            option_button(
                                self.ui.settings.theme_mode == ThemeMode::Frappe,
                                ThemeMode::Frappe.label(),
                                UiMessage::SetThemeMode(ThemeMode::Frappe),
                            ),
                            option_button(
                                self.ui.settings.theme_mode == ThemeMode::Macchiato,
                                ThemeMode::Macchiato.label(),
                                UiMessage::SetThemeMode(ThemeMode::Macchiato),
                            ),
                            option_button(
                                self.ui.settings.theme_mode == ThemeMode::Mocha,
                                ThemeMode::Mocha.label(),
                                UiMessage::SetThemeMode(ThemeMode::Mocha),
                            ),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                    theme_category(
                        "Gruvbox",
                        self.ui.theme_categories.gruvbox,
                        UiMessage::ToggleThemeCategory(ThemeCategory::Gruvbox),
                        row![
                            option_button(
                                self.ui.settings.theme_mode == ThemeMode::GruvboxLight,
                                "Light Mode",
                                UiMessage::SetThemeMode(ThemeMode::GruvboxLight),
                            ),
                            option_button(
                                self.ui.settings.theme_mode == ThemeMode::GruvboxDark,
                                "Dark Mode",
                                UiMessage::SetThemeMode(ThemeMode::GruvboxDark),
                            ),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                    theme_category(
                        "Everblush",
                        self.ui.theme_categories.everblush,
                        UiMessage::ToggleThemeCategory(ThemeCategory::Everblush),
                        row![
                            option_button(
                                self.ui.settings.theme_mode == ThemeMode::EverblushLight,
                                "Light Mode",
                                UiMessage::SetThemeMode(ThemeMode::EverblushLight),
                            ),
                            option_button(
                                self.ui.settings.theme_mode == ThemeMode::EverblushDark,
                                "Dark Mode",
                                UiMessage::SetThemeMode(ThemeMode::EverblushDark),
                            ),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                    theme_category(
                        "Kanagawa",
                        self.ui.theme_categories.kanagawa,
                        UiMessage::ToggleThemeCategory(ThemeCategory::Kanagawa),
                        row![
                            option_button(
                                self.ui.settings.theme_mode == ThemeMode::KanagawaLight,
                                "Light Mode",
                                UiMessage::SetThemeMode(ThemeMode::KanagawaLight),
                            ),
                            option_button(
                                self.ui.settings.theme_mode == ThemeMode::KanagawaDark,
                                "Dark Mode",
                                UiMessage::SetThemeMode(ThemeMode::KanagawaDark),
                            ),
                            option_button(
                                self.ui.settings.theme_mode == ThemeMode::KanagawaJournal,
                                "Journal Mode",
                                UiMessage::SetThemeMode(ThemeMode::KanagawaJournal),
                            ),
                        ]
                        .spacing(8)
                        .into(),
                    ),
                ]
                .spacing(12)
                .width(Length::Fill)
                .wrap(),
            ]
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
            let notice = self.ui.audio_notice.as_deref().map(|label| {
                row![
                    text(label)
                        .size(theme.size(12))
                        .font(style::font_propo(Weight::Light))
                        .style(move |_| style::text_style_muted(theme))
                        .width(Length::Fill),
                    button(
                        text("OK")
                            .size(theme.size(12))
                            .font(style::font_propo(Weight::Medium))
                            .style(move |_| style::text_style_primary(theme)),
                    )
                    .style(move |_, status| {
                        style::button_style(
                            theme,
                            style::ButtonKind::Tab { selected: false },
                            status,
                        )
                    })
                    .padding([6, 10])
                    .on_press(UiMessage::DismissAudioNotice)
                ]
                .align_y(Alignment::Center)
                .spacing(12)
            });
            column![
                section_hint("Choisissez la sortie audio principale."),
                if let Some(notice) = notice {
                    notice
                } else {
                    row![]
                },
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
            let band_controls = if self.ui.settings.eq_enabled {
                eq_band_controls(theme, &self.ui.settings)
            } else {
                column![].into()
            };
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
                if self.ui.settings.eq_enabled {
                    band_controls
                } else {
                    column![].into()
                },
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
                    setting_label(reset_audio_action.title(), reset_audio_action.description()),
                    controls(action_controls(reset_audio_action)),
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
    fn new(catalog: Catalog, library_root_override: Option<PathBuf>) -> Self {
        let mut settings = config::load_settings();
        if let Some(root) = library_root_override {
            settings.library_folder = root.display().to_string();
        }
        let mut player = match Player::new_with_settings(&settings) {
            Ok(player) => Some(player),
            Err(err) => {
                error!(error = %err, "Failed to initialize audio player");
                None
            }
        };
        let mut audio_notice = None;
        if let Some(player) = player.as_mut() {
            if let Some(fallback) = player.take_last_fallback_notice() {
                audio_notice = Some(fallback.notice());
                Self::apply_audio_fallback_to_settings(&mut settings, &fallback);
                if let Err(err) = config::save_settings(&settings) {
                    error!(error = %err, "Failed to persist audio fallback settings");
                }
            }
        }
        let playlists = PlaylistManager::load_or_default();
        let playback_queue = Self::playback_queue_from_playlist(&playlists);
        let mut ui = UiState::new(settings);
        ui.audio_notice = audio_notice;
        if !catalog.artists.is_empty() {
            ui.needs_initial_scan = false;
        }
        if let Some(active) = playlists.active() {
            ui.selection.selected_playlist = Some(playlists.active_index);
            ui.selection.playlist_name_draft = active.name.clone();
        }
        Self {
            catalog,
            player,
            playlists,
            playback_queue,
            ui,
            cover_preloads: Vec::new(),
        }
    }

    fn title(&self) -> String {
        "Grape".to_string()
    }

    fn update(&mut self, message: UiMessage) -> Task<UiMessage> {
        let should_select_genre_album = matches!(message, UiMessage::SelectGenre(_));
        let selected_artist = match &message {
            UiMessage::SelectArtist(artist) => Some(artist.clone()),
            _ => None,
        };
        let selected_album = match &message {
            UiMessage::SelectAlbum(album) => Some(album.clone()),
            _ => None,
        };
        let selected_folder = match &message {
            UiMessage::SelectFolder(folder) => Some(folder.clone()),
            _ => None,
        };
        let should_reset_limits = matches!(
            message,
            UiMessage::TabSelected(_)
                | UiMessage::Search(SearchMessage::QueryChanged(_))
                | UiMessage::Search(SearchMessage::SortChanged(_))
                | UiMessage::Search(SearchMessage::ToggleFilter(_))
                | UiMessage::SelectArtist(_)
                | UiMessage::SelectGenre(_)
                | UiMessage::SelectFolder(_)
        );
        let should_refresh_preloads = matches!(
            message,
            UiMessage::TabSelected(_)
                | UiMessage::Search(_)
                | UiMessage::SelectArtist(_)
                | UiMessage::SelectAlbum(_)
                | UiMessage::SelectGenre(_)
                | UiMessage::SelectFolder(_)
                | UiMessage::LoadMoreArtists
                | UiMessage::LoadMoreAlbums
                | UiMessage::LoadMoreTracks
                | UiMessage::LoadMoreGenres
                | UiMessage::LoadMoreFolders
                | UiMessage::LibraryScanCompleted(_)
        );
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
                | UiMessage::SetAccessibilityLargeText(_)
                | UiMessage::SetAccessibilityHighContrast(_)
                | UiMessage::SetAccessibilityReduceMotion(_)
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
                | UiMessage::SetEqBandGain(_, _)
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
        let should_refresh_audio = matches!(
            message,
            UiMessage::SetAudioOutputDevice(_)
                | UiMessage::SetMissingDeviceBehavior(_)
                | UiMessage::SetNormalizeVolume(_)
                | UiMessage::SetVolumeLevel(_)
                | UiMessage::SetEqEnabled(_)
                | UiMessage::SetEqPreset(_)
                | UiMessage::SetEqBandGain(_, _)
                | UiMessage::ResetEq
                | UiMessage::SetAudioStabilityMode(_)
                | UiMessage::SetAudioDebugLogs(_)
        );
        let mut task = Task::none();
        match &message {
            UiMessage::SelectTrack(track) => {
                self.handle_track_selection(track);
            }
            UiMessage::NavigateLibrary(navigation) => {
                self.handle_library_navigation(*navigation);
            }
            UiMessage::ActivateSelection => {
                self.activate_selection();
            }
            UiMessage::SelectPlaylist(index) => {
                if self.playlists.set_active(*index) {
                    if let Some(active) = self.playlists.active() {
                        self.ui.selection.selected_playlist = Some(*index);
                        self.ui.selection.playlist_name_draft = active.name.clone();
                    }
                    self.refresh_playback_queue(None);
                }
                self.ui.selection.playlist_drag_source = None;
            }
            UiMessage::Playback(playback_message) => {
                self.handle_playback_message(playback_message);
            }
            UiMessage::SaveAlbumMetadata => {
                self.handle_album_metadata_save();
            }
            UiMessage::CreatePlaylist => {
                let (index, name) = self
                    .playlists
                    .create_playlist(self.ui.selection.playlist_name_draft.clone());
                self.ui.selection.selected_playlist = Some(index);
                self.ui.selection.playlist_name_draft = name;
                self.ui.selection.playlist_drag_source = None;
                self.refresh_playback_queue(None);
                self.persist_playlist();
            }
            UiMessage::RenamePlaylist => {
                let index = self.playlists.active_index;
                if let Some(name) = self
                    .playlists
                    .rename_playlist(index, self.ui.selection.playlist_name_draft.clone())
                {
                    self.ui.selection.playlist_name_draft = name;
                    self.persist_playlist();
                }
            }
            UiMessage::DeletePlaylist => {
                let index = self.playlists.active_index;
                if self.playlists.remove_playlist(index) {
                    if let Some(active) = self.playlists.active() {
                        self.ui.selection.selected_playlist = Some(self.playlists.active_index);
                        self.ui.selection.playlist_name_draft = active.name.clone();
                    }
                    self.ui.selection.playlist_drag_source = None;
                    self.refresh_playback_queue(None);
                    self.persist_playlist();
                }
            }
            UiMessage::MovePlaylistItemUp(index) => {
                if *index > 0 {
                    self.playlist_reorder(*index, (*index).saturating_sub(1));
                }
            }
            UiMessage::MovePlaylistItemDown(index) => {
                let can_move = self
                    .playlists
                    .active()
                    .map(|playlist| *index + 1 < playlist.items.len())
                    .unwrap_or(false);
                if can_move {
                    self.playlist_reorder(*index, *index + 1);
                }
            }
            UiMessage::StartPlaylistItemDrag(index) => {
                if self.ui.selection.playlist_drag_source == Some(*index) {
                    self.ui.selection.playlist_drag_source = None;
                } else {
                    self.ui.selection.playlist_drag_source = Some(*index);
                }
            }
            UiMessage::MovePlaylistItemDrag { from, to } => {
                if self
                    .playlists
                    .active()
                    .map(|playlist| *from < playlist.items.len() && *to < playlist.items.len())
                    .unwrap_or(false)
                {
                    self.playlist_reorder(*from, *to);
                }
                self.ui.selection.playlist_drag_source = None;
            }
            UiMessage::DeletePlaylistItem(index) => {
                self.playlist_remove(*index);
                self.ui.selection.playlist_drag_source = None;
            }
            UiMessage::SavePlaylistOrder => {
                self.playlist_save_order();
                self.ui.selection.playlist_drag_source = None;
            }
            UiMessage::AddSelectedTrackToPlaylist => {
                if let Some(track) = self.ui.selection.selected_track.as_ref() {
                    let now_playing = Self::now_playing_from_ui_track(track);
                    self.playlist_add(now_playing);
                }
            }
            UiMessage::ClearQueue => {
                self.playlist_clear();
            }
            UiMessage::MoveQueueItemUp(index) => {
                if *index > 0 {
                    self.playlist_reorder(*index, (*index).saturating_sub(1));
                }
            }
            UiMessage::MoveQueueItemDown(index) => {
                if *index + 1 < self.playback_queue.items().len() {
                    self.playlist_reorder(*index, *index + 1);
                }
            }
            UiMessage::RemoveQueueItem(index) => {
                self.playlist_remove(*index);
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
            UiMessage::LibraryFolderPicked(path) => {
                if let Some(path) = path {
                    let root = PathBuf::from(path);
                    if !root.is_dir() {
                        warn!(
                            path = %root.display(),
                            "Selected library folder is invalid"
                        );
                    } else {
                        task = self.begin_scan(root, false);
                    }
                }
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
                task = self.handle_declarative_action(DeclarativeAction::ClearCache);
            }
            UiMessage::ClearHistory => {
                if let Err(err) = config::clear_history() {
                    error!(error = %err, "Failed to clear local history");
                } else {
                    info!("Local history cleared");
                }
            }
            UiMessage::OpenLogsFolder => {
                self.open_logs_folder();
            }
            UiMessage::ReindexLibrary => {
                task = self.handle_declarative_action(DeclarativeAction::ReindexLibrary);
            }
            UiMessage::ResetAudioEngine => {
                task = self.handle_declarative_action(DeclarativeAction::ResetAudioEngine);
            }
            UiMessage::ConfirmDeclarativeAction(action) => {
                if self.ui.pending_action == Some(*action) {
                    task = self.handle_declarative_action(*action);
                }
            }
            UiMessage::StartInitialScan => {
                if self.ui.needs_initial_scan {
                    self.ui.needs_initial_scan = false;
                    task = self.begin_scan_from_settings(true);
                }
            }
            UiMessage::ScanTick => {
                if let Some(status) = self.ui.scan_status.as_mut() {
                    let next = status.progress + 0.02;
                    status.progress = if next >= 0.95 { 0.2 } else { next };
                }
            }
            UiMessage::LibraryScanCompleted(result) => {
                let scan_root = self
                    .ui
                    .scan_status
                    .as_ref()
                    .map(|status| status.root.clone());
                self.ui.scan_status = None;
                match result {
                    Ok(catalog) => {
                        let catalog = catalog.clone();
                        let root = scan_root
                            .or_else(|| self.library_root())
                            .unwrap_or_default();
                        let has_root_album = catalog
                            .artists
                            .iter()
                            .any(|artist| artist.albums.iter().any(|album| album.path == root));
                        info!(
                            path = %root.display(),
                            root_tracks = has_root_album,
                            "Library scan completed"
                        );
                        self.catalog = catalog;
                        self.ui.selection = SelectionState::default();
                        self.ui.album_genre_draft.clear();
                        self.ui.album_year_draft.clear();
                        self.ui.search = SearchState::default();
                        self.ui.list_limits = ListLimits::default();
                    }
                    Err(err) => {
                        error!(error = %err, "Failed to scan library");
                    }
                }
            }
            UiMessage::LoadMoreArtists => {
                self.ui.list_limits.artists += 50;
            }
            UiMessage::LoadMoreAlbums => {
                self.ui.list_limits.albums += 30;
            }
            UiMessage::LoadMoreTracks => {
                self.ui.list_limits.tracks += 50;
            }
            UiMessage::LoadMoreGenres => {
                self.ui.list_limits.genres += 50;
            }
            UiMessage::LoadMoreFolders => {
                self.ui.list_limits.folders += 50;
            }
            _ => {}
        }
        self.ui.update(message);
        if let Some(artist) = selected_artist {
            self.apply_artist_selection(artist);
        }
        if let Some(album) = selected_album {
            self.apply_album_selection(album);
        }
        if let Some(folder) = selected_folder {
            self.apply_folder_selection(folder);
        }
        if should_reset_limits {
            self.ui.list_limits = ListLimits::default();
        }
        if should_select_genre_album && self.ui.active_tab == ActiveTab::Genres {
            if let Some(album) = self.filtered_albums_from_catalog().into_iter().next() {
                let album_id = album.id;
                self.ui.selection.selected_album = Some(album);
                self.ui.selection.selected_track =
                    self.album_entry_by_id(album_id)
                        .and_then(|(artist, entry)| {
                            self.filtered_tracks_for_album(artist, entry)
                                .into_iter()
                                .next()
                        });
            } else {
                self.ui.selection.selected_album = None;
                self.ui.selection.selected_track = None;
            }
        }
        if should_persist {
            if let Err(err) = config::save_settings(&self.ui.settings) {
                error!(error = %err, "Failed to save preferences");
            }
        }
        if should_refresh_audio {
            self.apply_audio_settings();
        }
        if should_refresh_preloads {
            self.refresh_cover_preloads();
        }
        self.sync_playback_state();
        if self.ui.settings.reduce_animations || self.ui.settings.accessibility_reduce_motion {
            self.ui.playback.animated_progress =
                progress_ratio(self.ui.playback.position, self.ui.playback.duration);
        } else {
            self.ui.playback.update_animated_progress();
        }
        task
    }

    fn view(&self) -> Element<'_, UiMessage> {
        let theme = self.theme_tokens();
        if self.ui.queue_open {
            return self.queue_view();
        }
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

        let mut layout = column![self.top_bar()]
            .spacing(16)
            .padding(16)
            .height(Length::Fill);
        if let Some(banner) = self.scan_banner() {
            layout = layout.push(banner);
        }
        layout = layout.push(content).push(self.player_bar());

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| style::surface_style(theme, style::Surface::AppBackground))
            .into()
    }

    fn theme(&self) -> Theme {
        match self.ui.settings.theme_mode {
            ThemeMode::Latte
            | ThemeMode::GruvboxLight
            | ThemeMode::EverblushLight
            | ThemeMode::KanagawaLight
            | ThemeMode::KanagawaJournal => Theme::Light,
            ThemeMode::Frappe
            | ThemeMode::Macchiato
            | ThemeMode::Mocha
            | ThemeMode::GruvboxDark
            | ThemeMode::EverblushDark
            | ThemeMode::KanagawaDark => Theme::Dark,
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

        if self.ui.queue_open {
            subscriptions.push(event::listen_with(|event, _status, _| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                    if matches!(key, keyboard::Key::Named(keyboard::key::Named::Escape)) =>
                {
                    Some(UiMessage::CloseQueue)
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

        if !self.ui.menu_open
            && !self.ui.playlist_open
            && !self.ui.queue_open
            && !self.ui.preferences_open
        {
            subscriptions.push(event::listen_with(|event, status, _| match event {
                event::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. })
                    if status == event::Status::Ignored =>
                {
                    match key {
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                            Some(UiMessage::NavigateLibrary(LibraryNavigation::Up))
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                            Some(UiMessage::NavigateLibrary(LibraryNavigation::Down))
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => {
                            Some(UiMessage::NavigateLibrary(LibraryNavigation::Left))
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => {
                            Some(UiMessage::NavigateLibrary(LibraryNavigation::Right))
                        }
                        keyboard::Key::Named(keyboard::key::Named::Tab) => {
                            if modifiers.shift() {
                                Some(UiMessage::NavigateLibrary(LibraryNavigation::PreviousPanel))
                            } else {
                                Some(UiMessage::NavigateLibrary(LibraryNavigation::NextPanel))
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::Enter) => {
                            Some(UiMessage::ActivateSelection)
                        }
                        _ => None,
                    }
                }
                _ => None,
            }));
        }

        if self.ui.needs_initial_scan {
            subscriptions
                .push(time::every(Duration::from_millis(16)).map(|_| UiMessage::StartInitialScan));
        }

        if self.ui.scan_status.is_some() {
            subscriptions
                .push(time::every(Duration::from_millis(120)).map(|_| UiMessage::ScanTick));
        }

        let target_progress = progress_ratio(self.ui.playback.position, self.ui.playback.duration);
        let needs_animation = (self.ui.playback.animated_progress - target_progress).abs() > 0.001;
        if (self.ui.playback.is_playing || needs_animation)
            && !self.ui.settings.accessibility_reduce_motion
            && !self.ui.settings.reduce_animations
        {
            subscriptions
                .push(time::every(Duration::from_millis(33)).map(|_| UiMessage::PlaybackTick));
        }

        Subscription::batch(subscriptions)
    }
}
