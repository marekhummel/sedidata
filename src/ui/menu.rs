use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::ui::{views::*, Controller};

pub struct Menu {
    menu_entries: Vec<MenuEntry>,
    selected: usize,
}

struct MenuEntry {
    description: &'static str,
    factory: Option<ViewFactory>,
}

type ViewFactory = fn(&Controller) -> Box<dyn RenderableView>;

impl Menu {
    pub fn new() -> Self {
        Self {
            menu_entries: Self::get_menu_entries(),
            selected: Self::get_menu_entries()
                .iter()
                .position(|e| e.factory.is_some())
                .unwrap_or(0),
        }
    }

    pub fn next(&mut self) {
        if self.menu_entries.is_empty() {
            return;
        }
        let len = self.menu_entries.len();
        let mut i = self.selected;
        loop {
            i = (i + 1) % len;
            if self.menu_entries[i].factory.is_some() {
                self.selected = i;
                break;
            }
            if i == self.selected {
                break; // no selectable entries
            }
        }
    }

    pub fn previous(&mut self) {
        if self.menu_entries.is_empty() {
            return;
        }
        let len = self.menu_entries.len();
        let mut i = self.selected;
        loop {
            i = if i == 0 { len - 1 } else { i - 1 };
            if self.menu_entries[i].factory.is_some() {
                self.selected = i;
                break;
            }
            if i == self.selected {
                break; // no selectable entries
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Build list items; headers (factory == None) are styled and non-selectable.
        let mut items: Vec<ListItem> = Vec::with_capacity(self.menu_entries.len());
        for (i, entry) in self.menu_entries.iter().enumerate() {
            if entry.factory.is_none() {
                // Group header - cyan bold
                items.push(
                    ListItem::new(format!("━━ {} ━━", entry.description))
                        .style(Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),
                );
            } else {
                // Regular menu item - subtle indicator for selected item
                let prefix = if i == self.selected { "  ► " } else { "    " };
                items.push(ListItem::new(format!("{}{}", prefix, entry.description)));
            }
        }

        let mut list_state = ListState::default();
        // Ensure selected points to a selectable entry (it should already), but guard anyway
        let sel = if self
            .menu_entries
            .get(self.selected)
            .map(|e| e.factory.is_some())
            .unwrap_or(false)
        {
            Some(self.selected)
        } else {
            // find first selectable
            self.menu_entries.iter().position(|e| e.factory.is_some())
        };
        list_state.select(sel);

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(200, 150, 0)))
                    .padding(ratatui::widgets::Padding::uniform(1))
                    .title("Commands")
                    .title_style(
                        Style::default()
                            .fg(Color::Rgb(200, 150, 0))
                            .add_modifier(Modifier::BOLD),
                    ),
            )
            .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
            .highlight_symbol("");

        // Render the selectable menu in the top chunk
        frame.render_stateful_widget(list, area, &mut list_state);
    }

    pub fn get_factory(&self) -> Option<ViewFactory> {
        self.menu_entries.get(self.selected).and_then(|e| e.factory)
    }

    fn get_menu_entries() -> Vec<MenuEntry> {
        macro_rules! menu_entry {
            (group: $desc:expr) => {
                MenuEntry {
                    description: $desc,
                    factory: None,
                }
            };
            (item: $desc:expr, $view:ty) => {
                MenuEntry {
                    description: $desc,
                    factory: Some(|ctrl| Box::new(<$view>::new(ctrl))),
                }
            };
            (item: $desc:expr, $view:ty, $($args:expr),+) => {
                MenuEntry {
                    description: $desc,
                    factory: Some(|ctrl| Box::new(<$view>::new(ctrl, $($args),+))),
                }
            };
        }

        vec![
            // Basic
            menu_entry!(group: "Basic"),
            menu_entry!(item: "Show Summoner Info", SummonerInfoView),
            // Live game
            menu_entry!(group: "Live Game"),
            menu_entry!(item: "Champ Select Info (ARAM)", ChampSelectAramView),
            menu_entry!(item: "Live Game Player Info", LivePlayerInfoView),
            // Mastery
            menu_entry!(group: "Mastery"),
            menu_entry!(item: "All Masteries", AllMasteriesView),
            menu_entry!(item: "Sky is the Limit", NextMasteryView, (10..=1000).collect(), "10+"),
            menu_entry!(item: "Mastery 10 Milestone", NextMasteryView, vec![7, 8, 9], "7-9"),
            menu_entry!(item: "Mastery  7 Milestone", NextMasteryView, vec![5, 6], "5-6"),
            menu_entry!(item: "Mastery  5 Milestone", NextMasteryView, vec![3, 4], "3-4"),
            menu_entry!(item: "Unplayed Champions", UnplayedChampsView),
            // Progress
            menu_entry!(group: "Progress"),
            menu_entry!(item: "Challenges Overview", ChallengesOverviewView),
            // Inventory
            menu_entry!(group: "Inventory"),
            menu_entry!(item: "Champions Without Skin", ChampionsWithoutSkinView),
            menu_entry!(item: "Chromas Without Skin", ChromasWithoutSkinView),
            // Loot
            menu_entry!(group: "Loot"),
            menu_entry!(item: "Blue Essence Info", BlueEssenceOverviewView),
            menu_entry!(item: "Missing Champion Shards", MissingChampShardsView),
            menu_entry!(item: "Interesting Skins", InterestingSkinsView),
            menu_entry!(item: "Skin Shards for First Skin", SkinShardsFirstSkinView),
            menu_entry!(item: "Disenchantable Skin Shards", SkinShardsDisenchantableView),
        ]
    }
}
