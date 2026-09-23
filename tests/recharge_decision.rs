use game_balance_keeper::balance_guard::{decide, BalanceDecision};
use game_balance_keeper::game_load::{
    AssetKind, GameLoad, LiveEvent, ModerationQueue, PlayerAsset,
};

#[test]
fn low_balance_preserves_busy_game_workloads() {
    let load = GameLoad {
        player_assets: vec![PlayerAsset {
            asset_id: "skin-7".into(),
            creator_id: "player-9".into(),
            kind: AssetKind::Skin,
        }],
        live_events: vec![LiveEvent {
            event_id: "final-round".into(),
            concurrent_players: 400,
        }],
        moderation_queues: vec![ModerationQueue {
            queue_name: "player-assets".into(),
            pending_reviews: 12,
        }],
    };

    assert_eq!(
        decide(9.50, 10.0),
        BalanceDecision::RechargeRequired { balance: 9.50 }
    );
    assert_eq!(
        load.operational_summary(),
        "1 player assets, 400 live players, 12 moderation reviews pending"
    );
}
