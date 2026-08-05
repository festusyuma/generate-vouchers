use crate::config::Config;
use crate::generator::{Generator, GeneratorAction};
use crate::logger::Logger;
use crate::store::{StoreAction, VoucherStore};
use crate::voucher::Voucher;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct MemoryStore {
    pins: HashSet<String>,
    serials: HashSet<String>,
    batch_size: usize,
    total_vouchers: usize,
    saved_vouchers: usize,
}

impl MemoryStore {
    pub fn new(config: &Config) -> Self {
        MemoryStore {
            pins: HashSet::new(),
            serials: HashSet::new(),
            batch_size: config.batch_size,
            total_vouchers: config.no_of_vouchers,
            saved_vouchers: 0,
        }
    }

    fn save(&mut self, voucher: Voucher) {
        let (pin, serial) = voucher.get();

        if self.pins.contains(pin) || self.serials.contains(serial) {
            return;
        }

        self.saved_vouchers += 1;
        self.pins.insert(String::from(pin));
        self.serials.insert(String::from(serial));

        // self.logger.log(voucher);
    }
}

impl VoucherStore for MemoryStore {
    fn start(
        store: Arc<Mutex<Self>>,
        generator: mpsc::Sender<GeneratorAction>,
    ) -> (mpsc::Sender<StoreAction>, JoinHandle<()>) {
        let (send, mut rx) = mpsc::channel(100);

        let stream = tokio::spawn(async move {
            let mut store = store.lock().await;

            loop {
                while let Some(action) = rx.recv().await {
                    match action {
                        StoreAction::Save(voucher) => {
                            store.save(voucher);
                        }
                        StoreAction::Stop => {
                            break;
                        }
                    }

                    if store.saved_vouchers % store.batch_size == 0 {
                        println!("loading: saved {} vouchers", store.saved_vouchers);
                    }
                }

                let to_generate = store.total_vouchers - store.saved_vouchers;
                println!("to_generate: {}", to_generate);

                if to_generate != 0 {
                    let _ = generator.send(GeneratorAction::Generate(to_generate)).await;
                    continue;
                }

                println!("completed: saved {} vouchers", store.saved_vouchers);
                break;
            }
        });

        (send, stream)
    }
}
