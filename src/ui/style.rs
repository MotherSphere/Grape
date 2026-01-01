use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Theme};

pub const BG_DARK: Color = Color::from_rgb8(0x12, 0x12, 0x12);
pub const BG_PANEL: Color = Color::from_rgb8(0x18, 0x18, 0x18);
pub const BG_ELEVATED: Color = Color::from_rgb8(0x1f, 0x1f, 0x1f);
pub const BG_HOVER: Color = Color::from_rgb8(0x2a, 0x2a, 0x2a);
pub const BG_SELECTED: Color = Color::from_rgb8(0x24, 0x2f, 0x47);
pub const ACCENT: Color = Color::from_rgb8(0x3d, 0x7c, 0xff);
pub const TEXT_PRIMARY: Color = Color::from_rgb8(0xf1, 0xf1, 0xf1);
pub const TEXT_MUTED: Color = Color::from_rgb8(0xa7, 0xa7, 0xa7);

#[derive(Debug, Clone, Copy)]
pub enum Surface {
    AppBackground,
    TopBar,
    Panel,
    Sidebar,
    PlayerBar,
    AlbumCover,
    Avatar,
}

#[derive(Debug, Clone, Copy)]
pub struct SurfaceStyle(pub Surface);

impl container::StyleSheet for SurfaceStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Theme) -> container::Appearance {
        let (background, border) = match self.0 {
            Surface::AppBackground => (
                BG_DARK,
                Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            ),
            Surface::TopBar => (
                BG_ELEVATED,
                Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            ),
            Surface::Panel => (
                BG_PANEL,
                Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(0x26, 0x26, 0x26),
                },
            ),
            Surface::Sidebar => (
                BG_ELEVATED,
                Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(0x2a, 0x2a, 0x2a),
                },
            ),
            Surface::PlayerBar => (
                Color::from_rgb8(0x0f, 0x0f, 0x0f),
                Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            ),
            Surface::AlbumCover => (
                Color::from_rgb8(0x27, 0x27, 0x27),
                Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(0x33, 0x33, 0x33),
                },
            ),
            Surface::Avatar => (
                Color::from_rgb8(0x2f, 0x3b, 0x55),
                Border {
                    radius: 999.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            ),
        };

        container::Appearance {
            background: Some(Background::Color(background)),
            text_color: Some(TEXT_PRIMARY),
            border,
            shadow: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonKind {
    Tab { selected: bool },
    ListItem { selected: bool },
    AlbumCard { selected: bool },
    Control,
    Icon,
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonStyle(pub ButtonKind);

impl button::StyleSheet for ButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Theme) -> button::Appearance {
        match self.0 {
            ButtonKind::Tab { selected } => button::Appearance {
                background: Some(Background::Color(if selected {
                    BG_HOVER
                } else {
                    Color::TRANSPARENT
                })),
                text_color: if selected { ACCENT } else { TEXT_MUTED },
                border: Border {
                    radius: 8.0.into(),
                    width: if selected { 1.0 } else { 0.0 },
                    color: if selected { ACCENT } else { Color::TRANSPARENT },
                },
                ..Default::default()
            },
            ButtonKind::ListItem { selected } => button::Appearance {
                background: Some(Background::Color(if selected {
                    BG_SELECTED
                } else {
                    Color::TRANSPARENT
                })),
                text_color: TEXT_PRIMARY,
                border: Border {
                    radius: 10.0.into(),
                    width: if selected { 1.0 } else { 0.0 },
                    color: if selected { ACCENT } else { Color::TRANSPARENT },
                },
                ..Default::default()
            },
            ButtonKind::AlbumCard { selected } => button::Appearance {
                background: Some(Background::Color(if selected {
                    BG_SELECTED
                } else {
                    Color::TRANSPARENT
                })),
                text_color: TEXT_PRIMARY,
                border: Border {
                    radius: 12.0.into(),
                    width: if selected { 1.0 } else { 0.0 },
                    color: if selected { ACCENT } else { Color::TRANSPARENT },
                },
                ..Default::default()
            },
            ButtonKind::Control => button::Appearance {
                background: Some(Background::Color(BG_ELEVATED)),
                text_color: TEXT_PRIMARY,
                border: Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: Color::from_rgb8(0x2c, 0x2c, 0x2c),
                },
                ..Default::default()
            },
            ButtonKind::Icon => button::Appearance {
                background: Some(Background::Color(Color::TRANSPARENT)),
                text_color: TEXT_MUTED,
                border: Border {
                    radius: 8.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            },
        }
    }

    fn hovered(&self, style: &Theme) -> button::Appearance {
        let mut appearance = self.active(style);
        appearance.background = Some(Background::Color(BG_HOVER));
        appearance
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SearchInput;

impl text_input::StyleSheet for SearchInput {
    type Style = Theme;

    fn active(&self, _style: &Theme) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(Color::from_rgb8(0x1b, 0x1b, 0x1b)),
            border: Border {
                radius: 10.0.into(),
                width: 1.0,
                color: Color::from_rgb8(0x2c, 0x2c, 0x2c),
            },
            icon_color: TEXT_MUTED,
        }
    }

    fn focused(&self, style: &Theme) -> text_input::Appearance {
        let mut appearance = self.active(style);
        appearance.border.color = ACCENT;
        appearance
    }

    fn placeholder_color(&self, _style: &Theme) -> Color {
        TEXT_MUTED
    }

    fn value_color(&self, _style: &Theme) -> Color {
        TEXT_PRIMARY
    }

    fn disabled_color(&self, _style: &Theme) -> Color {
        TEXT_MUTED
    }

    fn selection_color(&self, _style: &Theme) -> Color {
        Color::from_rgba8(0x3d, 0x7c, 0xff, 0.3)
    }
}
