use crate::{
    empty_row, fill_row, header_row,
    ui::{
        views::{eval_color_scale_descending, RenderableView},
        Controller,
    },
};
use crossterm::event::KeyCode;
use itertools::Itertools;
use ratatui::{text::Line, widgets::Paragraph};
use std::cmp::Ordering;

// ============================================================================
// Challenges Overview View
// ============================================================================

use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Padding, Row, Table},
};

pub struct ChallengesOverviewView {
    categories: Vec<(String, Vec<crate::model::challenge::Challenge>)>,
    error: Option<String>,
}

impl ChallengesOverviewView {
    pub fn new(controller: &Controller) -> Self {
        match Self::load_challenges(controller) {
            Ok(categories) => Self {
                categories,
                error: None,
            },
            Err(e) => Self {
                categories: Vec::new(),
                error: Some(format!("Failed to load challenges: {}", e)),
            },
        }
    }

    fn truncate_with_ellipsis(text: &str, max_len: usize) -> String {
        if text.chars().count() <= max_len {
            text.to_string()
        } else {
            let truncated: String = text.chars().take(max_len.saturating_sub(3)).collect();
            format!("{}...", truncated)
        }
    }

    fn load_challenges(
        ctrl: &Controller,
    ) -> Result<Vec<(String, Vec<crate::model::challenge::Challenge>)>, crate::ui::ViewError> {
        let mut challenges = ctrl.manager.get_challenges()?.to_vec();
        challenges.retain(|c| !c.is_capstone && !c.is_completed() && c.category != "LEGACY");

        // Sort by category first, then by progress percentage (descending) and points to next (ascending)
        challenges.sort_by(|a, b| {
            let progress_a = (a.current_value / a.threshold_value * 100.0).min(100.0);
            let progress_b = (b.current_value / b.threshold_value * 100.0).min(100.0);

            a.category
                .cmp(&b.category)
                .then_with(|| a.gamemode_short().cmp(b.gamemode_short()).reverse())
                .then_with(|| progress_a.partial_cmp(&progress_b).unwrap_or(Ordering::Equal).reverse())
                .then_with(|| a.points_to_next().cmp(&b.points_to_next()))
        });

        let mut categories = Vec::new();
        for (category, iterator) in &challenges.iter().chunk_by(|c| c.category.clone()) {
            let challenges_in_category: Vec<_> = iterator.cloned().collect();
            categories.push((category, challenges_in_category));
        }

        Ok(categories)
    }

    fn columns(&self) -> [Constraint; 5] {
        [
            Constraint::Length(32), // Name
            Constraint::Min(40),    // Description (takes remaining space)
            Constraint::Length(6),  // Game mode
            Constraint::Length(6),  // Points to next
            Constraint::Length(12), // Progress (current/threshold)
        ]
    }

    fn points_scale(&self) -> Vec<(u16, Color)> {
        vec![
            (95, Color::Rgb(0, 255, 0)),
            (80, Color::Rgb(100, 255, 100)),
            (65, Color::Rgb(150, 255, 100)),
            (50, Color::Rgb(200, 255, 100)),
            (35, Color::Rgb(255, 255, 100)),
            (20, Color::Rgb(255, 220, 100)),
            (10, Color::Rgb(255, 200, 150)),
            (0, Color::White),
        ]
    }

    fn progress_scale(&self) -> Vec<(f32, Color)> {
        vec![
            (95.0, Color::Rgb(0, 255, 0)),
            (90.0, Color::Rgb(100, 255, 100)),
            (70.0, Color::Rgb(150, 255, 100)),
            (60.0, Color::Rgb(200, 255, 100)),
            (50.0, Color::Rgb(255, 255, 100)),
            (40.0, Color::Rgb(255, 220, 100)),
            (20.0, Color::Rgb(255, 200, 150)),
            (0.0, Color::White),
        ]
    }
}

impl RenderableView for ChallengesOverviewView {
    fn title(&self) -> &str {
        "Challenges Overview"
    }

    fn render(&self, rc: crate::ui::RenderContext) -> crate::ui::ViewResult {
        if let Some(error) = &self.error {
            let paragraph = Paragraph::new(format!("\n  [!] Error: {}", error)).block(rc.block);
            rc.frame.render_widget(paragraph, rc.area);
            return Ok(());
        }

        // Build rows for the table
        let mut rows = Vec::new();

        for (category, challenges) in &self.categories {
            // Add category header row spanning multiple columns
            rows.push(
                fill_row!(5; Cell::from(Line::from(vec![Span::styled(
                    format!("━━ {} ━━", category),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )])))
                .style(Style::default().fg(Color::Cyan)),
            );

            // Add challenge rows
            for challenge in challenges {
                let points_to_next = challenge.points_to_next();
                let points_color = eval_color_scale_descending(points_to_next, &self.points_scale());

                // Determine game mode display with color coding
                let gamemode_cell = match challenge.gamemode_short() {
                    "SR" => Cell::from(Span::styled("SR", Style::default().fg(Color::Rgb(210, 180, 140)))),
                    "HA" => Cell::from(Span::styled("HA", Style::default().fg(Color::Cyan))),
                    _ => Cell::from("-"),
                };

                // Color code progress based on completion percentage (traffic light gradient)
                let progress_pct = (challenge.current_value / challenge.threshold_value * 100.0).min(100.0);
                let progress_color = eval_color_scale_descending(progress_pct, &self.progress_scale());

                // Calculate available width for description
                // Total terminal width minus: Columns + borders/padding
                let desc_width = rc.area.width.saturating_sub(32 + 6 + 6 + 12 + 12).max(40) as usize;
                let truncated_desc = Self::truncate_with_ellipsis(&challenge.description, desc_width);

                rows.push(Row::new(vec![
                    Cell::from(challenge.name.clone()),
                    Cell::from(truncated_desc),
                    gamemode_cell,
                    Cell::from(Span::styled(
                        format!("{}", points_to_next),
                        Style::default().fg(points_color),
                    )),
                    Cell::from(Span::styled(
                        format!("{:.0}/{:.0}", challenge.current_value, challenge.threshold_value),
                        Style::default().fg(progress_color),
                    )),
                ]));
            }

            // Empty row after each category
            rows.push(empty_row!(5));
        }

        // Skip rows based on scroll offset for manual scrolling
        let visible_rows: Vec<_> = rows.into_iter().skip(rc.scroll_offset as usize).collect();

        let table = Table::new(visible_rows, self.columns())
            .header(
                header_row!("Challenge", "Description", "Mode", "Points", "Progress")
                    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    .bottom_margin(1),
            )
            .block(rc.block)
            .column_spacing(2)
            .style(Style::default().fg(Color::White));

        rc.frame.render_widget(table, rc.area);
        Ok(())
    }
}
