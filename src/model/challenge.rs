pub struct ChallengeCategory {
    pub id: i32,
    pub name: String,
    pub children: Vec<Challenge>,
}

pub struct Challenge {
    pub id: i32,
    pub name: String,
    pub current_level: String,
    pub next_level: String,
    pub thresholds: Vec<Threshold>,
    pub parent_id: i32,
}

pub struct Threshold {
    pub level: String,
    pub value: u16,
}
