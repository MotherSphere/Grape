use crate::library::Catalog;
use crate::ui::components::albums_grid::AlbumsGrid;
use crate::ui::components::player_bar::PlayerBar;
use crate::ui::message::{SearchMessage, UiMessage};
use crate::ui::state::{ActiveTab, Album as UiAlbum, SortOption, UiState};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme};

pub struct GrapeApp {
    catalog: Catalog,
    ui: UiState,
}

impl GrapeApp {
    pub fn run(catalog: Catalog) -> iced::Result {
        <Self as Application>::run(Settings::with_flags(catalog))
    }

    pub fn run_with(catalog: Catalog, settings: Settings<Catalog>) -> iced::Result {
        let mut settings = settings;
        settings.flags = catalog;
        <Self as Application>::run(settings)
    }

    fn tab_label(&self, tab: ActiveTab, label: &str) -> String {
        if self.ui.active_tab == tab {
            format!("{label} •")
        } else {
            label.to_string()
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
                    year: Some(album.year as u32),
                });
                id += 1;
            }
        }

        albums
    }

    fn top_bar(&self) -> Element<UiMessage> {
        let logo = text("Grape").size(22);
        let tabs = row![
            button(text(self.tab_label(ActiveTab::Artists, "Artists")))
                .on_press(UiMessage::TabSelected(ActiveTab::Artists)),
            button(text(self.tab_label(ActiveTab::Genres, "Genres")))
                .on_press(UiMessage::TabSelected(ActiveTab::Genres)),
            button(text(self.tab_label(ActiveTab::Albums, "Albums")))
                .on_press(UiMessage::TabSelected(ActiveTab::Albums)),
            button(text(self.tab_label(ActiveTab::Folders, "Folders")))
                .on_press(UiMessage::TabSelected(ActiveTab::Folders)),
        ]
        .spacing(16)
        .align_items(Alignment::Center);
        let search_input = text_input("Search...", &self.ui.search.query)
            .on_input(|value| UiMessage::Search(SearchMessage::QueryChanged(value)));
        let search = row![search_input, text("⎯ ☐ ✕")]
            .spacing(12)
            .align_items(Alignment::Center);

        let layout = row![
            container(logo).width(Length::Shrink),
            container(tabs).width(Length::Fill).center_x(),
            container(search).width(Length::Shrink)
        ]
        .spacing(24)
        .align_items(Alignment::Center);

        container(layout)
            .padding(12)
            .width(Length::Fill)
            .into()
    }

    fn artists_panel(&self) -> Element<UiMessage> {
        let selection = self
            .ui
            .selection
            .selected_artist
            .as_ref()
            .map(|artist| artist.name.as_str())
            .unwrap_or("None");
        let content = column![
            text("Artists").size(16),
            text("A–Z index"),
            text(format!("Selected: {selection}")),
            text("Artists list placeholder"),
        ]
        .spacing(8);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .into()
    }

    fn albums_panel(&self) -> Element<UiMessage> {
        let sort_label = match self.ui.search.sort {
            SortOption::Alphabetical => "A–Z",
            SortOption::ByAlbum => "By album",
        };
        let selected_id = self.ui.selection.selected_album.as_ref().map(|album| album.id);
        let albums = self.albums_from_catalog();
        let grid = AlbumsGrid::new(albums)
            .with_sort_label(sort_label)
            .with_selection(selected_id)
            .view();

        container(grid)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn songs_panel(&self) -> Element<UiMessage> {
        let selection = self
            .ui
            .selection
            .selected_track
            .as_ref()
            .map(|track| track.title.as_str())
            .unwrap_or("None");
        let content = column![
            text("Songs").size(16),
            text("Album title / artist placeholder"),
            text(format!("Selected: {selection}")),
            text("Songs list placeholder"),
        ]
        .spacing(8);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .into()
    }

    fn player_bar(&self) -> Element<UiMessage> {
        let (title, artist) = self
            .ui
            .selection
            .selected_track
            .as_ref()
            .map(|track| (track.title.clone(), track.artist.clone()))
            .or_else(|| {
                self.ui
                    .selection
                    .selected_album
                    .as_ref()
                    .map(|album| (album.title.clone(), album.artist.clone()))
            })
            .unwrap_or_else(|| ("No track selected".to_string(), "Pick a track to play".to_string()));

        PlayerBar::new(title, artist)
            .with_playback(self.ui.playback)
            .with_volume(72)
            .with_queue(false)
            .view()
    }
}

impl Application for GrapeApp {
    type Executor = iced::executor::Default;
    type Message = UiMessage;
    type Theme = Theme;
    type Flags = Catalog;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                catalog: flags,
                ui: UiState::default(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Grape".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        self.ui.update(message);
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let content = row![
            self.artists_panel(),
            self.albums_panel(),
            self.songs_panel()
        ]
        .spacing(16)
        .height(Length::Fill);

        column![self.top_bar(), content, self.player_bar()]
            .spacing(12)
            .padding(12)
            .height(Length::Fill)
            .into()
    }
}
