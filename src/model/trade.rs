use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::model::OrderId;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    pub price: Decimal,
    pub quantity: Decimal,
    pub buy_order_id: OrderId,
    pub sell_order_id: OrderId,
}

impl Trade {
    pub fn new(
        price: Decimal,
        quantity: Decimal,
        buy_order_id: OrderId,
        sell_order_id: OrderId,
    ) -> Self {
        Self {
            price,
            quantity,
            buy_order_id,
            sell_order_id,
        }
    }
}
