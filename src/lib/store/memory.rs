use crate::config::Config;
use crate::store::VoucherStore;
use crate::voucher::Voucher;
use crate::logger::Logger;
use std::collections::HashSet;

pub struct MemoryStore {
    pins: HashSet<String>,
    serials: HashSet<String>,
}

impl MemoryStore {
    pub fn init(_: &Config) -> Self {
        MemoryStore {
            pins: HashSet::new(),
            serials: HashSet::new(),
        }
    }
}

impl VoucherStore for MemoryStore {
    fn save<T: Iterator<Item = Voucher>, TR: Logger>(
        &mut self,
        vouchers: &mut T,
        logger: &mut TR,
    ) -> usize {
        let mut saved_vouchers = 0;

        while let Some(voucher) = vouchers.next() {
            let (pin, serial) = voucher.get();

            if self.pins.contains(pin) || self.serials.contains(serial) {
                continue;
            }

            saved_vouchers += 1;
            self.pins.insert(String::from(pin));
            self.serials.insert(String::from(serial));
            logger.log(voucher);
        }

        saved_vouchers
    }
}
