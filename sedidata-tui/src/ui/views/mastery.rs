use crate::{
    empty_row, fill_row, header_row, impl_text_view,
    model::{
        champion::Champion,
        mastery::{Mastery, Milestone},
    },
    styled_line, styled_span,
    ui::{
        views::{eval_color_scale_ascending, eval_color_scale_descending, RenderableView},
        Controller, RenderContext, TextCreationResult, ViewError, ViewResult,
    },
};
use crossterm::event::KeyCode;
use itertools::Itertools;
use ratatui::{
    layout::{Alignment, Constraint},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Cell, Row, Table},
};

// ============================================================================
// Unplayed Champions View
// ============================================================================

fn unplayed_champs_view(ctrl: &Controller) -> TextCreationResult {
    let champs = ctrl.manager.get_champions().recv().unwrap()?;
    let played_champs = ctrl.util.get_played_champions_set().recv().unwrap()?;

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
    lines.push(styled_line!("{} champ(s) total", unplayed.len(); Color::Rgb(200, 150, 0)));
    Ok(lines)
}

impl_text_view!(UnplayedChampsView, unplayed_champs_view, "Unplayed Champions");

// ============================================================================
// All Masteries View
// ============================================================================

struct MasteryView {
    data: Vec<(Mastery, Champion)>,
    error: Option<String>,
    with_role: bool,
    grouping_enabled: bool,
    sorting_enabled: bool,
    grouping_status: bool,
    sorting_status: u8,
}

impl MasteryView {
    pub fn new(grouping: bool, sorting: bool, with_role: bool) -> Self {
        Self {
            data: Vec::new(),
            error: None,
            with_role,
            grouping_enabled: grouping,
            sorting_enabled: sorting,
            grouping_status: false,
            sorting_status: 0,
        }
    }

    pub fn load_masteries(&mut self, ctrl: &Controller, lvl_range: Option<Vec<u16>>) {
        let masteries: Result<Vec<(Mastery, Champion)>, ViewError> = (|| {
            let mastery_list = match lvl_range {
                None => ctrl.manager.get_masteries().recv().unwrap()?.to_vec(),
                Some(rng) => ctrl.util.get_masteries_with_level(rng).recv().unwrap()?,
            };

            let mut result = Vec::new();
            for mastery in mastery_list {
                let champ = ctrl.lookup.get_champion(&mastery.champ_id)?;
                result.push((mastery.clone(), champ.clone()));
            }
            result.sort_by_key(|(m, _)| (m.level, m.points));
            result.reverse();

            Ok(result)
        })();

        match masteries {
            Ok(data) => self.data = data,
            Err(e) => self.error = Some(format!("Failed to load masteries: {}", e)),
        }
    }

    pub fn check_keys(&mut self, keys: &[KeyCode]) {
        if self.grouping_enabled && keys.contains(&KeyCode::Char('g')) {
            self.grouping_status = !self.grouping_status;
        }

        if self.sorting_enabled && keys.contains(&KeyCode::Char('s')) {
            self.sorting_status = (self.sorting_status + 1) % 2;
            match self.sorting_status {
                0 => {
                    self.data.sort_by_key(|(m, _)| (m.level, m.points));
                    self.data.reverse();
                }
                1 => {
                    self.data.sort_by_key(|(_, champ)| champ.name.clone());
                }
                _ => unreachable!(),
            }
        }
    }

    fn columns(&self) -> Vec<Constraint> {
        if self.with_role {
            vec![
                Constraint::Length(20), // Champion
                Constraint::Length(10), // Roles (3 chars * 3 roles + spaces)
                Constraint::Length(4),  // Level
                Constraint::Length(20), // Points (right padded)
                Constraint::Length(8),  // Missing (right padded)
                Constraint::Length(10), // Marks (right padded)
                Constraint::Min(20),    // Next Milestone
            ]
        } else {
            vec![
                Constraint::Length(20), // Champion
                Constraint::Length(4),  // Level
                Constraint::Length(20), // Points (right padded)
                Constraint::Length(8),  // Missing (right padded)
                Constraint::Length(10), // Marks (right padded)
                Constraint::Min(20),    // Next Milestone
            ]
        }
    }

    pub fn render(&self, rc: RenderContext) -> ViewResult {
        if let Some(error) = &self.error {
            rc.error(error);
            return Ok(());
        }

        let mut rows = vec![];
        if self.grouping_status {
            assert!(self.with_role, "Grouping by role requires 'with_role' to be enabled.");

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
                        Self::format_role(role, false),
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
            if !self.data.is_empty() {
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
        }

        // Skip rows based on scroll offset for manual scrolling
        let visible_rows: Vec<_> = rows.into_iter().skip(rc.scroll_offset as usize).collect();

        let table = Table::new(visible_rows, self.columns())
            .header(
                self.header_row()
                    .style(Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD))
                    .bottom_margin(1),
            )
            .block(self.modify_block(rc.block))
            .column_spacing(2)
            .style(Style::default().fg(Color::White));

        rc.frame.render_widget(table, rc.area);
        Ok(())
    }

    fn format_role(role: &str, abbreviate: bool) -> Span<'_> {
        match role.to_lowercase().as_str() {
            "assassin" => styled_span!(if abbreviate { "ASS" } else { "Assassin" }; Color::Red),
            "mage" => styled_span!(if abbreviate { "MGE" } else { "Mage" }; Color::Blue),
            "tank" => styled_span!(if abbreviate { "TNK" } else { "Tank" }; Color::Green),
            "fighter" => styled_span!(if abbreviate { "FGT" } else { "Fighter" }; Color::Yellow),
            "marksman" => styled_span!(if abbreviate { "MRK" } else { "Marksman" }; Color::Cyan),
            "support" => styled_span!(if abbreviate { "SUP" } else { "Support" }; Color::Magenta),
            _ => styled_span!("???"; Color::White),
        }
    }

    fn missing_points_scale() -> Vec<(i32, Color)> {
        vec![
            (0, Color::Green),
            (2000, Color::Rgb(200, 255, 100)),
            (5000, Color::Yellow),
            (i32::MAX, Color::White),
        ]
    }

    fn progress_scale() -> Vec<(f32, Color)> {
        vec![
            (0.99, Color::Green),
            (0.7, Color::Rgb(200, 255, 100)),
            (0.5, Color::Yellow),
            (0.0, Color::White),
        ]
    }

    fn render_row<'a>(&'a self, mastery: &'a Mastery, champ: &'a Champion, max_points_log: usize) -> Row<'a> {
        // Color code missing points
        let missing = mastery.missing_points.max(0);
        let points_color = eval_color_scale_ascending(missing, &MasteryView::missing_points_scale());

        // Color code marks progress
        let marks_progress = mastery.marks as f32 / mastery.required_marks as f32;
        let marks_color = eval_color_scale_descending(marks_progress, &MasteryView::progress_scale());
        let marks_line = (mastery.required_marks > 0)
            .then(|| styled_line!("{}/{}", mastery.marks, mastery.required_marks; marks_color))
            .unwrap_or(styled_line!(""));

        let milestone = (mastery.required_marks > 0)
            .then(|| styled_line!(Self::format_milestone(&mastery.next_milestone)))
            .unwrap_or(styled_line!(""));

        // Cells
        let mut cells = vec![
            Cell::from(champ.name.clone()),
            Cell::from(styled_line!(mastery.level).alignment(Alignment::Right)),
            Cell::from(
                styled_line!(LIST [
                    styled_span!(mastery.points),
                    styled_span!(" / {:>1$}", mastery.required_points(), max_points_log; Color::DarkGray),
                ])
                .alignment(Alignment::Right),
            ),
            Cell::from(styled_line!("{}", missing; points_color).alignment(Alignment::Right)),
            Cell::from(marks_line.alignment(Alignment::Right)),
            Cell::from(milestone),
        ];

        if self.with_role {
            // Create colored role abbreviations
            let role_spans: Vec<Span> = champ
                .roles
                .iter()
                .sorted_by_key(|r| *r)
                .map(|r| Self::format_role(r.trim(), true))
                .enumerate()
                .flat_map(|(i, span)| if i > 0 { vec![Span::raw(" "), span] } else { vec![span] })
                .collect();

            cells.insert(1, Cell::from(styled_line!(VAR role_spans)));
        }

        Row::new(cells)
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

    fn header_row<'a>(&'a self) -> Row<'a> {
        if self.with_role {
            if self.sorting_enabled {
                match self.sorting_status {
                    0 => header_row!(
                        "Champion",
                        "Roles",
                        "Lvl↓",
                        "Points↓",
                        "Missing",
                        "Marks",
                        "Next Milestone"
                    ),
                    1 => header_row!(
                        "Champion↑",
                        "Roles",
                        "Lvl",
                        "Points",
                        "Missing",
                        "Marks",
                        "Next Milestone"
                    ),
                    _ => unreachable!(),
                }
            } else {
                header_row!(
                    "Champion",
                    "Roles",
                    "Lvl",
                    "Points",
                    "Missing",
                    "Marks",
                    "Next Milestone"
                )
            }
        } else if self.sorting_enabled {
            match self.sorting_status {
                0 => header_row!("Champion", "Lvl↓", "Points↓", "Missing", "Marks", "Next Milestone"),
                1 => header_row!("Champion↑", "Lvl", "Points", "Missing", "Marks", "Next Milestone"),
                _ => unreachable!(),
            }
        } else {
            header_row!("Champion", "Lvl", "Points", "Missing", "Marks", "Next Milestone")
        }
    }

    fn modify_block<'a>(&'a self, block: Block<'a>) -> Block<'a> {
        let mut block_mod = block;
        if self.grouping_enabled {
            block_mod = block_mod.title("(G to toggle grouping) ");
        }
        if self.sorting_enabled {
            block_mod = block_mod.title("(S to toggle sorting) ");
        }

        block_mod
    }
}

pub struct AllMasteriesView {
    internal: MasteryView,
}

impl AllMasteriesView {
    pub fn new(controller: &Controller) -> Self {
        let mut internal = MasteryView::new(false, true, false);
        internal.load_masteries(controller, None);
        Self { internal }
    }
}

impl RenderableView for AllMasteriesView {
    fn title(&self) -> &str {
        "All Masteries"
    }

    fn update(&mut self, _controller: &Controller, keys: &[KeyCode]) {
        self.internal.check_keys(keys);
    }

    fn render(&self, rc: RenderContext) -> ViewResult {
        self.internal.render(rc)
    }
}

// ============================================================================
// Next mastery view
// ============================================================================

pub struct NextMasteryView {
    internal: MasteryView,
    title: String,
}

impl NextMasteryView {
    pub fn new(controller: &Controller, lvl_range: Vec<u16>, title_range: &str) -> Self {
        let mut internal = MasteryView::new(true, false, true);
        internal.load_masteries(controller, Some(lvl_range));
        Self {
            internal,
            title: format!("Mastery Level {} Champions", title_range),
        }
    }
}

impl RenderableView for NextMasteryView {
    fn title(&self) -> &str {
        self.title.as_str()
    }

    fn update(&mut self, _controller: &Controller, keys: &[KeyCode]) {
        self.internal.check_keys(keys);
    }

    fn render(&self, rc: RenderContext) -> ViewResult {
        self.internal.render(rc)
    }
}
