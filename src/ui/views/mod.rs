use crate::ui::{RenderContext, ViewResult};

pub mod collection;
pub mod game;
pub mod loot;
pub mod mastery;
pub mod progress;
pub mod summoner;

pub use collection::*;
pub use game::*;
pub use loot::*;
pub use mastery::*;
pub use progress::*;
pub use summoner::*;

/// Trait for rendering views in the TUI
pub trait RenderableView {
    /// Render the view into a ratatui Frame with scroll support
    fn render(&self, rc: RenderContext) -> ViewResult;

    fn title(&self) -> &str;
}

#[macro_export]
macro_rules! styled_span {
    // More specific patterns FIRST

    // Expression with color and bold (expr; Color::X Bold)
    ($expr:expr; $color:ident Bold) => {
        ratatui::text::Span::styled(
            format!("{}", $expr),
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::$color)
                .add_modifier(ratatui::style::Modifier::BOLD)
        )
    };

    // Expression with color (expr; Color::X)
    ($expr:expr; $color:ident) => {
        ratatui::text::Span::styled(
            format!("{}", $expr),
            ratatui::style::Style::default().fg(ratatui::style::Color::$color)
        )
    };

    // Formatted text with color and bold (text, args...; Color::X Bold)
    ($text:literal, $($arg:expr),+; $color:ident Bold) => {
        ratatui::text::Span::styled(
            format!($text, $($arg),+),
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::$color)
                .add_modifier(ratatui::style::Modifier::BOLD)
        )
    };

    // Formatted text with color (text, args...; Color::X)
    ($text:literal, $($arg:expr),+; $color:ident) => {
        ratatui::text::Span::styled(
            format!($text, $($arg),+),
            ratatui::style::Style::default().fg(ratatui::style::Color::$color)
        )
    };

    // Plain text with color and bold (text; Color::X Bold)
    ($text:literal; $color:ident Bold) => {
        ratatui::text::Span::styled(
            $text,
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::$color)
                .add_modifier(ratatui::style::Modifier::BOLD)
        )
    };

    // Plain text with color (text; Color::X)
    ($text:literal; $color:ident) => {
        ratatui::text::Span::styled(
            $text,
            ratatui::style::Style::default().fg(ratatui::style::Color::$color)
        )
    };

    // Formatted text (text, args...)
    ($text:literal, $($arg:expr),+) => {
        ratatui::text::Span::raw(format!($text, $($arg),+))
    };

    // Plain text literal (LAST - most general)
    ($text:literal) => {
        ratatui::text::Span::raw($text)
    };

    // Plain expression
    ($expr:expr) => {
        ratatui::text::Span::raw(format!("{}", $expr))
    };
}

#[macro_export]
macro_rules! styled_line {
    // Empty line
    () => {
        ratatui::text::Line::raw("")
    };

    // Span list
    (LIST [$($args:expr),+ $(,)?]) => {
        ratatui::text::Line::from(vec![$($args),+])
    };

    // Full styled line
    ($($args:tt)+) => {
        ratatui::text::Line::from($crate::styled_span!($($args)+))
    };
}

/// Macro for simple text-based views
#[macro_export]
macro_rules! impl_text_view {
    ($name:ident, $text_render_fn:expr, $title:expr, $description:expr) => {
        pub struct $name {
            lines: Vec<ratatui::text::Line<'static>>,
            error: Option<String>,
        }

        impl $name {
            pub fn new(controller: &Controller) -> Self {
                match $text_render_fn(controller) {
                    Ok(lines) => Self { lines, error: None },
                    Err(e) => Self {
                        lines: Vec::new(),
                        error: Some(format!("{}", e)),
                    },
                }
            }
        }

        impl $crate::ui::views::RenderableView for $name {
            fn title(&self) -> &str {
                $title
            }

            fn render(&self, rc: $crate::ui::RenderContext) -> $crate::ui::ViewResult {
                use ratatui::style::{Color, Style};
                use ratatui::text::{Line, Span};

                let text = if let Some(error) = &self.error {
                    vec![Line::from(vec![
                        Span::raw("\n  [!] Error: "),
                        Span::styled(error, Style::default().fg(Color::Red)),
                    ])]
                } else {
                    self.lines.clone()
                };

                let paragraph = ratatui::widgets::Paragraph::new(text)
                    .block(rc.block)
                    .wrap(ratatui::widgets::Wrap { trim: false })
                    .scroll((rc.scroll_offset, 0));

                rc.frame.render_widget(paragraph, rc.area);
                Ok(())
            }
        }
    };
}
