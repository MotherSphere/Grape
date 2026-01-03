use iced::font::{Family, Weight};
use iced::widget::{button, container, text, text_input};
use iced::{Background, Border, Color, Font, Shadow};

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
    pub fn latte() -> Self {
        Self {
            background: Color::from_rgb8(0xef, 0xf1, 0xf5),
            panel: Color::from_rgb8(0xe6, 0xe9, 0xef),
            elevated: Color::from_rgb8(0xcc, 0xd0, 0xda),
            hover: Color::from_rgb8(0xbc, 0xc0, 0xcc),
            selected: Color::from_rgb8(0x72, 0x87, 0xfd),
            accent: Color::from_rgb8(0x1e, 0x66, 0xf5),
            text_primary: Color::from_rgb8(0x4c, 0x4f, 0x69),
            text_muted: Color::from_rgb8(0x6c, 0x6f, 0x85),
            border: Color::from_rgb8(0xac, 0xb0, 0xbe),
            border_subtle: Color::from_rgb8(0xbc, 0xc0, 0xcc),
            avatar: Color::from_rgb8(0xcc, 0xd0, 0xda),
            player_bar: Color::from_rgb8(0xdc, 0xe0, 0xe8),
            album_cover: Color::from_rgb8(0xcc, 0xd0, 0xda),
            input_background: Color::from_rgb8(0xe6, 0xe9, 0xef),
            input_border: Color::from_rgb8(0xac, 0xb0, 0xbe),
        }
    }

    pub fn frappe() -> Self {
        Self {
            background: Color::from_rgb8(0x30, 0x34, 0x46),
            panel: Color::from_rgb8(0x29, 0x2c, 0x3c),
            elevated: Color::from_rgb8(0x41, 0x45, 0x59),
            hover: Color::from_rgb8(0x51, 0x57, 0x6d),
            selected: Color::from_rgb8(0xba, 0xbb, 0xf1),
            accent: Color::from_rgb8(0x8c, 0xaa, 0xee),
            text_primary: Color::from_rgb8(0xc6, 0xd0, 0xf5),
            text_muted: Color::from_rgb8(0xa5, 0xad, 0xce),
            border: Color::from_rgb8(0x62, 0x68, 0x80),
            border_subtle: Color::from_rgb8(0x51, 0x57, 0x6d),
            avatar: Color::from_rgb8(0x41, 0x45, 0x59),
            player_bar: Color::from_rgb8(0x23, 0x26, 0x34),
            album_cover: Color::from_rgb8(0x41, 0x45, 0x59),
            input_background: Color::from_rgb8(0x29, 0x2c, 0x3c),
            input_border: Color::from_rgb8(0x62, 0x68, 0x80),
        }
    }

    pub fn macchiato() -> Self {
        Self {
            background: Color::from_rgb8(0x24, 0x27, 0x3a),
            panel: Color::from_rgb8(0x1e, 0x20, 0x30),
            elevated: Color::from_rgb8(0x36, 0x3a, 0x4f),
            hover: Color::from_rgb8(0x49, 0x4d, 0x64),
            selected: Color::from_rgb8(0xb7, 0xbd, 0xf8),
            accent: Color::from_rgb8(0x8a, 0xad, 0xf4),
            text_primary: Color::from_rgb8(0xca, 0xd3, 0xf5),
            text_muted: Color::from_rgb8(0xa5, 0xad, 0xcb),
            border: Color::from_rgb8(0x5b, 0x60, 0x78),
            border_subtle: Color::from_rgb8(0x49, 0x4d, 0x64),
            avatar: Color::from_rgb8(0x36, 0x3a, 0x4f),
            player_bar: Color::from_rgb8(0x18, 0x19, 0x26),
            album_cover: Color::from_rgb8(0x36, 0x3a, 0x4f),
            input_background: Color::from_rgb8(0x1e, 0x20, 0x30),
            input_border: Color::from_rgb8(0x5b, 0x60, 0x78),
        }
    }

    pub fn mocha() -> Self {
        Self {
            background: Color::from_rgb8(0x1e, 0x1e, 0x2e),
            panel: Color::from_rgb8(0x18, 0x18, 0x25),
            elevated: Color::from_rgb8(0x31, 0x32, 0x44),
            hover: Color::from_rgb8(0x45, 0x47, 0x5a),
            selected: Color::from_rgb8(0xb4, 0xbe, 0xfe),
            accent: Color::from_rgb8(0x89, 0xb4, 0xfa),
            text_primary: Color::from_rgb8(0xcd, 0xd6, 0xf4),
            text_muted: Color::from_rgb8(0xa6, 0xad, 0xc8),
            border: Color::from_rgb8(0x58, 0x5b, 0x70),
            border_subtle: Color::from_rgb8(0x45, 0x47, 0x5a),
            avatar: Color::from_rgb8(0x31, 0x32, 0x44),
            player_bar: Color::from_rgb8(0x11, 0x11, 0x1b),
            album_cover: Color::from_rgb8(0x31, 0x32, 0x44),
            input_background: Color::from_rgb8(0x18, 0x18, 0x25),
            input_border: Color::from_rgb8(0x58, 0x5b, 0x70),
        }
    }

    pub fn gruvbox() -> Self {
        Self {
            background: Color::from_rgb8(0x28, 0x28, 0x28),
            panel: Color::from_rgb8(0x3c, 0x38, 0x36),
            elevated: Color::from_rgb8(0x50, 0x49, 0x45),
            hover: Color::from_rgb8(0x66, 0x5c, 0x54),
            selected: Color::from_rgb8(0x83, 0xa5, 0x98),
            accent: Color::from_rgb8(0xfa, 0xbd, 0x2f),
            text_primary: Color::from_rgb8(0xeb, 0xdb, 0xb2),
            text_muted: Color::from_rgb8(0xbd, 0xae, 0x93),
            border: Color::from_rgb8(0x50, 0x49, 0x45),
            border_subtle: Color::from_rgb8(0x3c, 0x38, 0x36),
            avatar: Color::from_rgb8(0x50, 0x49, 0x45),
            player_bar: Color::from_rgb8(0x1d, 0x20, 0x21),
            album_cover: Color::from_rgb8(0x50, 0x49, 0x45),
            input_background: Color::from_rgb8(0x3c, 0x38, 0x36),
            input_border: Color::from_rgb8(0x50, 0x49, 0x45),
        }
    }

    pub fn everblush() -> Self {
        Self {
            background: Color::from_rgb8(0x14, 0x1b, 0x1e),
            panel: Color::from_rgb8(0x23, 0x2a, 0x2d),
            elevated: Color::from_rgb8(0x2d, 0x34, 0x37),
            hover: Color::from_rgb8(0x3a, 0x42, 0x45),
            selected: Color::from_rgb8(0x67, 0xb0, 0xe8),
            accent: Color::from_rgb8(0x8c, 0xcf, 0x7e),
            text_primary: Color::from_rgb8(0xda, 0xda, 0xda),
            text_muted: Color::from_rgb8(0xb3, 0xb9, 0xb8),
            border: Color::from_rgb8(0x2d, 0x34, 0x37),
            border_subtle: Color::from_rgb8(0x23, 0x2a, 0x2d),
            avatar: Color::from_rgb8(0x2d, 0x34, 0x37),
            player_bar: Color::from_rgb8(0x10, 0x16, 0x18),
            album_cover: Color::from_rgb8(0x2d, 0x34, 0x37),
            input_background: Color::from_rgb8(0x23, 0x2a, 0x2d),
            input_border: Color::from_rgb8(0x3a, 0x42, 0x45),
        }
    }

    pub fn kanagawa() -> Self {
        Self {
            background: Color::from_rgb8(0x1f, 0x1f, 0x28),
            panel: Color::from_rgb8(0x18, 0x18, 0x20),
            elevated: Color::from_rgb8(0x2a, 0x2a, 0x37),
            hover: Color::from_rgb8(0x36, 0x36, 0x46),
            selected: Color::from_rgb8(0x7e, 0x9c, 0xd8),
            accent: Color::from_rgb8(0x95, 0x7f, 0xb8),
            text_primary: Color::from_rgb8(0xdc, 0xd7, 0xba),
            text_muted: Color::from_rgb8(0x72, 0x71, 0x69),
            border: Color::from_rgb8(0x2a, 0x2a, 0x37),
            border_subtle: Color::from_rgb8(0x1a, 0x1a, 0x22),
            avatar: Color::from_rgb8(0x2a, 0x2a, 0x37),
            player_bar: Color::from_rgb8(0x16, 0x16, 0x1d),
            album_cover: Color::from_rgb8(0x2a, 0x2a, 0x37),
            input_background: Color::from_rgb8(0x18, 0x18, 0x20),
            input_border: Color::from_rgb8(0x36, 0x36, 0x46),
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
            ThemeMode::Latte => Palette::latte(),
            ThemeMode::Frappe => Palette::frappe(),
            ThemeMode::Macchiato => Palette::macchiato(),
            ThemeMode::Gruvbox => Palette::gruvbox(),
            ThemeMode::Everblush => Palette::everblush(),
            ThemeMode::Kanagawa => Palette::kanagawa(),
            ThemeMode::Mocha => Palette::mocha(),
        };
        Self { palette, scale }
    }

    pub fn size(&self, base: u16) -> u32 {
        ((base as f32 * self.scale).round().max(10.0)) as u32
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

pub fn text_style_primary(theme: ThemeTokens) -> text::Style {
    text::Style {
        color: Some(text_primary(theme)),
        ..text::Style::default()
    }
}

pub fn text_style_muted(theme: ThemeTokens) -> text::Style {
    text::Style {
        color: Some(text_muted(theme)),
        ..text::Style::default()
    }
}

pub fn text_style(color: Color) -> text::Style {
    text::Style {
        color: Some(color),
        ..text::Style::default()
    }
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

pub fn surface_style(theme: ThemeTokens, surface: Surface) -> container::Style {
    let palette = theme.palette;
    let (background, border) = match surface {
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

    container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(text_primary(theme)),
        border,
        shadow: Shadow::default(),
        snap: cfg!(feature = "crisp"),
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

pub fn button_style(
    theme: ThemeTokens,
    kind: ButtonKind,
    status: button::Status,
) -> button::Style {
    let palette = theme.palette;
    let mut style = match kind {
        ButtonKind::Tab { selected } => button::Style {
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
            shadow: Shadow::default(),
            snap: cfg!(feature = "crisp"),
        },
        ButtonKind::ListItem { selected } => button::Style {
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
            shadow: Shadow::default(),
            snap: cfg!(feature = "crisp"),
        },
        ButtonKind::AlbumCard { selected } => button::Style {
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
            shadow: Shadow::default(),
            snap: cfg!(feature = "crisp"),
        },
        ButtonKind::Control => button::Style {
            background: Some(Background::Color(palette.elevated)),
            text_color: palette.text_primary,
            border: Border {
                radius: 12.0.into(),
                width: 1.0,
                color: palette.border_subtle,
            },
            shadow: Shadow::default(),
            snap: cfg!(feature = "crisp"),
        },
        ButtonKind::Icon => button::Style {
            background: Some(Background::Color(Color::TRANSPARENT)),
            text_color: palette.text_muted,
            border: Border {
                radius: 8.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: Shadow::default(),
            snap: cfg!(feature = "crisp"),
        },
    };

    match status {
        button::Status::Hovered | button::Status::Pressed => {
            style.background = Some(Background::Color(palette.hover));
        }
        button::Status::Disabled => {
            style.background = Some(Background::Color(palette.elevated));
            style.text_color = palette.text_muted;
            style.border.color = palette.border_subtle;
        }
        button::Status::Active => {}
    }

    style
}

pub fn text_input_style(
    theme: ThemeTokens,
    status: text_input::Status,
) -> text_input::Style {
    let base = text_input::Style {
        background: Background::Color(theme.palette.input_background),
        border: Border {
            radius: 10.0.into(),
            width: 1.0,
            color: theme.palette.input_border,
        },
        icon: text_muted(theme),
        placeholder: text_muted(theme),
        value: text_primary(theme),
        selection: accent_alpha(theme, 0.25),
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Hovered => text_input::Style {
            border: Border {
                color: theme.palette.border,
                ..base.border
            },
            ..base
        },
        text_input::Status::Focused { .. } => text_input::Style {
            border: Border {
                color: accent(theme),
                ..base.border
            },
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: Background::Color(theme.palette.elevated),
            border: Border {
                color: theme.palette.border,
                ..base.border
            },
            value: text_muted(theme),
            ..base
        },
    }
}
