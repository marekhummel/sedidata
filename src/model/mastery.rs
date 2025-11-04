use super::ids::ChampionId;

#[derive(Debug)]
pub struct Mastery {
    pub champ_id: ChampionId,
    pub level: u16,
    pub points: u32,
    pub tokens: Option<u16>,
    pub points_to_next_level: i32,
    // pub chest_granted: bool,
}

//   {
//     "championId": 157,
//     "championLevel": 31,
//     "championPoints": 359288,
//     "championPointsSinceLastLevel": 52688,
//     "championPointsUntilNextLevel": -41688,
//     "championSeasonMilestone": 1,
//     "highestGrade": "S+",
//     "lastPlayTime": 1730930003000,
//     "markRequiredForNextLevel": 2,
//     "milestoneGrades": [
//       "A+",
//       "S",
//       "A"
//     ],
//     "nextSeasonMilestone": {
//       "bonus": false,
//       "requireGradeCounts": {
//         "A-": 1,
//         "C-": 4
//       },
//       "rewardConfig": {
//         "maximumReward": 0,
//         "rewardValue": ""
//       },
//       "rewardMarks": 1
//     },
//     "puuid": "7496d9e2-1ec1-5303-b3e5-02caab8c00aa",
//     "tokensEarned": 0
//   },
