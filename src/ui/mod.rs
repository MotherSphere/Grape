#[derive(Debug, Default, Clone, Copy)]
pub struct TopBar;

#[derive(Debug, Default, Clone, Copy)]
pub struct LeftSidebar;

#[derive(Debug, Default, Clone, Copy)]
pub struct AlbumsGrid;

#[derive(Debug, Default, Clone, Copy)]
pub struct RightPanel;

#[derive(Debug, Default, Clone, Copy)]
pub struct PlayerBar;

#[derive(Debug, Default, Clone, Copy)]
pub struct ContentColumns {
    pub left: LeftSidebar,
    pub center: AlbumsGrid,
    pub right: RightPanel,
}

impl ContentColumns {
    pub fn new() -> Self {
        Self {
            left: LeftSidebar,
            center: AlbumsGrid,
            right: RightPanel,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MainLayout {
    pub top_bar: TopBar,
    pub content: ContentColumns,
    pub footer: PlayerBar,
}

impl MainLayout {
    pub fn new() -> Self {
        Self {
            top_bar: TopBar,
            content: ContentColumns::new(),
            footer: PlayerBar,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MainView {
    pub layout: MainLayout,
}

impl MainView {
    pub fn new() -> Self {
        Self {
            layout: MainLayout::new(),
        }
    }
}

pub fn main_view() -> MainView {
    MainView::new()
}
