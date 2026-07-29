use chrono::offset;
use dotenvy::dotenv;
use generate_vouchers::config::Config;
use generate_vouchers::store::VoucherStore;
use generate_vouchers::store::db::DbStore;
use generate_vouchers::voucher::Voucher;
use generate_vouchers::writer::file::FileWriter;
use rand::Rng;
use rand::distr::{Alphanumeric, SampleString};
use std::collections::HashSet;
use std::env;
use std::str::FromStr;

fn validate_voucher_pin(pin: &str) -> bool {
    for c in pin.chars() {
        if c.is_alphabetic() {
            return true;
        }
    }

    false
}

fn unwrap<Ta: Iterator<Item = String>, T: FromStr>(args: &mut Ta, message: &str) -> T {
    args.next().and_then(|v| v.parse().ok()).expect(message)
}

fn main() {
    dotenv().ok();

    let args = env::args();
    let config = Config::new(args);

    let mut store = DbStore::init(&config);
    let writer = FileWriter::new(&config);

    let mut existing_pins = HashSet::new();
    let mut existing_serials = HashSet::new();
    let mut vouchers_generated = 0;
    let mut rng = rand::rng();

    while vouchers_generated < config.no_of_vouchers {
        let serial: String = (0..20).map(|_| rng.random_range('0'..'9')).collect();
        if existing_serials.contains(&serial) {
            continue;
        }

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

                    if generated.len() == 5 {
                        break;
                    }
                }

                if validate_voucher_pin(&generated) {
                    break;
                }
            }

            generated
        };

        if existing_pins.contains(&pin) {
            continue;
        }

        vouchers_generated += 1;
        if vouchers_generated != 0 && vouchers_generated % config.batch_size == 0 {
            println!(
                "Generated {} vouchers at {}",
                vouchers_generated,
                offset::Local::now()
            );
        }

        existing_pins.insert(pin);
        existing_serials.insert(serial);
    }

    let mut vouchers = {
        let mut vouchers = vec![];
        let mut existing_pins = existing_pins.iter();
        let mut existing_serials = existing_serials.iter();

        while let Some(pin) = existing_pins.next() {
            if let Some(serial) = existing_serials.next() {
                vouchers.push(Voucher::new(pin, serial));
            } else {
                break;
            }
        }

        vouchers.into_iter()
    };

    store.save(&mut vouchers, Some(writer))
}
