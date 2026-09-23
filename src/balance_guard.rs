use crate::game_load::GameLoad;
use crate::infrai_client::{InfraiClient, InfraiError};

#[derive(Debug, Clone, PartialEq)]
pub enum BalanceDecision {
    Healthy { balance: f64 },
    RechargeRequired { balance: f64 },
}

pub fn decide(balance: f64, trigger_balance: f64) -> BalanceDecision {
    if balance <= trigger_balance {
        BalanceDecision::RechargeRequired { balance }
    } else {
        BalanceDecision::Healthy { balance }
    }
}

pub async fn run_guard(
    client: &InfraiClient,
    load: &GameLoad,
    notify_to: &str,
    trigger_balance: f64,
    recharge_amount: f64,
) -> Result<BalanceDecision, InfraiError> {
    let balance = client.account_balance().await?;
    let decision = decide(balance, trigger_balance);

    if let BalanceDecision::RechargeRequired { balance } = decision {
        client
            .configure_autorecharge(trigger_balance, recharge_amount)
            .await?;
        client
            .send_email(
                notify_to,
                "Game backend recharge triggered",
                &format!(
                    "Balance {balance:.2} reached the {trigger_balance:.2} trigger. Recharge amount: {recharge_amount:.2}. Workload: {}.",
                    load.operational_summary()
                ),
            )
            .await?;
    }

    Ok(decision)
}
