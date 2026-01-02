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
