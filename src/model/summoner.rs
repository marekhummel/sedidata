use super::ids::SummonerId;

#[derive(Debug)]
pub struct Summoner {
    pub id: SummonerId,
    pub display_name: String,
    pub internal_name: String,
    pub level: u16,
}
