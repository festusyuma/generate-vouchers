#[derive(Debug)]
pub struct Voucher {
    pin: String,
    serial: String,
}

impl Voucher {
    pub fn new(pin: &str, serial: &str) -> Self {
        Voucher {
            pin: pin.to_string().to_uppercase(),
            serial: serial.to_string(),
        }
    }

    pub fn get(&self) -> (&str, &str) {
        (&self.pin, &self.serial)
    }
}
