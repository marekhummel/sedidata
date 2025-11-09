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

/// Macro to create styled text lines easily
///
/// Usage:
/// ```
/// styled_text!("Plain text")
/// styled_text!("Plain {}", variable)
/// styled_text!(Color::Red, "Red text")
/// styled_text!(Color::Green, Bold, "Green bold text")
/// styled_text!("Mix " Color::Blue "blue" " and " Color::Red "red" " text")
/// ```
#[macro_export]
#[allow(clippy::vec_init_then_push)]
macro_rules! styled_text {
    // Empty line
    () => {
        ratatui::text::Line::raw("")
    };

    // Plain text with format args
    ($fmt:literal $(, $arg:expr)*) => {
        ratatui::text::Line::raw(format!($fmt $(, $arg)*))
    };

    // Single colored segment with format
    ($color:expr, $fmt:literal $(, $arg:expr)*) => {{
        use ratatui::text::{Line, Span};
        use ratatui::style::{Style, Color};
        Line::from(Span::styled(
            format!($fmt $(, $arg)*),
            Style::default().fg($color)
        ))
    }};

    // Colored + Bold with format
    ($color:expr, Bold, $fmt:literal $(, $arg:expr)*) => {{
        use ratatui::text::{Line, Span};
        use ratatui::style::{Style, Color, Modifier};
        Line::from(Span::styled(
            format!($fmt $(, $arg)*),
            Style::default().fg($color).add_modifier(Modifier::BOLD)
        ))
    }};

    // Multiple segments - this is the complex one
    ($($segment:tt)+) => {{
        let mut spans = Vec::new();
        $crate::parse_segments!(spans, $($segment)+);
        ratatui::text::Line::from(spans)
    }};
}

/// Helper macro to parse multiple segments
#[macro_export]
macro_rules! parse_segments {
    // Base case - empty
    ($spans:expr,) => {};

    // Plain string literal
    ($spans:expr, $text:literal $($rest:tt)*) => {
        $spans.push(ratatui::text::Span::raw($text));
        $crate::parse_segments!($spans, $($rest)*);
    };

    // Plain expression in braces (for formatted strings)
    ($spans:expr, {$expr:expr} $($rest:tt)*) => {
        $spans.push(ratatui::text::Span::raw($expr));
        $crate::parse_segments!($spans, $($rest)*);
    };

    // Color then string
    ($spans:expr, Color::$color:ident $text:literal $($rest:tt)*) => {
        $spans.push(ratatui::text::Span::styled($text, ratatui::style::Style::default().fg(ratatui::style::Color::$color)));
        $crate::parse_segments!($spans, $($rest)*);
    };

    // Color then expression in braces
    ($spans:expr, Color::$color:ident {$expr:expr} $($rest:tt)*) => {
        $spans.push(ratatui::text::Span::styled($expr, ratatui::style::Style::default().fg(ratatui::style::Color::$color)));
        $crate::parse_segments!($spans, $($rest)*);
    };

    // Color Bold then string
    ($spans:expr, Color::$color:ident Bold $text:literal $($rest:tt)*) => {
        $spans.push(ratatui::text::Span::styled(
            $text,
            ratatui::style::Style::default().fg(ratatui::style::Color::$color).add_modifier(ratatui::style::Modifier::BOLD)
        ));
        $crate::parse_segments!($spans, $($rest)*);
    };

    // Color Bold then expression in braces
    ($spans:expr, Color::$color:ident Bold {$expr:expr} $($rest:tt)*) => {
        $spans.push(ratatui::text::Span::styled(
            $expr,
            ratatui::style::Style::default().fg(ratatui::style::Color::$color).add_modifier(ratatui::style::Modifier::BOLD)
        ));
        $crate::parse_segments!($spans, $($rest)*);
    };

    // RGB color then string
    ($spans:expr, Rgb($r:expr, $g:expr, $b:expr) $text:literal $($rest:tt)*) => {
        $spans.push(ratatui::text::Span::styled($text, ratatui::style::Style::default().fg(ratatui::style::Color::Rgb($r, $g, $b))));
        $crate::parse_segments!($spans, $($rest)*);
    };

    // RGB color then expression in braces
    ($spans:expr, Rgb($r:expr, $g:expr, $b:expr) {$expr:expr} $($rest:tt)*) => {
        $spans.push(ratatui::text::Span::styled($expr, ratatui::style::Style::default().fg(ratatui::style::Color::Rgb($r, $g, $b))));
        $crate::parse_segments!($spans, $($rest)*);
    };

    // RGB color Bold then string
    ($spans:expr, Rgb($r:expr, $g:expr, $b:expr) Bold $text:literal $($rest:tt)*) => {
        $spans.push(ratatui::text::Span::styled(
            $text,
            ratatui::style::Style::default().fg(ratatui::style::Color::Rgb($r, $g, $b)).add_modifier(ratatui::style::Modifier::BOLD)
        ));
        $crate::parse_segments!($spans, $($rest)*);
    };

    // RGB color Bold then expression in braces
    ($spans:expr, Rgb($r:expr, $g:expr, $b:expr) Bold {$expr:expr} $($rest:tt)*) => {
        $spans.push(ratatui::text::Span::styled(
            $expr,
            ratatui::style::Style::default().fg(ratatui::style::Color::Rgb($r, $g, $b)).add_modifier(ratatui::style::Modifier::BOLD)
        ));
        $crate::parse_segments!($spans, $($rest)*);
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
