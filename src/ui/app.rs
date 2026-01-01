use crate::library::Catalog;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Alignment, Application, Command, Element, Length, Settings, Theme};

#[derive(Debug, Clone)]
pub enum Message {
    None,
}

pub struct GrapeApp {
    catalog: Catalog,
}

impl GrapeApp {
    pub fn run(catalog: Catalog) -> iced::Result {
        Self::run_with(catalog, Settings::<Catalog>::default())
    }

    pub fn run_with(catalog: Catalog, settings: Settings<Catalog>) -> iced::Result {
        GrapeApp::run(settings.with_flags(catalog))
    }

    fn top_bar(&self) -> Element<Message> {
        let tabs = row![
            text("Artists"),
            text("Genres"),
            text("Albums"),
            text("Folders"),
        ]
        .spacing(16)
        .align_items(Alignment::Center);

        container(tabs)
            .padding(12)
            .width(Length::Fill)
            .into()
    }

    fn artists_panel(&self) -> Element<Message> {
        let mut content = column![text("Artists")].spacing(6);

        if self.catalog.artists.is_empty() {
            content = content.push(text("No artists found"));
        } else {
            for artist in self.catalog.artists.iter().take(30) {
                content = content.push(text(&artist.name));
            }
        }

        container(scrollable(content))
            .width(Length::Fill)
            .padding(12)
            .into()
    }

    fn albums_panel(&self) -> Element<Message> {
        let mut content = column![text("Albums")].spacing(6);

        let albums: Vec<_> = self
            .catalog
            .artists
            .iter()
            .flat_map(|artist| artist.albums.iter().map(move |album| (artist, album)))
            .take(12)
            .collect();

        if albums.is_empty() {
            content = content.push(text("No albums found"));
        } else {
            for (artist, album) in albums {
                content = content.push(text(format!("{} — {}", album.title, artist.name)));
            }
        }

        container(scrollable(content))
            .width(Length::Fill)
            .padding(12)
            .into()
    }

    fn songs_panel(&self) -> Element<Message> {
        let mut content = column![text("Songs")].spacing(6);

        let tracks: Vec<_> = self
            .catalog
            .artists
            .iter()
            .flat_map(|artist| {
                artist
                    .albums
                    .iter()
                    .flat_map(|album| album.tracks.iter().map(move |track| (album, track)))
            })
            .take(12)
            .collect();

        if tracks.is_empty() {
            content = content.push(text("No tracks found"));
        } else {
            for (album, track) in tracks {
                content = content.push(text(format!("{} — {}", track.title, album.title)));
            }
        }

        container(scrollable(content))
            .width(Length::Fill)
            .padding(12)
            .into()
    }

    fn player_bar(&self) -> Element<Message> {
        let content = row![
            text("Now Playing"),
            text("⏮ ⏯ ⏭"),
            text("00:00 / 03:34")
        ]
        .spacing(20)
        .align_items(Alignment::Center);

        container(content)
            .padding(12)
            .width(Length::Fill)
            .into()
    }
}

impl Application for GrapeApp {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Catalog;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self { catalog: flags }, Command::none())
    }

    fn title(&self) -> String {
        "Grape".to_string()
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
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
            .into()
    }
}
