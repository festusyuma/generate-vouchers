use crate::config::Config;
use crate::logger::Logger;
use crate::store::VoucherStore;
use crate::voucher::Voucher;
use chrono::offset;
use rand::TryRngCore;
use rand::rngs::OsRng;

const PIN_ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const SERIAL_ALPHABET: &[u8] = b"0123456789";

pub struct Generator {}

fn validate_voucher_pin(pin: &str) -> bool {
    let mut char_iter = pin.chars().into_iter();

    let mut has_alpha = false;
    let mut has_numeric = false;

    while let Some(first) = char_iter.next() {
        if let Some(second) = char_iter.next() {
            // prevent consecutive characters
            if second == first {
                return false;
            }

            if !has_alpha && (first.is_alphabetic() | second.is_alphabetic()) {
                has_alpha = true;
            }

            if !has_numeric && (first.is_numeric() | second.is_numeric()) {
                has_numeric = true;
            }
        }
    }

    has_alpha & has_numeric
}

fn generate_random(rng: &mut OsRng, chars: &[u8], size: usize) -> String {
    let mut pin = String::with_capacity(size);
    let max_allowed = u8::MAX - chars.len() as u8;

    while pin.len() < size {
        let mut byte = [0u8; 1];

        rng.try_fill_bytes(&mut byte)
            .expect("failed to read secure random bytes from OS");

        if byte[0] < max_allowed {
            let index = byte[0] as usize % chars.len();
            pin.push(chars[index] as char);
        }
    }

    pin
}

impl Generator {
    pub fn generate<VS: VoucherStore, W: Logger>(config: &Config, store: &mut VS, logger: &mut W) {
        let mut rng = OsRng;
        let mut vouchers_generated = 0;
        let mut vouchers = Vec::with_capacity(config.batch_size);
        let mut total_created = 0;

        loop {
            let serial = generate_random(&mut rng, SERIAL_ALPHABET, 20);

            let pin = {
                let mut generated;

                loop {
                    generated = generate_random(&mut rng, PIN_ALPHABET, config.pin_size);

                    if validate_voucher_pin(&generated) {
                        break;
                    }
                }

                generated
            };

            vouchers.push(Voucher::new(&pin, &serial));
            vouchers_generated += 1;

            if vouchers_generated == (config.no_of_vouchers - total_created)
                || (vouchers_generated != 0 && vouchers_generated % config.batch_size == 0)
            {
                println!(
                    "[{}] Generated {} vouchers, attempting to save...",
                    offset::Local::now(),
                    vouchers_generated,
                );

                let saved_vouchers = store.save(&mut vouchers.into_iter(), logger);
                total_created += saved_vouchers;

                vouchers = Vec::with_capacity(config.batch_size);
                vouchers_generated = 0;

                println!(
                    "[{}] Total generated: {}",
                    offset::Local::now(),
                    total_created,
                );

                if total_created == config.no_of_vouchers {
                    break;
                }
            }
        }
    }
}
