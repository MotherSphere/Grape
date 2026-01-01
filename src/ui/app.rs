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
        <Self as Application>::run(Settings::with_flags(catalog))
    }

    pub fn run_with(catalog: Catalog, settings: Settings<Catalog>) -> iced::Result {
        let mut settings = settings;
        settings.flags = catalog;
        <Self as Application>::run(settings)
    }

    fn top_bar(&self) -> Element<Message> {
        let logo = text("Grape").size(22);
        let tabs = row![
            text("Artists"),
            text("Genres"),
            text("Albums"),
            text("Folders"),
        ]
        .spacing(16)
        .align_items(Alignment::Center);
        let search = row![text("Search..."), text("⎯ ☐ ✕")]
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

    fn artists_panel(&self) -> Element<Message> {
        let content = column![
            text("Artists").size(16),
            text("A–Z index"),
            text("Artists list placeholder"),
        ]
        .spacing(8);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .into()
    }

    fn albums_panel(&self) -> Element<Message> {
        let content = column![
            text("Albums").size(16),
            text("Sort: A–Z"),
            text("Albums grid placeholder"),
        ]
        .spacing(8);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .into()
    }

    fn songs_panel(&self) -> Element<Message> {
        let content = column![
            text("Songs").size(16),
            text("Album title / artist placeholder"),
            text("Songs list placeholder"),
        ]
        .spacing(8);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .into()
    }

    fn player_bar(&self) -> Element<Message> {
        let content = row![
            text("Now Playing • Artwork + title"),
            text("Shuffle • Prev • Play/Pause • Next • Repeat"),
            text("Progress • Volume • Queue")
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
            .height(Length::Fill)
            .into()
    }
}
