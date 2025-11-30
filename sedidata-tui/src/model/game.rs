use itertools::Itertools;

use super::{ids::ChampionId, summoner::SummonerWithStats};

#[derive(Debug, Clone)]
pub struct ChampSelectSession {
    pub session_id: String,
    pub queue_id: u16,
    pub local_player_cell: u8,
    pub benched_champs: Vec<ChampionId>,
    pub my_team: Vec<ChampSelectPlayerInfo>,
    pub their_team: Vec<ChampSelectPlayerInfo>,
}

#[derive(Debug, Clone)]
pub struct ChampSelectPlayerInfo {
    pub cell_id: u8,
    pub position: String,
    pub _puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub is_ally: bool,
    pub selected_champion: ChampionId,
}

#[derive(Debug, Clone)]
pub struct QueueInfo {
    pub queue_id: u16,
    pub _category: String,
    pub _description: String,
    pub _gamemode: String,
    pub _type_descriptor: String,
    pub _select_mode_group: String,
    pub pick_mode: String,
}

#[derive(Debug, Clone)]
pub struct LiveGameSession {
    pub players: Vec<LiveGamePlayerInfo>,
}

#[derive(Debug, Clone)]
pub struct LiveGamePlayerInfo {
    pub game_name: String,
    pub tag_line: String,
    pub position: String,
    pub _champion_name: String,
    pub team: String,
    pub _is_bot: bool,
}

#[derive(Debug, Clone)]
pub enum GameState {
    ChampSelect {
        session: ChampSelectSession,
        players: Option<Vec<SummonerWithStats>>,
    },
    LiveGame {
        session: LiveGameSession,
        players: Option<Vec<SummonerWithStats>>,
    },
    NotInGame,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub game_name: String,
    pub tag_line: String,
    pub position: String,
    pub is_ally: bool,
}

impl GameState {
    pub fn ranked_players(&self) -> Option<&[SummonerWithStats]> {
        match self {
            GameState::ChampSelect { players: Some(p), .. } | GameState::LiveGame { players: Some(p), .. } => {
                Some(p.as_slice())
            }
            _ => None,
        }
    }

    pub fn player_infos(&self, ally: &(String, String)) -> Vec<PlayerInfo> {
        match self {
            GameState::ChampSelect { session, .. } => session
                .my_team
                .iter()
                .chain(session.their_team.iter())
                .map(|p| PlayerInfo {
                    game_name: p.game_name.clone(),
                    tag_line: p.tag_line.clone(),
                    position: p.position.clone(),
                    is_ally: p.is_ally,
                })
                .collect(),
            GameState::LiveGame { session, .. } => {
                let ally_team = session
                    .players
                    .iter()
                    .find(|p| p.game_name == ally.0 && p.tag_line == ally.1)
                    .map(|p| p.team.clone())
                    .unwrap();
                session
                    .players
                    .iter()
                    .map(|p| PlayerInfo {
                        game_name: p.game_name.clone(),
                        tag_line: p.tag_line.clone(),
                        position: p.position.clone(),
                        is_ally: p.team == ally_team,
                    })
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}

impl PartialEq for ChampSelectSession {
    fn eq(&self, other: &Self) -> bool {
        self.session_id == other.session_id
    }
}

impl PartialEq for LiveGameSession {
    fn eq(&self, other: &Self) -> bool {
        if self.players.len() != other.players.len() {
            return false;
        }

        let p1_sorted = self.players.iter().sorted_by_key(|p| (&p.game_name, &p.tag_line));
        let p2_sorted = other.players.iter().sorted_by_key(|p| (&p.game_name, &p.tag_line));
        p1_sorted
            .zip(p2_sorted)
            .all(|(a, b)| a.game_name == b.game_name && a.tag_line == b.tag_line)
    }
}
