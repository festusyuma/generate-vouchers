use crate::voucher::Voucher;

pub mod file;

pub trait Logger {
    fn log(&self, vouchers: Voucher);
    fn log_all(&self, vouchers: Vec<Voucher>);
}
