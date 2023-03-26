use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use market::Market;
pub use messages::{MessageChannel, MessagePort};
pub use order::{Order, OrderId, OrderStatus};
pub use order_book::{OrderBook, PricePair};
pub use order_book_side::OrderBookSide;
pub use order_type::OrderType;
pub use side::Side;
pub use state::State;
pub use trade::Trade;
pub use user::User;
pub use wal::WriteAheadLog;

mod compare;
mod market;
mod messages;
mod order;
mod order_book;
mod order_book_side;
mod order_type;
mod side;
mod state;
mod trade;
mod user;
mod wal;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenOrder {
    pub quantity: Decimal,
    pub price: Decimal,
    pub side: Side,
    pub order_type: OrderType,
}
