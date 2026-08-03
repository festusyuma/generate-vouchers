use crate::voucher::Voucher;
use crate::logger::Logger;

pub mod db;
pub mod memory;

pub trait VoucherStore {
    fn save<T: Iterator<Item = Voucher>, TR: Logger>(
        &mut self,
        voucher: &mut T,
        logger: &mut TR,
    ) -> usize;
}
