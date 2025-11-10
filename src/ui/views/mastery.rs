use crate::{
    impl_text_view,
    model::{champion::Champion, mastery::Mastery},
    styled_line, styled_span,
    ui::{
        views::{eval_color_scale_ascending, eval_color_scale_descending},
        Controller, TextCreationResult,
    },
};

// ============================================================================
// Unplayed Champions View
// ============================================================================

fn unplayed_champs_view(ctrl: &Controller) -> TextCreationResult {
    let champs = ctrl.manager.get_champions()?;
    let played_champs = ctrl.util.get_played_champions_set()?;

    let mut unplayed = champs
        .iter()
        .filter(|c| !played_champs.contains(&c.id))
        .collect::<Vec<_>>();
    unplayed.sort_by_key(|c| c.name.as_str());

    let mut lines = vec![styled_line!()];

    for c in &unplayed {
        lines.push(styled_line!("  {}", c.name));
    }

    lines.push(styled_line!());
    lines.push(styled_line!("{} champ(s) total", unplayed.len(); Color::Cyan));
    Ok(lines)
}

impl_text_view!(
    UnplayedChampsView,
    unplayed_champs_view,
    "Unplayed Champions",
    "Unplayed Champions"
);

// ============================================================================
// Next mastery view
// ============================================================================

use itertools::Itertools;
use ratatui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Cell, Row, Table},
};

pub struct NextMasteryView {
    // Store all the data we need as owned values for display
    data: Vec<(Mastery, Champion)>, // (name, roles, level, points, req_points, missing, marks, req_marks, milestone)
    error: Option<String>,
}

impl NextMasteryView {
    pub fn new(controller: &Controller, lvl_range: Vec<u16>) -> Self {
        match Self::load_masteries(controller, lvl_range) {
            Ok(data) => Self { data, error: None },
            Err(e) => Self {
                data: Vec::new(),
                error: Some(format!("Failed to load masteries: {}", e)),
            },
        }
    }

    fn load_masteries(
        ctrl: &Controller,
        lvl_range: Vec<u16>,
    ) -> Result<Vec<(Mastery, Champion)>, crate::ui::ViewError> {
        let mut masteries = ctrl.util.get_masteries_with_level(lvl_range)?;
        masteries.sort_by_key(|m| (m.level, m.points));
        masteries.reverse();

        let mut result = Vec::new();
        for mastery in masteries {
            let champ = ctrl.lookup.get_champion(&mastery.champ_id)?;
            result.push((mastery.clone(), champ.clone()));
        }

        Ok(result)
    }

    fn role_abbreviation(role: &str) -> Span<'_> {
        match role.to_lowercase().as_str() {
            "assassin" => styled_span!("ASS"; Color::Red),
            "mage" => styled_span!("MGE"; Color::Blue),
            "tank" => styled_span!("TNK"; Color::Green),
            "fighter" => styled_span!("FGT"; Color::Yellow),
            "marksman" => styled_span!("MRK"; Color::Cyan),
            "support" => styled_span!("SUP"; Color::Magenta),
            _ => styled_span!("???"; Color::White),
        }
    }

    fn columns(&self) -> [Constraint; 7] {
        [
            Constraint::Length(25), // Champion
            Constraint::Length(15), // Roles (3 chars * 3 roles + spaces)
            Constraint::Length(4),  // Level
            Constraint::Length(22), // Points (right padded)
            Constraint::Length(8),  // Missing (right padded)
            Constraint::Length(8),  // Marks (right padded)
            Constraint::Min(30),    // Next Milestone
        ]
    }

    fn missing_points_scale(&self) -> Vec<(i32, Color)> {
        vec![
            (0, Color::Green),
            (2000, Color::Rgb(200, 255, 100)),
            (5000, Color::Yellow),
            (i32::MAX, Color::White),
        ]
    }

    fn progress_scale(&self) -> Vec<(f32, Color)> {
        vec![
            (0.99, Color::Green),
            (0.7, Color::Rgb(200, 255, 100)),
            (0.5, Color::Yellow),
            (0.0, Color::White),
        ]
    }

    fn format_milestone(milestone: &crate::model::mastery::Milestone) -> String {
        let grades = milestone
            .require_grade_counts
            .iter()
            .map(|(grade, count)| format!("{}x {}", count, grade))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{} ⇒ {} mark(s)", grades, milestone.reward_marks)
    }
}

impl crate::ui::views::RenderableView for NextMasteryView {
    fn title(&self) -> &str {
        "Next Mastery Champions"
    }

    fn render(&self, rc: crate::ui::RenderContext) -> crate::ui::ViewResult {
        if let Some(error) = &self.error {
            let paragraph = ratatui::widgets::Paragraph::new(format!("\n  [!] Error: {}", error)).block(rc.block);
            rc.frame.render_widget(paragraph, rc.area);
            return Ok(());
        }

        // Group champions by role (each champion can appear in multiple groups)
        let role_order = vec!["assassin", "fighter", "mage", "marksman", "support", "tank"];
        let mut rows = vec![];

        for role in &role_order {
            // Find all champions that have this role
            let champions_with_role: Vec<_> = self
                .data
                .iter()
                .filter(|(_, champ)| champ.roles.iter().any(|r| r.to_lowercase() == *role))
                .collect();

            if champions_with_role.is_empty() {
                continue;
            }

            // Add role header row
            let role_abbr_span = Self::role_abbreviation(role);
            rows.push(
                Row::new(vec![
                    Cell::from(styled_line!(LIST [
                        styled_span!("━━ "),
                        role_abbr_span,
                        styled_span!(" ━━"),
                    ])),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ])
                .style(Style::default().add_modifier(Modifier::BOLD)),
            );

            // Add champion rows for this role
            for (mastery, champ) in champions_with_role {
                // Create colored role abbreviations
                let role_spans: Vec<Span> = champ
                    .roles
                    .iter()
                    .sorted_by_key(|r| *r)
                    .map(|r| Self::role_abbreviation(r.trim()))
                    .enumerate()
                    .flat_map(|(i, span)| if i > 0 { vec![Span::raw(" "), span] } else { vec![span] })
                    .collect();

                // Color code missing points
                let missing = mastery.missing_points.max(0);
                let points_color = eval_color_scale_ascending(missing, &self.missing_points_scale());

                // Color code marks progress
                let marks_progress = mastery.marks as f32 / mastery.required_marks as f32;
                let marks_color = eval_color_scale_descending(marks_progress, &self.progress_scale());

                rows.push(Row::new(vec![
                    Cell::from(champ.name.clone()),
                    Cell::from(styled_line!(VAR role_spans)),
                    Cell::from(format!("{:>3}", mastery.level)),
                    Cell::from(styled_line!(LIST [
                        styled_span!("{:>8}", mastery.points),
                        styled_span!(" / {:<8}", mastery.required_points(); if missing == 0  { Color::DarkGray } else { Color::White }),
                    ])),
                    Cell::from(styled_line!("{:>6}", missing; points_color)),
                    Cell::from(styled_line!("{:>2}/{:<2}", mastery.marks, mastery.required_marks; marks_color)),
                    Cell::from(Self::format_milestone(&mastery.next_milestone)),
                ]));
            }

            // Add empty row after each role group
            rows.push(Row::new(vec![
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
                Cell::from(""),
            ]));
        }

        // Skip rows based on scroll offset for manual scrolling
        let visible_rows: Vec<_> = rows.into_iter().skip(rc.scroll_offset as usize).collect();

        let table = Table::new(visible_rows, self.columns())
            .header(
                Row::new(vec![
                    Cell::from("Champion"),
                    Cell::from("Roles"),
                    Cell::from("Lvl"),
                    Cell::from("Points"),
                    Cell::from("Missing"),
                    Cell::from("Marks"),
                    Cell::from("Next Milestone"),
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
