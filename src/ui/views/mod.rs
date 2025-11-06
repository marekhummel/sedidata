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
}

/// Macro for simple text-based views
#[macro_export]
macro_rules! impl_text_view {
    ($name:ident, $text_render_fn:expr, $title:expr, $description:expr) => {
        pub struct $name {
            content: String,
        }

        impl $name {
            pub fn new(controller: &Controller) -> Self {
                match $text_render_fn(controller) {
                    Ok(content) => Self { content },
                    Err(e) => Self {
                        content: format!("\n  [!] Error: {}", e),
                    },
                }
            }
        }

        impl $crate::ui::views::RenderableView for $name {
            fn render(&self, rc: $crate::ui::RenderContext) -> $crate::ui::ViewResult {
                let lines: Vec<ratatui::text::Line> = self
                    .content
                    .lines()
                    .map(|s| ratatui::text::Line::from(s.to_string()))
                    .collect();
                let paragraph = ratatui::widgets::Paragraph::new(lines)
                    .block(
                        ratatui::widgets::Block::default()
                            .borders(ratatui::widgets::Borders::ALL)
                            .padding(ratatui::widgets::Padding::horizontal(1))
                            .title(format!(
                                "{} (↑/↓ or PgUp/PgDown to scroll, Esc to return)",
                                $title
                            )),
                    )
                    .wrap(ratatui::widgets::Wrap { trim: false })
                    .scroll((rc.scroll_offset, 0));

                rc.frame.render_widget(paragraph, rc.area);
                Ok(())
            }
        }
    };
}
