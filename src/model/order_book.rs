use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use super::Side;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderBook {
    pub last: Option<Decimal>,
    pub best_bid: Option<Decimal>,
    pub best_ask: Option<Decimal>,
    pub bids: Vec<PricePair>,
    pub asks: Vec<PricePair>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            last: None,
            best_bid: None,
            best_ask: None,
            bids: Vec::new(),
            asks: Vec::new(),
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
                    if index == 0 {
                        self.best_bid = Some(existing_bid.quantity);
                    }
                    return;
                }
                Ordering::Less => {
                    self.bids.insert(index, PricePair::new(bid_price, bid_qty));
                    if index == 0 {
                        self.best_bid = Some(bid_price);
                    }
                    return;
                }
                Ordering::Greater => {}
            };
        }
        // The price is lower than all other bids
        let empty = self.bids.is_empty();
        self.bids.push(PricePair::new(bid_price, bid_qty));
        if empty {
            self.best_bid = Some(bid_price);
        }
    }

    pub fn place_ask(&mut self, ask_price: Decimal, ask_qty: Decimal) {
        for (index, existing_ask) in self.asks.iter_mut().enumerate() {
            match existing_ask.price.cmp(&ask_price) {
                Ordering::Equal => {
                    existing_ask.quantity += ask_qty;
                    if index == 0 {
                        self.best_ask = Some(existing_ask.quantity);
                    }
                    return;
                }
                Ordering::Greater => {
                    self.asks.insert(index, PricePair::new(ask_price, ask_qty));
                    if index == 0 {
                        self.best_ask = Some(ask_price);
                    }
                    return;
                }
                Ordering::Less => {}
            }
        }
        // The price is higher than all other asks
        let empty = self.asks.is_empty();
        self.asks.push(PricePair::new(ask_price, ask_qty));
        if empty {
            self.best_ask = Some(ask_price);
        }
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
                    if index == 0 {
                        self.best_bid = self.bids.first().map(|p| p.price);
                    }
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
                    if index == 0 {
                        self.best_ask = self.asks.first().map(|p| p.price);
                    }
                }
                return;
            }
        }
    }

    pub fn last(&mut self, price: Decimal) {
        self.last = Some(price);
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn should_be_created_empty() {
        let o = OrderBook::new();
        assert_eq!(o.bids, Vec::new());
        assert_eq!(o.best_bid, None);
        assert_eq!(o.asks, Vec::new());
        assert_eq!(o.best_ask, None);
    }

    #[test]
    fn should_bid_a_new_price() {
        let mut o = OrderBook::new();
        o.place_bid(dec!(11), dec!(200));
        assert_eq!(o.bids, vec![PricePair::new(dec!(11), dec!(200))]);
        assert_eq!(o.best_bid, Some(dec!(11)));

        o.place_bid(dec!(10), dec!(300));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(11), dec!(200)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );
        assert_eq!(o.best_bid, Some(dec!(11)));

        o.place_bid(dec!(12), dec!(500));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(12), dec!(500)),
                PricePair::new(dec!(11), dec!(200)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );
        assert_eq!(o.best_bid, Some(dec!(12)));

        o.place_bid(dec!(11), dec!(500));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(12), dec!(500)),
                PricePair::new(dec!(11), dec!(700)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );
        assert_eq!(o.best_bid, Some(dec!(12)));

        o.take_bid(dec!(11), dec!(300));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(12), dec!(500)),
                PricePair::new(dec!(11), dec!(400)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );
        assert_eq!(o.best_bid, Some(dec!(12)));

        o.take_bid(dec!(11), dec!(400));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(12), dec!(500)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );
        assert_eq!(o.best_bid, Some(dec!(12)));

        o.take_bid(dec!(12), dec!(500));
        assert_eq!(o.bids, vec![PricePair::new(dec!(10), dec!(300))]);
        assert_eq!(o.best_bid, Some(dec!(10)));

        o.take_bid(dec!(10), dec!(300));
        assert_eq!(o.bids, vec![]);
        assert_eq!(o.best_bid, None);
    }

    #[test]
    fn should_ask_a_new_price() {
        let mut o = OrderBook::new();
        o.place_ask(dec!(11), dec!(200));
        assert_eq!(o.asks, vec![PricePair::new(dec!(11), dec!(200))]);
        assert_eq!(o.best_ask, Some(dec!(11)));

        o.place_ask(dec!(10), dec!(300));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(11), dec!(200)),
            ]
        );
        assert_eq!(o.best_ask, Some(dec!(10)));

        o.place_ask(dec!(12), dec!(500));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(11), dec!(200)),
                PricePair::new(dec!(12), dec!(500)),
            ]
        );
        assert_eq!(o.best_ask, Some(dec!(10)));

        o.place_ask(dec!(11), dec!(500));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(11), dec!(700)),
                PricePair::new(dec!(12), dec!(500)),
            ]
        );
        assert_eq!(o.best_ask, Some(dec!(10)));

        o.take_ask(dec!(11), dec!(300));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(11), dec!(400)),
                PricePair::new(dec!(12), dec!(500)),
            ]
        );
        assert_eq!(o.best_ask, Some(dec!(10)));

        o.take_ask(dec!(11), dec!(400));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(12), dec!(500)),
            ]
        );
        assert_eq!(o.best_ask, Some(dec!(10)));

        o.take_ask(dec!(10), dec!(300));
        assert_eq!(o.asks, vec![PricePair::new(dec!(12), dec!(500))]);
        assert_eq!(o.best_ask, Some(dec!(12)));

        o.take_ask(dec!(12), dec!(500));
        assert_eq!(o.asks, vec![]);
        assert_eq!(o.best_ask, None);
    }

    #[test]
    fn should_handle_a_trade() {
        let mut o = OrderBook::new();
        assert_eq!(o.last, None);

        o.last(dec!(15));
        assert_eq!(o.last, Some(dec!(15)));
    }
}
