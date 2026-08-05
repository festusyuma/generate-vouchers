use crate::generator::GeneratorAction;
use crate::voucher::Voucher;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;

pub mod db;
pub mod memory;
pub mod redis;

pub enum StoreAction {
    Save(Voucher),
    Stop,
}

pub trait VoucherStore {
    fn start(
        store: Arc<Mutex<Self>>,
        generator: mpsc::Sender<GeneratorAction>,
    ) -> (mpsc::Sender<StoreAction>, JoinHandle<()>);
}
