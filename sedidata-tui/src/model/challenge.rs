#[derive(Clone, Debug)]
pub struct Challenge {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub current_level: String,
    pub next_level: String,
    pub current_value: f32,
    pub threshold_value: f32,
    pub thresholds: Vec<Threshold>,
    pub gamemodes: Vec<String>,
    pub _parent_id: i32,
    pub _children: Vec<i32>,
    pub is_capstone: bool,
    pub category: String,
}

pub const LEVELS: [&str; 10] = [
    "IRON",
    "BRONZE",
    "SILVER",
    "GOLD",
    "PLATINUM",
    "EMERALD",
    "DIAMOND",
    "MASTER",
    "GRANDMASTER",
    "CHALLENGER",
];

#[derive(Clone, Debug)]
pub struct Threshold {
    pub level: String,
    pub value: u16,
}

impl Challenge {
    pub fn is_completed(&self) -> bool {
        if self.next_level.is_empty() {
            return true;
        }

        if self.current_level == "NONE" {
            return false;
        }

        self.reward_in_pts() == 0
    }

    pub fn reward_in_pts(&self) -> u16 {
        let current = self.get_threshold(&self.current_level);
        let next = self.get_threshold(&self.next_level);
        next.value - current.value
    }

    pub fn gamemode_short(&self) -> &str {
        if self.gamemodes.iter().any(|g| g == "CLASSIC") {
            "SR"
        } else if self.gamemodes.iter().any(|g| g == "ARAM") {
            "HA"
        } else if self.gamemodes.iter().any(|g| g == "URF") {
            "URF"
        } else {
            "-"
        }
    }

    fn get_threshold(&self, level: &str) -> Threshold {
        if level == "NONE" {
            return Threshold {
                level: "NONE".into(),
                value: 0,
            };
        }

        self.thresholds
            .iter()
            .find(|t| t.level == level)
            .unwrap_or_else(|| panic!("Invalid level: '{}'", level))
            .clone()
    }
}
