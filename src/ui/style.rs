use iced::font::{Family, Weight};
use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Font, Theme};

use crate::config::ThemeMode;

pub const FONT_PROPO: &str = "JetBrainsMono Nerd Font Propo";
pub const FONT_MONO: &str = "JetBrainsMono Nerd Font Mono";

pub fn font_propo(weight: Weight) -> Font {
    Font {
        family: Family::Name(FONT_PROPO),
        weight,
        ..Font::DEFAULT
    }
}

pub fn font_mono(weight: Weight) -> Font {
    Font {
        family: Family::Name(FONT_MONO),
        weight,
        ..Font::DEFAULT
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: Color,
    pub panel: Color,
    pub elevated: Color,
    pub hover: Color,
    pub selected: Color,
    pub accent: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub border: Color,
    pub border_subtle: Color,
    pub avatar: Color,
    pub player_bar: Color,
    pub album_cover: Color,
    pub input_background: Color,
    pub input_border: Color,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            background: Color::from_rgb8(0x12, 0x12, 0x12),
            panel: Color::from_rgb8(0x18, 0x18, 0x18),
            elevated: Color::from_rgb8(0x1f, 0x1f, 0x1f),
            hover: Color::from_rgb8(0x2a, 0x2a, 0x2a),
            selected: Color::from_rgb8(0x24, 0x2f, 0x47),
            accent: Color::from_rgb8(0x3d, 0x7c, 0xff),
            text_primary: Color::from_rgb8(0xf1, 0xf1, 0xf1),
            text_muted: Color::from_rgb8(0xa7, 0xa7, 0xa7),
            border: Color::from_rgb8(0x26, 0x26, 0x26),
            border_subtle: Color::from_rgb8(0x2a, 0x2a, 0x2a),
            avatar: Color::from_rgb8(0x2f, 0x3b, 0x55),
            player_bar: Color::from_rgb8(0x0f, 0x0f, 0x0f),
            album_cover: Color::from_rgb8(0x27, 0x27, 0x27),
            input_background: Color::from_rgb8(0x1b, 0x1b, 0x1b),
            input_border: Color::from_rgb8(0x2c, 0x2c, 0x2c),
        }
    }

    pub fn light() -> Self {
        Self {
            background: Color::from_rgb8(0xf6, 0xf7, 0xfb),
            panel: Color::from_rgb8(0xff, 0xff, 0xff),
            elevated: Color::from_rgb8(0xee, 0xf1, 0xf7),
            hover: Color::from_rgb8(0xe3, 0xe7, 0xf0),
            selected: Color::from_rgb8(0xd9, 0xe6, 0xff),
            accent: Color::from_rgb8(0x2f, 0x6b, 0xff),
            text_primary: Color::from_rgb8(0x1f, 0x23, 0x2a),
            text_muted: Color::from_rgb8(0x5c, 0x64, 0x6f),
            border: Color::from_rgb8(0xdd, 0xe2, 0xee),
            border_subtle: Color::from_rgb8(0xe5, 0xe9, 0xf2),
            avatar: Color::from_rgb8(0xc8, 0xd5, 0xf4),
            player_bar: Color::from_rgb8(0xe9, 0xed, 0xf4),
            album_cover: Color::from_rgb8(0xf1, 0xf4, 0xfa),
            input_background: Color::from_rgb8(0xff, 0xff, 0xff),
            input_border: Color::from_rgb8(0xd0, 0xd7, 0xe2),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeTokens {
    pub palette: Palette,
    pub scale: f32,
}

impl ThemeTokens {
    pub fn new(mode: ThemeMode, scale: f32) -> Self {
        let palette = match mode {
            ThemeMode::Dark => Palette::dark(),
            ThemeMode::Light => Palette::light(),
            ThemeMode::System => Palette::dark(),
        };
        Self { palette, scale }
    }

    pub fn size(&self, base: u16) -> u16 {
        ((base as f32 * self.scale).round().max(10.0)) as u16
    }
}

pub fn accent(theme: ThemeTokens) -> Color {
    theme.palette.accent
}

pub fn text_primary(theme: ThemeTokens) -> Color {
    theme.palette.text_primary
}

pub fn text_muted(theme: ThemeTokens) -> Color {
    theme.palette.text_muted
}

pub fn accent_alpha(theme: ThemeTokens, alpha: f32) -> Color {
    Color {
        a: alpha,
        ..theme.palette.accent
    }
}

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
pub struct SurfaceStyle {
    surface: Surface,
    theme: ThemeTokens,
}

impl SurfaceStyle {
    pub fn new(surface: Surface, theme: ThemeTokens) -> Self {
        Self { surface, theme }
    }
}

impl container::StyleSheet for SurfaceStyle {
    type Style = Theme;

    fn appearance(&self, _style: &Theme) -> container::Appearance {
        let palette = self.theme.palette;
        let (background, border) = match self.surface {
            Surface::AppBackground => (
                palette.background,
                Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            ),
            Surface::TopBar => (
                palette.elevated,
                Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            ),
            Surface::Panel => (
                palette.panel,
                Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: palette.border,
                },
            ),
            Surface::Sidebar => (
                palette.elevated,
                Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: palette.border_subtle,
                },
            ),
            Surface::PlayerBar => (
                palette.player_bar,
                Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            ),
            Surface::AlbumCover => (
                palette.album_cover,
                Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: palette.border_subtle,
                },
            ),
            Surface::Avatar => (
                palette.avatar,
                Border {
                    radius: 999.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            ),
        };

        container::Appearance {
            background: Some(Background::Color(background)),
            text_color: Some(text_primary(self.theme)),
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
pub struct ButtonStyle {
    kind: ButtonKind,
    theme: ThemeTokens,
}

impl ButtonStyle {
    pub fn new(kind: ButtonKind, theme: ThemeTokens) -> Self {
        Self { kind, theme }
    }
}

impl button::StyleSheet for ButtonStyle {
    type Style = Theme;

    fn active(&self, _style: &Theme) -> button::Appearance {
        let palette = self.theme.palette;
        match self.kind {
            ButtonKind::Tab { selected } => button::Appearance {
                background: Some(Background::Color(if selected {
                    palette.hover
                } else {
                    Color::TRANSPARENT
                })),
                text_color: if selected {
                    palette.accent
                } else {
                    palette.text_muted
                },
                border: Border {
                    radius: 8.0.into(),
                    width: if selected { 1.0 } else { 0.0 },
                    color: if selected {
                        palette.accent
                    } else {
                        Color::TRANSPARENT
                    },
                },
                ..Default::default()
            },
            ButtonKind::ListItem { selected } => button::Appearance {
                background: Some(Background::Color(if selected {
                    palette.selected
                } else {
                    Color::TRANSPARENT
                })),
                text_color: palette.text_primary,
                border: Border {
                    radius: 10.0.into(),
                    width: if selected { 1.0 } else { 0.0 },
                    color: if selected {
                        palette.accent
                    } else {
                        Color::TRANSPARENT
                    },
                },
                ..Default::default()
            },
            ButtonKind::AlbumCard { selected } => button::Appearance {
                background: Some(Background::Color(if selected {
                    palette.selected
                } else {
                    Color::TRANSPARENT
                })),
                text_color: palette.text_primary,
                border: Border {
                    radius: 12.0.into(),
                    width: if selected { 1.0 } else { 0.0 },
                    color: if selected {
                        palette.accent
                    } else {
                        Color::TRANSPARENT
                    },
                },
                ..Default::default()
            },
            ButtonKind::Control => button::Appearance {
                background: Some(Background::Color(palette.elevated)),
                text_color: palette.text_primary,
                border: Border {
                    radius: 12.0.into(),
                    width: 1.0,
                    color: palette.border_subtle,
                },
                ..Default::default()
            },
            ButtonKind::Icon => button::Appearance {
                background: Some(Background::Color(Color::TRANSPARENT)),
                text_color: palette.text_muted,
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
        appearance.background = Some(Background::Color(self.theme.palette.hover));
        appearance
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SearchInput {
    theme: ThemeTokens,
}

impl SearchInput {
    pub fn new(theme: ThemeTokens) -> Self {
        Self { theme }
    }
}

impl text_input::StyleSheet for SearchInput {
    type Style = Theme;

    fn active(&self, _style: &Theme) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(self.theme.palette.input_background),
            border: Border {
                radius: 10.0.into(),
                width: 1.0,
                color: self.theme.palette.input_border,
            },
            icon_color: text_muted(self.theme),
        }
    }

    fn focused(&self, style: &Theme) -> text_input::Appearance {
        let mut appearance = self.active(style);
        appearance.border.color = accent(self.theme);
        appearance
    }

    fn placeholder_color(&self, _style: &Theme) -> Color {
        text_muted(self.theme)
    }

    fn value_color(&self, _style: &Theme) -> Color {
        text_primary(self.theme)
    }

    fn disabled_color(&self, _style: &Theme) -> Color {
        text_muted(self.theme)
    }

    fn selection_color(&self, _style: &Theme) -> Color {
        accent_alpha(self.theme, 0.25)
    }

    fn disabled(&self, style: &Theme) -> text_input::Appearance {
        let mut appearance = self.active(style);
        appearance.background = Background::Color(self.theme.palette.elevated);
        appearance.border.color = self.theme.palette.border;
        appearance
    }
}
