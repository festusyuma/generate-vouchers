use crate::voucher::Voucher;
use crate::writer::Writer;

pub mod db;
pub mod memory;

pub trait VoucherStore {
    fn save<T: Iterator<Item = Voucher>, TR: Writer>(
        &mut self,
        voucher: &mut T,
        writer: &mut TR,
    ) -> usize;
}
