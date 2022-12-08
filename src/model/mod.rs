use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use messages::{MessageChannel, MessagePort};
pub use order::{Order, OrderId, OrderStatus};
pub use order_book::{OrderBook, PricePair};
pub use side::Side;
pub use state::State;
pub use order_type::OrderType;
pub use trade::Trade;

mod messages;
mod order;
mod order_book;
mod order_type;
mod side;
mod state;
mod trade;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenOrder {
    pub quantity: Decimal,
    pub price: Decimal,
    pub side: Side,
    pub order_type: OrderType,
}
