use crate::ui::{Controller, RenderContext, ViewResult};

pub mod collection;
pub mod game;
pub mod loot;
pub mod mastery;
pub mod progress;
pub mod summoner;

pub use collection::*;
use crossterm::event::KeyCode;
pub use game::*;
pub use loot::*;
pub use mastery::*;
pub use progress::*;
pub use summoner::*;

/// Trait for rendering views in the TUI
pub trait RenderableView {
    /// Render the view into a ratatui Frame with scroll support
    fn render(&self, rc: RenderContext) -> ViewResult;

    fn update(&mut self, _controller: &Controller, _keys: &[KeyCode]) {}

    fn title(&self) -> &str;

    /// Returns the auto-refresh interval in seconds, or None if no auto-refresh
    fn auto_refresh_interval(&self) -> Option<f32> {
        None
    }

    /// Called when the view should refresh its data
    fn refresh_data(&mut self, _controller: &Controller) -> Result<(), String> {
        Ok(())
    }
}

pub fn eval_color_scale_descending<T: PartialOrd>(
    value: T,
    scale: &[(T, ratatui::style::Color)],
) -> ratatui::style::Color {
    for (threshold, color) in scale {
        if value >= *threshold {
            return *color;
        }
    }
    // Default to the last color if no thresholds matched
    scale
        .last()
        .map(|(_, color)| *color)
        .unwrap_or(ratatui::style::Color::White)
}

pub fn eval_color_scale_ascending<T: PartialOrd>(
    value: T,
    scale: &[(T, ratatui::style::Color)],
) -> ratatui::style::Color {
    for (threshold, color) in scale {
        if value <= *threshold {
            return *color;
        }
    }
    // Default to the first color if no thresholds matched
    scale
        .first()
        .map(|(_, color)| *color)
        .unwrap_or(ratatui::style::Color::White)
}

#[macro_export]
macro_rules! styled_span {
    // More specific patterns FIRST

    // Expression with color and bold (expr; Color::X Bold)
    ($expr:expr; Bold $color:expr) => {
        ratatui::text::Span::styled(
            format!("{}", $expr),
            ratatui::style::Style::default()
                .fg($color)
                .add_modifier(ratatui::style::Modifier::BOLD)
        )
    };

    // Expression with color (expr; Color::X)
    ($expr:expr; $color:expr) => {
        ratatui::text::Span::styled(
            format!("{}", $expr),
            ratatui::style::Style::default().fg($color)
        )
    };

    // Formatted text with color and bold (text, args...; Color::X Bold)
    ($text:literal, $($arg:expr),+; Bold $color:ident) => {
        ratatui::text::Span::styled(
            format!($text, $($arg),+),
            ratatui::style::Style::default()
                .fg($color)
                .add_modifier(ratatui::style::Modifier::BOLD)
        )
    };

    // Formatted text with color (text, args...; Color::X)
    ($text:literal, $($arg:expr),+; $color:expr) => {
        ratatui::text::Span::styled(
            format!($text, $($arg),+),
            ratatui::style::Style::default().fg($color)
        )
    };

    // Plain text with color and bold (text; Color::X Bold)
    ($text:literal; Bold $color:expr) => {
        ratatui::text::Span::styled(
            $text,
            ratatui::style::Style::default()
                .fg($color)
                .add_modifier(ratatui::style::Modifier::BOLD)
        )
    };

    // Plain text with color (text; Color::X)
    ($text:literal; $color:expr) => {
        ratatui::text::Span::styled(
            $text,
            ratatui::style::Style::default().fg($color)
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
    (VAR $vec:expr) => {
        ratatui::text::Line::from($vec)
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

#[macro_export]
macro_rules! empty_row {
    ($cell_count:expr) => {
        Row::new((0..$cell_count).map(|_| Cell::from("")).collect::<Vec<_>>())
    };
}

#[macro_export]
macro_rules! fill_row {
    ($cell_count:expr; $($cells:expr),+) => {
        {
            let mut cells = Vec::new();
            $(
                cells.push(Cell::from($cells));
            )+
            while cells.len() < $cell_count {
                cells.push(Cell::from(""));
            }
            Row::new(cells)
        }
    };
}

#[macro_export]
macro_rules! header_row {
    ($($header:expr),+) => {
        Row::new(vec![$(Cell::from($header)),+])
    };
}

/// Macro for simple text-based views
#[macro_export]
macro_rules! impl_text_view {
    // Without auto-refresh
    ($name:ident, $text_render_fn:expr, $title:expr) => {
        $crate::impl_text_view!(@internal $name, $text_render_fn, $title, None);
    };

    // With auto-refresh interval
    ($name:ident, $text_render_fn:expr, $title:expr, auto_refresh: $interval:expr) => {
        $crate::impl_text_view!(@internal $name, $text_render_fn, $title, Some($interval));
    };

    // Internal implementation
    (@internal $name:ident, $text_render_fn:expr, $title:expr, $interval:expr) => {
        pub struct $name {
            data: $crate::ui::AsyncData<Result<Vec<ratatui::text::Line<'static>>, String>>,
        }

        impl $name {
            pub fn new(controller: &Controller) -> Self {
                Self {
                    data: Self::load_data(controller),
                }
            }

            fn load_data(controller: &Controller) -> $crate::ui::AsyncData<Result<Vec<ratatui::text::Line<'static>>, String>> {
                let (tx, rx) = std::sync::mpsc::channel();

                // Execute the render function and capture result
                let result = $text_render_fn(controller).map_err(|e| format!("{}", e));

                // Spawn a thread to send the result
                std::thread::spawn(move || {
                    let _ = tx.send(Ok(result));
                });

                $crate::ui::AsyncData::new(rx)
            }
        }

        impl $crate::ui::views::RenderableView for $name {
            fn title(&self) -> &str {
                $title
            }

            fn auto_refresh_interval(&self) -> Option<f32> {
                $interval
            }

            fn update(&mut self, _controller: &Controller, _keys: &[crossterm::event::KeyCode]) {
                self.data.try_update();
            }

            fn refresh_data(&mut self, _controller: &Controller) -> Result<(), String> {
                self.data = Self::load_data(_controller);
                Ok(())
            }

            fn render(&self, rc: $crate::ui::RenderContext) -> $crate::ui::ViewResult {
                // Check if still loading
                if self.data.is_loading() {
                    let loading_text = vec![$crate::styled_line!("Loading data...")];
                    let paragraph = ratatui::widgets::Paragraph::new(loading_text)
                        .block(rc.block)
                        .wrap(ratatui::widgets::Wrap { trim: false });
                    rc.frame.render_widget(paragraph, rc.area);
                    return Ok(());
                }

                // Check for errors
                if let Some(err) = self.data.error() {
                    rc.error(err);
                    return Ok(());
                }

                // Get the loaded data
                let Some(result) = self.data.get_data() else {
                    rc.error("Data not available");
                    return Ok(());
                };

                // Check if the result itself is an error
                let lines = match result {
                    Ok(lines) => lines,
                    Err(e) => {
                        rc.error(e);
                        return Ok(());
                    }
                };

                let paragraph = ratatui::widgets::Paragraph::new(lines.clone())
                    .block(rc.block)
                    .wrap(ratatui::widgets::Wrap { trim: false })
                    .scroll((rc.scroll_offset, 0));

                rc.frame.render_widget(paragraph, rc.area);
                Ok(())
            }
        }
    };
}
