use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use api_context::ApiContext;
pub use order::{Order, OrderId, OrderStatus};
pub use order_book::{OrderBook, PricePair};
pub use side::Side;
pub use state::State;
pub use trade::Trade;

mod api_context;
mod order;
mod order_book;
mod side;
mod state;
mod trade;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenOrder {
    pub quantity: Decimal,
    pub price: Decimal,
    pub side: Side,
}
