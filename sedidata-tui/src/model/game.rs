use itertools::Itertools;

use crate::model::{
    champion::Champion,
    ids::ChampionId,
    summoner::{SummonerName, SummonerWithStats},
};

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
    pub name: Option<SummonerName>,
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
    pub name: Option<SummonerName>,
    pub position: String,
    pub champion_name: String,
    pub team: String,
    pub _is_bot: bool,
}

#[derive(Debug, Clone)]
pub struct PostGameSession {
    pub game_id: u64,
    pub teams: Vec<PostGameTeamInfo>,
}

#[derive(Debug, Clone)]
pub struct PostGameTeamInfo {
    pub is_player_team: bool,
    pub _is_winning_team: bool,
    pub players: Vec<PostGamePlayerInfo>,
}

#[derive(Debug, Clone)]
pub struct PostGamePlayerInfo {
    pub name: SummonerName,
    pub position: String,
    pub champion_name: String,
    pub team_id: u16,
    pub _is_bot: bool,
}

#[derive(Debug, Clone)]
pub enum GameState {
    ChampSelect {
        session_info: ChampSelectSession,
        players: Vec<PlayerInfo>,
        ranked_info: Option<Vec<SummonerWithStats>>,
    },
    LiveGame {
        session_info: LiveGameSession,
        players: Vec<PlayerInfo>,
        ranked_info: Option<Vec<SummonerWithStats>>,
    },
    PostGame {
        session_info: PostGameSession,
        players: Vec<PlayerInfo>,
        ranked_info: Option<Vec<SummonerWithStats>>,
    },
    NotInGame,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct PlayerInfo {
    pub name: Option<SummonerName>,
    pub position: String,
    pub is_ally: Option<bool>,
    pub champion: Option<Champion>,
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

        let sorter = |p: &&LiveGamePlayerInfo| p.name.as_ref().map(|name| name.tuple());
        let p1_sorted = self.players.iter().sorted_by_key(sorter);
        let p2_sorted = other.players.iter().sorted_by_key(sorter);
        p1_sorted.zip(p2_sorted).all(|(a, b)| a.name == b.name)
    }
}

impl PartialEq for PostGameSession {
    fn eq(&self, other: &Self) -> bool {
        self.game_id == other.game_id
    }
}
