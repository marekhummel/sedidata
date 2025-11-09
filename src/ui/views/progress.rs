use crate::ui::Controller;
use itertools::Itertools;
use ratatui::text::Line;
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
                error: Some(format!("{}", e)),
            },
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
}

impl crate::ui::views::RenderableView for ChallengesOverviewView {
    fn title(&self) -> &str {
        "Challenges Overview"
    }

    fn render(&self, rc: crate::ui::RenderContext) -> crate::ui::ViewResult {
        if let Some(error) = &self.error {
            let paragraph = ratatui::widgets::Paragraph::new(format!("\n  [!] Error: {}", error)).block(
                Block::default()
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1))
                    .title("Challenges Overview (↑/↓ or PgUp/PgDown to scroll, Esc to return)"),
            );
            rc.frame.render_widget(paragraph, rc.area);
            return Ok(());
        }

        // Build rows for the table
        let mut rows = Vec::new();

        for (category, challenges) in &self.categories {
            // Add category header row spanning multiple columns
            rows.push(
                Row::new(vec![
                    Cell::from(Line::from(vec![Span::styled(
                        format!("━━ {} ━━", category),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )])),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])
                .style(Style::default().fg(Color::Cyan)),
            );

            // Add challenge rows
            for challenge in challenges {
                let points_to_next = challenge.points_to_next();

                // Determine game mode display with color coding
                let gamemode_cell = match challenge.gamemode_short() {
                    "SR" => Cell::from(Span::styled("SR", Style::default().fg(Color::Rgb(210, 180, 140)))), // Tan/light brown
                    "HA" => Cell::from(Span::styled("HA", Style::default().fg(Color::Cyan))),
                    _ => Cell::from("-"),
                };

                // Color code points based on value (traffic light gradient)
                // Higher points = better (green), lower points = less valuable (white)
                let points_color = if points_to_next >= 95 {
                    Color::Rgb(0, 255, 0) // 95-100+: Bright green - highest value!
                } else if points_to_next >= 80 {
                    Color::Rgb(100, 255, 100) // 80-95: Green
                } else if points_to_next >= 65 {
                    Color::Rgb(150, 255, 100) // 65-80: Light green
                } else if points_to_next >= 50 {
                    Color::Rgb(200, 255, 100) // 50-65: Yellow-green
                } else if points_to_next >= 35 {
                    Color::Rgb(255, 255, 100) // 35-50: Yellow
                } else if points_to_next >= 20 {
                    Color::Rgb(255, 220, 100) // 20-35: Light yellow
                } else if points_to_next >= 10 {
                    Color::Rgb(255, 200, 150) // 10-20: Cream
                } else {
                    Color::White // 0-10: White
                };

                // Color code progress based on completion percentage (traffic light gradient)
                let progress_pct = (challenge.current_value / challenge.threshold_value * 100.0).min(100.0);
                let progress_color = if progress_pct >= 95.0 {
                    Color::Rgb(0, 255, 0) // 95-100%: Bright green - almost done!
                } else if progress_pct >= 90.0 {
                    Color::Rgb(100, 255, 100) // 90-95%: Green
                } else if progress_pct >= 70.0 {
                    Color::Rgb(150, 255, 100) // 80-90%: Light green
                } else if progress_pct >= 60.0 {
                    Color::Rgb(200, 255, 100) // 70-80%: Yellow-green
                } else if progress_pct >= 50.0 {
                    Color::Rgb(255, 255, 100) // 60-70%: Yellow
                } else if progress_pct >= 40.0 {
                    Color::Rgb(255, 220, 100) // 50-60%: Light yellow
                } else if progress_pct >= 20.0 {
                    Color::Rgb(255, 200, 150) // 40-50%: Cream
                } else {
                    Color::White // 0-40%: White
                };

                rows.push(Row::new(vec![
                    Cell::from(challenge.name.clone()),
                    Cell::from(challenge.description.clone()),
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
            rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ]));
        }

        let widths = [
            Constraint::Length(32), // Name
            Constraint::Min(40),    // Description (takes remaining space)
            Constraint::Length(6),  // Game mode
            Constraint::Length(6),  // Points to next
            Constraint::Length(12), // Progress (current/threshold)
        ];

        // Skip rows based on scroll offset for manual scrolling
        let visible_rows: Vec<_> = rows.into_iter().skip(rc.scroll_offset as usize).collect();

        let table = Table::new(visible_rows, widths)
            .header(
                Row::new(vec![
                    Cell::from("Challenge"),
                    Cell::from("Description"),
                    Cell::from("Mode"),
                    Cell::from("Points"),
                    Cell::from("Progress"),
                ])
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
