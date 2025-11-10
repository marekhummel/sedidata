use crate::{
    empty_row, fill_row, header_row, impl_text_view,
    model::{
        champion::Champion,
        mastery::{Mastery, Milestone},
    },
    styled_line, styled_span,
    ui::{
        views::{eval_color_scale_ascending, eval_color_scale_descending, RenderableView},
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

use crossterm::event::KeyCode;
use itertools::Itertools;
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Cell, Paragraph, Row, Table},
};

pub struct NextMasteryView {
    data: Vec<(Mastery, Champion)>,
    error: Option<String>,
    grouping_enabled: bool,
}

impl NextMasteryView {
    pub fn new(controller: &Controller, lvl_range: Vec<u16>) -> Self {
        match Self::load_masteries(controller, lvl_range) {
            Ok(data) => Self {
                data,
                error: None,
                grouping_enabled: true,
            },
            Err(e) => Self {
                data: Vec::new(),
                error: Some(format!("Failed to load masteries: {}", e)),
                grouping_enabled: true,
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

    fn columns(&self) -> [Constraint; 7] {
        [
            Constraint::Length(20), // Champion
            Constraint::Length(10), // Roles (3 chars * 3 roles + spaces)
            Constraint::Length(4),  // Level
            Constraint::Length(22), // Points (right padded)
            Constraint::Length(8),  // Missing (right padded)
            Constraint::Length(8),  // Marks (right padded)
            Constraint::Min(20),    // Next Milestone
        ]
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

    fn format_milestone(milestone: &Milestone) -> String {
        let grades = milestone
            .require_grade_counts
            .iter()
            .map(|(grade, count)| format!("{}x {}", count, grade))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{} ⇒ {} mark(s)", grades, milestone.reward_marks)
    }

    fn render_row<'a>(&'a self, mastery: &'a Mastery, champ: &'a Champion, max_points_log: usize) -> Row<'a> {
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

        Row::new(vec![
            Cell::from(champ.name.clone()),
            Cell::from(styled_line!(VAR role_spans)),
            Cell::from(styled_line!(mastery.level).alignment(Alignment::Right)),
            Cell::from(
                styled_line!(LIST [
                    styled_span!(mastery.points),
                    styled_span!(" / {:>1$}", mastery.required_points(), max_points_log; Color::DarkGray),
                ])
                .alignment(Alignment::Right),
            ),
            Cell::from(styled_line!("{}", missing; points_color).alignment(Alignment::Right)),
            Cell::from(
                styled_line!("{}/{}", mastery.marks, mastery.required_marks; marks_color).alignment(Alignment::Right),
            ),
            Cell::from(Self::format_milestone(&mastery.next_milestone)),
        ])
    }
}

impl RenderableView for NextMasteryView {
    fn title(&self) -> &str {
        "Mastery Level X Champions"
    }

    fn interact(&mut self, keys: &[KeyCode]) {
        if keys.contains(&KeyCode::Char('g')) {
            self.grouping_enabled = !self.grouping_enabled;
        }
    }

    fn render(&self, rc: crate::ui::RenderContext) -> crate::ui::ViewResult {
        if let Some(error) = &self.error {
            let paragraph = Paragraph::new(format!("\n  [!] Error: {}", error)).block(rc.block);
            rc.frame.render_widget(paragraph, rc.area);
            return Ok(());
        }

        let mut rows = vec![];
        if self.grouping_enabled {
            // Group champions by role (each champion can appear in multiple groups)
            let role_order = vec!["assassin", "fighter", "mage", "marksman", "support", "tank"];

            for role in &role_order {
                let champions_with_role: Vec<_> = self.data.iter().filter(|(_, champ)| champ.has_role(role)).collect();
                if champions_with_role.is_empty() {
                    continue;
                }

                // Add role header row
                rows.push(
                    fill_row!(7; Cell::from(styled_line!(LIST [
                        styled_span!("━━ "),
                        Self::role_abbreviation(role),
                        styled_span!(" ━━"),
                    ])))
                    .style(Style::default().add_modifier(Modifier::BOLD)),
                );

                // Add champion rows for this role
                let max_points_log = champions_with_role
                    .iter()
                    .map(|(m, _)| (m.points as f32).log10().ceil() as usize)
                    .max()
                    .unwrap();
                for (mastery, champ) in champions_with_role {
                    rows.push(self.render_row(mastery, champ, max_points_log));
                }

                // Add empty row after each role group
                rows.push(empty_row!(7));
            }
        } else {
            // Add champion rows for this role
            let max_points_log = self
                .data
                .iter()
                .map(|(m, _)| (m.points as f32).log10().ceil() as usize)
                .max()
                .unwrap();
            for (mastery, champ) in &self.data {
                rows.push(self.render_row(mastery, champ, max_points_log));
            }
        }

        // Skip rows based on scroll offset for manual scrolling
        let visible_rows: Vec<_> = rows.into_iter().skip(rc.scroll_offset as usize).collect();

        let table = Table::new(visible_rows, self.columns())
            .header(
                header_row!(
                    "Champion",
                    "Roles",
                    "Lvl",
                    "Points",
                    "Missing",
                    "Marks",
                    "Next Milestone"
                )
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .bottom_margin(1),
            )
            .block(rc.block.title("(G to toggle grouping)"))
            .column_spacing(2)
            .style(Style::default().fg(Color::White));

        rc.frame.render_widget(table, rc.area);
        Ok(())
    }
}
