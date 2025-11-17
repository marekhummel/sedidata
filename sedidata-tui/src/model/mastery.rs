use super::ids::ChampionId;

#[derive(Debug, Clone)]
pub struct Mastery {
    pub champ_id: ChampionId,
    pub level: u16,
    pub points: u32,
    pub missing_points: i32,
    pub marks: u16,
    pub required_marks: u16,
    pub next_milestone: Milestone,
}

impl Mastery {
    pub fn required_points(&self) -> u32 {
        (self.points as i32 + self.missing_points) as u32
    }
}

#[derive(Debug, Clone)]
pub struct Milestone {
    pub reward_marks: u16,
    pub require_grade_counts: Vec<(String, u16)>,
}

// LUX
//   {
//     "championId": 99,
//     "championLevel": 9,
//     "championPoints": 82371,
//     "championPointsSinceLastLevel": 17771,
//     "championPointsUntilNextLevel": -6771,
//     "championSeasonMilestone": 0,
//     "highestGrade": "C+",
//     "lastPlayTime": 1762382764000,
//     "markRequiredForNextLevel": 2,
//     "milestoneGrades": [
//       "C+"
//     ],
//     "nextSeasonMilestone": {
//       "bonus": false,
//       "requireGradeCounts": {
//         "A-": 1
//       },
//       "rewardConfig": {
//         "maximumReward": 0,
//         "rewardValue": ""
//       },
//       "rewardMarks": 1
//     },
//     "puuid": "7496d9e2-1ec1-5303-b3e5-02caab8c00aa",
//     "tokensEarned": 1
//   },
