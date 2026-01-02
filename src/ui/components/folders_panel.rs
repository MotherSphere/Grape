#![allow(dead_code)]

use crate::ui::message::UiMessage;
use crate::ui::style;
use iced::font::Weight;
use iced::theme::{Button, Container};
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderEntry {
    pub id: usize,
    pub name: String,
    pub track_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderLayout {
    Grid,
    List,
}

#[derive(Debug, Clone)]
pub struct FoldersPanel {
    sort_label: String,
    folders: Vec<FolderEntry>,
    selected_folder_id: Option<usize>,
    layout: FolderLayout,
    columns: usize,
    scroll_offset: usize,
    viewport_rows: usize,
}

impl FoldersPanel {
    pub fn new(folders: Vec<FolderEntry>) -> Self {
        Self {
            sort_label: "By name".to_string(),
            folders,
            selected_folder_id: None,
            layout: FolderLayout::Grid,
            columns: 3,
            scroll_offset: 0,
            viewport_rows: 3,
        }
    }

    pub fn with_sort_label(mut self, sort_label: impl Into<String>) -> Self {
        self.sort_label = sort_label.into();
        self
    }

    pub fn with_selection(mut self, selected_folder_id: Option<usize>) -> Self {
        self.selected_folder_id = selected_folder_id;
        self
    }

    pub fn with_layout_grid(
        mut self,
        columns: usize,
        scroll_offset: usize,
        viewport_rows: usize,
    ) -> Self {
        self.layout = FolderLayout::Grid;
        self.columns = columns.max(1);
        self.scroll_offset = scroll_offset;
        self.viewport_rows = viewport_rows.max(1);
        self
    }

    pub fn with_layout_list(mut self, scroll_offset: usize, viewport_size: usize) -> Self {
        self.layout = FolderLayout::List;
        self.scroll_offset = scroll_offset;
        self.viewport_rows = viewport_size.max(1);
        self
    }

    pub fn view(self) -> Element<'static, UiMessage> {
        let header = row![
            text(format!("{} Folders", self.folders.len()))
                .size(16)
                .font(style::font_propo(Weight::Semibold))
                .style(style::text_primary()),
            text(self.sort_label)
                .size(12)
                .font(style::font_propo(Weight::Light))
                .style(style::text_muted())
        ]
        .spacing(8)
        .align_items(Alignment::Center);

        let content = match self.layout {
            FolderLayout::Grid => self.grid_view(header),
            FolderLayout::List => self.list_view(header),
        };

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(12)
            .style(Container::Custom(Box::new(style::SurfaceStyle(
                style::Surface::Panel,
            ))))
            .into()
    }

    fn grid_view(&self, header: Element<'static, UiMessage>) -> Element<'static, UiMessage> {
        let rows = self
            .folders
            .chunks(self.columns)
            .map(|chunk| {
                let cells = chunk
                    .iter()
                    .map(|folder| {
                        let is_selected = Some(folder.id) == self.selected_folder_id;
                        let icon = container(
                            text("▣")
                                .size(26)
                                .font(style::font_propo(Weight::Medium))
                                .style(style::text_muted()),
                        )
                        .width(Length::Fixed(120.0))
                        .height(Length::Fixed(120.0))
                        .center_x()
                        .center_y()
                        .style(Container::Custom(Box::new(
                            style::SurfaceStyle(style::Surface::AlbumCover),
                        )));
                        let title = text(folder.name.clone())
                            .size(14)
                            .font(style::font_propo(Weight::Medium))
                            .style(style::text_primary());
                        let count = text(format!("{} tracks", folder.track_count))
                            .size(12)
                            .font(style::font_propo(Weight::Light))
                            .style(style::text_muted());
                        let card = column![icon, title, count]
                            .spacing(6)
                            .align_items(Alignment::Center)
                            .width(Length::Fill);

                        button(card)
                            .style(Button::Custom(Box::new(style::ButtonStyle(
                                style::ButtonKind::AlbumCard {
                                    selected: is_selected,
                                },
                            ))))
                            .width(Length::FillPortion(1))
                            .into()
                    })
                    .collect::<Vec<Element<UiMessage>>>();

                row(cells)
                    .spacing(16)
                    .align_items(Alignment::Start)
                    .width(Length::Fill)
                    .into()
            })
            .collect::<Vec<Element<UiMessage>>>();
        let grid = column(rows)
            .spacing(20)
            .width(Length::Fill)
            .align_items(Alignment::Start);
        column![header, grid]
            .spacing(12)
            .width(Length::Fill)
            .align_items(Alignment::Start)
            .into()
    }

    fn list_view(&self, header: Element<'static, UiMessage>) -> Element<'static, UiMessage> {
        let list_items = self
            .folders
            .iter()
            .map(|folder| {
                let is_selected = Some(folder.id) == self.selected_folder_id;
                let icon = container(
                    text("▣")
                        .size(16)
                        .font(style::font_propo(Weight::Medium))
                        .style(style::text_primary()),
                )
                .width(Length::Fixed(28.0))
                .height(Length::Fixed(28.0))
                .center_x()
                .center_y()
                .style(Container::Custom(Box::new(
                    style::SurfaceStyle(style::Surface::AlbumCover),
                )));
                let title = text(folder.name.clone())
                    .size(14)
                    .font(style::font_propo(Weight::Medium))
                    .style(style::text_primary());
                let count = text(format!("{} tracks", folder.track_count))
                    .size(12)
                    .font(style::font_propo(Weight::Light))
                    .style(style::text_muted());
                let details = column![title, count]
                    .spacing(2)
                    .width(Length::Fill)
                    .align_items(Alignment::Start);
                let row_content = row![icon, details]
                    .spacing(10)
                    .align_items(Alignment::Center)
                    .width(Length::Fill);

                button(row_content)
                    .style(Button::Custom(Box::new(style::ButtonStyle(
                        style::ButtonKind::ListItem {
                            selected: is_selected,
                        },
                    ))))
                    .width(Length::Fill)
                    .into()
            })
            .collect::<Vec<Element<UiMessage>>>();
        let list = column(list_items)
            .spacing(8)
            .width(Length::Fill)
            .align_items(Alignment::Start);
        column![header, list]
            .spacing(12)
            .width(Length::Fill)
            .align_items(Alignment::Start)
            .into()
    }

    pub fn render(&self) -> String {
        let header = format!("{} Folders · {}", self.folders.len(), self.sort_label);
        let mut lines = vec![header];

        match self.layout {
            FolderLayout::Grid => {
                let total_rows = self.folders.len().div_ceil(self.columns);
                let scroll_offset = self.scroll_offset.min(total_rows.saturating_sub(1));
                let rows = self.folders.chunks(self.columns).collect::<Vec<_>>();
                let visible_rows = rows.iter().skip(scroll_offset).take(self.viewport_rows);

                for row in visible_rows {
                    let row_labels = row
                        .iter()
                        .map(|folder| {
                            let name = if Some(folder.id) == self.selected_folder_id {
                                format!("> {} <", folder.name)
                            } else {
                                folder.name.clone()
                            };
                            format!("{: <14} {:>3}", name, folder.track_count)
                        })
                        .collect::<Vec<_>>();
                    lines.push(row_labels.join("  "));
                }
            }
            FolderLayout::List => {
                let visible = self
                    .folders
                    .iter()
                    .skip(self.scroll_offset)
                    .take(self.viewport_rows)
                    .collect::<Vec<_>>();
                for folder in visible {
                    let name = if Some(folder.id) == self.selected_folder_id {
                        format!("> {} <", folder.name)
                    } else {
                        folder.name.clone()
                    };
                    lines.push(format!("{:<20} {:>3} tracks", name, folder.track_count));
                }
            }
        }

        lines.join("\n")
    }
}
