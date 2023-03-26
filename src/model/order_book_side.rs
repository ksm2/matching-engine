use crate::model::compare::Compare;
use crate::model::{Order, Side, Trade};
use log::debug;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashSet, VecDeque};

#[derive(Debug, Clone)]
pub struct OrderBookSide {
    levels: BTreeMap<Compare<Decimal>, VecDeque<Order>>,
    reverse: bool,
}

impl OrderBookSide {
    pub fn new(reverse: bool) -> Self {
        Self {
            levels: BTreeMap::new(),
            reverse,
        }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.levels.values().map(|lvl| lvl.len()).sum()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.levels.is_empty()
    }

    #[cfg(test)]
    pub fn into_vec(mut self) -> Vec<Order> {
        let mut vec = Vec::with_capacity(self.len());
        while let Some((_, mut orders)) = self.levels.pop_first() {
            while let Some(order) = orders.pop_front() {
                vec.push(order);
            }
        }
        vec
    }

    #[cfg(test)]
    pub fn peek(&self) -> Option<&Order> {
        self.levels
            .first_key_value()
            .and_then(|entry| entry.1.iter().next())
    }

    pub fn fill(&mut self, order: &mut Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        let mut levels_to_delete = HashSet::new();

        for (&opposite_order_price, opposite_orders) in self.levels.iter_mut() {
            if order.is_filled() || !order.crosses(*opposite_order_price) {
                break;
            }

            while !order.is_filled() {
                let Some(mut opposite_order) = opposite_orders.pop_front() else {
                    break;
                };

                let trade = Self::execute_trade(order, &mut opposite_order);
                trades.push(trade);

                if !opposite_order.is_filled() {
                    opposite_orders.push_front(opposite_order);
                }
            }

            if opposite_orders.is_empty() {
                levels_to_delete.insert(opposite_order_price);
            }
        }

        for level in levels_to_delete {
            self.levels.remove(&level);
        }

        trades
    }

    fn execute_trade(order: &mut Order, other: &mut Order) -> Trade {
        let (buy_order_id, sell_order_id) = match order.side {
            Side::Buy => (order.id, other.id),
            Side::Sell => (other.id, order.id),
        };

        let used_qty = other.fill(order.unfilled());
        order.fill(used_qty);
        debug!("Filled bid at {}", other.price);

        Trade::new(other.price, used_qty, buy_order_id, sell_order_id)
    }

    pub fn push(&mut self, order: Order) {
        self.levels
            .entry(Compare::new(order.price, self.reverse))
            .or_default()
            .push_back(order);
    }
}
