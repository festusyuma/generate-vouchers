use crate::voucher::Voucher;

pub mod file;

pub trait Logger {
    fn log(&mut self, voucher: Voucher);
}
