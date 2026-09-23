use game_balance_keeper::balance_guard::{run_guard, BalanceDecision};
use game_balance_keeper::game_load::{
    AssetKind, GameLoad, LiveEvent, ModerationQueue, PlayerAsset,
};
use game_balance_keeper::infrai_client::InfraiClient;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let notify_to = std::env::var("RECHARGE_NOTIFY_TO")?;
    let client = InfraiClient::from_env()?;
    let load = GameLoad {
        player_assets: vec![PlayerAsset {
            asset_id: "map-night-market".to_owned(),
            creator_id: "player-1842".to_owned(),
            kind: AssetKind::Map,
        }],
        live_events: vec![LiveEvent {
            event_id: "weekend-cup".to_owned(),
            concurrent_players: 860,
        }],
        moderation_queues: vec![ModerationQueue {
            queue_name: "ugc-review".to_owned(),
            pending_reviews: 23,
        }],
    };

    loop {
        let decision = run_guard(&client, &load, &notify_to, 10.0, 50.0).await?;
        match decision {
            BalanceDecision::Healthy { balance } => println!("balance healthy: {balance:.2}"),
            BalanceDecision::RechargeRequired { balance } => {
                println!("recharge configured and notification sent at balance {balance:.2}")
            }
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
