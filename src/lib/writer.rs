use crate::voucher::Voucher;

pub mod file;

pub trait Writer {
    fn write(&mut self, voucher: Voucher);
}
