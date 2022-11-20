use crate::model::Side;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderBook {
    pub last: Option<Decimal>,
    pub bids: Vec<PricePair>,
    pub asks: Vec<PricePair>,
    pub trades: Vec<PricePair>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            last: None,
            bids: Vec::new(),
            asks: Vec::new(),
            trades: Vec::new(),
        }
    }

    pub fn place(&mut self, side: Side, price: Decimal, qty: Decimal) {
        match side {
            Side::Buy => self.place_bid(price, qty),
            Side::Sell => self.place_ask(price, qty),
        }
    }

    pub fn place_bid(&mut self, bid_price: Decimal, bid_qty: Decimal) {
        for (index, existing_bid) in self.bids.iter_mut().enumerate() {
            match existing_bid.price.cmp(&bid_price) {
                Ordering::Equal => {
                    existing_bid.quantity += bid_qty;
                    return;
                }
                Ordering::Less => {
                    self.bids.insert(index, PricePair::new(bid_price, bid_qty));
                    return;
                }
                Ordering::Greater => {}
            };
        }
        // The price is lower than all other bids
        self.bids.push(PricePair::new(bid_price, bid_qty));
    }

    pub fn place_ask(&mut self, ask_price: Decimal, ask_qty: Decimal) {
        for (index, existing_ask) in self.asks.iter_mut().enumerate() {
            match existing_ask.price.cmp(&ask_price) {
                Ordering::Equal => {
                    existing_ask.quantity += ask_qty;
                    return;
                }
                Ordering::Greater => {
                    self.asks.insert(index, PricePair::new(ask_price, ask_qty));
                    return;
                }
                Ordering::Less => {}
            }
        }
        // The price is higher than all other asks
        self.asks.push(PricePair::new(ask_price, ask_qty));
    }

    pub fn take(&mut self, side: Side, price: Decimal, qty: Decimal) {
        match side {
            Side::Buy => self.take_bid(price, qty),
            Side::Sell => self.take_ask(price, qty),
        }
    }

    pub fn take_bid(&mut self, bid_price: Decimal, bid_qty: Decimal) {
        for (index, existing_bid) in self.bids.iter_mut().enumerate() {
            if existing_bid.price == bid_price {
                existing_bid.quantity -= bid_qty;
                if existing_bid.quantity.is_zero() {
                    self.bids.remove(index);
                }
                return;
            }
        }
    }

    pub fn take_ask(&mut self, ask_price: Decimal, ask_qty: Decimal) {
        for (index, existing_ask) in self.asks.iter_mut().enumerate() {
            if existing_ask.price == ask_price {
                existing_ask.quantity -= ask_qty;
                if existing_ask.quantity.is_zero() {
                    self.asks.remove(index);
                }
                return;
            }
        }
    }

    pub fn trade(&mut self, price: Decimal, quantity: Decimal) {
        let trade = PricePair::new(price, quantity);
        self.trades.push(trade);
        self.last = Some(price);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn should_be_created_empty() {
        let o = OrderBook::new();
        assert_eq!(o.asks, Vec::new());
        assert_eq!(o.bids, Vec::new());
    }

    #[test]
    fn should_bid_a_new_price() {
        let mut o = OrderBook::new();
        o.place_bid(dec!(11), dec!(200));
        assert_eq!(o.bids, vec![PricePair::new(dec!(11), dec!(200))]);

        o.place_bid(dec!(10), dec!(300));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(11), dec!(200)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );

        o.place_bid(dec!(12), dec!(500));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(12), dec!(500)),
                PricePair::new(dec!(11), dec!(200)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );

        o.place_bid(dec!(11), dec!(500));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(12), dec!(500)),
                PricePair::new(dec!(11), dec!(700)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );

        o.take_bid(dec!(11), dec!(300));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(12), dec!(500)),
                PricePair::new(dec!(11), dec!(400)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );

        o.take_bid(dec!(11), dec!(400));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(12), dec!(500)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );
    }

    #[test]
    fn should_ask_a_new_price() {
        let mut o = OrderBook::new();
        o.place_ask(dec!(11), dec!(200));
        assert_eq!(o.asks, vec![PricePair::new(dec!(11), dec!(200))]);

        o.place_ask(dec!(10), dec!(300));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(11), dec!(200)),
            ]
        );

        o.place_ask(dec!(12), dec!(500));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(11), dec!(200)),
                PricePair::new(dec!(12), dec!(500)),
            ]
        );

        o.place_ask(dec!(11), dec!(500));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(11), dec!(700)),
                PricePair::new(dec!(12), dec!(500)),
            ]
        );

        o.take_ask(dec!(11), dec!(300));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(11), dec!(400)),
                PricePair::new(dec!(12), dec!(500)),
            ]
        );

        o.take_ask(dec!(11), dec!(400));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(12), dec!(500)),
            ]
        );
    }

    #[test]
    fn should_handle_a_trade() {
        let mut o = OrderBook::new();
        assert_eq!(o.trades, Vec::new());
        assert_eq!(o.last, None);

        o.trade(dec!(15), dec!(500));
        assert_eq!(o.trades, vec![PricePair::new(dec!(15), dec!(500))]);
        assert_eq!(o.last, Some(dec!(15)));
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PricePair {
    pub price: Decimal,
    pub quantity: Decimal,
}

impl PricePair {
    pub fn new(price: Decimal, quantity: Decimal) -> Self {
        Self { price, quantity }
    }
}
