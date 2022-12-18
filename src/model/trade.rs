use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::model::OrderId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    pub price: Decimal,
    pub quantity: Decimal,
    pub buy_order_id: OrderId,
    pub sell_order_id: OrderId,
    pub executed_at: u128,
}

impl Trade {
    pub fn new(
        price: Decimal,
        quantity: Decimal,
        buy_order_id: OrderId,
        sell_order_id: OrderId,
    ) -> Self {
        let now = SystemTime::now();
        let executed_at = now.duration_since(UNIX_EPOCH).unwrap().as_nanos();

        Self {
            price,
            quantity,
            buy_order_id,
            sell_order_id,
            executed_at,
        }
    }
}
