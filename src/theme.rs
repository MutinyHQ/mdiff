use ratatui::style::Color;
use serde::Deserialize;

/// All semantic color slots for the mdiff UI.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,

    // General UI
    pub accent: Color,
    pub secondary: Color,
    pub text: Color,
    pub text_muted: Color,
    pub surface: Color,
    pub selection_bg: Color,
    pub selection_inactive_bg: Color,

    // Diff
    pub diff_add_bg: Color,
    pub diff_del_bg: Color,
    pub diff_add_fg: Color,
    pub diff_del_fg: Color,
    pub diff_context_fg: Color,
    pub diff_hunk_header_fg: Color,
    pub visual_select_bg: Color,
    pub cursor_line_fg: Color,
    pub collapsed_bg: Color,

    // Status indicators
    pub success: Color,
    pub error: Color,
    pub warning: Color,

    // Syntax highlighting
    pub syntax: SyntaxColors,
}

/// Syntax highlighting color slots.
#[derive(Debug, Clone)]
pub struct SyntaxColors {
    pub comment: Color,
    pub keyword: Color,
    pub string: Color,
    pub number: Color,
    pub function: Color,
    pub type_name: Color,
    pub variable: Color,
    pub operator: Color,
    pub property: Color,
    pub tag: Color,
    pub punctuation: Color,
    pub default_fg: Color,
}

pub const THEME_NAMES: &[&str] = &[
    "one-dark",
    "github-dark",
    "dracula",
    "catppuccin-mocha",
    "tokyo-night",
    "solarized-dark",
];

impl Theme {
    pub fn from_name(name: &str) -> Self {
        match name {
            "github-dark" => github_dark(),
            "dracula" => dracula(),
            "catppuccin-mocha" => catppuccin_mocha(),
            "tokyo-night" => tokyo_night(),
            "solarized-dark" => solarized_dark(),
            _ => one_dark(),
        }
    }
}

pub fn next_theme(current: &str) -> &'static str {
    let idx = THEME_NAMES.iter().position(|&n| n == current).unwrap_or(0);
    THEME_NAMES[(idx + 1) % THEME_NAMES.len()]
}

pub fn prev_theme(current: &str) -> &'static str {
    let idx = THEME_NAMES.iter().position(|&n| n == current).unwrap_or(0);
    if idx == 0 {
        THEME_NAMES[THEME_NAMES.len() - 1]
    } else {
        THEME_NAMES[idx - 1]
    }
}

pub fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

// ── Serde-compatible override struct ──────────────────────────────

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ThemeOverrides {
    pub accent: Option<String>,
    pub secondary: Option<String>,
    pub text: Option<String>,
    pub text_muted: Option<String>,
    pub surface: Option<String>,
    pub selection_bg: Option<String>,
    pub selection_inactive_bg: Option<String>,
    pub diff_add_bg: Option<String>,
    pub diff_del_bg: Option<String>,
    pub diff_add_fg: Option<String>,
    pub diff_del_fg: Option<String>,
    pub diff_context_fg: Option<String>,
    pub diff_hunk_header_fg: Option<String>,
    pub visual_select_bg: Option<String>,
    pub cursor_line_fg: Option<String>,
    pub collapsed_bg: Option<String>,
    pub success: Option<String>,
    pub error: Option<String>,
    pub warning: Option<String>,
    #[serde(default)]
    pub syntax: Option<SyntaxOverrides>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct SyntaxOverrides {
    pub comment: Option<String>,
    pub keyword: Option<String>,
    pub string: Option<String>,
    pub number: Option<String>,
    pub function: Option<String>,
    pub type_name: Option<String>,
    pub variable: Option<String>,
    pub operator: Option<String>,
    pub property: Option<String>,
    pub tag: Option<String>,
    pub punctuation: Option<String>,
    pub default_fg: Option<String>,
}

pub fn apply_overrides(theme: &mut Theme, overrides: &ThemeOverrides) {
    macro_rules! apply {
        ($field:ident) => {
            if let Some(ref hex) = overrides.$field {
                if let Some(c) = parse_hex_color(hex) {
                    theme.$field = c;
                }
            }
        };
    }
    apply!(accent);
    apply!(secondary);
    apply!(text);
    apply!(text_muted);
    apply!(surface);
    apply!(selection_bg);
    apply!(selection_inactive_bg);
    apply!(diff_add_bg);
    apply!(diff_del_bg);
    apply!(diff_add_fg);
    apply!(diff_del_fg);
    apply!(diff_context_fg);
    apply!(diff_hunk_header_fg);
    apply!(visual_select_bg);
    apply!(cursor_line_fg);
    apply!(collapsed_bg);
    apply!(success);
    apply!(error);
    apply!(warning);

    if let Some(ref syn) = overrides.syntax {
        macro_rules! apply_syn {
            ($field:ident) => {
                if let Some(ref hex) = syn.$field {
                    if let Some(c) = parse_hex_color(hex) {
                        theme.syntax.$field = c;
                    }
                }
            };
        }
        apply_syn!(comment);
        apply_syn!(keyword);
        apply_syn!(string);
        apply_syn!(number);
        apply_syn!(function);
        apply_syn!(type_name);
        apply_syn!(variable);
        apply_syn!(operator);
        apply_syn!(property);
        apply_syn!(tag);
        apply_syn!(punctuation);
        apply_syn!(default_fg);
    }
}

// ── Built-in themes ──────────────────────────────────────────────

fn one_dark() -> Theme {
    Theme {
        name: "one-dark".to_string(),
        accent: Color::Cyan,
        secondary: Color::Magenta,
        text: Color::White,
        text_muted: Color::DarkGray,
        surface: Color::Rgb(30, 30, 30),
        selection_bg: Color::Rgb(40, 40, 50),
        selection_inactive_bg: Color::Rgb(35, 35, 45),
        diff_add_bg: Color::Rgb(0, 30, 0),
        diff_del_bg: Color::Rgb(40, 0, 0),
        diff_add_fg: Color::Green,
        diff_del_fg: Color::Red,
        diff_context_fg: Color::Rgb(171, 178, 191),
        diff_hunk_header_fg: Color::Magenta,
        visual_select_bg: Color::Rgb(70, 50, 100),
        cursor_line_fg: Color::Yellow,
        collapsed_bg: Color::Rgb(20, 20, 20),
        success: Color::Green,
        error: Color::Red,
        warning: Color::Yellow,
        syntax: SyntaxColors {
            comment: Color::Rgb(106, 115, 125),
            keyword: Color::Rgb(198, 120, 221),
            string: Color::Rgb(152, 195, 121),
            number: Color::Rgb(209, 154, 102),
            function: Color::Rgb(97, 175, 239),
            type_name: Color::Rgb(229, 192, 123),
            variable: Color::Rgb(171, 178, 191),
            operator: Color::Rgb(86, 182, 194),
            property: Color::Rgb(224, 108, 117),
            tag: Color::Rgb(224, 108, 117),
            punctuation: Color::Rgb(140, 140, 140),
            default_fg: Color::Rgb(171, 178, 191),
        },
    }
}

fn github_dark() -> Theme {
    Theme {
        name: "github-dark".to_string(),
        accent: Color::Rgb(88, 166, 255),
        secondary: Color::Rgb(188, 140, 255),
        text: Color::Rgb(230, 237, 243),
        text_muted: Color::Rgb(125, 133, 144),
        surface: Color::Rgb(22, 27, 34),
        selection_bg: Color::Rgb(38, 50, 72),
        selection_inactive_bg: Color::Rgb(30, 40, 58),
        diff_add_bg: Color::Rgb(18, 40, 24),
        diff_del_bg: Color::Rgb(50, 18, 18),
        diff_add_fg: Color::Rgb(63, 185, 80),
        diff_del_fg: Color::Rgb(248, 81, 73),
        diff_context_fg: Color::Rgb(230, 237, 243),
        diff_hunk_header_fg: Color::Rgb(188, 140, 255),
        visual_select_bg: Color::Rgb(50, 60, 90),
        cursor_line_fg: Color::Rgb(210, 153, 34),
        collapsed_bg: Color::Rgb(13, 17, 23),
        success: Color::Rgb(63, 185, 80),
        error: Color::Rgb(248, 81, 73),
        warning: Color::Rgb(210, 153, 34),
        syntax: SyntaxColors {
            comment: Color::Rgb(125, 133, 144),
            keyword: Color::Rgb(255, 123, 114),
            string: Color::Rgb(165, 214, 255),
            number: Color::Rgb(121, 192, 255),
            function: Color::Rgb(210, 168, 255),
            type_name: Color::Rgb(255, 166, 87),
            variable: Color::Rgb(230, 237, 243),
            operator: Color::Rgb(255, 123, 114),
            property: Color::Rgb(121, 192, 255),
            tag: Color::Rgb(126, 231, 135),
            punctuation: Color::Rgb(125, 133, 144),
            default_fg: Color::Rgb(230, 237, 243),
        },
    }
}

fn dracula() -> Theme {
    Theme {
        name: "dracula".to_string(),
        accent: Color::Rgb(139, 233, 253),
        secondary: Color::Rgb(255, 121, 198),
        text: Color::Rgb(248, 248, 242),
        text_muted: Color::Rgb(98, 114, 164),
        surface: Color::Rgb(40, 42, 54),
        selection_bg: Color::Rgb(68, 71, 90),
        selection_inactive_bg: Color::Rgb(55, 58, 75),
        diff_add_bg: Color::Rgb(15, 40, 15),
        diff_del_bg: Color::Rgb(45, 10, 10),
        diff_add_fg: Color::Rgb(80, 250, 123),
        diff_del_fg: Color::Rgb(255, 85, 85),
        diff_context_fg: Color::Rgb(248, 248, 242),
        diff_hunk_header_fg: Color::Rgb(255, 121, 198),
        visual_select_bg: Color::Rgb(80, 60, 120),
        cursor_line_fg: Color::Rgb(241, 250, 140),
        collapsed_bg: Color::Rgb(30, 31, 40),
        success: Color::Rgb(80, 250, 123),
        error: Color::Rgb(255, 85, 85),
        warning: Color::Rgb(241, 250, 140),
        syntax: SyntaxColors {
            comment: Color::Rgb(98, 114, 164),
            keyword: Color::Rgb(255, 121, 198),
            string: Color::Rgb(241, 250, 140),
            number: Color::Rgb(189, 147, 249),
            function: Color::Rgb(80, 250, 123),
            type_name: Color::Rgb(139, 233, 253),
            variable: Color::Rgb(248, 248, 242),
            operator: Color::Rgb(255, 121, 198),
            property: Color::Rgb(189, 147, 249),
            tag: Color::Rgb(255, 121, 198),
            punctuation: Color::Rgb(248, 248, 242),
            default_fg: Color::Rgb(248, 248, 242),
        },
    }
}

fn catppuccin_mocha() -> Theme {
    Theme {
        name: "catppuccin-mocha".to_string(),
        accent: Color::Rgb(137, 180, 250),
        secondary: Color::Rgb(245, 194, 231),
        text: Color::Rgb(205, 214, 244),
        text_muted: Color::Rgb(108, 112, 134),
        surface: Color::Rgb(30, 30, 46),
        selection_bg: Color::Rgb(49, 50, 68),
        selection_inactive_bg: Color::Rgb(40, 40, 58),
        diff_add_bg: Color::Rgb(10, 35, 20),
        diff_del_bg: Color::Rgb(45, 10, 15),
        diff_add_fg: Color::Rgb(166, 227, 161),
        diff_del_fg: Color::Rgb(243, 139, 168),
        diff_context_fg: Color::Rgb(205, 214, 244),
        diff_hunk_header_fg: Color::Rgb(245, 194, 231),
        visual_select_bg: Color::Rgb(60, 50, 90),
        cursor_line_fg: Color::Rgb(249, 226, 175),
        collapsed_bg: Color::Rgb(24, 24, 37),
        success: Color::Rgb(166, 227, 161),
        error: Color::Rgb(243, 139, 168),
        warning: Color::Rgb(249, 226, 175),
        syntax: SyntaxColors {
            comment: Color::Rgb(108, 112, 134),
            keyword: Color::Rgb(203, 166, 247),
            string: Color::Rgb(166, 227, 161),
            number: Color::Rgb(250, 179, 135),
            function: Color::Rgb(137, 180, 250),
            type_name: Color::Rgb(249, 226, 175),
            variable: Color::Rgb(205, 214, 244),
            operator: Color::Rgb(137, 220, 235),
            property: Color::Rgb(242, 205, 205),
            tag: Color::Rgb(243, 139, 168),
            punctuation: Color::Rgb(147, 153, 178),
            default_fg: Color::Rgb(205, 214, 244),
        },
    }
}

fn tokyo_night() -> Theme {
    Theme {
        name: "tokyo-night".to_string(),
        accent: Color::Rgb(122, 162, 247),
        secondary: Color::Rgb(187, 154, 247),
        text: Color::Rgb(192, 202, 245),
        text_muted: Color::Rgb(86, 95, 137),
        surface: Color::Rgb(26, 27, 38),
        selection_bg: Color::Rgb(41, 46, 66),
        selection_inactive_bg: Color::Rgb(33, 37, 55),
        diff_add_bg: Color::Rgb(10, 35, 15),
        diff_del_bg: Color::Rgb(45, 10, 15),
        diff_add_fg: Color::Rgb(158, 206, 106),
        diff_del_fg: Color::Rgb(247, 118, 142),
        diff_context_fg: Color::Rgb(192, 202, 245),
        diff_hunk_header_fg: Color::Rgb(187, 154, 247),
        visual_select_bg: Color::Rgb(55, 50, 95),
        cursor_line_fg: Color::Rgb(224, 175, 104),
        collapsed_bg: Color::Rgb(20, 22, 30),
        success: Color::Rgb(158, 206, 106),
        error: Color::Rgb(247, 118, 142),
        warning: Color::Rgb(224, 175, 104),
        syntax: SyntaxColors {
            comment: Color::Rgb(86, 95, 137),
            keyword: Color::Rgb(187, 154, 247),
            string: Color::Rgb(158, 206, 106),
            number: Color::Rgb(255, 158, 100),
            function: Color::Rgb(122, 162, 247),
            type_name: Color::Rgb(42, 195, 222),
            variable: Color::Rgb(192, 202, 245),
            operator: Color::Rgb(137, 221, 255),
            property: Color::Rgb(115, 218, 202),
            tag: Color::Rgb(247, 118, 142),
            punctuation: Color::Rgb(86, 95, 137),
            default_fg: Color::Rgb(192, 202, 245),
        },
    }
}

fn solarized_dark() -> Theme {
    Theme {
        name: "solarized-dark".to_string(),
        accent: Color::Rgb(38, 139, 210),
        secondary: Color::Rgb(211, 54, 130),
        text: Color::Rgb(147, 161, 161),
        text_muted: Color::Rgb(88, 110, 117),
        surface: Color::Rgb(0, 34, 43),
        selection_bg: Color::Rgb(7, 54, 66),
        selection_inactive_bg: Color::Rgb(3, 44, 55),
        diff_add_bg: Color::Rgb(0, 30, 10),
        diff_del_bg: Color::Rgb(40, 5, 5),
        diff_add_fg: Color::Rgb(133, 153, 0),
        diff_del_fg: Color::Rgb(220, 50, 47),
        diff_context_fg: Color::Rgb(147, 161, 161),
        diff_hunk_header_fg: Color::Rgb(211, 54, 130),
        visual_select_bg: Color::Rgb(30, 60, 80),
        cursor_line_fg: Color::Rgb(181, 137, 0),
        collapsed_bg: Color::Rgb(0, 26, 33),
        success: Color::Rgb(133, 153, 0),
        error: Color::Rgb(220, 50, 47),
        warning: Color::Rgb(181, 137, 0),
        syntax: SyntaxColors {
            comment: Color::Rgb(88, 110, 117),
            keyword: Color::Rgb(133, 153, 0),
            string: Color::Rgb(42, 161, 152),
            number: Color::Rgb(211, 54, 130),
            function: Color::Rgb(38, 139, 210),
            type_name: Color::Rgb(181, 137, 0),
            variable: Color::Rgb(147, 161, 161),
            operator: Color::Rgb(133, 153, 0),
            property: Color::Rgb(38, 139, 210),
            tag: Color::Rgb(220, 50, 47),
            punctuation: Color::Rgb(88, 110, 117),
            default_fg: Color::Rgb(147, 161, 161),
        },
    }
}
