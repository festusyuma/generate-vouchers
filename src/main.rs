use chrono::offset;
use rand::Rng;
use rand::distr::{Alphanumeric, SampleString};
use std::collections::HashSet;
use std::{env, fs, io::Write};

fn validate_voucher_pin(pin: &str) -> bool {
    let mut has_char = false;
    let mut has_digit = false;

    for c in pin.chars() {
        if !has_char {
            has_char = c.is_alphabetic()
        }

        if !has_digit {
            has_digit = c.is_numeric()
        }

        if has_digit & has_char {
            return true;
        }
    }

    false
}

fn main() {
    let mut rng = rand::rng();

    let mut existing_pins = HashSet::new();
    let mut existing_serials = HashSet::new();
    let mut vouchers_generated = 0;

    let args: Vec<String> = env::args().collect();
    let no_of_vouchers: &i32 = &args[1].clone().parse().expect("enter number of vouchers");
    let output_file = &args[2];

    let mut file = fs::File::create(output_file).unwrap();
    file.write("pin, serial\n".as_bytes()).unwrap();

    loop {
        if &vouchers_generated == no_of_vouchers {
            break;
        }

        if &vouchers_generated % 1_000_000 == 0 {
            println!(
                "Generated {} vouchers at {}",
                vouchers_generated,
                offset::Local::now()
            )
        }

        let serial: String = (0..20).map(|_| rng.random_range('0'..'9')).collect();
        let pin = {
            let mut generated;

            loop {
                generated = Alphanumeric.sample_string(&mut rng, 5);
                if validate_voucher_pin(&generated) {
                    break;
                }
            }

            generated
        };

        if existing_pins.contains(&pin) || existing_serials.contains(&serial) {
            continue;
        }

        file.write(format!("\"{}\", \"{}\"\n", pin, serial).as_bytes())
            .unwrap();

        vouchers_generated += 1;
        existing_pins.insert(pin);
        existing_serials.insert(serial);
    }
}
