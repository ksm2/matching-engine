use serde::{Deserialize, Serialize};

use crate::model::{PricePair, Side};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<PricePair>,
    pub asks: Vec<PricePair>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    pub fn bid_or_ask(&mut self, side: Side, pair: PricePair) {
        match side {
            Side::Buy => self.bid(pair),
            Side::Sell => self.ask(pair),
        }
    }

    pub fn bid(&mut self, new_bid: PricePair) {
        for (index, existing_bid) in self.bids.iter_mut().enumerate() {
            if existing_bid.price == new_bid.price {
                existing_bid.quantity += new_bid.quantity;
                return;
            } else if existing_bid.price < new_bid.price {
                self.bids.insert(index, new_bid);
                return;
            }
        }
        // The price is lower than all other bids
        self.bids.push(new_bid);
    }

    pub fn ask(&mut self, new_ask: PricePair) {
        for (index, existing_ask) in self.asks.iter_mut().enumerate() {
            if existing_ask.price == new_ask.price {
                existing_ask.quantity += new_ask.quantity;
                return;
            } else if existing_ask.price > new_ask.price {
                self.asks.insert(index, new_ask);
                return;
            }
        }
        // The price is higher than all other asks
        self.asks.push(new_ask);
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
        o.bid(PricePair::new(dec!(11), dec!(200)));
        assert_eq!(o.bids, vec![PricePair::new(dec!(11), dec!(200))]);

        o.bid(PricePair::new(dec!(10), dec!(300)));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(11), dec!(200)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );

        o.bid(PricePair::new(dec!(12), dec!(500)));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(12), dec!(500)),
                PricePair::new(dec!(11), dec!(200)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );

        o.bid(PricePair::new(dec!(11), dec!(500)));
        assert_eq!(
            o.bids,
            vec![
                PricePair::new(dec!(12), dec!(500)),
                PricePair::new(dec!(11), dec!(700)),
                PricePair::new(dec!(10), dec!(300)),
            ]
        );
    }

    #[test]
    fn should_ask_a_new_price() {
        let mut o = OrderBook::new();
        o.ask(PricePair::new(dec!(11), dec!(200)));
        assert_eq!(o.asks, vec![PricePair::new(dec!(11), dec!(200))]);

        o.ask(PricePair::new(dec!(10), dec!(300)));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(11), dec!(200)),
            ]
        );

        o.ask(PricePair::new(dec!(12), dec!(500)));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(11), dec!(200)),
                PricePair::new(dec!(12), dec!(500)),
            ]
        );

        o.ask(PricePair::new(dec!(11), dec!(500)));
        assert_eq!(
            o.asks,
            vec![
                PricePair::new(dec!(10), dec!(300)),
                PricePair::new(dec!(11), dec!(700)),
                PricePair::new(dec!(12), dec!(500)),
            ]
        );
    }
}
