#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerAsset {
    pub asset_id: String,
    pub creator_id: String,
    pub kind: AssetKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetKind {
    Map,
    Skin,
    Replay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveEvent {
    pub event_id: String,
    pub concurrent_players: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModerationQueue {
    pub queue_name: String,
    pub pending_reviews: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GameLoad {
    pub player_assets: Vec<PlayerAsset>,
    pub live_events: Vec<LiveEvent>,
    pub moderation_queues: Vec<ModerationQueue>,
}

impl GameLoad {
    pub fn operational_summary(&self) -> String {
        let players: u32 = self
            .live_events
            .iter()
            .map(|event| event.concurrent_players)
            .sum();
        let reviews: u32 = self
            .moderation_queues
            .iter()
            .map(|queue| queue.pending_reviews)
            .sum();
        format!(
            "{} player assets, {} live players, {} moderation reviews pending",
            self.player_assets.len(),
            players,
            reviews
        )
    }
}
