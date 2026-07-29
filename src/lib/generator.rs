use crate::config::Config;
use crate::store::VoucherStore;
use crate::voucher::Voucher;
use crate::writer::Writer;
use chrono::offset;
use rand::Rng;
use rand::distr::{Alphanumeric, SampleString};

pub struct Generator<VS: VoucherStore, W: Writer> {
    pin_size: usize,
    batch_size: usize,
    store: VS,
    writer: W,
}

impl<VS: VoucherStore, W: Writer> Generator<VS, W> {
    pub fn new(config: &Config, store: VS, writer: W) -> Self {
        Generator {
            pin_size: config.pin_size,
            batch_size: config.batch_size,
            store,
            writer,
        }
    }

    pub fn generate(&mut self, no_of_vouchers: usize) {
        let mut rng = rand::rng();
        let mut vouchers_generated = 0;
        let mut vouchers = Vec::with_capacity(self.batch_size);

        while vouchers_generated < no_of_vouchers {
            let serial: String = (0..20).map(|_| rng.random_range('0'..'9')).collect();

            let pin = {
                let mut generated;

                loop {
                    generated = String::new();

                    loop {
                        let generated_char = Alphanumeric
                            .sample_string(&mut rng, 1)
                            .to_uppercase()
                            .chars()
                            .last()
                            .expect("unable to generate character");

                        /* Prevents same consecutive characters */
                        if generated.len() > 0 {
                            if generated.chars().last().unwrap() == generated_char {
                                continue;
                            }
                        }

                        generated.push(generated_char);

                        if generated.len() == self.pin_size {
                            break;
                        }
                    }

                    if validate_voucher_pin(&generated) {
                        break;
                    }
                }

                generated
            };

            vouchers.push(Voucher::new(&pin, &serial));
            vouchers_generated += 1;

            if vouchers_generated != 0 && vouchers_generated % self.batch_size == 0 {
                println!(
                    "[{}] Generated {} vouchers, attempting to save...",
                    offset::Local::now(),
                    self.batch_size,
                );

                let saved_vouchers = self.store.save(&mut vouchers.into_iter(), &mut self.writer);

                vouchers = Vec::with_capacity(self.batch_size);
                vouchers_generated = vouchers_generated + saved_vouchers - self.batch_size;

                println!(
                    "[{}] Total generated: {}",
                    offset::Local::now(),
                    vouchers_generated,
                );
            }
        }
    }
}

fn validate_voucher_pin(pin: &str) -> bool {
    for c in pin.chars() {
        if c.is_alphabetic() {
            return true;
        }
    }

    false
}
