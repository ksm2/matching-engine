use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    #[default]
    Limit,
    Market,
}
