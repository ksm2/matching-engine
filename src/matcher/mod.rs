use tokio::runtime::Runtime;
use tokio::sync::mpsc::Receiver;

use crate::model::OpenOrder;

pub fn matcher(rt: &Runtime, mut rx: Receiver<OpenOrder>) {
    while let Some(message) = rt.block_on(rx.recv()) {
        println!("Processing {:?}", message);
    }
}
